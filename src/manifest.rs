//! Typed file-population adapter for authored Nomos.
//!
//! A manifest associates validated relative `.nomos` paths with authored
//! top-level namespaces. It is already-decoded typed configuration; this module
//! does not define or hide a second textual parser. The complete reachable graph
//! is validated and read before structural planning begins, and every structural
//! plan is merged into one atomic naming-authority request.
//!
//! `[not-understood-by-psyche, Entry 7, NomosTrainAddendum-2026-07-30]`
//! New carrier shapes here are positional. All files resolved by one manifest
//! populate one self-contained v1 package namespace; no cross-package Invoke
//! lookup or fallback exists.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use encoded_name_table::{Name, OperationKey};
use signal_sema_translator::{
    AuthorityReply, DeclarationNode, SealUniversal, VocabularyEncodedId, VocabularyRoot,
    WritePrecondition,
};
use structural_codec::EncodedNameResolver;

use crate::textual::{PlannedNamePath, PlannedNomosLoad, validate_authority_reply};
use crate::{
    AuthoredTransformerSet, LoadedNomosPopulation, NomosLoadError, NomosModulePath, TextualNomos,
    TextualNomosError,
};

/// One normalized, root-relative `.nomos` source path.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NomosSourcePath(PathBuf);

impl NomosSourcePath {
    pub fn try_new(path: impl Into<PathBuf>) -> Result<Self, NomosManifestLoadError> {
        let path = path.into();
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(NomosManifestLoadError::InvalidSourcePath(path));
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("nomos") {
            return Err(NomosManifestLoadError::WrongSourceExtension(path));
        }
        Ok(Self(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// One file-to-namespace association and its explicit file dependencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NomosManifestFile(
    pub NomosSourcePath,
    pub NomosModulePath,
    pub Vec<NomosSourcePath>,
);

/// One entry point and the typed file associations available to its graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NomosFileManifest(pub NomosSourcePath, pub Vec<NomosManifestFile>);

/// The complete file-population plan awaiting one durable authority receipt.
pub struct PlannedNomosPopulation(
    Vec<PlannedNomosLoad>,
    SealUniversal,
    Vec<PlannedNamePath>,
    Vec<PlannedNamePath>,
    BTreeMap<VocabularyEncodedId, Name>,
);

impl PlannedNomosPopulation {
    /// The one canonical request for the entire reachable file graph.
    pub const fn request(&self) -> &SealUniversal {
        &self.1
    }
}

/// Typed refusal while resolving or materializing a Nomos file graph.
#[derive(Debug, thiserror::Error)]
pub enum NomosManifestLoadError {
    #[error("Nomos source path must be a non-empty lexical relative path: {0:?}")]
    InvalidSourcePath(PathBuf),
    #[error("Nomos source path must end in .nomos: {0:?}")]
    WrongSourceExtension(PathBuf),
    #[error("manifest associates the same Nomos source path more than once: {0:?}")]
    DuplicateSource(NomosSourcePath),
    #[error("manifest entry point is not associated with a namespace: {0:?}")]
    MissingEntryPoint(NomosSourcePath),
    #[error("Nomos source {0:?} depends on an unassociated source {1:?}")]
    MissingDependency(NomosSourcePath, NomosSourcePath),
    #[error("Nomos manifest dependency graph is cyclic: {0:?}")]
    DependencyCycle(Vec<NomosSourcePath>),
    #[error("Nomos source root cannot be resolved: {0:?}")]
    SourceRoot(PathBuf, #[source] io::Error),
    #[error("Nomos source cannot be resolved or read: {0:?}")]
    SourceRead(NomosSourcePath, #[source] io::Error),
    #[error("Nomos source resolves outside the configured root: {0:?}")]
    SourceEscapesRoot(NomosSourcePath),
    #[error("fixed vocabulary identity has conflicting spellings across files: {0:?}")]
    FixedVocabularyConflict(VocabularyEncodedId),
    #[error(transparent)]
    Load(#[from] NomosLoadError),
}

impl TextualNomos {
    /// Resolve, read, and structurally plan one manifest as a single mutation.
    ///
    /// The method returns no request unless the reachable dependency graph and
    /// every source read and per-file structural plan have all succeeded.
    pub fn plan_file_population<Resolver>(
        &self,
        source_root: &Path,
        manifest: &NomosFileManifest,
        resolver: &Resolver,
        operation_key: [u8; 32],
        expected: WritePrecondition,
    ) -> Result<PlannedNomosPopulation, NomosManifestLoadError>
    where
        Resolver: EncodedNameResolver<VocabularyRoot> + ?Sized,
    {
        let associations = manifest_associations(manifest)?;
        let mut source_order = resolve_reachable(&manifest.0, &associations)?;
        source_order.sort();

        let source_root = fs::canonicalize(source_root).map_err(|error| {
            NomosManifestLoadError::SourceRoot(source_root.to_path_buf(), error)
        })?;
        let mut sources = Vec::with_capacity(source_order.len());
        for source_path in source_order {
            let association = associations
                .get(&source_path)
                .expect("reachable paths came from the association map");
            let joined = source_root.join(source_path.as_path());
            let resolved = fs::canonicalize(&joined)
                .map_err(|error| NomosManifestLoadError::SourceRead(source_path.clone(), error))?;
            if !resolved.starts_with(&source_root) {
                return Err(NomosManifestLoadError::SourceEscapesRoot(source_path));
            }
            let source = fs::read_to_string(&resolved)
                .map_err(|error| NomosManifestLoadError::SourceRead(source_path.clone(), error))?;
            sources.push((association.1.clone(), source));
        }

        let mut plans = Vec::with_capacity(sources.len());
        for (module, source) in sources {
            plans.push(self.plan_load(
                &source,
                resolver,
                module,
                operation_key,
                expected.clone(),
            )?);
        }

        let mut modules = ModulePlan::default();
        let mut declaration_paths = BTreeSet::new();
        let mut reference_paths = Vec::new();
        let mut fixed_names = BTreeMap::new();
        for plan in &plans {
            modules.insert(plan.module.modules(), &plan.module_declarations);
            declaration_paths.extend(plan.declaration_paths.iter().cloned());
            reference_paths.extend(plan.reference_paths.iter().cloned());
            for (encoded_id, spelling) in &plan.fixed_names {
                match fixed_names.get(encoded_id) {
                    Some(existing) if existing != spelling => {
                        return Err(NomosManifestLoadError::FixedVocabularyConflict(
                            encoded_id.clone(),
                        ));
                    }
                    Some(_) => {}
                    None => {
                        fixed_names.insert(encoded_id.clone(), spelling.clone());
                    }
                }
            }
        }

        let declaration_paths = declaration_paths.into_iter().collect();
        reference_paths.sort();
        let references = reference_paths
            .iter()
            .map(PlannedNamePath::as_reference)
            .collect();
        let request = SealUniversal {
            operation_key: OperationKey::new(operation_key),
            expected,
            declarations: modules.into_declarations(),
            references,
        };
        Ok(PlannedNomosPopulation(
            plans,
            request,
            declaration_paths,
            reference_paths,
            fixed_names,
        ))
    }

    /// Verify the one receipt, decode every file, and expose source-neutral data.
    pub fn complete_file_population<Resolver>(
        &self,
        planned: &PlannedNomosPopulation,
        authoritative_reply: &AuthorityReply,
        resolver: &Resolver,
    ) -> Result<LoadedNomosPopulation, NomosManifestLoadError>
    where
        Resolver: EncodedNameResolver<VocabularyRoot> + ?Sized,
    {
        let (names, resolved_paths) = validate_authority_reply(
            &planned.1,
            &planned.2,
            &planned.3,
            &planned.4,
            authoritative_reply,
            resolver,
        )?;
        let mut declarations = Vec::new();
        for source in &planned.0 {
            let (_, source_declarations, _) =
                self.decode_planned_parts(source, &names, &resolved_paths)?;
            declarations.extend(source_declarations);
        }

        // [not-understood-by-psyche, Entry 7, NomosTrainAddendum-2026-07-30]
        // The whole file graph is one self-contained v1 package. This aggregate
        // validation permits in-package cross-file Invoke and refuses any
        // target absent from the package; it does not search another package.
        let transformers = AuthoredTransformerSet::try_new(declarations)
            .map_err(TextualNomosError::from)
            .map_err(NomosLoadError::from)?;
        Ok(LoadedNomosPopulation::from_typed(transformers, names))
    }
}

fn manifest_associations(
    manifest: &NomosFileManifest,
) -> Result<BTreeMap<NomosSourcePath, &NomosManifestFile>, NomosManifestLoadError> {
    let mut associations = BTreeMap::new();
    for association in &manifest.1 {
        if associations
            .insert(association.0.clone(), association)
            .is_some()
        {
            return Err(NomosManifestLoadError::DuplicateSource(
                association.0.clone(),
            ));
        }
    }
    if !associations.contains_key(&manifest.0) {
        return Err(NomosManifestLoadError::MissingEntryPoint(
            manifest.0.clone(),
        ));
    }
    Ok(associations)
}

fn resolve_reachable(
    entry_point: &NomosSourcePath,
    associations: &BTreeMap<NomosSourcePath, &NomosManifestFile>,
) -> Result<Vec<NomosSourcePath>, NomosManifestLoadError> {
    let mut states = BTreeMap::new();
    let mut stack = Vec::new();
    let mut resolved = Vec::new();
    visit(
        entry_point,
        associations,
        &mut states,
        &mut stack,
        &mut resolved,
    )?;
    Ok(resolved)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum VisitState {
    Visiting,
    Complete,
}

fn visit(
    source: &NomosSourcePath,
    associations: &BTreeMap<NomosSourcePath, &NomosManifestFile>,
    states: &mut BTreeMap<NomosSourcePath, VisitState>,
    stack: &mut Vec<NomosSourcePath>,
    resolved: &mut Vec<NomosSourcePath>,
) -> Result<(), NomosManifestLoadError> {
    match states.get(source) {
        Some(VisitState::Complete) => return Ok(()),
        Some(VisitState::Visiting) => {
            let start = stack.iter().position(|path| path == source).unwrap_or(0);
            let mut cycle = stack[start..].to_vec();
            cycle.push(source.clone());
            return Err(NomosManifestLoadError::DependencyCycle(cycle));
        }
        None => {}
    }

    let association = associations
        .get(source)
        .expect("entry and recursive dependencies are checked before visiting");
    states.insert(source.clone(), VisitState::Visiting);
    stack.push(source.clone());
    let mut dependencies = association.2.clone();
    dependencies.sort();
    for dependency in dependencies {
        if !associations.contains_key(&dependency) {
            return Err(NomosManifestLoadError::MissingDependency(
                source.clone(),
                dependency,
            ));
        }
        visit(&dependency, associations, states, stack, resolved)?;
    }
    stack.pop();
    states.insert(source.clone(), VisitState::Complete);
    resolved.push(source.clone());
    Ok(())
}

#[derive(Default)]
struct ModulePlan(Vec<DeclarationNode>, BTreeMap<Name, ModulePlan>);

impl ModulePlan {
    fn insert(&mut self, modules: &[Name], declarations: &[DeclarationNode]) {
        let Some((head, tail)) = modules.split_first() else {
            self.0.extend_from_slice(declarations);
            return;
        };
        self.1
            .entry(head.clone())
            .or_default()
            .insert(tail, declarations);
    }

    fn into_declarations(self) -> Vec<DeclarationNode> {
        let mut declarations = self.0;
        declarations.extend(
            self.1
                .into_iter()
                .map(|(spelling, child)| DeclarationNode::Module {
                    spelling,
                    declarations: child.into_declarations(),
                }),
        );
        declarations.sort();
        declarations
    }
}

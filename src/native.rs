//! Native authored-Nomos evaluation over complete encoded-ID chains.
//!
//! This module never reconstructs a legacy `MacroPackage`. It evaluates the
//! sealed authored declarations directly and retains every resolved
//! declaration-indexed term, including invocation and splice results.
//!
//! `GenerationClass` is deliberately absent from this evaluator and from
//! sealed authored content. The daemon carries the temporary six-class
//! selection as separate per-slot deployment metadata
//! `[to-be-reviewed-by-psyche]`; po2.8 owns moving that behavior into authored
//! Nomos and retiring the external selection.

use std::collections::{BTreeMap, BTreeSet};

use capsule_content_identity::{ArchiveError, PortableArchive};
use core_ethos::{
    WholeEthos, WholeEthosBody, WholeEthosEnumeration, WholeEthosFileKind, WholeEthosItem,
    WholeEthosNewtype, WholeEthosQuality, WholeEthosTypeApplication, WholeEthosTypeReference,
    WholeEthosVariant, WholeEthosVariantPayload,
};
use core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype, WholeLogosTupleFields,
    WholeLogosTypeApplication, WholeLogosTypeReference, WholeLogosVariant,
    WholeLogosVariantPayload, WholeLogosVisibility,
};
use protos::EncodedPopulation;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::{EncodedConstructorId, ScalarValue, StableRoleId};
use thiserror::Error;

use crate::template_language::RecursiveCallJudgment;
use crate::{
    AuthoredBindingIdentity, AuthoredTransformerDeclaration, AuthoredTransformerIdentity,
    AuthoredTransformerSet, InsertAt, MacroKind, MetaType, NameTransform, SectionDefault,
    TemplateFieldValue, TemplateFuture, TemplateRootOutputSelector, TemplateTerm, TemplateValue,
};

/// The future-free, declaration-indexed native Logos encoded form.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
pub struct NativeEvaluatedLogos(Vec<NativeEvaluatedValue>);

impl structural_codec::EncodedForm for NativeEvaluatedLogos {
    type VocabularyRoot = VocabularyRoot;
    type Language = protos::Logos;
}

impl NativeEvaluatedLogos {
    /// Every genuine evaluated Logos item value in source order.
    pub fn values(&self) -> &[NativeEvaluatedValue] {
        &self.0
    }
}

/// Cross-process native Logos population with semantic validation on restore.
///
/// The inner generic carrier is intentionally private so callers cannot mistake
/// its blanket rkyv round trip for the checked Nomos output boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeLogosPopulation<NameTree>(EncodedPopulation<NativeEvaluatedLogos, NameTree>);

impl<NameTree> NativeLogosPopulation<NameTree> {
    pub const fn encoded_form(&self) -> &NativeEvaluatedLogos {
        self.0.encoded_form()
    }

    pub const fn name_tree(&self) -> &NameTree {
        self.0.name_tree()
    }

    pub fn into_parts(self) -> (NativeEvaluatedLogos, NameTree) {
        self.0.into_parts()
    }

    pub fn to_archive_bytes(&self) -> Result<Vec<u8>, NativePopulationArchiveError>
    where
        EncodedPopulation<NativeEvaluatedLogos, NameTree>: NativePopulationArchiveCodec,
    {
        Ok(self.0.encode_population()?.as_ref().to_vec())
    }

    pub fn from_archive_bytes(
        bytes: &[u8],
        evaluator: &NativeAuthoredEvaluator<'_>,
    ) -> Result<Self, NativePopulationArchiveError>
    where
        NameTree: NativeLogosNameTree,
        EncodedPopulation<NativeEvaluatedLogos, NameTree>: NativePopulationArchiveCodec,
    {
        let population =
            EncodedPopulation::<NativeEvaluatedLogos, NameTree>::decode_population(bytes)?;
        evaluator.validate_output(population.encoded_form())?;
        population
            .name_tree()
            .validate_for(population.encoded_form())?;
        Ok(Self(population))
    }
}

#[derive(Debug, Error)]
pub enum NativePopulationArchiveError {
    #[error(transparent)]
    Archive(#[from] ArchiveError),
    #[error(transparent)]
    Semantic(#[from] NativeEvaluationError),
    #[error(transparent)]
    NameTree(#[from] NativeNameTreeRefusal),
}

#[doc(hidden)]
pub trait NativePopulationArchiveCodec: Sized {
    fn encode_population(&self) -> Result<rkyv::util::AlignedVec, ArchiveError>;
    fn decode_population(bytes: &[u8]) -> Result<Self, ArchiveError>;
}

impl<Value> NativePopulationArchiveCodec for Value
where
    Value: PortableArchive,
    Value::Archived: rkyv::Deserialize<Value, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>
        + for<'validation> rkyv::bytecheck::CheckBytes<
            rkyv::rancor::Strategy<
                rkyv::validation::Validator<
                    rkyv::validation::archive::ArchiveValidator<'validation>,
                    rkyv::validation::shared::SharedValidator,
                >,
                rkyv::rancor::Error,
            >,
        >,
{
    fn encode_population(&self) -> Result<rkyv::util::AlignedVec, ArchiveError> {
        self.to_archive_bytes()
    }

    fn decode_population(bytes: &[u8]) -> Result<Self, ArchiveError> {
        Self::from_archive_bytes(bytes)
    }
}

/// One transformed native population and its bounded whole-item projection.
///
/// The `EncodedPopulation` is the authoritative semantic output. `WholeLogos`
/// remains a derived compatibility projection while its vocabulary lacks
/// authored attributes and structs. Carrying both is a temporary po2.6 choice
/// `[to-be-reviewed-by-psyche]`; po2.8 owns retiring the bounded projection.
pub struct NativeTransformation<NameTree> {
    population: NativeLogosPopulation<NameTree>,
    whole_projection: WholeLogos,
}

impl<NameTree> NativeTransformation<NameTree> {
    pub const fn population(&self) -> &NativeLogosPopulation<NameTree> {
        &self.population
    }

    pub const fn whole_projection(&self) -> &WholeLogos {
        &self.whole_projection
    }

    pub fn into_parts(self) -> (NativeLogosPopulation<NameTree>, WholeLogos) {
        (self.population, self.whole_projection)
    }
}

/// A declaration-indexed Logos value after all authored futures have resolved.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct NativeEvaluatedValue {
    constructor: EncodedConstructorId<VocabularyRoot>,
    #[rkyv(omit_bounds)]
    fields: Vec<NativeEvaluatedField>,
}

impl NativeEvaluatedValue {
    pub const fn constructor(&self) -> &EncodedConstructorId<VocabularyRoot> {
        &self.constructor
    }

    pub fn fields(&self) -> &[NativeEvaluatedField] {
        &self.fields
    }
}

/// One stable Logos role and its resolved native value.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct NativeEvaluatedField {
    role: StableRoleId,
    #[rkyv(omit_bounds)]
    term: NativeEvaluatedTerm,
}

impl NativeEvaluatedField {
    pub const fn role(&self) -> StableRoleId {
        self.role
    }

    pub const fn term(&self) -> &NativeEvaluatedTerm {
        &self.term
    }
}

/// The future-free native landing algebra.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub enum NativeEvaluatedTerm {
    Declaration(VocabularyEncodedId),
    Reference(VocabularyEncodedId),
    Literal(VocabularyEncodedId),
    Scalar(ScalarValue),
    Nested(#[rkyv(omit_bounds)] Box<NativeEvaluatedValue>),
    Sequence(#[rkyv(omit_bounds)] Vec<NativeEvaluatedTerm>),
    TypeReference(WholeLogosTypeReference),
    Variant(WholeLogosVariant),
}

/// Component-owned complete NameTree behavior required by native evaluation.
///
/// The neutral `protos` carrier does not define either operation. An owning
/// component must authenticate its input tree, resolve name transformations,
/// and construct the complete Logos tree for the returned population.
/// The two-phase authenticated plan is a po2.6 boundary choice
/// `[to-be-reviewed-by-psyche]`.
pub trait NativeNameTreeBoundary {
    type LogosNameTree: NativeLogosNameTree;
    type Plan: NativeNameTreePlan<LogosNameTree = Self::LogosNameTree>;

    /// Authenticate the complete input tree and construct a mutation-free plan
    /// before evaluator entry.
    fn authenticated_plan(&self) -> Result<Self::Plan, NativeNameTreeRefusal>;
}

/// Pure derived-name and output-tree operations on one authenticated plan.
pub trait NativeNameTreePlan {
    type LogosNameTree: NativeLogosNameTree;

    fn realize_declaration(
        &self,
        source: &VocabularyEncodedId,
        transform: NameTransform,
    ) -> Result<VocabularyEncodedId, NativeNameTreeRefusal>;

    fn logos_name_tree(
        &self,
        evaluated: &NativeEvaluatedLogos,
    ) -> Result<Self::LogosNameTree, NativeNameTreeRefusal>;
}

/// Semantic validation owned by a complete Logos NameTree implementation.
pub trait NativeLogosNameTree {
    fn validate_for(&self, evaluated: &NativeEvaluatedLogos) -> Result<(), NativeNameTreeRefusal>;
}

/// A typed refusal at the component-owned NameTree boundary.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum NativeNameTreeRefusal {
    #[error("the complete NameTree cannot realize {transform:?} for {identity:?}")]
    CannotRealize {
        identity: VocabularyEncodedId,
        transform: NameTransform,
    },
    #[error("the complete Logos NameTree cannot be constructed from the evaluated population")]
    CannotConstructLogosTree,
}

/// One exact complete-chain reference rewrite in the active transformation
/// universe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeReferenceMapping {
    source: VocabularyEncodedId,
    target: VocabularyEncodedId,
}

impl NativeReferenceMapping {
    pub fn try_new(
        source: VocabularyEncodedId,
        target: VocabularyEncodedId,
    ) -> Result<Self, NativeEvaluationError> {
        if source.root_variant() != &VocabularyRoot::Universal {
            return Err(NativeEvaluationError::ReferenceMappingSource {
                found: *source.root_variant(),
            });
        }
        if target.root_variant() != &VocabularyRoot::Rust {
            return Err(NativeEvaluationError::ReferenceMappingTarget {
                found: *target.root_variant(),
            });
        }
        Ok(Self { source, target })
    }

    pub const fn source(&self) -> &VocabularyEncodedId {
        &self.source
    }

    pub const fn target(&self) -> &VocabularyEncodedId {
        &self.target
    }
}

/// Canonically ordered exact reference mappings for the complete runtime
/// universe. Unlisted complete identities are preserved exactly. The current
/// Universal-to-Rust target law is a po2.6 runtime-universe choice
/// `[to-be-reviewed-by-psyche]`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NativeReferenceUniverse(Vec<NativeReferenceMapping>);

impl NativeReferenceUniverse {
    pub fn try_new(
        mut mappings: Vec<NativeReferenceMapping>,
    ) -> Result<Self, NativeEvaluationError> {
        mappings.sort_by(|left, right| left.source.cmp(&right.source));
        for pair in mappings.windows(2) {
            if pair[0].source == pair[1].source {
                return Err(NativeEvaluationError::DuplicateReferenceMapping {
                    identity: pair[0].source.clone(),
                });
            }
        }
        Ok(Self(mappings))
    }

    pub const fn identity() -> Self {
        Self(Vec::new())
    }

    pub fn mappings(&self) -> &[NativeReferenceMapping] {
        &self.0
    }

    fn map(&self, source: &VocabularyEncodedId) -> VocabularyEncodedId {
        self.0
            .binary_search_by(|mapping| mapping.source.cmp(source))
            .map(|index| self.0[index].target.clone())
            .unwrap_or_else(|_| source.clone())
    }
}

/// Direct evaluator for one immutable authored transformer set.
pub struct NativeAuthoredEvaluator<'package> {
    package: &'package AuthoredTransformerSet,
    references: NativeReferenceUniverse,
}

// `[delegated-assent]` The designated Claude advisor accepted this whole-source
// preflight and evaluator design for po2.19. This records delegated review, not
// psyche intent.
struct RecursiveEnumeration<'ethos> {
    source: &'ethos WholeEthosEnumeration,
    children: Vec<VocabularyEncodedId>,
}

struct NativeRecursiveUniverse<'ethos> {
    enumerations: BTreeMap<VocabularyEncodedId, RecursiveEnumeration<'ethos>>,
}

enum NativeDocumentItem<'ethos> {
    Newtype(&'ethos WholeEthosNewtype),
    Enumeration(&'ethos WholeEthosEnumeration),
}

fn native_document_items(
    ethos: &WholeEthos,
) -> Result<Vec<NativeDocumentItem<'_>>, NativeEvaluationError> {
    let WholeEthosBody::Nexus(body) = ethos.body() else {
        return Err(NativeEvaluationError::UnsupportedDocumentBody {
            kind: ethos.body().kind(),
        });
    };
    if !body.traits().is_empty() {
        return Err(NativeEvaluationError::UnsupportedNexusTraits);
    }
    body.types()
        .iter()
        .map(|item| match item {
            WholeEthosItem::Newtype(newtype) => Ok(NativeDocumentItem::Newtype(newtype)),
            WholeEthosItem::Enumeration(enumeration) => Ok(NativeDocumentItem::Enumeration(enumeration)),
            WholeEthosItem::Struct(_) => Err(NativeEvaluationError::UnsupportedDocumentItem {
                item: "struct",
            }),
            WholeEthosItem::StreamInitiation(_) => Err(NativeEvaluationError::UnsupportedDocumentItem {
                item: "stream initiation",
            }),
        })
        .collect()
}

impl<'ethos> NativeRecursiveUniverse<'ethos> {
    fn preflight(
        package: &AuthoredTransformerSet,
        ethos: &'ethos WholeEthos,
    ) -> Result<Self, NativeEvaluationError> {
        if !package
            .declarations()
            .iter()
            .any(|declaration| matches!(declaration.kind(), MacroKind::Recursive { .. }))
        {
            return Ok(Self {
                enumerations: BTreeMap::new(),
            });
        }

        let mut identities = BTreeSet::new();
        let mut enumerations = BTreeMap::new();
        for item in native_document_items(ethos)? {
            let identity = match item {
                NativeDocumentItem::Newtype(newtype) => newtype.name(),
                NativeDocumentItem::Enumeration(enumeration) => enumeration.name(),
            };
            if !identities.insert(identity.clone()) {
                return Err(NativeEvaluationError::DuplicateRecursiveSourceIdentity {
                    identity: identity.clone(),
                });
            }
            if let NativeDocumentItem::Enumeration(enumeration) = item {
                enumerations.insert(
                    identity.clone(),
                    RecursiveEnumeration {
                        source: enumeration,
                        children: Vec::new(),
                    },
                );
            }
        }

        let mut indegrees = BTreeMap::<VocabularyEncodedId, usize>::new();
        let enumeration_identities = enumerations.keys().cloned().collect::<BTreeSet<_>>();
        for item in native_document_items(ethos)? {
            let NativeDocumentItem::Enumeration(enumeration) = item else {
                continue;
            };
            let mut children = Vec::new();
            for variant in enumeration.variants() {
                let WholeEthosVariantPayload::Tuple(fields) = variant.payload() else {
                    continue;
                };
                for field in fields.fields() {
                    match field {
                        WholeEthosTypeReference::Identity(identity)
                            if enumeration_identities.contains(identity) =>
                        {
                            children.push(identity.clone());
                            let indegree = indegrees.entry(identity.clone()).or_default();
                            *indegree += 1;
                            if *indegree > 1 {
                                return Err(NativeEvaluationError::RecursiveSourceSharing {
                                    enumeration: identity.clone(),
                                });
                            }
                        }
                        WholeEthosTypeReference::Identity(_) => {}
                        WholeEthosTypeReference::Parameter(parameter) => {
                            return Err(NativeEvaluationError::RecursiveSourceParameter {
                                enumeration: enumeration.name().clone(),
                                variant: variant.name().clone(),
                                parameter: parameter.name().clone(),
                            });
                        }
                        WholeEthosTypeReference::Application(_) => {
                            return Err(NativeEvaluationError::RecursiveSourceApplication {
                                enumeration: enumeration.name().clone(),
                                variant: variant.name().clone(),
                            });
                        }
                    }
                }
            }
            enumerations
                .get_mut(enumeration.name())
                .expect("the enumeration was indexed above")
                .children = children;
        }

        let universe = Self { enumerations };
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for identity in universe.enumerations.keys() {
            validate_recursive_source_acyclic(identity, &universe, &mut visiting, &mut visited)?;
        }
        Ok(universe)
    }

    fn enumeration(&self, identity: &VocabularyEncodedId) -> Option<&'ethos WholeEthosEnumeration> {
        self.enumerations.get(identity).map(|entry| entry.source)
    }
}

fn validate_recursive_source_acyclic(
    identity: &VocabularyEncodedId,
    universe: &NativeRecursiveUniverse<'_>,
    visiting: &mut BTreeSet<VocabularyEncodedId>,
    visited: &mut BTreeSet<VocabularyEncodedId>,
) -> Result<(), NativeEvaluationError> {
    if visited.contains(identity) {
        return Ok(());
    }
    if !visiting.insert(identity.clone()) {
        return Err(NativeEvaluationError::RecursiveSourceCycle {
            enumeration: identity.clone(),
        });
    }
    let entry = universe
        .enumerations
        .get(identity)
        .expect("recursive source edges only name indexed enumerations");
    for child in &entry.children {
        validate_recursive_source_acyclic(child, universe, visiting, visited)?;
    }
    visiting.remove(identity);
    visited.insert(identity.clone());
    Ok(())
}

#[derive(Clone, Copy)]
struct RecursiveFrame<'ethos> {
    enumeration: &'ethos WholeEthosEnumeration,
    variant: &'ethos WholeEthosVariant,
}

struct NativeEvaluationContext<'context, 'ethos, NameTree: ?Sized> {
    package: &'context AuthoredTransformerSet,
    names: &'context NameTree,
    references: &'context NativeReferenceUniverse,
    recursive_universe: &'context NativeRecursiveUniverse<'ethos>,
    recursive_subject: Option<&'ethos WholeEthosEnumeration>,
}

#[derive(Clone)]
struct EmittedSpan {
    binding: AuthoredBindingIdentity,
    start: usize,
    len: usize,
}

struct AppliedInsertion {
    binding: AuthoredBindingIdentity,
    boundary: u64,
}

impl<'package> NativeAuthoredEvaluator<'package> {
    /// Admit one immutable authored set before any transformation executes.
    ///
    /// Invocation cycles and text scalars are package refusals, not runtime
    /// partial-evaluation failures.
    pub fn try_new(
        package: &'package AuthoredTransformerSet,
        references: NativeReferenceUniverse,
    ) -> Result<Self, NativeEvaluationError> {
        validate_package(package)?;
        Ok(Self {
            package,
            references,
        })
    }

    /// Transform one complete encoded-form/NameTree population without a
    /// fixture or legacy execution carrier.
    pub fn transform<NameTree>(
        &self,
        population: &EncodedPopulation<WholeEthos, NameTree>,
    ) -> Result<NativeTransformation<NameTree::LogosNameTree>, NativeEvaluationError>
    where
        NameTree: NativeNameTreeBoundary,
    {
        let ethos = population.encoded_form();
        let recursive_universe = NativeRecursiveUniverse::preflight(self.package, ethos)?;
        let names = population.name_tree().authenticated_plan()?;
        let items = native_document_items(ethos)?;
        let mut values = Vec::with_capacity(items.len());
        let mut whole_items = Vec::with_capacity(items.len());
        for item in items {
            let evaluated = match item {
                NativeDocumentItem::Newtype(newtype) => {
                    self.evaluate_newtype(newtype, &names, &recursive_universe)?
                }
                NativeDocumentItem::Enumeration(enumeration) => {
                    self.evaluate_enumeration(enumeration, &names, &recursive_universe)?
                }
            };
            values.push(evaluated.0);
            whole_items.push(evaluated.1);
        }
        let evaluated = NativeEvaluatedLogos(values);
        let logos_names = names.logos_name_tree(&evaluated)?;
        self.validate_output(&evaluated)?;
        logos_names.validate_for(&evaluated)?;
        Ok(NativeTransformation {
            population: NativeLogosPopulation(EncodedPopulation::new(evaluated, logos_names)),
            whole_projection: WholeLogos::new(whole_items),
        })
    }

    fn evaluate_newtype<NameTree>(
        &self,
        newtype: &WholeEthosNewtype,
        names: &NameTree,
        recursive_universe: &NativeRecursiveUniverse<'_>,
    ) -> Result<(NativeEvaluatedValue, WholeLogosItem), NativeEvaluationError>
    where
        NameTree: NativeNameTreePlan,
    {
        let declaration = self.structural_transformer(SectionDefault::Newtype)?;
        let bindings = bind_newtype(declaration, newtype, &self.references)?;
        let result = self.evaluate_declaration(
            declaration,
            &bindings,
            names,
            recursive_universe,
            None,
            &mut Vec::new(),
        )?;
        let whole_projection = project_newtype(&result, declaration)?;
        Ok((result, WholeLogosItem::Newtype(whole_projection)))
    }

    fn evaluate_enumeration<NameTree>(
        &self,
        enumeration: &WholeEthosEnumeration,
        names: &NameTree,
        recursive_universe: &NativeRecursiveUniverse<'_>,
    ) -> Result<(NativeEvaluatedValue, WholeLogosItem), NativeEvaluationError>
    where
        NameTree: NativeNameTreePlan,
    {
        let declaration = self.structural_transformer(SectionDefault::Enumeration)?;
        let bindings = bind_enumeration(declaration, enumeration, &self.references)?;
        let result = self.evaluate_declaration(
            declaration,
            &bindings,
            names,
            recursive_universe,
            Some(enumeration),
            &mut Vec::new(),
        )?;
        let whole_projection = project_enumeration(&result, declaration)?;
        Ok((result, WholeLogosItem::Enumeration(whole_projection)))
    }

    fn structural_transformer(
        &self,
        section: SectionDefault,
    ) -> Result<&AuthoredTransformerDeclaration, NativeEvaluationError> {
        let mut matching = self
            .package
            .declarations()
            .iter()
            .filter(|declaration| declaration.kind() == MacroKind::Structural(section));
        let first = matching
            .next()
            .ok_or(NativeEvaluationError::MissingStructuralTransformer { section })?;
        if matching.next().is_some() {
            return Err(NativeEvaluationError::AmbiguousStructuralTransformer { section });
        }
        Ok(first)
    }

    fn validate_output(
        &self,
        evaluated: &NativeEvaluatedLogos,
    ) -> Result<(), NativeEvaluationError> {
        let schemas = native_schemas(self.package)?;
        for value in evaluated.values() {
            validate_native_value(value, &schemas)?;
            let matches_structural_root = self.package.declarations().iter().any(|declaration| {
                matches!(declaration.kind(), MacroKind::Structural(_))
                    && value.constructor == *declaration.result().constructor()
                    && value
                        .fields
                        .iter()
                        .map(NativeEvaluatedField::role)
                        .eq(declaration
                            .result()
                            .fields()
                            .iter()
                            .map(TemplateFieldValue::role))
            });
            if !matches_structural_root {
                return Err(NativeEvaluationError::RestoredTopLevelSchema {
                    constructor: value.constructor.clone(),
                });
            }
        }
        Ok(())
    }

    fn evaluate_declaration<NameTree>(
        &self,
        declaration: &AuthoredTransformerDeclaration,
        bindings: &[NativeBinding],
        names: &NameTree,
        recursive_universe: &NativeRecursiveUniverse<'_>,
        recursive_subject: Option<&WholeEthosEnumeration>,
        stack: &mut Vec<AuthoredTransformerIdentity>,
    ) -> Result<NativeEvaluatedValue, NativeEvaluationError>
    where
        NameTree: NativeNameTreePlan,
    {
        if stack.contains(declaration.name()) {
            return Err(NativeEvaluationError::InvocationCycle {
                transformer: declaration.name().clone(),
            });
        }
        stack.push(declaration.name().clone());
        let context = NativeEvaluationContext {
            package: self.package,
            names,
            references: &self.references,
            recursive_universe,
            recursive_subject,
        };
        let result = evaluate_value(&context, declaration.result(), bindings, None, stack);
        stack.pop();
        result
    }
}

#[derive(Clone)]
struct NativeBinding {
    identity: AuthoredBindingIdentity,
    value: NativeBindingValue,
}

#[derive(Clone)]
enum NativeBindingValue {
    Name(VocabularyEncodedId),
    Type(WholeLogosTypeReference),
    SourceVariants(Vec<WholeLogosVariant>),
    EvaluatedChildren(Vec<NativeEvaluatedTerm>),
}

fn bind_newtype(
    declaration: &AuthoredTransformerDeclaration,
    newtype: &WholeEthosNewtype,
    references: &NativeReferenceUniverse,
) -> Result<Vec<NativeBinding>, NativeEvaluationError> {
    declaration
        .input()
        .parameters()
        .iter()
        .map(|parameter| {
            let value = match parameter.meta() {
                MetaType::Name => NativeBindingValue::Name(newtype.name().clone()),
                MetaType::Type => NativeBindingValue::Type(lower_reference(
                    newtype.wrapped_field().reference(),
                    references,
                )?),
                found => {
                    return Err(NativeEvaluationError::InputMetaMismatch {
                        section: SectionDefault::Newtype,
                        found,
                    });
                }
            };
            Ok(NativeBinding {
                identity: parameter.binding().clone(),
                value,
            })
        })
        .collect()
}

fn bind_enumeration(
    declaration: &AuthoredTransformerDeclaration,
    enumeration: &WholeEthosEnumeration,
    references: &NativeReferenceUniverse,
) -> Result<Vec<NativeBinding>, NativeEvaluationError> {
    declaration
        .input()
        .parameters()
        .iter()
        .map(|parameter| {
            let value = match parameter.meta() {
                MetaType::Name => NativeBindingValue::Name(enumeration.name().clone()),
                MetaType::Variants => NativeBindingValue::SourceVariants(
                    enumeration
                        .variants()
                        .iter()
                        .map(|variant| lower_variant(variant, references))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                found => {
                    return Err(NativeEvaluationError::InputMetaMismatch {
                        section: SectionDefault::Enumeration,
                        found,
                    });
                }
            };
            Ok(NativeBinding {
                identity: parameter.binding().clone(),
                value,
            })
        })
        .collect()
}

fn evaluate_value<NameTree>(
    context: &NativeEvaluationContext<'_, '_, NameTree>,
    value: &TemplateValue<VocabularyRoot>,
    bindings: &[NativeBinding],
    recursive_frame: Option<RecursiveFrame<'_>>,
    stack: &mut Vec<AuthoredTransformerIdentity>,
) -> Result<NativeEvaluatedValue, NativeEvaluationError>
where
    NameTree: NativeNameTreePlan,
{
    let fields = value
        .fields()
        .iter()
        .map(|field| evaluate_field(context, field, bindings, recursive_frame, stack))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NativeEvaluatedValue {
        constructor: value.constructor().clone(),
        fields,
    })
}

fn evaluate_field<NameTree>(
    context: &NativeEvaluationContext<'_, '_, NameTree>,
    field: &TemplateFieldValue<VocabularyRoot>,
    bindings: &[NativeBinding],
    recursive_frame: Option<RecursiveFrame<'_>>,
    stack: &mut Vec<AuthoredTransformerIdentity>,
) -> Result<NativeEvaluatedField, NativeEvaluationError>
where
    NameTree: NativeNameTreePlan,
{
    Ok(NativeEvaluatedField {
        role: field.role(),
        term: evaluate_term(context, field.term(), bindings, recursive_frame, stack)?,
    })
}

fn evaluate_term<NameTree>(
    context: &NativeEvaluationContext<'_, '_, NameTree>,
    term: &TemplateTerm<VocabularyRoot>,
    bindings: &[NativeBinding],
    recursive_frame: Option<RecursiveFrame<'_>>,
    stack: &mut Vec<AuthoredTransformerIdentity>,
) -> Result<NativeEvaluatedTerm, NativeEvaluationError>
where
    NameTree: NativeNameTreePlan,
{
    match term {
        TemplateTerm::Declaration(value) => Ok(NativeEvaluatedTerm::Declaration(value.clone())),
        TemplateTerm::Reference(value) => Ok(NativeEvaluatedTerm::Reference(
            context.references.map(value),
        )),
        TemplateTerm::Literal(value) => Ok(NativeEvaluatedTerm::Literal(value.clone())),
        TemplateTerm::Scalar(value) => Ok(NativeEvaluatedTerm::Scalar(value.clone())),
        TemplateTerm::Nested(value) => Ok(NativeEvaluatedTerm::Nested(Box::new(evaluate_value(
            context,
            value,
            bindings,
            recursive_frame,
            stack,
        )?))),
        TemplateTerm::Sequence(values) => {
            let mut evaluated = Vec::new();
            let mut sequence_bindings = bindings.to_vec();
            let mut spans = Vec::<EmittedSpan>::new();
            let mut insertions = Vec::<AppliedInsertion>::new();
            let mut index = 0;
            while index < values.len() {
                match &values[index] {
                    TemplateTerm::Future(TemplateFuture::Invoke(transformer)) => {
                        let NativeEvaluatedTerm::Sequence(items) =
                            evaluate_invocation(context, transformer, stack)?
                        else {
                            return Err(NativeEvaluationError::InvocationOutputShape {
                                transformer: transformer.clone(),
                            });
                        };
                        evaluated.extend(items);
                    }
                    TemplateTerm::Future(TemplateFuture::Splice { binding }) => {
                        let emitted = match binding_value(&sequence_bindings, binding)? {
                            NativeBindingValue::SourceVariants(variants) => variants
                                .iter()
                                .cloned()
                                .map(NativeEvaluatedTerm::Variant)
                                .collect::<Vec<_>>(),
                            NativeBindingValue::EvaluatedChildren(children) => children.clone(),
                            NativeBindingValue::Name(_) | NativeBindingValue::Type(_) => {
                                return Err(NativeEvaluationError::InvalidSplice {
                                    binding: binding.clone(),
                                });
                            }
                        };
                        spans.push(EmittedSpan {
                            binding: binding.clone(),
                            start: evaluated.len(),
                            len: emitted.len(),
                        });
                        evaluated.extend(emitted);
                    }
                    TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
                        let frame = recursive_frame.ok_or_else(|| {
                            NativeEvaluationError::RecursiveInvokeOutsidePreflight {
                                transformer: payload.target().clone(),
                            }
                        })?;
                        let children = evaluate_recursive_children(context, payload, frame, stack)?;
                        replace_binding_value(
                            &mut sequence_bindings,
                            payload.children_binding(),
                            NativeBindingValue::EvaluatedChildren(children),
                        )?;
                    }
                    TemplateTerm::Future(TemplateFuture::InsertAt { payload }) => {
                        let inserted = values.get(index + 1).ok_or_else(|| {
                            NativeEvaluationError::InvalidInsertAt {
                                binding: payload.target().clone(),
                            }
                        })?;
                        let inserted = evaluate_term(
                            context,
                            inserted,
                            &sequence_bindings,
                            recursive_frame,
                            stack,
                        )?;
                        apply_insert_at(
                            &mut evaluated,
                            &mut spans,
                            &mut insertions,
                            payload,
                            inserted,
                        )?;
                        index += 1;
                    }
                    _ => evaluated.push(evaluate_term(
                        context,
                        &values[index],
                        &sequence_bindings,
                        recursive_frame,
                        stack,
                    )?),
                }
                index += 1;
            }
            Ok(NativeEvaluatedTerm::Sequence(evaluated))
        }
        TemplateTerm::Future(TemplateFuture::Realize { binding, transform }) => {
            let value = binding_value(bindings, binding)?;
            match value {
                NativeBindingValue::Name(source) => {
                    let result = if *transform == NameTransform::Identity {
                        source.clone()
                    } else {
                        context.names.realize_declaration(source, *transform)?
                    };
                    validate_realized_declaration(&result)?;
                    Ok(NativeEvaluatedTerm::Declaration(result))
                }
                NativeBindingValue::Type(reference) if *transform == NameTransform::Identity => {
                    Ok(NativeEvaluatedTerm::TypeReference(reference.clone()))
                }
                NativeBindingValue::Type(_)
                | NativeBindingValue::SourceVariants(_)
                | NativeBindingValue::EvaluatedChildren(_) => {
                    Err(NativeEvaluationError::InvalidRealize {
                        binding: binding.clone(),
                        transform: *transform,
                    })
                }
            }
        }
        TemplateTerm::Future(TemplateFuture::Splice { binding }) => {
            Err(NativeEvaluationError::InvalidSplice {
                binding: binding.clone(),
            })
        }
        TemplateTerm::Future(TemplateFuture::Invoke(transformer)) => {
            evaluate_invocation(context, transformer, stack)
        }
        TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
            Err(NativeEvaluationError::RecursiveInvokeOutsidePreflight {
                transformer: payload.target().clone(),
            })
        }
        TemplateTerm::Future(TemplateFuture::InsertAt { payload }) => {
            Err(NativeEvaluationError::InvalidInsertAt {
                binding: payload.target().clone(),
            })
        }
    }
}

fn replace_binding_value(
    bindings: &mut [NativeBinding],
    identity: &AuthoredBindingIdentity,
    value: NativeBindingValue,
) -> Result<(), NativeEvaluationError> {
    let binding = bindings
        .iter_mut()
        .find(|binding| &binding.identity == identity)
        .ok_or_else(|| NativeEvaluationError::MissingBinding {
            binding: identity.clone(),
        })?;
    binding.value = value;
    Ok(())
}

fn apply_insert_at(
    evaluated: &mut Vec<NativeEvaluatedTerm>,
    spans: &mut [EmittedSpan],
    insertions: &mut Vec<AppliedInsertion>,
    insertion: &InsertAt,
    value: NativeEvaluatedTerm,
) -> Result<(), NativeEvaluationError> {
    let span_index = spans
        .iter()
        .position(|span| &span.binding == insertion.target())
        .ok_or_else(|| NativeEvaluationError::InvalidInsertAt {
            binding: insertion.target().clone(),
        })?;
    let span = &spans[span_index];
    let boundary = usize::try_from(insertion.boundary()).map_err(|_| {
        NativeEvaluationError::InsertBoundaryOutOfRange {
            binding: insertion.target().clone(),
            boundary: insertion.boundary(),
            len: span.len,
        }
    })?;
    if boundary > span.len {
        return Err(NativeEvaluationError::InsertBoundaryOutOfRange {
            binding: insertion.target().clone(),
            boundary: insertion.boundary(),
            len: span.len,
        });
    }
    let prior_insertions = insertions
        .iter()
        .filter(|prior| {
            prior.binding == *insertion.target() && prior.boundary <= insertion.boundary()
        })
        .count();
    let insertion_index = span.start + boundary + prior_insertions;
    if insertion_index > evaluated.len() {
        return Err(NativeEvaluationError::InvalidInsertAt {
            binding: insertion.target().clone(),
        });
    }
    evaluated.insert(insertion_index, value);
    for later_span in spans.iter_mut().skip(span_index + 1) {
        later_span.start += 1;
    }
    insertions.push(AppliedInsertion {
        binding: insertion.target().clone(),
        boundary: insertion.boundary(),
    });
    Ok(())
}

fn evaluate_recursive_children<NameTree>(
    context: &NativeEvaluationContext<'_, '_, NameTree>,
    judgment: &RecursiveCallJudgment,
    frame: RecursiveFrame<'_>,
    stack: &mut Vec<AuthoredTransformerIdentity>,
) -> Result<Vec<NativeEvaluatedTerm>, NativeEvaluationError>
where
    NameTree: NativeNameTreePlan,
{
    let declaration = context
        .package
        .declarations()
        .iter()
        .find(|candidate| candidate.name() == judgment.target())
        .ok_or_else(|| NativeEvaluationError::MissingInvocation {
            transformer: judgment.target().clone(),
        })?;
    let mut children = Vec::new();
    let WholeEthosVariantPayload::Tuple(fields) = frame.variant.payload() else {
        return Ok(children);
    };
    for field in fields.fields() {
        let WholeEthosTypeReference::Identity(identity) = field else {
            return Err(NativeEvaluationError::RecursiveSourceApplication {
                enumeration: frame.enumeration.name().clone(),
                variant: frame.variant.name().clone(),
            });
        };
        let Some(enumeration) = context.recursive_universe.enumeration(identity) else {
            continue;
        };
        for variant in enumeration.variants() {
            let term = evaluate_recursive_variant(
                context,
                declaration,
                judgment,
                enumeration,
                variant,
                stack,
            )?;
            let NativeEvaluatedTerm::Sequence(items) = term else {
                return Err(NativeEvaluationError::RecursiveOutputShape {
                    transformer: judgment.target().clone(),
                });
            };
            children.extend(items);
        }
    }
    Ok(children)
}

fn evaluate_recursive_variant<NameTree>(
    context: &NativeEvaluationContext<'_, '_, NameTree>,
    declaration: &AuthoredTransformerDeclaration,
    judgment: &RecursiveCallJudgment,
    enumeration: &WholeEthosEnumeration,
    variant: &WholeEthosVariant,
    stack: &mut Vec<AuthoredTransformerIdentity>,
) -> Result<NativeEvaluatedTerm, NativeEvaluationError>
where
    NameTree: NativeNameTreePlan,
{
    let bindings = vec![
        NativeBinding {
            identity: judgment.subject_binding().clone(),
            value: NativeBindingValue::SourceVariants(vec![lower_variant(
                variant,
                context.references,
            )?]),
        },
        NativeBinding {
            identity: judgment.constructor_binding().clone(),
            value: NativeBindingValue::Name(variant.name().clone()),
        },
        NativeBinding {
            identity: judgment.children_binding().clone(),
            value: NativeBindingValue::EvaluatedChildren(Vec::new()),
        },
    ];
    let recursive_context = NativeEvaluationContext {
        package: context.package,
        names: context.names,
        references: context.references,
        recursive_universe: context.recursive_universe,
        recursive_subject: Some(enumeration),
    };
    let result = evaluate_value(
        &recursive_context,
        declaration.result(),
        &bindings,
        Some(RecursiveFrame {
            enumeration,
            variant,
        }),
        stack,
    )?;
    select_evaluated_output(declaration, result)
}

fn recursive_judgment(
    declaration: &AuthoredTransformerDeclaration,
) -> Result<&RecursiveCallJudgment, NativeEvaluationError> {
    fn collect<'value>(
        term: &'value TemplateTerm<VocabularyRoot>,
        judgments: &mut Vec<&'value RecursiveCallJudgment>,
    ) {
        match term {
            TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
                judgments.push(payload);
            }
            TemplateTerm::Nested(value) => {
                for field in value.fields() {
                    collect(field.term(), judgments);
                }
            }
            TemplateTerm::Sequence(items) => {
                for item in items {
                    collect(item, judgments);
                }
            }
            TemplateTerm::Declaration(_)
            | TemplateTerm::Reference(_)
            | TemplateTerm::Literal(_)
            | TemplateTerm::Scalar(_)
            | TemplateTerm::Future(_) => {}
        }
    }

    let mut judgments = Vec::new();
    for field in declaration.result().fields() {
        collect(field.term(), &mut judgments);
    }
    if let [judgment] = judgments.as_slice() {
        Ok(judgment)
    } else {
        Err(NativeEvaluationError::RecursiveJudgmentMissing {
            transformer: declaration.name().clone(),
        })
    }
}

fn evaluate_invocation<NameTree>(
    context: &NativeEvaluationContext<'_, '_, NameTree>,
    transformer: &AuthoredTransformerIdentity,
    stack: &mut Vec<AuthoredTransformerIdentity>,
) -> Result<NativeEvaluatedTerm, NativeEvaluationError>
where
    NameTree: NativeNameTreePlan,
{
    let declaration = context
        .package
        .declarations()
        .iter()
        .find(|candidate| candidate.name() == transformer)
        .ok_or_else(|| NativeEvaluationError::MissingInvocation {
            transformer: transformer.clone(),
        })?;
    if matches!(declaration.kind(), MacroKind::Recursive { .. }) {
        let subject = context.recursive_subject.ok_or_else(|| {
            NativeEvaluationError::RecursiveSubjectUnavailable {
                transformer: transformer.clone(),
            }
        })?;
        if stack.contains(transformer) {
            return Err(NativeEvaluationError::InvocationCycle {
                transformer: transformer.clone(),
            });
        }
        let judgment = recursive_judgment(declaration)?;
        stack.push(transformer.clone());
        let result = subject
            .variants()
            .iter()
            .try_fold(Vec::new(), |mut output, variant| {
                let term = evaluate_recursive_variant(
                    context,
                    declaration,
                    judgment,
                    subject,
                    variant,
                    stack,
                )?;
                let NativeEvaluatedTerm::Sequence(items) = term else {
                    return Err(NativeEvaluationError::RecursiveOutputShape {
                        transformer: transformer.clone(),
                    });
                };
                output.extend(items);
                Ok(output)
            });
        stack.pop();
        return result.map(NativeEvaluatedTerm::Sequence);
    }
    if !declaration.input().parameters().is_empty() {
        return Err(NativeEvaluationError::InvocationRequiresInput {
            transformer: transformer.clone(),
        });
    }
    if stack.contains(transformer) {
        return Err(NativeEvaluationError::InvocationCycle {
            transformer: transformer.clone(),
        });
    }
    stack.push(transformer.clone());
    let result = evaluate_value(context, declaration.result(), &[], None, stack);
    stack.pop();
    select_evaluated_output(declaration, result?)
}

fn select_evaluated_output(
    declaration: &AuthoredTransformerDeclaration,
    mut result: NativeEvaluatedValue,
) -> Result<NativeEvaluatedTerm, NativeEvaluationError> {
    match declaration.root_output().selector() {
        TemplateRootOutputSelector::WholeValue => Ok(NativeEvaluatedTerm::Nested(Box::new(result))),
        TemplateRootOutputSelector::Field(role) => {
            let index = result
                .fields
                .iter()
                .position(|field| field.role == role)
                .ok_or_else(|| NativeEvaluationError::InvocationOutputSelector {
                    transformer: declaration.name().clone(),
                    role,
                })?;
            Ok(result.fields.remove(index).term)
        }
    }
}

fn binding_value<'a>(
    bindings: &'a [NativeBinding],
    identity: &AuthoredBindingIdentity,
) -> Result<&'a NativeBindingValue, NativeEvaluationError> {
    bindings
        .iter()
        .find(|binding| &binding.identity == identity)
        .map(|binding| &binding.value)
        .ok_or_else(|| NativeEvaluationError::MissingBinding {
            binding: identity.clone(),
        })
}

fn validate_realized_declaration(
    identity: &VocabularyEncodedId,
) -> Result<(), NativeEvaluationError> {
    if identity.root_variant() != &VocabularyRoot::Universal {
        return Err(NativeEvaluationError::RealizedDeclarationRoot {
            found: *identity.root_variant(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeSchema {
    constructor: EncodedConstructorId<VocabularyRoot>,
    fields: Vec<(StableRoleId, NativeTermShape)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NativeTermShape {
    Declaration,
    Reference,
    Literal(VocabularyEncodedId),
    ScalarInteger(i64),
    ScalarFloat(u64),
    ScalarBoolean(bool),
    Nested(EncodedConstructorId<VocabularyRoot>),
    Sequence(Vec<NativeSequenceShape>),
    DynamicSequence(Vec<NativeTermShape>),
    TypeReference,
    Variant,
    Field,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum NativeSequenceShape {
    One(NativeTermShape),
    Repeated(NativeTermShape),
}

fn native_schemas(
    package: &AuthoredTransformerSet,
) -> Result<Vec<NativeSchema>, NativeEvaluationError> {
    let mut schemas = Vec::new();
    for declaration in package.declarations() {
        collect_native_schemas(package, declaration, declaration.result(), &mut schemas)?;
    }
    Ok(schemas)
}

fn collect_native_schemas(
    package: &AuthoredTransformerSet,
    declaration: &AuthoredTransformerDeclaration,
    value: &TemplateValue<VocabularyRoot>,
    schemas: &mut Vec<NativeSchema>,
) -> Result<(), NativeEvaluationError> {
    let fields = value
        .fields()
        .iter()
        .map(|field| {
            Ok((
                field.role(),
                native_term_shape(package, declaration, field.term())?,
            ))
        })
        .collect::<Result<Vec<_>, NativeEvaluationError>>()?;
    let schema = NativeSchema {
        constructor: value.constructor().clone(),
        fields,
    };
    if !schemas.contains(&schema) {
        schemas.push(schema);
    }
    for field in value.fields() {
        collect_native_term_schemas(package, declaration, field.term(), schemas)?;
    }
    Ok(())
}

fn collect_native_term_schemas(
    package: &AuthoredTransformerSet,
    declaration: &AuthoredTransformerDeclaration,
    term: &TemplateTerm<VocabularyRoot>,
    schemas: &mut Vec<NativeSchema>,
) -> Result<(), NativeEvaluationError> {
    match term {
        TemplateTerm::Nested(value) => {
            collect_native_schemas(package, declaration, value, schemas)?
        }
        TemplateTerm::Sequence(items) => {
            for item in items {
                collect_native_term_schemas(package, declaration, item, schemas)?;
            }
        }
        TemplateTerm::Declaration(_)
        | TemplateTerm::Reference(_)
        | TemplateTerm::Literal(_)
        | TemplateTerm::Scalar(_)
        | TemplateTerm::Future(_) => {}
    }
    Ok(())
}

fn native_term_shape(
    package: &AuthoredTransformerSet,
    declaration: &AuthoredTransformerDeclaration,
    term: &TemplateTerm<VocabularyRoot>,
) -> Result<NativeTermShape, NativeEvaluationError> {
    match term {
        TemplateTerm::Declaration(_) => Ok(NativeTermShape::Declaration),
        TemplateTerm::Reference(_) => Ok(NativeTermShape::Reference),
        TemplateTerm::Literal(identity) => Ok(NativeTermShape::Literal(identity.clone())),
        TemplateTerm::Scalar(ScalarValue::Integer(value)) => {
            Ok(NativeTermShape::ScalarInteger(*value))
        }
        TemplateTerm::Scalar(ScalarValue::Float(value)) => {
            Ok(NativeTermShape::ScalarFloat(value.to_bits()))
        }
        TemplateTerm::Scalar(ScalarValue::Boolean(value)) => {
            Ok(NativeTermShape::ScalarBoolean(*value))
        }
        TemplateTerm::Scalar(ScalarValue::Text(_)) => Err(NativeEvaluationError::TextScalar {
            transformer: declaration.name().clone(),
        }),
        TemplateTerm::Nested(value) => Ok(NativeTermShape::Nested(value.constructor().clone())),
        TemplateTerm::Sequence(items) => {
            if sequence_has_dynamic_output(package, items)? {
                return native_dynamic_sequence_shape(package, declaration, items);
            }
            let mut shapes = Vec::new();
            for item in items {
                match item {
                    TemplateTerm::Future(TemplateFuture::Invoke(target)) => {
                        let NativeTermShape::Sequence(invoked) =
                            native_invocation_shape(package, target)?
                        else {
                            return Err(NativeEvaluationError::InvocationOutputShape {
                                transformer: target.clone(),
                            });
                        };
                        shapes.extend(invoked);
                    }
                    TemplateTerm::Future(TemplateFuture::Splice { binding }) => {
                        let parameter = declaration
                            .input()
                            .parameters()
                            .iter()
                            .find(|parameter| parameter.binding() == binding)
                            .ok_or_else(|| NativeEvaluationError::MissingBinding {
                                binding: binding.clone(),
                            })?;
                        let shape = match parameter.meta() {
                            MetaType::Variants => NativeTermShape::Variant,
                            MetaType::Fields => NativeTermShape::Field,
                            MetaType::Name | MetaType::Type => {
                                return Err(NativeEvaluationError::InvalidSplice {
                                    binding: binding.clone(),
                                });
                            }
                        };
                        shapes.push(NativeSequenceShape::Repeated(shape));
                    }
                    TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
                        unreachable!(
                            "recursive marker was routed through dynamic sequence shape: {:?}",
                            payload.target()
                        );
                    }
                    TemplateTerm::Future(TemplateFuture::InsertAt { payload }) => {
                        unreachable!(
                            "InsertAt was routed through dynamic sequence shape: {:?}",
                            payload.target()
                        );
                    }
                    _ => shapes.push(NativeSequenceShape::One(native_term_shape(
                        package,
                        declaration,
                        item,
                    )?)),
                }
            }
            Ok(NativeTermShape::Sequence(shapes))
        }
        TemplateTerm::Future(TemplateFuture::Realize { binding, .. }) => {
            let parameter = declaration
                .input()
                .parameters()
                .iter()
                .find(|parameter| parameter.binding() == binding)
                .ok_or_else(|| NativeEvaluationError::MissingBinding {
                    binding: binding.clone(),
                })?;
            match parameter.meta() {
                MetaType::Name => Ok(NativeTermShape::Declaration),
                MetaType::Type => Ok(NativeTermShape::TypeReference),
                MetaType::Fields | MetaType::Variants => {
                    Err(NativeEvaluationError::InvalidRealize {
                        binding: binding.clone(),
                        transform: NameTransform::Identity,
                    })
                }
            }
        }
        TemplateTerm::Future(TemplateFuture::Invoke(target)) => {
            native_invocation_shape(package, target)
        }
        TemplateTerm::Future(TemplateFuture::Splice { binding }) => {
            Err(NativeEvaluationError::InvalidSplice {
                binding: binding.clone(),
            })
        }
        TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
            Err(NativeEvaluationError::RecursiveInvokeOutsidePreflight {
                transformer: payload.target().clone(),
            })
        }
        TemplateTerm::Future(TemplateFuture::InsertAt { payload }) => {
            Err(NativeEvaluationError::InvalidInsertAt {
                binding: payload.target().clone(),
            })
        }
    }
}

fn sequence_has_dynamic_output(
    package: &AuthoredTransformerSet,
    items: &[TemplateTerm<VocabularyRoot>],
) -> Result<bool, NativeEvaluationError> {
    for item in items {
        match item {
            TemplateTerm::Future(
                TemplateFuture::RecursiveInvoke { .. } | TemplateFuture::InsertAt { .. },
            ) => return Ok(true),
            TemplateTerm::Future(TemplateFuture::Invoke(target))
                if matches!(
                    native_invocation_shape(package, target)?,
                    NativeTermShape::DynamicSequence(_)
                ) =>
            {
                return Ok(true);
            }
            _ => {}
        }
    }
    Ok(false)
}

fn native_dynamic_sequence_shape(
    package: &AuthoredTransformerSet,
    declaration: &AuthoredTransformerDeclaration,
    items: &[TemplateTerm<VocabularyRoot>],
) -> Result<NativeTermShape, NativeEvaluationError> {
    let children_bindings = items
        .iter()
        .filter_map(|item| match item {
            TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
                Some(payload.children_binding().clone())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut admitted = Vec::new();
    let mut index = 0;
    while index < items.len() {
        match &items[index] {
            TemplateTerm::Future(TemplateFuture::RecursiveInvoke { .. }) => {}
            TemplateTerm::Future(TemplateFuture::InsertAt { payload }) => {
                let value =
                    items
                        .get(index + 1)
                        .ok_or_else(|| NativeEvaluationError::InvalidInsertAt {
                            binding: payload.target().clone(),
                        })?;
                admit_native_shape(
                    native_term_shape(package, declaration, value)?,
                    &mut admitted,
                );
                index += 1;
            }
            TemplateTerm::Future(TemplateFuture::Invoke(target)) => {
                admit_invoked_shape(native_invocation_shape(package, target)?, &mut admitted);
            }
            TemplateTerm::Future(TemplateFuture::Splice { binding })
                if children_bindings.contains(binding) => {}
            TemplateTerm::Future(TemplateFuture::Splice { binding }) => {
                let parameter = declaration
                    .input()
                    .parameters()
                    .iter()
                    .find(|parameter| parameter.binding() == binding)
                    .ok_or_else(|| NativeEvaluationError::MissingBinding {
                        binding: binding.clone(),
                    })?;
                let shape = match parameter.meta() {
                    MetaType::Variants => NativeTermShape::Variant,
                    MetaType::Fields => NativeTermShape::Field,
                    MetaType::Name | MetaType::Type => {
                        return Err(NativeEvaluationError::InvalidSplice {
                            binding: binding.clone(),
                        });
                    }
                };
                admit_native_shape(shape, &mut admitted);
            }
            item => admit_native_shape(
                native_term_shape(package, declaration, item)?,
                &mut admitted,
            ),
        }
        index += 1;
    }
    Ok(NativeTermShape::DynamicSequence(admitted))
}

fn admit_invoked_shape(shape: NativeTermShape, admitted: &mut Vec<NativeTermShape>) {
    match shape {
        NativeTermShape::Sequence(shapes) => {
            for shape in shapes {
                match shape {
                    NativeSequenceShape::One(shape) | NativeSequenceShape::Repeated(shape) => {
                        admit_native_shape(shape, admitted);
                    }
                }
            }
        }
        NativeTermShape::DynamicSequence(shapes) => {
            for shape in shapes {
                admit_native_shape(shape, admitted);
            }
        }
        shape => admit_native_shape(shape, admitted),
    }
}

fn admit_native_shape(shape: NativeTermShape, admitted: &mut Vec<NativeTermShape>) {
    if !admitted.contains(&shape) {
        admitted.push(shape);
    }
}

fn native_invocation_shape(
    package: &AuthoredTransformerSet,
    transformer: &AuthoredTransformerIdentity,
) -> Result<NativeTermShape, NativeEvaluationError> {
    let target = package
        .declarations()
        .iter()
        .find(|declaration| declaration.name() == transformer)
        .ok_or_else(|| NativeEvaluationError::MissingInvocation {
            transformer: transformer.clone(),
        })?;
    match target.root_output().selector() {
        TemplateRootOutputSelector::WholeValue => Ok(NativeTermShape::Nested(
            target.result().constructor().clone(),
        )),
        TemplateRootOutputSelector::Field(role) => {
            let field = target
                .result()
                .fields()
                .iter()
                .find(|field| field.role() == role)
                .ok_or_else(|| NativeEvaluationError::InvocationOutputSelector {
                    transformer: transformer.clone(),
                    role,
                })?;
            native_term_shape(package, target, field.term())
        }
    }
}

fn validate_native_value(
    value: &NativeEvaluatedValue,
    schemas: &[NativeSchema],
) -> Result<(), NativeEvaluationError> {
    validate_nonempty_identity(value.constructor.type_id().encoded_id())?;
    if value.constructor.type_id().encoded_id().root_variant() != &VocabularyRoot::Universal {
        return Err(NativeEvaluationError::RestoredConstructorRoot {
            found: *value.constructor.type_id().encoded_id().root_variant(),
        });
    }
    let roles = value
        .fields
        .iter()
        .map(NativeEvaluatedField::role)
        .collect::<Vec<_>>();
    if roles.iter().any(|role| role.value() == 0) || roles.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(NativeEvaluationError::RestoredValueSchema {
            constructor: value.constructor.clone(),
        });
    }
    for field in &value.fields {
        validate_native_term_roots(&field.term, schemas)?;
    }
    let valid = schemas.iter().any(|schema| {
        schema.constructor == value.constructor
            && schema.fields.len() == value.fields.len()
            && schema
                .fields
                .iter()
                .zip(&value.fields)
                .all(|((role, shape), field)| {
                    *role == field.role && native_term_matches(&field.term, shape, schemas)
                })
    });
    if !valid {
        return Err(NativeEvaluationError::RestoredValueSchema {
            constructor: value.constructor.clone(),
        });
    }
    Ok(())
}

fn validate_native_term_roots(
    term: &NativeEvaluatedTerm,
    schemas: &[NativeSchema],
) -> Result<(), NativeEvaluationError> {
    match term {
        NativeEvaluatedTerm::Declaration(identity) => validate_restored_declaration(identity),
        NativeEvaluatedTerm::Reference(identity) => validate_restored_reference(identity),
        NativeEvaluatedTerm::Literal(identity) => validate_restored_declaration(identity),
        NativeEvaluatedTerm::Scalar(ScalarValue::Text(_)) => {
            Err(NativeEvaluationError::RestoredTextScalar)
        }
        NativeEvaluatedTerm::Scalar(_) => Ok(()),
        NativeEvaluatedTerm::Nested(value) => validate_native_value(value, schemas),
        NativeEvaluatedTerm::Sequence(items) => {
            for item in items {
                validate_native_term_roots(item, schemas)?;
            }
            Ok(())
        }
        NativeEvaluatedTerm::TypeReference(reference) => {
            validate_restored_type_reference(reference)
        }
        NativeEvaluatedTerm::Variant(variant) => {
            validate_restored_declaration(variant.name())?;
            if let WholeLogosVariantPayload::Tuple(fields) = variant.payload() {
                for field in fields.fields() {
                    validate_restored_type_reference(field)?;
                }
            }
            Ok(())
        }
    }
}

fn native_term_matches(
    term: &NativeEvaluatedTerm,
    shape: &NativeTermShape,
    schemas: &[NativeSchema],
) -> bool {
    match (term, shape) {
        (NativeEvaluatedTerm::Declaration(_), NativeTermShape::Declaration)
        | (NativeEvaluatedTerm::Reference(_), NativeTermShape::Reference)
        | (NativeEvaluatedTerm::TypeReference(_), NativeTermShape::TypeReference)
        | (NativeEvaluatedTerm::Variant(_), NativeTermShape::Variant) => true,
        (NativeEvaluatedTerm::Literal(found), NativeTermShape::Literal(expected)) => {
            found == expected
        }
        (
            NativeEvaluatedTerm::Scalar(ScalarValue::Integer(found)),
            NativeTermShape::ScalarInteger(expected),
        ) if found == expected => true,
        (
            NativeEvaluatedTerm::Scalar(ScalarValue::Float(found)),
            NativeTermShape::ScalarFloat(expected),
        ) if found.to_bits() == *expected => true,
        (
            NativeEvaluatedTerm::Scalar(ScalarValue::Boolean(found)),
            NativeTermShape::ScalarBoolean(expected),
        ) if found == expected => true,
        (NativeEvaluatedTerm::Nested(value), NativeTermShape::Nested(constructor)) => {
            &value.constructor == constructor && validate_native_value(value, schemas).is_ok()
        }
        (NativeEvaluatedTerm::Sequence(items), NativeTermShape::Sequence(expected)) => {
            native_sequence_matches(items, expected, schemas)
        }
        (NativeEvaluatedTerm::Sequence(items), NativeTermShape::DynamicSequence(admitted)) => {
            items.iter().all(|item| {
                admitted
                    .iter()
                    .any(|shape| native_term_matches(item, shape, schemas))
            })
        }
        _ => false,
    }
}

fn native_sequence_matches(
    items: &[NativeEvaluatedTerm],
    shapes: &[NativeSequenceShape],
    schemas: &[NativeSchema],
) -> bool {
    let Some((shape, remaining)) = shapes.split_first() else {
        return items.is_empty();
    };
    match shape {
        NativeSequenceShape::One(expected) => items.first().is_some_and(|item| {
            native_term_matches(item, expected, schemas)
                && native_sequence_matches(&items[1..], remaining, schemas)
        }),
        NativeSequenceShape::Repeated(expected) => (0..=items.len()).any(|count| {
            items[..count]
                .iter()
                .all(|item| native_term_matches(item, expected, schemas))
                && native_sequence_matches(&items[count..], remaining, schemas)
        }),
    }
}

fn validate_restored_declaration(
    identity: &VocabularyEncodedId,
) -> Result<(), NativeEvaluationError> {
    validate_nonempty_identity(identity)?;
    if identity.root_variant() != &VocabularyRoot::Universal {
        return Err(NativeEvaluationError::RestoredDeclarationRoot {
            found: *identity.root_variant(),
        });
    }
    Ok(())
}

fn validate_restored_reference(
    identity: &VocabularyEncodedId,
) -> Result<(), NativeEvaluationError> {
    validate_nonempty_identity(identity)?;
    if !matches!(
        identity.root_variant(),
        VocabularyRoot::Universal | VocabularyRoot::Rust
    ) {
        return Err(NativeEvaluationError::RestoredReferenceRoot {
            found: *identity.root_variant(),
        });
    }
    Ok(())
}

fn validate_nonempty_identity(identity: &VocabularyEncodedId) -> Result<(), NativeEvaluationError> {
    validate_nonempty_chain(identity.chain())
}

fn validate_nonempty_chain<T>(chain: &[T]) -> Result<(), NativeEvaluationError> {
    if chain.is_empty() {
        return Err(NativeEvaluationError::RestoredEmptyEncodedId);
    }
    Ok(())
}

fn validate_restored_type_reference(
    reference: &WholeLogosTypeReference,
) -> Result<(), NativeEvaluationError> {
    match reference {
        WholeLogosTypeReference::Identity(identity) => validate_restored_reference(identity),
        WholeLogosTypeReference::Parameter(name) => validate_restored_reference(name),
        WholeLogosTypeReference::Application(application) => {
            validate_restored_reference(application.head())?;
            for argument in application.arguments() {
                validate_restored_type_reference(argument)?;
            }
            Ok(())
        }
    }
}

fn validate_package(package: &AuthoredTransformerSet) -> Result<(), NativeEvaluationError> {
    for declaration in package.declarations() {
        validate_authored_identity(
            declaration.name().encoded_id(),
            NativeIdentityPosition::Transformer,
        )?;
        for parameter in declaration.input().parameters() {
            validate_authored_identity(
                parameter.binding().encoded_id(),
                NativeIdentityPosition::Binding,
            )?;
        }
        validate_authored_value_ids(declaration.result())?;
        validate_root_selector(declaration)?;
        reject_text_scalars(declaration.name(), declaration.result())?;
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for declaration in package.declarations() {
        validate_invocation_acyclic(package, declaration.name(), &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn validate_authored_value_ids(
    value: &TemplateValue<VocabularyRoot>,
) -> Result<(), NativeEvaluationError> {
    validate_authored_identity(
        value.constructor().type_id().encoded_id(),
        NativeIdentityPosition::Constructor,
    )?;
    for field in value.fields() {
        validate_authored_term_ids(field.term())?;
    }
    Ok(())
}

fn validate_authored_term_ids(
    term: &TemplateTerm<VocabularyRoot>,
) -> Result<(), NativeEvaluationError> {
    match term {
        TemplateTerm::Declaration(identity) => {
            validate_authored_identity(identity, NativeIdentityPosition::Declaration)
        }
        TemplateTerm::Reference(identity) => {
            validate_authored_identity(identity, NativeIdentityPosition::Reference)
        }
        TemplateTerm::Literal(identity) => {
            validate_authored_identity(identity, NativeIdentityPosition::Literal)
        }
        TemplateTerm::Nested(value) => validate_authored_value_ids(value),
        TemplateTerm::Sequence(items) => {
            for item in items {
                validate_authored_term_ids(item)?;
            }
            Ok(())
        }
        TemplateTerm::Future(TemplateFuture::Realize { binding, .. })
        | TemplateTerm::Future(TemplateFuture::Splice { binding }) => {
            validate_authored_identity(binding.encoded_id(), NativeIdentityPosition::Binding)
        }
        TemplateTerm::Future(TemplateFuture::Invoke(transformer)) => validate_authored_identity(
            transformer.encoded_id(),
            NativeIdentityPosition::Transformer,
        ),
        TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
            validate_authored_identity(
                payload.target().encoded_id(),
                NativeIdentityPosition::Transformer,
            )?;
            validate_authored_identity(
                payload.subject_binding().encoded_id(),
                NativeIdentityPosition::Binding,
            )?;
            validate_authored_identity(
                payload.constructor_binding().encoded_id(),
                NativeIdentityPosition::Binding,
            )?;
            validate_authored_identity(
                payload.children_binding().encoded_id(),
                NativeIdentityPosition::Binding,
            )
        }
        TemplateTerm::Future(TemplateFuture::InsertAt { payload }) => validate_authored_identity(
            payload.target().encoded_id(),
            NativeIdentityPosition::Binding,
        ),
        TemplateTerm::Scalar(_) => Ok(()),
    }
}

fn validate_authored_identity(
    identity: &VocabularyEncodedId,
    position: NativeIdentityPosition,
) -> Result<(), NativeEvaluationError> {
    if identity.chain().is_empty() {
        return Err(NativeEvaluationError::AuthoredEmptyEncodedId { position });
    }
    if identity.root_variant() != &VocabularyRoot::Universal {
        return Err(NativeEvaluationError::AuthoredIdentityRoot {
            position,
            found: *identity.root_variant(),
        });
    }
    Ok(())
}

fn validate_root_selector(
    declaration: &AuthoredTransformerDeclaration,
) -> Result<(), NativeEvaluationError> {
    if let TemplateRootOutputSelector::Field(role) = declaration.root_output().selector() {
        let found = declaration
            .result()
            .fields()
            .iter()
            .filter(|field| field.role() == role)
            .count();
        if found != 1 {
            return Err(NativeEvaluationError::InvalidRootOutputSelector {
                transformer: declaration.name().clone(),
                role,
                found,
            });
        }
    }
    Ok(())
}

fn reject_text_scalars(
    transformer: &AuthoredTransformerIdentity,
    value: &TemplateValue<VocabularyRoot>,
) -> Result<(), NativeEvaluationError> {
    for field in value.fields() {
        reject_text_term(transformer, field.term())?;
    }
    Ok(())
}

fn reject_text_term(
    transformer: &AuthoredTransformerIdentity,
    term: &TemplateTerm<VocabularyRoot>,
) -> Result<(), NativeEvaluationError> {
    match term {
        TemplateTerm::Scalar(ScalarValue::Text(_)) => Err(NativeEvaluationError::TextScalar {
            transformer: transformer.clone(),
        }),
        TemplateTerm::Nested(value) => reject_text_scalars(transformer, value),
        TemplateTerm::Sequence(items) => {
            for item in items {
                reject_text_term(transformer, item)?;
            }
            Ok(())
        }
        TemplateTerm::Declaration(_)
        | TemplateTerm::Reference(_)
        | TemplateTerm::Literal(_)
        | TemplateTerm::Scalar(_)
        | TemplateTerm::Future(_) => Ok(()),
    }
}

fn validate_invocation_acyclic(
    package: &AuthoredTransformerSet,
    transformer: &AuthoredTransformerIdentity,
    visiting: &mut BTreeSet<AuthoredTransformerIdentity>,
    visited: &mut BTreeSet<AuthoredTransformerIdentity>,
) -> Result<(), NativeEvaluationError> {
    if visited.contains(transformer) {
        return Ok(());
    }
    if !visiting.insert(transformer.clone()) {
        return Err(NativeEvaluationError::InvocationCycle {
            transformer: transformer.clone(),
        });
    }
    let declaration = package
        .declarations()
        .iter()
        .find(|candidate| candidate.name() == transformer)
        .ok_or_else(|| NativeEvaluationError::MissingInvocation {
            transformer: transformer.clone(),
        })?;
    let mut targets = Vec::new();
    collect_invocations(declaration.result(), &mut targets);
    for target in targets {
        let target_declaration = package
            .declarations()
            .iter()
            .find(|candidate| candidate.name() == target)
            .ok_or_else(|| NativeEvaluationError::MissingInvocation {
                transformer: target.clone(),
            })?;
        if !matches!(target_declaration.kind(), MacroKind::Recursive { .. })
            && !target_declaration.input().parameters().is_empty()
        {
            return Err(NativeEvaluationError::InvocationRequiresInput {
                transformer: target.clone(),
            });
        }
        validate_invocation_acyclic(package, target, visiting, visited)?;
    }
    visiting.remove(transformer);
    visited.insert(transformer.clone());
    Ok(())
}

fn collect_invocations<'value>(
    value: &'value TemplateValue<VocabularyRoot>,
    targets: &mut Vec<&'value AuthoredTransformerIdentity>,
) {
    for field in value.fields() {
        collect_invocations_from_term(field.term(), targets);
    }
}

fn collect_invocations_from_term<'value>(
    term: &'value TemplateTerm<VocabularyRoot>,
    targets: &mut Vec<&'value AuthoredTransformerIdentity>,
) {
    match term {
        TemplateTerm::Future(TemplateFuture::Invoke(target)) => targets.push(target),
        TemplateTerm::Nested(value) => collect_invocations(value, targets),
        TemplateTerm::Sequence(items) => {
            for item in items {
                collect_invocations_from_term(item, targets);
            }
        }
        TemplateTerm::Declaration(_)
        | TemplateTerm::Reference(_)
        | TemplateTerm::Literal(_)
        | TemplateTerm::Scalar(_)
        | TemplateTerm::Future(_) => {}
    }
}

fn project_newtype(
    value: &NativeEvaluatedValue,
    declaration: &AuthoredTransformerDeclaration,
) -> Result<WholeLogosNewtype, NativeEvaluationError> {
    validate_projection_schema(value, declaration)?;
    let [visibility, _attributes, name, wrapped_visibility, wrapped] = value.fields.as_slice()
    else {
        return Err(NativeEvaluationError::ProjectionShape {
            constructor: value.constructor.clone(),
        });
    };
    let authored = declaration.result().fields();
    Ok(WholeLogosNewtype::new(
        project_visibility(&visibility.term, authored[0].term())?,
        project_declaration(&name.term)?,
        project_visibility(&wrapped_visibility.term, authored[3].term())?,
        project_type(&wrapped.term)?,
    ))
}

fn project_enumeration(
    value: &NativeEvaluatedValue,
    declaration: &AuthoredTransformerDeclaration,
) -> Result<WholeLogosEnumeration, NativeEvaluationError> {
    validate_projection_schema(value, declaration)?;
    let [visibility, _attributes, name, _generics, variants] = value.fields.as_slice() else {
        return Err(NativeEvaluationError::ProjectionShape {
            constructor: value.constructor.clone(),
        });
    };
    let authored = declaration.result().fields();
    Ok(WholeLogosEnumeration::new(
        project_visibility(&visibility.term, authored[0].term())?,
        project_declaration(&name.term)?,
        project_variants(&variants.term)?,
    ))
}

fn validate_projection_schema(
    value: &NativeEvaluatedValue,
    declaration: &AuthoredTransformerDeclaration,
) -> Result<(), NativeEvaluationError> {
    let authored = declaration.result();
    if value.constructor != *authored.constructor()
        || value.fields.len() != authored.fields().len()
        || value
            .fields
            .iter()
            .zip(authored.fields())
            .any(|(evaluated, expected)| evaluated.role != expected.role())
    {
        return Err(NativeEvaluationError::ProjectionSchema {
            transformer: declaration.name().clone(),
        });
    }
    Ok(())
}

fn project_visibility(
    term: &NativeEvaluatedTerm,
    authored: &TemplateTerm<VocabularyRoot>,
) -> Result<WholeLogosVisibility, NativeEvaluationError> {
    let NativeEvaluatedTerm::Nested(value) = term else {
        return Err(NativeEvaluationError::ProjectionTerm);
    };
    let TemplateTerm::Nested(authored) = authored else {
        return Err(NativeEvaluationError::ProjectionTerm);
    };
    if value.constructor != *authored.constructor()
        || value
            .fields
            .iter()
            .map(NativeEvaluatedField::role)
            .ne(authored.fields().iter().map(TemplateFieldValue::role))
    {
        return Err(NativeEvaluationError::ProjectionTerm);
    }
    match value.constructor.local() {
        1 => Ok(WholeLogosVisibility::Public),
        2 => Ok(WholeLogosVisibility::Private),
        _ => Err(NativeEvaluationError::ProjectionTerm),
    }
}

fn project_declaration(
    term: &NativeEvaluatedTerm,
) -> Result<VocabularyEncodedId, NativeEvaluationError> {
    match term {
        NativeEvaluatedTerm::Declaration(value) => Ok(value.clone()),
        _ => Err(NativeEvaluationError::ProjectionTerm),
    }
}

fn project_type(
    term: &NativeEvaluatedTerm,
) -> Result<WholeLogosTypeReference, NativeEvaluationError> {
    match term {
        NativeEvaluatedTerm::TypeReference(value) => Ok(value.clone()),
        _ => Err(NativeEvaluationError::ProjectionTerm),
    }
}

fn project_variants(
    term: &NativeEvaluatedTerm,
) -> Result<Vec<WholeLogosVariant>, NativeEvaluationError> {
    let NativeEvaluatedTerm::Sequence(items) = term else {
        return Err(NativeEvaluationError::ProjectionTerm);
    };
    let mut projected = Vec::new();
    for item in items {
        match item {
            NativeEvaluatedTerm::Variant(variant) => projected.push(variant.clone()),
            _ => return Err(NativeEvaluationError::ProjectionTerm),
        }
    }
    Ok(projected)
}

fn lower_reference(
    reference: &WholeEthosTypeReference,
    references: &NativeReferenceUniverse,
) -> Result<WholeLogosTypeReference, NativeEvaluationError> {
    Ok(match reference {
        WholeEthosTypeReference::Identity(identity) => {
            WholeLogosTypeReference::Identity(references.map(identity))
        }
        WholeEthosTypeReference::Parameter(parameter) => {
            WholeLogosTypeReference::Parameter(parameter.name().clone())
        }
        WholeEthosTypeReference::Application(application) => {
            WholeLogosTypeReference::Application(lower_application(application, references)?)
        }
    })
}

fn lower_application(
    application: &WholeEthosTypeApplication,
    references: &NativeReferenceUniverse,
) -> Result<WholeLogosTypeApplication, NativeEvaluationError> {
    let WholeEthosQuality::Shape(head) = application.head() else {
        return Err(NativeEvaluationError::TypeApplicationHeadMustBeShape {
            quality: application.head().identity().clone(),
        });
    };
    WholeLogosTypeApplication::new(
        references.map(head),
        application
            .arguments()
            .iter()
            .map(|argument| lower_reference(argument, references))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .map_err(|_| NativeEvaluationError::EmptyTypeApplicationArguments {
        head: head.clone(),
    })
}

fn lower_variant(
    variant: &WholeEthosVariant,
    references: &NativeReferenceUniverse,
) -> Result<WholeLogosVariant, NativeEvaluationError> {
    let payload = match variant.payload() {
        WholeEthosVariantPayload::Unit => WholeLogosVariantPayload::Unit,
        WholeEthosVariantPayload::Tuple(fields) => {
            let fields = WholeLogosTupleFields::new(
                fields
                    .fields()
                    .iter()
                    .map(|field| lower_reference(field, references))
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(|_| NativeEvaluationError::EmptyVariantTuple {
                variant: variant.name().clone(),
            })?;
            WholeLogosVariantPayload::Tuple(fields)
        }
    };
    Ok(WholeLogosVariant::new(variant.name().clone(), payload))
}

/// A typed refusal from native authored evaluation or the bounded po2.6
/// whole-item projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeIdentityPosition {
    Transformer,
    Binding,
    Constructor,
    Declaration,
    Reference,
    Literal,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum NativeEvaluationError {
    #[error("native authored evaluation only admits a Nexus document, found {kind:?}")]
    UnsupportedDocumentBody { kind: WholeEthosFileKind },
    #[error("native authored evaluation does not lower {item} declarations")]
    UnsupportedDocumentItem { item: &'static str },
    #[error("native authored evaluation does not lower Nexus trait declarations")]
    UnsupportedNexusTraits,
    #[error("recursive source {enumeration:?} variant {variant:?} uses parameter {parameter:?}")]
    RecursiveSourceParameter {
        enumeration: VocabularyEncodedId,
        variant: VocabularyEncodedId,
        parameter: VocabularyEncodedId,
    },
    #[error("type application head must be a Shape, found trait quality {quality:?}")]
    TypeApplicationHeadMustBeShape { quality: VocabularyEncodedId },
    #[error("type application {head:?} has no arguments")]
    EmptyTypeApplicationArguments { head: VocabularyEncodedId },
    #[error("variant {variant:?} has no tuple fields")]
    EmptyVariantTuple { variant: VocabularyEncodedId },
    #[error(transparent)]
    NameTree(#[from] NativeNameTreeRefusal),
    #[error("reference mapping source belongs to {found:?}; expected Universal")]
    ReferenceMappingSource { found: VocabularyRoot },
    #[error("reference mapping target belongs to {found:?}; expected Rust")]
    ReferenceMappingTarget { found: VocabularyRoot },
    #[error("realized declaration belongs to {found:?}; expected Universal")]
    RealizedDeclarationRoot { found: VocabularyRoot },
    #[error("restored native Logos contains a text scalar")]
    RestoredTextScalar,
    #[error("restored native Logos contains an empty encoded-ID chain")]
    RestoredEmptyEncodedId,
    #[error("restored native constructor belongs to {found:?}; expected Universal")]
    RestoredConstructorRoot { found: VocabularyRoot },
    #[error("restored native declaration/literal belongs to {found:?}; expected Universal")]
    RestoredDeclarationRoot { found: VocabularyRoot },
    #[error("restored native reference belongs to unsupported root {found:?}")]
    RestoredReferenceRoot { found: VocabularyRoot },
    #[error("restored native value {constructor:?} has an unknown constructor/role schema")]
    RestoredValueSchema {
        constructor: EncodedConstructorId<VocabularyRoot>,
    },
    #[error("restored top-level value {constructor:?} is not an admitted structural output")]
    RestoredTopLevelSchema {
        constructor: EncodedConstructorId<VocabularyRoot>,
    },
    #[error("reference mapping repeats complete source {identity:?}")]
    DuplicateReferenceMapping { identity: VocabularyEncodedId },
    #[error("authored {position:?} contains an empty encoded-ID chain")]
    AuthoredEmptyEncodedId { position: NativeIdentityPosition },
    #[error("authored {position:?} belongs to {found:?}; expected Universal")]
    AuthoredIdentityRoot {
        position: NativeIdentityPosition,
        found: VocabularyRoot,
    },
    #[error("transformer {transformer:?} contains a forbidden text scalar")]
    TextScalar {
        transformer: AuthoredTransformerIdentity,
    },
    #[error(
        "transformer {transformer:?} root selector role {role:?} resolves {found} fields; expected one"
    )]
    InvalidRootOutputSelector {
        transformer: AuthoredTransformerIdentity,
        role: StableRoleId,
        found: usize,
    },
    #[error("no structural transformer is declared for {section:?}")]
    MissingStructuralTransformer { section: SectionDefault },
    #[error("more than one structural transformer is declared for {section:?}")]
    AmbiguousStructuralTransformer { section: SectionDefault },
    #[error("the {section:?} transformer cannot bind input meta-type {found:?}")]
    InputMetaMismatch {
        section: SectionDefault,
        found: MetaType,
    },
    #[error("authored binding {binding:?} has no native input value")]
    MissingBinding { binding: AuthoredBindingIdentity },
    #[error("Realize on {binding:?} cannot apply {transform:?} to that input")]
    InvalidRealize {
        binding: AuthoredBindingIdentity,
        transform: NameTransform,
    },
    #[error("Splice on {binding:?} does not reference a native sequence")]
    InvalidSplice { binding: AuthoredBindingIdentity },
    #[error("InsertAt cannot assemble target binding {binding:?}")]
    InvalidInsertAt { binding: AuthoredBindingIdentity },
    #[error(
        "InsertAt boundary {boundary} exceeds original span length {len} for binding {binding:?}"
    )]
    InsertBoundaryOutOfRange {
        binding: AuthoredBindingIdentity,
        boundary: u64,
        len: usize,
    },
    #[error("recursive source repeats complete declaration identity {identity:?}")]
    DuplicateRecursiveSourceIdentity { identity: VocabularyEncodedId },
    #[error(
        "recursive source enumeration {enumeration:?} variant {variant:?} contains an application edge"
    )]
    RecursiveSourceApplication {
        enumeration: VocabularyEncodedId,
        variant: VocabularyEncodedId,
    },
    #[error("recursive source enumeration {enumeration:?} has more than one incoming edge")]
    RecursiveSourceSharing { enumeration: VocabularyEncodedId },
    #[error("recursive source cycle reaches enumeration {enumeration:?}")]
    RecursiveSourceCycle { enumeration: VocabularyEncodedId },
    #[error("recursive transformer {transformer:?} has no unique compiled self judgment")]
    RecursiveJudgmentMissing {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("recursive transformer {transformer:?} has no ambient enumeration subject")]
    RecursiveSubjectUnavailable {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("recursive transformer {transformer:?} did not produce a sequence")]
    RecursiveOutputShape {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("recursive Invoke for {transformer:?} escaped whole-universe preflight")]
    RecursiveInvokeOutsidePreflight {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("invoked transformer {transformer:?} disappeared from the package")]
    MissingInvocation {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("invoked transformer {transformer:?} requires unavailable input")]
    InvocationRequiresInput {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("invocation cycle reaches {transformer:?}")]
    InvocationCycle {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("invoked transformer {transformer:?} did not produce the required sequence fragment")]
    InvocationOutputShape {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("invoked transformer {transformer:?} omitted sealed output role {role:?}")]
    InvocationOutputSelector {
        transformer: AuthoredTransformerIdentity,
        role: StableRoleId,
    },
    #[error("resolved value does not match transformer {transformer:?}'s sealed projection schema")]
    ProjectionSchema {
        transformer: AuthoredTransformerIdentity,
    },
    #[error("resolved constructor {constructor:?} cannot inhabit the bounded whole projection")]
    ProjectionShape {
        constructor: EncodedConstructorId<VocabularyRoot>,
    },
    #[error("resolved term cannot inhabit the bounded whole projection")]
    ProjectionTerm,
}

#[cfg(test)]
mod tests {
    use encoded_name_table::LocalEncodedId;

    use super::*;

    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
    struct TestLogosNameTree(usize);

    impl NativeLogosNameTree for TestLogosNameTree {
        fn validate_for(
            &self,
            evaluated: &NativeEvaluatedLogos,
        ) -> Result<(), NativeNameTreeRefusal> {
            if self.0 == evaluated.values().len() {
                Ok(())
            } else {
                Err(NativeNameTreeRefusal::CannotConstructLogosTree)
            }
        }
    }

    fn identity(chain: &[u16]) -> VocabularyEncodedId {
        VocabularyEncodedId::new(
            VocabularyRoot::Universal,
            chain.iter().copied().map(LocalEncodedId::new).collect(),
        )
        .expect("non-empty test identity")
    }

    fn binding(chain: &[u16]) -> AuthoredBindingIdentity {
        AuthoredBindingIdentity::try_new(identity(chain)).expect("Universal binding identity")
    }

    fn integer(value: i64) -> NativeEvaluatedTerm {
        NativeEvaluatedTerm::Scalar(ScalarValue::Integer(value))
    }

    fn integer_values(values: &[NativeEvaluatedTerm]) -> Vec<i64> {
        values
            .iter()
            .map(|value| {
                let NativeEvaluatedTerm::Scalar(ScalarValue::Integer(value)) = value else {
                    panic!("insertion witness remains integral")
                };
                *value
            })
            .collect()
    }

    #[test]
    fn insert_at_preserves_template_order_for_equal_and_mixed_boundaries() {
        let target = binding(&[90, 1]);
        let mut equal_values = vec![integer(10), integer(20), integer(30)];
        let mut equal_spans = vec![EmittedSpan {
            binding: target.clone(),
            start: 0,
            len: 3,
        }];
        let mut equal_insertions = Vec::new();
        apply_insert_at(
            &mut equal_values,
            &mut equal_spans,
            &mut equal_insertions,
            &InsertAt::new(target.clone(), 1),
            integer(100),
        )
        .expect("first equal-boundary insertion");
        apply_insert_at(
            &mut equal_values,
            &mut equal_spans,
            &mut equal_insertions,
            &InsertAt::new(target.clone(), 1),
            integer(101),
        )
        .expect("second equal-boundary insertion");
        assert_eq!(integer_values(&equal_values), vec![10, 100, 101, 20, 30]);

        let mut mixed_values = vec![integer(10), integer(20), integer(30)];
        let mut mixed_spans = vec![EmittedSpan {
            binding: target.clone(),
            start: 0,
            len: 3,
        }];
        let mut mixed_insertions = Vec::new();
        apply_insert_at(
            &mut mixed_values,
            &mut mixed_spans,
            &mut mixed_insertions,
            &InsertAt::new(target.clone(), 2),
            integer(200),
        )
        .expect("later-boundary insertion");
        apply_insert_at(
            &mut mixed_values,
            &mut mixed_spans,
            &mut mixed_insertions,
            &InsertAt::new(target.clone(), 0),
            integer(100),
        )
        .expect("earlier-boundary insertion");
        apply_insert_at(
            &mut mixed_values,
            &mut mixed_spans,
            &mut mixed_insertions,
            &InsertAt::new(target, 2),
            integer(201),
        )
        .expect("second later-boundary insertion");
        assert_eq!(
            integer_values(&mixed_values),
            vec![100, 10, 20, 200, 201, 30]
        );

        let out_of_range = binding(&[90, 2]);
        let mut bounded_values = vec![integer(10), integer(20), integer(30)];
        let mut bounded_spans = vec![EmittedSpan {
            binding: out_of_range.clone(),
            start: 0,
            len: 3,
        }];
        let error = apply_insert_at(
            &mut bounded_values,
            &mut bounded_spans,
            &mut Vec::new(),
            &InsertAt::new(out_of_range.clone(), 4),
            integer(40),
        )
        .expect_err("boundary beyond the original span refuses");
        assert!(matches!(
            error,
            NativeEvaluationError::InsertBoundaryOutOfRange {
                binding,
                boundary: 4,
                len: 3
            } if binding == out_of_range
        ));
    }

    #[test]
    fn insert_at_shifts_only_later_emitted_spans() {
        let first = binding(&[91, 1]);
        let second = binding(&[91, 2]);
        let mut values = vec![integer(10), integer(20), integer(30), integer(40)];
        let mut spans = vec![
            EmittedSpan {
                binding: first.clone(),
                start: 0,
                len: 2,
            },
            EmittedSpan {
                binding: second.clone(),
                start: 2,
                len: 2,
            },
        ];
        let mut insertions = Vec::new();
        apply_insert_at(
            &mut values,
            &mut spans,
            &mut insertions,
            &InsertAt::new(first.clone(), 2),
            integer(25),
        )
        .expect("append to first span");
        apply_insert_at(
            &mut values,
            &mut spans,
            &mut insertions,
            &InsertAt::new(second.clone(), 0),
            integer(29),
        )
        .expect("prepend to shifted second span");
        apply_insert_at(
            &mut values,
            &mut spans,
            &mut insertions,
            &InsertAt::new(first, 0),
            integer(5),
        )
        .expect("prepend to first span");
        apply_insert_at(
            &mut values,
            &mut spans,
            &mut insertions,
            &InsertAt::new(second, 1),
            integer(35),
        )
        .expect("insert within twice-shifted second span");
        assert_eq!(integer_values(&values), vec![5, 10, 20, 25, 29, 30, 35, 40]);
    }

    #[test]
    fn nested_sequence_text_scalar_is_refused_by_evaluator_admission() {
        let package = crate::authored::native_text_admission_package_for_test();
        assert!(matches!(
            NativeAuthoredEvaluator::try_new(&package, NativeReferenceUniverse::identity()),
            Err(NativeEvaluationError::TextScalar { .. })
        ));
    }

    #[test]
    fn reference_universe_refuses_wrong_roots_and_duplicate_complete_sources() {
        let universal = identity(&[4, 1]);
        let rust = VocabularyEncodedId::new(
            VocabularyRoot::Rust,
            vec![LocalEncodedId::new(8), LocalEncodedId::new(1)],
        )
        .expect("Rust identity");
        assert!(matches!(
            NativeReferenceMapping::try_new(rust.clone(), rust.clone()),
            Err(NativeEvaluationError::ReferenceMappingSource {
                found: VocabularyRoot::Rust
            })
        ));
        assert!(matches!(
            NativeReferenceMapping::try_new(universal.clone(), universal.clone()),
            Err(NativeEvaluationError::ReferenceMappingTarget {
                found: VocabularyRoot::Universal
            })
        ));
        let mapping = NativeReferenceMapping::try_new(universal.clone(), rust.clone())
            .expect("valid mapping");
        assert!(matches!(
            NativeReferenceUniverse::try_new(vec![mapping.clone(), mapping]),
            Err(NativeEvaluationError::DuplicateReferenceMapping { identity })
                if identity == universal
        ));
        let homonym = identity(&[5, 1]);
        let exact = NativeReferenceUniverse::try_new(vec![
            NativeReferenceMapping::try_new(universal.clone(), rust.clone())
                .expect("valid exact mapping"),
        ])
        .expect("one mapping");
        assert_eq!(exact.map(&universal), rust);
        assert_eq!(exact.map(&homonym), homonym);
        assert_eq!(
            validate_realized_declaration(&encoded_rust(&[7, 1])),
            Err(NativeEvaluationError::RealizedDeclarationRoot {
                found: VocabularyRoot::Rust
            })
        );
    }

    #[test]
    fn checked_restore_refuses_forged_semantic_archives() {
        let package = crate::authored::native_restore_validation_package_for_test();
        let evaluator =
            NativeAuthoredEvaluator::try_new(&package, NativeReferenceUniverse::identity())
                .expect("valid restore-validation package");
        let declaration = &package.declarations()[0];
        let fields = declaration.result().fields();
        let valid = NativeEvaluatedValue {
            constructor: declaration.result().constructor().clone(),
            fields: vec![
                NativeEvaluatedField {
                    role: fields[0].role(),
                    term: NativeEvaluatedTerm::Declaration(identity(&[31, 11])),
                },
                NativeEvaluatedField {
                    role: fields[1].role(),
                    term: NativeEvaluatedTerm::Scalar(ScalarValue::Boolean(true)),
                },
            ],
        };

        let restore = |value| {
            let population = NativeLogosPopulation(EncodedPopulation::new(
                NativeEvaluatedLogos(vec![value]),
                TestLogosNameTree(1),
            ));
            let bytes = population
                .to_archive_bytes()
                .expect("forged test population serializes");
            NativeLogosPopulation::<TestLogosNameTree>::from_archive_bytes(&bytes, &evaluator)
        };

        assert!(restore(valid.clone()).is_ok());

        let mut wrong_term = valid.clone();
        wrong_term.fields[0].term = NativeEvaluatedTerm::Scalar(ScalarValue::Boolean(true));
        assert!(matches!(
            restore(wrong_term),
            Err(NativePopulationArchiveError::Semantic(
                NativeEvaluationError::RestoredValueSchema { .. }
            ))
        ));

        let mut wrong_root = valid.clone();
        wrong_root.fields[0].term = NativeEvaluatedTerm::Declaration(encoded_rust(&[31, 11]));
        assert!(matches!(
            restore(wrong_root),
            Err(NativePopulationArchiveError::Semantic(
                NativeEvaluationError::RestoredDeclarationRoot {
                    found: VocabularyRoot::Rust
                }
            ))
        ));

        let mut wrong_scalar = valid;
        wrong_scalar.fields[1].term = NativeEvaluatedTerm::Scalar(ScalarValue::Boolean(false));
        assert!(matches!(
            restore(wrong_scalar),
            Err(NativePopulationArchiveError::Semantic(
                NativeEvaluationError::RestoredValueSchema { .. }
            ))
        ));
    }

    #[test]
    fn restored_encoded_ids_require_nonempty_complete_chains() {
        assert_eq!(
            validate_nonempty_chain::<LocalEncodedId>(&[]),
            Err(NativeEvaluationError::RestoredEmptyEncodedId)
        );
        assert_eq!(validate_nonempty_chain(&[LocalEncodedId::new(1)]), Ok(()));
    }

    fn encoded_rust(chain: &[u16]) -> VocabularyEncodedId {
        VocabularyEncodedId::new(
            VocabularyRoot::Rust,
            chain.iter().copied().map(LocalEncodedId::new).collect(),
        )
        .expect("non-empty Rust identity")
    }
}

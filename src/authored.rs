//! Phase-stable authored Nomos declarations.
//!
//! Transformer and binding identities are complete translator-issued chains.
//! The result is the one declaration-indexed [`crate::TemplateValue`] substrate;
//! no Logos type has an authored Rust twin in this module.

use std::collections::BTreeSet;

use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::LandingShape;
use thiserror::Error;

use crate::template_language::{RecursiveCallJudgment, RecursiveEdgePolicy};
use crate::{
    MacroKind, MetaType, TemplateFuture, TemplateFutureKind, TemplateFutureOutput,
    TemplateFutureRequirement, TemplateLanguage, TemplateRootOutput, TemplateValue,
    TemplateValueError,
};

/// A durable identity position whose production root is fixed by the authored
/// Nomos contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoredIdentityPosition {
    Transformer,
    Binding,
}

/// A typed refusal while constructing the authored-stage value.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum AuthoredNomosError {
    #[error("{position:?} identity belongs to {found:?}; expected Universal")]
    WrongRoot {
        position: AuthoredIdentityPosition,
        found: VocabularyRoot,
    },

    #[error("input signature declares binding {binding:?} more than once")]
    DuplicateBinding { binding: VocabularyEncodedId },

    #[error("typed Logos skeleton references undeclared binding {binding:?}")]
    UndeclaredBinding { binding: VocabularyEncodedId },

    #[error("authored package declares transformer {transformer:?} more than once")]
    DuplicateTransformer { transformer: VocabularyEncodedId },

    #[error("authored package invokes undeclared transformer {transformer:?}")]
    UndeclaredTransformer { transformer: VocabularyEncodedId },

    #[error("recursive declaration {transformer:?} does not contain an authored self-Invoke")]
    MissingRecursiveInvoke { transformer: VocabularyEncodedId },

    #[error("recursive declaration {transformer:?} retains an uncompiled self-Invoke")]
    UncompiledRecursiveInvoke { transformer: VocabularyEncodedId },

    #[error(
        "transformer {transformer:?} contains a recursive judgment outside a recursive declaration"
    )]
    UnexpectedRecursiveInvoke { transformer: VocabularyEncodedId },

    #[error("transformer {transformer:?} contains an invalid recursive termination judgment")]
    InvalidRecursiveJudgment { transformer: VocabularyEncodedId },

    #[error("recursive section {section:?} is not admitted by the finite owned-tree evaluator")]
    UnsupportedRecursiveSection { section: crate::SectionDefault },

    #[error("transformer {transformer:?} persisted invocation requirements drift from its result")]
    InvocationRequirementDrift { transformer: VocabularyEncodedId },

    #[error("transformer {transformer:?} persisted InsertAt requirements drift from its result")]
    InsertionRequirementDrift { transformer: VocabularyEncodedId },

    #[error(
        "recursive declaration {transformer:?} has no distinct Variants/Name/Variants binding contract"
    )]
    InvalidRecursiveBindings { transformer: VocabularyEncodedId },

    #[error("recursive declaration {transformer:?} does not produce a zero-admitting sequence")]
    InvalidRecursiveLanding { transformer: VocabularyEncodedId },

    #[error(
        "recursive declaration {transformer:?} has an invalid self-marker/children-Splice order"
    )]
    InvalidRecursiveSequence { transformer: VocabularyEncodedId },

    #[error("InsertAt target binding {binding:?} does not produce a sequence")]
    InsertionTargetNotSequence { binding: VocabularyEncodedId },

    #[error(
        "{future:?} produces {found:?}, which cannot inhabit the computed landing {expected:?}"
    )]
    FutureOutputMismatch {
        future: TemplateFutureKind,
        expected: TemplateFutureOutput<VocabularyRoot>,
        found: TemplateFutureOutput<VocabularyRoot>,
    },

    #[error("computed template root {encoded_type:?} has no coherent output")]
    InvalidTemplateRoot {
        encoded_type: structural_codec::EncodedTypeId<VocabularyRoot>,
    },

    #[error(transparent)]
    Template(Box<TemplateValueError<VocabularyRoot>>),
}

impl From<TemplateValueError<VocabularyRoot>> for AuthoredNomosError {
    fn from(error: TemplateValueError<VocabularyRoot>) -> Self {
        Self::Template(Box::new(error))
    }
}

/// The translator-issued durable identity of one transformer.
#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct AuthoredTransformerIdentity(VocabularyEncodedId);

impl AuthoredTransformerIdentity {
    pub fn try_new(encoded_id: VocabularyEncodedId) -> Result<Self, AuthoredNomosError> {
        require_universal(&encoded_id, AuthoredIdentityPosition::Transformer)?;
        Ok(Self(encoded_id))
    }

    pub fn encoded_id(&self) -> &VocabularyEncodedId {
        &self.0
    }

    pub fn into_encoded_id(self) -> VocabularyEncodedId {
        self.0
    }
}

/// The translator-issued durable identity of one input binding.
#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct AuthoredBindingIdentity(VocabularyEncodedId);

impl AuthoredBindingIdentity {
    pub fn try_new(encoded_id: VocabularyEncodedId) -> Result<Self, AuthoredNomosError> {
        require_universal(&encoded_id, AuthoredIdentityPosition::Binding)?;
        Ok(Self(encoded_id))
    }

    pub fn encoded_id(&self) -> &VocabularyEncodedId {
        &self.0
    }

    pub fn into_encoded_id(self) -> VocabularyEncodedId {
        self.0
    }
}

fn require_universal(
    encoded_id: &VocabularyEncodedId,
    position: AuthoredIdentityPosition,
) -> Result<(), AuthoredNomosError> {
    let found = *encoded_id.root_variant();
    if found != VocabularyRoot::Universal {
        return Err(AuthoredNomosError::WrongRoot { position, found });
    }
    Ok(())
}

/// One authored input position: durable binding identity, its Ethos meta-type,
/// and the encoded landing output available to Realize or Splice.
///
/// The output is supplied by the resolved input type declaration. No Logos type
/// or binding spelling is switched on here.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredInputParameter {
    binding: AuthoredBindingIdentity,
    meta: MetaType,
    output: TemplateFutureOutput<VocabularyRoot>,
}

impl AuthoredInputParameter {
    pub fn new(
        binding: AuthoredBindingIdentity,
        meta: MetaType,
        output: TemplateFutureOutput<VocabularyRoot>,
    ) -> Self {
        Self {
            binding,
            meta,
            output,
        }
    }

    pub fn binding(&self) -> &AuthoredBindingIdentity {
        &self.binding
    }

    pub const fn meta(&self) -> MetaType {
        self.meta
    }

    pub const fn output(&self) -> &TemplateFutureOutput<VocabularyRoot> {
        &self.output
    }
}

/// The ordered, positional input signature of an authored transformer.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredInputSignature(Vec<AuthoredInputParameter>);

impl AuthoredInputSignature {
    pub fn try_new(parameters: Vec<AuthoredInputParameter>) -> Result<Self, AuthoredNomosError> {
        let mut bindings = BTreeSet::new();
        for parameter in &parameters {
            let binding = parameter.binding().encoded_id();
            if !bindings.insert(binding.clone()) {
                return Err(AuthoredNomosError::DuplicateBinding {
                    binding: binding.clone(),
                });
            }
        }
        Ok(Self(parameters))
    }

    pub fn unit() -> Self {
        Self(Vec::new())
    }

    pub fn parameters(&self) -> &[AuthoredInputParameter] {
        &self.0
    }

    fn parameter(&self, binding: &AuthoredBindingIdentity) -> Option<&AuthoredInputParameter> {
        self.0
            .iter()
            .find(|parameter| parameter.binding() == binding)
    }
}

/// One authored transformer declaration in durable pre-seal form.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
pub struct AuthoredTransformerDeclaration {
    name: AuthoredTransformerIdentity,
    kind: MacroKind,
    input: AuthoredInputSignature,
    result: TemplateValue<VocabularyRoot>,
    output: TemplateRootOutput<VocabularyRoot>,
    future_requirements: Vec<TemplateFutureRequirement<VocabularyRoot>>,
}

impl AuthoredTransformerDeclaration {
    /// Validate the generic result against its computed Template(X) declaration.
    ///
    /// Binding futures are resolved immediately from the input signature.
    /// Invoke and InsertAt requirements remain durable so complete-set restore
    /// can revalidate their lexical positions, targets and output landings.
    pub fn try_new(
        name: AuthoredTransformerIdentity,
        kind: MacroKind,
        input: AuthoredInputSignature,
        result: TemplateValue<VocabularyRoot>,
        language: &TemplateLanguage<VocabularyRoot>,
    ) -> Result<Self, AuthoredNomosError> {
        let requirements = language.analyze_value(&result)?;
        let output = language.root_output_contract().map_err(|_| {
            AuthoredNomosError::InvalidTemplateRoot {
                encoded_type: language.root().clone(),
            }
        })?;
        let result = match kind {
            MacroKind::Recursive { .. } => {
                compile_recursive_result(result, &name, &input, &requirements, output.output())?
            }
            MacroKind::Named | MacroKind::Structural(_) => {
                if contains_internal_recursive_call(&result) {
                    return Err(AuthoredNomosError::UnexpectedRecursiveInvoke {
                        transformer: name.encoded_id().clone(),
                    });
                }
                result
            }
        };
        let persisted_requirements = resolve_binding_outputs(&input, &requirements)?;
        let declaration = Self {
            name,
            kind,
            input,
            result,
            output,
            future_requirements: persisted_requirements,
        };
        validate_requirement_mirror(&declaration)?;
        validate_insertions(&declaration)?;
        validate_recursive_sequence(&declaration)?;
        Ok(declaration)
    }

    pub fn name(&self) -> &AuthoredTransformerIdentity {
        &self.name
    }

    pub const fn kind(&self) -> MacroKind {
        self.kind
    }

    pub fn input(&self) -> &AuthoredInputSignature {
        &self.input
    }

    pub fn result(&self) -> &TemplateValue<VocabularyRoot> {
        &self.result
    }

    pub const fn output(&self) -> &TemplateFutureOutput<VocabularyRoot> {
        self.output.output()
    }

    pub const fn root_output(&self) -> &TemplateRootOutput<VocabularyRoot> {
        &self.output
    }

    pub fn future_requirements(&self) -> &[TemplateFutureRequirement<VocabularyRoot>] {
        &self.future_requirements
    }
}

// `[delegated-assent]` The designated Claude advisor accepted these compile,
// mirror, and recursive-sequence equations for po2.19. This is delegated review
// evidence, not a claim of psyche intent.
struct RecursiveBindings {
    subject: AuthoredBindingIdentity,
    constructor: AuthoredBindingIdentity,
    children: AuthoredBindingIdentity,
}

fn recursive_bindings(
    transformer: &AuthoredTransformerIdentity,
    input: &AuthoredInputSignature,
    landing: &TemplateFutureOutput<VocabularyRoot>,
) -> Result<RecursiveBindings, AuthoredNomosError> {
    if !matches!(landing.landing(), LandingShape::Sequence { minimum: 0, .. }) {
        return Err(AuthoredNomosError::InvalidRecursiveLanding {
            transformer: transformer.encoded_id().clone(),
        });
    }
    let mut variants = input
        .parameters()
        .iter()
        .filter(|parameter| parameter.meta() == MetaType::Variants);
    let subject = variants.next();
    let children = variants.next();
    let constructor = input
        .parameters()
        .iter()
        .filter(|parameter| parameter.meta() == MetaType::Name)
        .collect::<Vec<_>>();
    let bindings_valid = subject.is_some_and(|subject| {
        children.is_some_and(|children| {
            subject.binding() != children.binding() && children.output() == landing
        })
    });
    let valid = bindings_valid
        && children.is_some()
        && variants.next().is_none()
        && constructor.len() == 1
        && input.parameters().len() == 3;
    if !valid {
        return Err(AuthoredNomosError::InvalidRecursiveBindings {
            transformer: transformer.encoded_id().clone(),
        });
    }
    let mut variants = input
        .parameters()
        .iter()
        .filter(|parameter| parameter.meta() == MetaType::Variants);
    Ok(RecursiveBindings {
        subject: variants
            .next()
            .expect("validated subject binding")
            .binding()
            .clone(),
        constructor: constructor[0].binding().clone(),
        children: variants
            .next()
            .expect("validated children binding")
            .binding()
            .clone(),
    })
}

struct RecursiveCompiler<'requirements> {
    transformer: &'requirements AuthoredTransformerIdentity,
    bindings: RecursiveBindings,
    requirements: &'requirements [TemplateFutureRequirement<VocabularyRoot>],
    next_requirement: usize,
    compiled_calls: usize,
}

impl RecursiveCompiler<'_> {
    fn compile_value(
        &mut self,
        value: TemplateValue<VocabularyRoot>,
    ) -> Result<TemplateValue<VocabularyRoot>, AuthoredNomosError> {
        let fields = value
            .fields()
            .iter()
            .map(|field| {
                Ok(crate::TemplateFieldValue::new(
                    field.role(),
                    self.compile_term(field.term().clone())?,
                ))
            })
            .collect::<Result<Vec<_>, AuthoredNomosError>>()?;
        Ok(TemplateValue::try_new(value.constructor().clone(), fields)?)
    }

    fn compile_term(
        &mut self,
        term: crate::TemplateTerm<VocabularyRoot>,
    ) -> Result<crate::TemplateTerm<VocabularyRoot>, AuthoredNomosError> {
        match term {
            crate::TemplateTerm::Nested(value) => Ok(crate::TemplateTerm::Nested(Box::new(
                self.compile_value(*value)?,
            ))),
            crate::TemplateTerm::Sequence(items) => Ok(crate::TemplateTerm::Sequence(
                items
                    .into_iter()
                    .map(|item| self.compile_term(item))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            crate::TemplateTerm::Future(future) => {
                let requirement =
                    self.requirements
                        .get(self.next_requirement)
                        .ok_or_else(|| AuthoredNomosError::InvocationRequirementDrift {
                            transformer: self.transformer.encoded_id().clone(),
                        })?;
                if requirement.future() != &future {
                    return Err(AuthoredNomosError::InvocationRequirementDrift {
                        transformer: self.transformer.encoded_id().clone(),
                    });
                }
                self.next_requirement += 1;
                match future {
                    TemplateFuture::Invoke(target) if &target == self.transformer => {
                        self.compiled_calls += 1;
                        Ok(crate::TemplateTerm::Future(
                            TemplateFuture::RecursiveInvoke {
                                payload: Box::new(RecursiveCallJudgment::new(
                                    target,
                                    self.bindings.subject.clone(),
                                    self.bindings.constructor.clone(),
                                    self.bindings.children.clone(),
                                    requirement.output().clone(),
                                )),
                            },
                        ))
                    }
                    TemplateFuture::RecursiveInvoke { .. } => {
                        Err(AuthoredNomosError::UnexpectedRecursiveInvoke {
                            transformer: self.transformer.encoded_id().clone(),
                        })
                    }
                    other => Ok(crate::TemplateTerm::Future(other)),
                }
            }
            other => Ok(other),
        }
    }
}

fn compile_recursive_result(
    result: TemplateValue<VocabularyRoot>,
    transformer: &AuthoredTransformerIdentity,
    input: &AuthoredInputSignature,
    requirements: &[TemplateFutureRequirement<VocabularyRoot>],
    landing: &TemplateFutureOutput<VocabularyRoot>,
) -> Result<TemplateValue<VocabularyRoot>, AuthoredNomosError> {
    let bindings = recursive_bindings(transformer, input, landing)?;
    let mut compiler = RecursiveCompiler {
        transformer,
        bindings,
        requirements,
        next_requirement: 0,
        compiled_calls: 0,
    };
    let compiled = compiler.compile_value(result)?;
    if compiler.next_requirement != requirements.len() {
        return Err(AuthoredNomosError::InvocationRequirementDrift {
            transformer: transformer.encoded_id().clone(),
        });
    }
    if compiler.compiled_calls == 0 {
        return Err(AuthoredNomosError::MissingRecursiveInvoke {
            transformer: transformer.encoded_id().clone(),
        });
    }
    Ok(compiled)
}

fn contains_internal_recursive_call(value: &TemplateValue<VocabularyRoot>) -> bool {
    value
        .fields()
        .iter()
        .any(|field| term_contains_internal_recursive_call(field.term()))
}

fn term_contains_internal_recursive_call(term: &crate::TemplateTerm<VocabularyRoot>) -> bool {
    match term {
        crate::TemplateTerm::Nested(value) => contains_internal_recursive_call(value),
        crate::TemplateTerm::Sequence(items) => {
            items.iter().any(term_contains_internal_recursive_call)
        }
        crate::TemplateTerm::Future(TemplateFuture::RecursiveInvoke { .. }) => true,
        crate::TemplateTerm::Declaration(_)
        | crate::TemplateTerm::Reference(_)
        | crate::TemplateTerm::Literal(_)
        | crate::TemplateTerm::Scalar(_)
        | crate::TemplateTerm::Future(_) => false,
    }
}

fn resolve_binding_outputs(
    input: &AuthoredInputSignature,
    requirements: &[TemplateFutureRequirement<VocabularyRoot>],
) -> Result<Vec<TemplateFutureRequirement<VocabularyRoot>>, AuthoredNomosError> {
    let mut persisted = Vec::new();
    for requirement in requirements {
        match requirement.future() {
            TemplateFuture::Realize { binding, .. } | TemplateFuture::Splice { binding } => {
                let parameter = input.parameter(binding).ok_or_else(|| {
                    AuthoredNomosError::UndeclaredBinding {
                        binding: binding.encoded_id().clone(),
                    }
                })?;
                if parameter.output() != requirement.output() {
                    return Err(AuthoredNomosError::FutureOutputMismatch {
                        future: requirement.future().kind(),
                        expected: requirement.output().clone(),
                        found: parameter.output().clone(),
                    });
                }
            }
            TemplateFuture::InsertAt { payload } => {
                let parameter = input.parameter(payload.target()).ok_or_else(|| {
                    AuthoredNomosError::UndeclaredBinding {
                        binding: payload.target().encoded_id().clone(),
                    }
                })?;
                let LandingShape::Sequence { element, .. } = parameter.output().landing() else {
                    return Err(AuthoredNomosError::InsertionTargetNotSequence {
                        binding: payload.target().encoded_id().clone(),
                    });
                };
                if element.as_ref() != requirement.output().landing() {
                    return Err(AuthoredNomosError::FutureOutputMismatch {
                        future: requirement.future().kind(),
                        expected: requirement.output().clone(),
                        found: TemplateFutureOutput::new(element.as_ref().clone()),
                    });
                }
                persisted.push(requirement.clone());
            }
            TemplateFuture::Invoke(_) | TemplateFuture::RecursiveInvoke { .. } => {
                persisted.push(requirement.clone())
            }
        }
    }
    Ok(persisted)
}

enum CompiledFutureSite<'value> {
    Authored {
        future: &'value TemplateFuture,
    },
    Recursive {
        payload: &'value RecursiveCallJudgment,
    },
}

fn validate_requirement_mirror(
    declaration: &AuthoredTransformerDeclaration,
) -> Result<(), AuthoredNomosError> {
    let mut sites = Vec::new();
    collect_compiled_future_sites(declaration.result(), &mut sites);
    let requirements = declaration.future_requirements();
    if sites.len() != requirements.len() {
        return Err(requirement_drift(declaration, requirements, &sites));
    }
    for index in 0..sites.len() {
        let requirement = &requirements[index];
        let matches = match &sites[index] {
            CompiledFutureSite::Authored { future } => *future == requirement.future(),
            CompiledFutureSite::Recursive { payload } => {
                matches!(
                    requirement.future(),
                    TemplateFuture::Invoke(target) if target == payload.target()
                ) && requirement.output() == payload.landing()
            }
        };
        if !matches {
            return Err(requirement_drift(declaration, requirements, &sites));
        }
    }
    Ok(())
}

fn requirement_drift(
    declaration: &AuthoredTransformerDeclaration,
    requirements: &[TemplateFutureRequirement<VocabularyRoot>],
    sites: &[CompiledFutureSite<'_>],
) -> AuthoredNomosError {
    let insertion = requirements
        .iter()
        .any(|requirement| matches!(requirement.future(), TemplateFuture::InsertAt { .. }))
        || sites.iter().any(|site| {
            matches!(
                site,
                CompiledFutureSite::Authored {
                    future: TemplateFuture::InsertAt { .. }
                }
            )
        });
    if insertion {
        AuthoredNomosError::InsertionRequirementDrift {
            transformer: declaration.name().encoded_id().clone(),
        }
    } else {
        AuthoredNomosError::InvocationRequirementDrift {
            transformer: declaration.name().encoded_id().clone(),
        }
    }
}

fn collect_compiled_future_sites<'value>(
    value: &'value TemplateValue<VocabularyRoot>,
    sites: &mut Vec<CompiledFutureSite<'value>>,
) {
    for field in value.fields() {
        collect_compiled_future_sites_from_term(field.term(), sites);
    }
}

fn collect_compiled_future_sites_from_term<'value>(
    term: &'value crate::TemplateTerm<VocabularyRoot>,
    sites: &mut Vec<CompiledFutureSite<'value>>,
) {
    match term {
        crate::TemplateTerm::Nested(value) => collect_compiled_future_sites(value, sites),
        crate::TemplateTerm::Sequence(items) => {
            for item in items {
                collect_compiled_future_sites_from_term(item, sites);
            }
        }
        crate::TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
            sites.push(CompiledFutureSite::Recursive { payload });
        }
        crate::TemplateTerm::Future(
            future @ (TemplateFuture::Invoke(_) | TemplateFuture::InsertAt { .. }),
        ) => {
            sites.push(CompiledFutureSite::Authored { future });
        }
        crate::TemplateTerm::Declaration(_)
        | crate::TemplateTerm::Reference(_)
        | crate::TemplateTerm::Literal(_)
        | crate::TemplateTerm::Scalar(_)
        | crate::TemplateTerm::Future(_) => {}
    }
}

struct InsertionValidator<'declaration> {
    declaration: &'declaration AuthoredTransformerDeclaration,
    next_requirement: usize,
}

fn validate_insertions(
    declaration: &AuthoredTransformerDeclaration,
) -> Result<(), AuthoredNomosError> {
    let mut validator = InsertionValidator {
        declaration,
        next_requirement: 0,
    };
    validator.value(declaration.result())?;
    if validator.next_requirement != declaration.future_requirements().len() {
        return Err(AuthoredNomosError::InsertionRequirementDrift {
            transformer: declaration.name().encoded_id().clone(),
        });
    }
    Ok(())
}

impl InsertionValidator<'_> {
    fn value(&mut self, value: &TemplateValue<VocabularyRoot>) -> Result<(), AuthoredNomosError> {
        for field in value.fields() {
            self.term(field.term(), false)?;
        }
        Ok(())
    }

    fn term(
        &mut self,
        term: &crate::TemplateTerm<VocabularyRoot>,
        in_sequence: bool,
    ) -> Result<(), AuthoredNomosError> {
        match term {
            crate::TemplateTerm::Nested(value) => self.value(value),
            crate::TemplateTerm::Sequence(items) => self.sequence(items),
            crate::TemplateTerm::Future(TemplateFuture::InsertAt { .. }) if !in_sequence => {
                Err(self.drift())
            }
            crate::TemplateTerm::Future(
                TemplateFuture::Invoke(_)
                | TemplateFuture::RecursiveInvoke { .. }
                | TemplateFuture::InsertAt { .. },
            ) => {
                self.next_requirement += 1;
                Ok(())
            }
            crate::TemplateTerm::Declaration(_)
            | crate::TemplateTerm::Reference(_)
            | crate::TemplateTerm::Literal(_)
            | crate::TemplateTerm::Scalar(_)
            | crate::TemplateTerm::Future(_) => Ok(()),
        }
    }

    fn sequence(
        &mut self,
        items: &[crate::TemplateTerm<VocabularyRoot>],
    ) -> Result<(), AuthoredNomosError> {
        for index in 0..items.len() {
            if let crate::TemplateTerm::Future(site_future) = &items[index]
                && let TemplateFuture::InsertAt { payload } = site_future
            {
                let requirement = self
                    .declaration
                    .future_requirements()
                    .get(self.next_requirement)
                    .ok_or_else(|| self.drift())?;
                if requirement.future() != site_future {
                    return Err(self.drift());
                }
                let matching_splices = items
                    .iter()
                    .filter(|item| {
                        matches!(
                            item,
                            crate::TemplateTerm::Future(TemplateFuture::Splice { binding })
                                if binding == payload.target()
                        )
                    })
                    .count();
                let preceding_splices = items[..index]
                    .iter()
                    .filter(|item| {
                        matches!(
                            item,
                            crate::TemplateTerm::Future(TemplateFuture::Splice { binding })
                                if binding == payload.target()
                        )
                    })
                    .count();
                if matching_splices != 1 || preceding_splices != 1 {
                    return Err(self.drift());
                }
                let value = items.get(index + 1).ok_or_else(|| self.drift())?;
                if !term_matches_landing(value, requirement.output().landing(), self.declaration) {
                    return Err(self.drift());
                }
            }
            self.term(&items[index], true)?;
        }
        Ok(())
    }

    fn drift(&self) -> AuthoredNomosError {
        AuthoredNomosError::InsertionRequirementDrift {
            transformer: self.declaration.name().encoded_id().clone(),
        }
    }
}

fn term_matches_landing(
    term: &crate::TemplateTerm<VocabularyRoot>,
    landing: &LandingShape<VocabularyRoot>,
    declaration: &AuthoredTransformerDeclaration,
) -> bool {
    match (term, landing) {
        (crate::TemplateTerm::Declaration(_), LandingShape::Declaration)
        | (crate::TemplateTerm::Reference(_), LandingShape::Reference) => true,
        (crate::TemplateTerm::Literal(found), LandingShape::Literal(expected)) => found == expected,
        (
            crate::TemplateTerm::Scalar(structural_codec::ScalarValue::Integer(_)),
            LandingShape::Scalar(structural_codec::LeafCodec::Integer),
        )
        | (
            crate::TemplateTerm::Scalar(structural_codec::ScalarValue::Float(_)),
            LandingShape::Scalar(structural_codec::LeafCodec::Float),
        )
        | (
            crate::TemplateTerm::Scalar(structural_codec::ScalarValue::Text(_)),
            LandingShape::Scalar(structural_codec::LeafCodec::Text),
        )
        | (
            crate::TemplateTerm::Scalar(structural_codec::ScalarValue::Boolean(_)),
            LandingShape::Scalar(structural_codec::LeafCodec::Boolean),
        ) => true,
        (crate::TemplateTerm::Nested(value), LandingShape::Type(target)) => {
            value.constructor().type_id() == target
        }
        (
            crate::TemplateTerm::Sequence(items),
            LandingShape::Sequence {
                minimum,
                maximum,
                element,
            },
        ) => {
            let length = items.len() as u64;
            length >= *minimum
                && maximum.is_none_or(|limit| length <= limit)
                && items
                    .iter()
                    .all(|item| term_matches_landing(item, element, declaration))
        }
        (
            crate::TemplateTerm::Future(
                TemplateFuture::Realize { binding, .. } | TemplateFuture::Splice { binding },
            ),
            expected,
        ) => declaration
            .input()
            .parameter(binding)
            .is_some_and(|parameter| parameter.output().landing() == expected),
        (
            crate::TemplateTerm::Future(
                TemplateFuture::RecursiveInvoke { .. } | TemplateFuture::InsertAt { .. },
            ),
            _,
        ) => false,
        (crate::TemplateTerm::Future(future), expected) => {
            declaration.future_requirements().iter().any(|requirement| {
                requirement.future() == future && requirement.output().landing() == expected
            })
        }
        _ => false,
    }
}

#[derive(Default)]
struct RecursiveSequenceCounts {
    markers: usize,
    children_splices: usize,
}

fn validate_recursive_sequence(
    declaration: &AuthoredTransformerDeclaration,
) -> Result<(), AuthoredNomosError> {
    let MacroKind::Recursive { .. } = declaration.kind() else {
        return Ok(());
    };
    if !matches!(
        declaration.output().landing(),
        LandingShape::Sequence { minimum: 0, .. }
    ) {
        return Err(AuthoredNomosError::InvalidRecursiveLanding {
            transformer: declaration.name().encoded_id().clone(),
        });
    }
    let bindings = recursive_bindings(
        declaration.name(),
        declaration.input(),
        declaration.output(),
    )?;
    let mut counts = RecursiveSequenceCounts::default();
    validate_recursive_value(
        declaration,
        declaration.result(),
        &bindings.children,
        &mut counts,
    )?;
    if counts.markers != 1 || counts.children_splices != 1 {
        return Err(AuthoredNomosError::InvalidRecursiveSequence {
            transformer: declaration.name().encoded_id().clone(),
        });
    }
    Ok(())
}

fn validate_recursive_value(
    declaration: &AuthoredTransformerDeclaration,
    value: &TemplateValue<VocabularyRoot>,
    children: &AuthoredBindingIdentity,
    counts: &mut RecursiveSequenceCounts,
) -> Result<(), AuthoredNomosError> {
    for field in value.fields() {
        validate_recursive_term(declaration, field.term(), children, counts, false)?;
    }
    Ok(())
}

fn validate_recursive_term(
    declaration: &AuthoredTransformerDeclaration,
    term: &crate::TemplateTerm<VocabularyRoot>,
    children: &AuthoredBindingIdentity,
    counts: &mut RecursiveSequenceCounts,
    in_sequence: bool,
) -> Result<(), AuthoredNomosError> {
    match term {
        crate::TemplateTerm::Nested(value) => {
            validate_recursive_value(declaration, value, children, counts)
        }
        crate::TemplateTerm::Sequence(items) => {
            let marker_indices = items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    matches!(
                        item,
                        crate::TemplateTerm::Future(TemplateFuture::RecursiveInvoke { .. })
                    )
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            let children_indices = items
                .iter()
                .enumerate()
                .filter_map(|(index, item)| {
                    matches!(
                        item,
                        crate::TemplateTerm::Future(TemplateFuture::Splice { binding })
                            if binding == children
                    )
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            if (!marker_indices.is_empty() || !children_indices.is_empty())
                && (marker_indices.len() != 1
                    || children_indices.len() != 1
                    || marker_indices[0] >= children_indices[0])
            {
                return Err(AuthoredNomosError::InvalidRecursiveSequence {
                    transformer: declaration.name().encoded_id().clone(),
                });
            }
            counts.markers += marker_indices.len();
            counts.children_splices += children_indices.len();
            for item in items {
                validate_recursive_term(declaration, item, children, counts, true)?;
            }
            Ok(())
        }
        crate::TemplateTerm::Future(TemplateFuture::RecursiveInvoke { .. }) if !in_sequence => {
            Err(AuthoredNomosError::InvalidRecursiveSequence {
                transformer: declaration.name().encoded_id().clone(),
            })
        }
        crate::TemplateTerm::Future(TemplateFuture::Splice { binding })
            if !in_sequence && binding == children =>
        {
            Err(AuthoredNomosError::InvalidRecursiveSequence {
                transformer: declaration.name().encoded_id().clone(),
            })
        }
        crate::TemplateTerm::Declaration(_)
        | crate::TemplateTerm::Reference(_)
        | crate::TemplateTerm::Literal(_)
        | crate::TemplateTerm::Scalar(_)
        | crate::TemplateTerm::Future(_) => Ok(()),
    }
}

/// A complete authored transformer set whose invocation outputs have been
/// resolved and checked before evaluator entry.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
pub struct AuthoredTransformerSet(Vec<AuthoredTransformerDeclaration>);

impl AuthoredTransformerSet {
    pub fn try_new(
        mut declarations: Vec<AuthoredTransformerDeclaration>,
    ) -> Result<Self, AuthoredNomosError> {
        declarations.sort_by(|left, right| left.name().cmp(right.name()));
        for pair in declarations.windows(2) {
            if pair[0].name() == pair[1].name() {
                return Err(AuthoredNomosError::DuplicateTransformer {
                    transformer: pair[0].name().encoded_id().clone(),
                });
            }
        }
        for declaration in &declarations {
            validate_requirement_mirror(declaration)?;
            let recovered =
                resolve_binding_outputs(declaration.input(), declaration.future_requirements())?;
            if recovered != declaration.future_requirements() {
                return Err(AuthoredNomosError::InvocationRequirementDrift {
                    transformer: declaration.name().encoded_id().clone(),
                });
            }
            validate_insertions(declaration)?;
            validate_recursive_sequence(declaration)?;
            let invocation_requirements = declaration
                .future_requirements()
                .iter()
                .filter(|requirement| matches!(requirement.future(), TemplateFuture::Invoke(_)))
                .collect::<Vec<_>>();
            let mut sites = Vec::new();
            collect_compiled_invocation_sites(declaration.result(), &mut sites);
            if sites.len() != invocation_requirements.len() {
                return Err(AuthoredNomosError::InvocationRequirementDrift {
                    transformer: declaration.name().encoded_id().clone(),
                });
            }
            let mut saw_recursive = false;
            for index in 0..sites.len() {
                let requirement = invocation_requirements[index];
                let TemplateFuture::Invoke(authored_target) = requirement.future() else {
                    return Err(AuthoredNomosError::InvocationRequirementDrift {
                        transformer: declaration.name().encoded_id().clone(),
                    });
                };
                match &sites[index] {
                    CompiledInvocationSite::Ordinary { target } => {
                        if *target != authored_target {
                            return Err(AuthoredNomosError::InvocationRequirementDrift {
                                transformer: declaration.name().encoded_id().clone(),
                            });
                        }
                        if matches!(declaration.kind(), MacroKind::Recursive { .. })
                            && *target == declaration.name()
                        {
                            return Err(AuthoredNomosError::UncompiledRecursiveInvoke {
                                transformer: target.encoded_id().clone(),
                            });
                        }
                        let target = declarations
                            .iter()
                            .find(|candidate| candidate.name() == *target)
                            .ok_or_else(|| AuthoredNomosError::UndeclaredTransformer {
                                transformer: target.encoded_id().clone(),
                            })?;
                        if target.output() != requirement.output() {
                            return Err(AuthoredNomosError::FutureOutputMismatch {
                                future: TemplateFutureKind::Invoke,
                                expected: requirement.output().clone(),
                                found: target.output().clone(),
                            });
                        }
                    }
                    CompiledInvocationSite::Recursive { payload } => {
                        saw_recursive = true;
                        validate_recursive_judgment(declaration, requirement, payload)?;
                    }
                }
            }
            match declaration.kind() {
                MacroKind::Recursive { section } => {
                    if section != crate::SectionDefault::Enumeration {
                        return Err(AuthoredNomosError::UnsupportedRecursiveSection { section });
                    }
                    if !saw_recursive {
                        return Err(AuthoredNomosError::MissingRecursiveInvoke {
                            transformer: declaration.name().encoded_id().clone(),
                        });
                    }
                }
                MacroKind::Named | MacroKind::Structural(_) if saw_recursive => {
                    return Err(AuthoredNomosError::UnexpectedRecursiveInvoke {
                        transformer: declaration.name().encoded_id().clone(),
                    });
                }
                MacroKind::Named | MacroKind::Structural(_) => {}
            }
        }
        Ok(Self(declarations))
    }

    pub fn declarations(&self) -> &[AuthoredTransformerDeclaration] {
        &self.0
    }
}

fn validate_recursive_judgment(
    declaration: &AuthoredTransformerDeclaration,
    requirement: &TemplateFutureRequirement<VocabularyRoot>,
    payload: &RecursiveCallJudgment,
) -> Result<(), AuthoredNomosError> {
    let MacroKind::Recursive { section } = declaration.kind() else {
        return Err(AuthoredNomosError::UnexpectedRecursiveInvoke {
            transformer: declaration.name().encoded_id().clone(),
        });
    };
    let parameters = declaration.input().parameters();
    let subject_matches = parameters
        .iter()
        .filter(|parameter| {
            parameter.meta() == MetaType::Variants
                && parameter.binding() == payload.subject_binding()
        })
        .count();
    let constructor_matches = parameters
        .iter()
        .filter(|parameter| {
            parameter.meta() == MetaType::Name
                && parameter.binding() == payload.constructor_binding()
        })
        .count();
    let children_matches = parameters
        .iter()
        .filter(|parameter| {
            parameter.meta() == MetaType::Variants
                && parameter.binding() == payload.children_binding()
        })
        .count();
    let authored_target = match requirement.future() {
        TemplateFuture::Invoke(target) => Some(target),
        TemplateFuture::Realize { .. }
        | TemplateFuture::Splice { .. }
        | TemplateFuture::RecursiveInvoke { .. }
        | TemplateFuture::InsertAt { .. } => None,
    };
    let judgment_matches = payload.target() == declaration.name()
        && authored_target == Some(payload.target())
        && section == crate::SectionDefault::Enumeration
        && subject_matches == 1
        && constructor_matches == 1
        && children_matches == 1
        && payload.subject_binding() != payload.children_binding()
        && parameters.len() == 3
        && payload.edge_policy() == RecursiveEdgePolicy::FiniteOwnedTree
        && payload.landing() == requirement.output()
        && declaration.output() == requirement.output();
    if !judgment_matches {
        return Err(AuthoredNomosError::InvalidRecursiveJudgment {
            transformer: declaration.name().encoded_id().clone(),
        });
    }
    Ok(())
}

enum CompiledInvocationSite<'value> {
    Ordinary {
        target: &'value AuthoredTransformerIdentity,
    },
    Recursive {
        payload: &'value RecursiveCallJudgment,
    },
}

fn collect_compiled_invocation_sites<'value>(
    value: &'value TemplateValue<VocabularyRoot>,
    sites: &mut Vec<CompiledInvocationSite<'value>>,
) {
    for field in value.fields() {
        collect_compiled_invocation_sites_from_term(field.term(), sites);
    }
}

fn collect_compiled_invocation_sites_from_term<'value>(
    term: &'value crate::TemplateTerm<VocabularyRoot>,
    sites: &mut Vec<CompiledInvocationSite<'value>>,
) {
    match term {
        crate::TemplateTerm::Nested(value) => collect_compiled_invocation_sites(value, sites),
        crate::TemplateTerm::Sequence(items) => {
            for item in items {
                collect_compiled_invocation_sites_from_term(item, sites);
            }
        }
        crate::TemplateTerm::Future(TemplateFuture::Invoke(target)) => {
            sites.push(CompiledInvocationSite::Ordinary { target });
        }
        crate::TemplateTerm::Future(TemplateFuture::RecursiveInvoke { payload }) => {
            sites.push(CompiledInvocationSite::Recursive { payload });
        }
        crate::TemplateTerm::Declaration(_)
        | crate::TemplateTerm::Reference(_)
        | crate::TemplateTerm::Literal(_)
        | crate::TemplateTerm::Scalar(_)
        | crate::TemplateTerm::Future(_) => {}
    }
}

#[cfg(test)]
pub(crate) fn native_text_admission_package_for_test() -> AuthoredTransformerSet {
    use encoded_name_table::LocalEncodedId;
    use structural_codec::LandingShape;

    use crate::{
        TemplateRootOutput, TemplateRootOutputSelector,
        template_language::nested_text_value_for_native_admission_test,
    };

    let value = nested_text_value_for_native_admission_test();
    let name = VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        vec![LocalEncodedId::new(31), LocalEncodedId::new(9)],
    )
    .expect("non-empty test transformer identity");
    let output = TemplateRootOutput::for_native_admission_test(
        TemplateRootOutputSelector::WholeValue,
        TemplateFutureOutput::new(LandingShape::Type(value.constructor().type_id().clone())),
    );
    AuthoredTransformerSet(vec![AuthoredTransformerDeclaration {
        name: AuthoredTransformerIdentity(name),
        kind: MacroKind::Named,
        input: AuthoredInputSignature::unit(),
        result: value,
        output,
        future_requirements: Vec::new(),
    }])
}

#[cfg(test)]
pub(crate) fn native_restore_validation_package_for_test() -> AuthoredTransformerSet {
    use encoded_name_table::LocalEncodedId;
    use structural_codec::LandingShape;

    use crate::{
        SectionDefault, TemplateRootOutput, TemplateRootOutputSelector,
        template_language::restore_validation_value_for_native_test,
    };

    let value = restore_validation_value_for_native_test();
    let name = VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        vec![LocalEncodedId::new(31), LocalEncodedId::new(19)],
    )
    .expect("non-empty test transformer identity");
    let output = TemplateRootOutput::for_native_admission_test(
        TemplateRootOutputSelector::WholeValue,
        TemplateFutureOutput::new(LandingShape::Type(value.constructor().type_id().clone())),
    );
    AuthoredTransformerSet(vec![AuthoredTransformerDeclaration {
        name: AuthoredTransformerIdentity(name),
        kind: MacroKind::Structural(SectionDefault::Newtype),
        input: AuthoredInputSignature::unit(),
        result: value,
        output,
        future_requirements: Vec::new(),
    }])
}

#[cfg(test)]
mod restore_mutation_tests {
    use capsule_content_identity::PortableArchive;
    use encoded_name_table::LocalEncodedId;
    use structural_codec::{
        EncodedConstructorId, EncodedTypeId, FieldRole, LandingShape, LeafCodec, Position,
        ScalarValue, SharedDescriptor,
    };

    use super::*;
    use crate::{
        InsertAt, SectionDefault, TemplateFieldValue, TemplateFutureRequirement,
        TemplateRootOutputSelector, TemplateTerm,
    };

    #[derive(rkyv::Archive)]
    struct RecursiveResultRole;

    impl FieldRole for RecursiveResultRole {
        const STABLE_ID: u16 = 32_001;
    }

    struct RecursiveRestoreFixture {
        set: AuthoredTransformerSet,
        transformer: AuthoredTransformerIdentity,
        subject: AuthoredBindingIdentity,
        constructor: AuthoredBindingIdentity,
        children: AuthoredBindingIdentity,
        landing: TemplateFutureOutput<VocabularyRoot>,
    }

    fn identity(chain: &[u16]) -> VocabularyEncodedId {
        VocabularyEncodedId::new(
            VocabularyRoot::Universal,
            chain.iter().copied().map(LocalEncodedId::new).collect(),
        )
        .expect("non-empty recursive restore identity")
    }

    fn binding(local: u16) -> AuthoredBindingIdentity {
        AuthoredBindingIdentity::try_new(identity(&[32, local]))
            .expect("Universal recursive binding")
    }

    fn integer_landing() -> LandingShape<VocabularyRoot> {
        LandingShape::Scalar(LeafCodec::Integer)
    }

    fn sequence_landing(element: LandingShape<VocabularyRoot>) -> LandingShape<VocabularyRoot> {
        LandingShape::sequence(0, None, element)
    }

    fn output(landing: LandingShape<VocabularyRoot>) -> TemplateFutureOutput<VocabularyRoot> {
        TemplateFutureOutput::new(landing)
    }

    fn fixture() -> RecursiveRestoreFixture {
        let transformer =
            AuthoredTransformerIdentity::try_new(identity(&[32, 10])).expect("transformer");
        let subject = binding(1);
        let constructor = binding(2);
        let children = binding(3);
        let landing = output(sequence_landing(integer_landing()));
        let input = AuthoredInputSignature::try_new(vec![
            AuthoredInputParameter::new(
                constructor.clone(),
                MetaType::Name,
                output(LandingShape::Declaration),
            ),
            AuthoredInputParameter::new(
                subject.clone(),
                MetaType::Variants,
                output(sequence_landing(integer_landing())),
            ),
            AuthoredInputParameter::new(children.clone(), MetaType::Variants, landing.clone()),
        ])
        .expect("three distinct recursive bindings");
        let role =
            Position::<RecursiveResultRole, VocabularyRoot>::try_new(SharedDescriptor::Repeated {
                minimum: 0,
                maximum: None,
                element: Box::new(SharedDescriptor::Leaf(LeafCodec::Integer)),
            })
            .expect("non-zero recursive result role")
            .role();
        let result_type = EncodedTypeId::new(identity(&[32, 20]));
        let judgment = RecursiveCallJudgment::new(
            transformer.clone(),
            subject.clone(),
            constructor.clone(),
            children.clone(),
            landing.clone(),
        );
        let insertion = InsertAt::new(children.clone(), 0);
        let result = TemplateValue::try_new(
            EncodedConstructorId::under(&result_type, 1),
            vec![TemplateFieldValue::new(
                role,
                TemplateTerm::Sequence(vec![
                    TemplateTerm::Future(TemplateFuture::RecursiveInvoke {
                        payload: Box::new(judgment),
                    }),
                    TemplateTerm::Future(TemplateFuture::Splice {
                        binding: children.clone(),
                    }),
                    TemplateTerm::Future(TemplateFuture::InsertAt {
                        payload: Box::new(insertion.clone()),
                    }),
                    TemplateTerm::Scalar(ScalarValue::Integer(7)),
                ]),
            )],
        )
        .expect("one recursive result field");
        let root_output = crate::TemplateRootOutput::for_native_admission_test(
            TemplateRootOutputSelector::Field(role),
            landing.clone(),
        );
        let requirements = vec![
            TemplateFutureRequirement::for_restore_mutation_test(
                TemplateFuture::Invoke(transformer.clone()),
                landing.clone(),
            ),
            TemplateFutureRequirement::for_restore_mutation_test(
                TemplateFuture::InsertAt {
                    payload: Box::new(insertion),
                },
                output(integer_landing()),
            ),
        ];
        let declaration = AuthoredTransformerDeclaration {
            name: transformer.clone(),
            kind: MacroKind::Recursive {
                section: SectionDefault::Enumeration,
            },
            input,
            result,
            output: root_output,
            future_requirements: requirements,
        };
        let set =
            AuthoredTransformerSet::try_new(vec![declaration]).expect("valid recursive fixture");
        RecursiveRestoreFixture {
            set,
            transformer,
            subject,
            constructor,
            children,
            landing,
        }
    }

    fn replace_judgment(set: &mut AuthoredTransformerSet, judgment: RecursiveCallJudgment) {
        let declaration = &mut set.0[0];
        let mut fields = declaration.result.fields().to_vec();
        let TemplateTerm::Sequence(mut items) = fields[0].term().clone() else {
            panic!("recursive fixture result is a sequence")
        };
        items[0] = TemplateTerm::Future(TemplateFuture::RecursiveInvoke {
            payload: Box::new(judgment),
        });
        fields[0] = TemplateFieldValue::new(fields[0].role(), TemplateTerm::Sequence(items));
        declaration.result =
            TemplateValue::try_new(declaration.result.constructor().clone(), fields)
                .expect("role-preserving judgment mutation");
    }

    fn restore_and_validate(
        forged: &AuthoredTransformerSet,
    ) -> Result<AuthoredTransformerSet, AuthoredNomosError> {
        let bytes = forged
            .to_archive_bytes()
            .expect("archive forged recursive set");
        let restored = AuthoredTransformerSet::from_archive_bytes(bytes.as_ref())
            .expect("restore structurally valid forged set");
        AuthoredTransformerSet::try_new(restored.declarations().to_vec())
    }

    #[test]
    fn restored_recursive_judgment_equations_refuse_field_mutations() {
        let fixture = fixture();

        let mut target = fixture.set.clone();
        let other_target =
            AuthoredTransformerIdentity::try_new(identity(&[32, 11])).expect("other target");
        replace_judgment(
            &mut target,
            RecursiveCallJudgment::new(
                other_target.clone(),
                fixture.subject.clone(),
                fixture.constructor.clone(),
                fixture.children.clone(),
                fixture.landing.clone(),
            ),
        );
        target.0[0].future_requirements[0] = TemplateFutureRequirement::for_restore_mutation_test(
            TemplateFuture::Invoke(other_target),
            fixture.landing.clone(),
        );
        assert!(matches!(
            restore_and_validate(&target),
            Err(AuthoredNomosError::InvalidRecursiveJudgment { .. })
        ));

        let mut subject = fixture.set.clone();
        replace_judgment(
            &mut subject,
            RecursiveCallJudgment::new(
                fixture.transformer.clone(),
                fixture.constructor.clone(),
                fixture.constructor.clone(),
                fixture.children.clone(),
                fixture.landing.clone(),
            ),
        );
        assert!(matches!(
            restore_and_validate(&subject),
            Err(AuthoredNomosError::InvalidRecursiveJudgment { .. })
        ));

        let mut constructor = fixture.set.clone();
        replace_judgment(
            &mut constructor,
            RecursiveCallJudgment::new(
                fixture.transformer.clone(),
                fixture.subject.clone(),
                fixture.subject.clone(),
                fixture.children.clone(),
                fixture.landing.clone(),
            ),
        );
        assert!(matches!(
            restore_and_validate(&constructor),
            Err(AuthoredNomosError::InvalidRecursiveJudgment { .. })
        ));

        let mut children = fixture.set.clone();
        replace_judgment(
            &mut children,
            RecursiveCallJudgment::new(
                fixture.transformer.clone(),
                fixture.subject.clone(),
                fixture.constructor.clone(),
                fixture.subject.clone(),
                fixture.landing.clone(),
            ),
        );
        assert!(matches!(
            restore_and_validate(&children),
            Err(AuthoredNomosError::InvalidRecursiveJudgment { .. })
        ));

        let changed_landing = output(sequence_landing(LandingShape::Scalar(LeafCodec::Boolean)));
        let mut landing = fixture.set.clone();
        replace_judgment(
            &mut landing,
            RecursiveCallJudgment::new(
                fixture.transformer.clone(),
                fixture.subject.clone(),
                fixture.constructor.clone(),
                fixture.children.clone(),
                changed_landing.clone(),
            ),
        );
        landing.0[0].future_requirements[0] = TemplateFutureRequirement::for_restore_mutation_test(
            TemplateFuture::Invoke(fixture.transformer.clone()),
            changed_landing,
        );
        assert!(matches!(
            restore_and_validate(&landing),
            Err(AuthoredNomosError::InvalidRecursiveJudgment { .. })
        ));

        let restored = restore_and_validate(&fixture.set).expect("valid judgment round trip");
        let mut sites = Vec::new();
        collect_compiled_invocation_sites(restored.declarations()[0].result(), &mut sites);
        let [CompiledInvocationSite::Recursive { payload }] = sites.as_slice() else {
            panic!("one restored recursive judgment")
        };
        assert_eq!(
            payload.edge_policy(),
            RecursiveEdgePolicy::FiniteOwnedTree,
            "the single-variant edge policy is closed rather than forgeable"
        );
    }

    #[test]
    fn restored_insert_at_requirements_refuse_vector_shape_and_marker_mutations() {
        let fixture = fixture();

        let mut reordered = fixture.set.clone();
        reordered.0[0].future_requirements.swap(0, 1);
        assert!(matches!(
            restore_and_validate(&reordered),
            Err(AuthoredNomosError::InsertionRequirementDrift { .. })
        ));

        let mut wrong_shape = fixture.set.clone();
        let insertion = wrong_shape.0[0].future_requirements[1].future().clone();
        wrong_shape.0[0].future_requirements[1] =
            TemplateFutureRequirement::for_restore_mutation_test(
                insertion,
                output(LandingShape::Scalar(LeafCodec::Boolean)),
            );
        assert!(matches!(
            restore_and_validate(&wrong_shape),
            Err(AuthoredNomosError::FutureOutputMismatch {
                future: TemplateFutureKind::InsertAt,
                ..
            })
        ));

        let mut nested_marker = fixture.set.clone();
        let declaration = &mut nested_marker.0[0];
        let mut fields = declaration.result.fields().to_vec();
        let TemplateTerm::Sequence(mut items) = fields[0].term().clone() else {
            panic!("recursive fixture result is a sequence")
        };
        let insertion = items[2].clone();
        items.insert(3, insertion);
        fields[0] = TemplateFieldValue::new(fields[0].role(), TemplateTerm::Sequence(items));
        declaration.result =
            TemplateValue::try_new(declaration.result.constructor().clone(), fields)
                .expect("role-preserving nested marker mutation");
        declaration
            .future_requirements
            .push(declaration.future_requirements[1].clone());
        assert!(matches!(
            restore_and_validate(&nested_marker),
            Err(AuthoredNomosError::InsertionRequirementDrift { .. })
        ));
    }
}

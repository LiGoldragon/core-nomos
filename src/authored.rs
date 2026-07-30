//! Phase-stable authored Nomos declarations.
//!
//! Transformer and binding identities are complete translator-issued chains.
//! The result is the one declaration-indexed [`crate::TemplateValue`] substrate;
//! no Logos type has an authored Rust twin in this module.

use std::collections::BTreeSet;

use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use thiserror::Error;

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
pub struct AuthoredInputParameter(
    AuthoredBindingIdentity,
    MetaType,
    TemplateFutureOutput<VocabularyRoot>,
);

impl AuthoredInputParameter {
    pub fn new(
        binding: AuthoredBindingIdentity,
        meta: MetaType,
        output: TemplateFutureOutput<VocabularyRoot>,
    ) -> Self {
        Self(binding, meta, output)
    }

    pub fn binding(&self) -> &AuthoredBindingIdentity {
        &self.0
    }

    pub const fn meta(&self) -> MetaType {
        self.1
    }

    pub const fn output(&self) -> &TemplateFutureOutput<VocabularyRoot> {
        &self.2
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
pub struct AuthoredTransformerDeclaration(
    AuthoredTransformerIdentity,
    MacroKind,
    AuthoredInputSignature,
    TemplateValue<VocabularyRoot>,
    TemplateRootOutput<VocabularyRoot>,
    Vec<TemplateFutureRequirement<VocabularyRoot>>,
);

impl AuthoredTransformerDeclaration {
    /// Validate the generic result against its computed Template(X) declaration.
    ///
    /// Binding futures are resolved immediately from the input signature.
    /// Invoke requirements remain durable until the complete within-package
    /// declaration set can resolve their lookup-only target identities.
    pub fn try_new(
        name: AuthoredTransformerIdentity,
        kind: MacroKind,
        input: AuthoredInputSignature,
        result: TemplateValue<VocabularyRoot>,
        language: &TemplateLanguage<VocabularyRoot>,
    ) -> Result<Self, AuthoredNomosError> {
        let requirements = language.analyze_value(&result)?;
        let invokes = resolve_binding_outputs(&input, requirements)?;
        let output = language.root_output_contract().map_err(|_| {
            AuthoredNomosError::InvalidTemplateRoot {
                encoded_type: language.root().clone(),
            }
        })?;
        Ok(Self(name, kind, input, result, output, invokes))
    }

    pub fn name(&self) -> &AuthoredTransformerIdentity {
        &self.0
    }

    pub const fn kind(&self) -> MacroKind {
        self.1
    }

    pub fn input(&self) -> &AuthoredInputSignature {
        &self.2
    }

    pub fn result(&self) -> &TemplateValue<VocabularyRoot> {
        &self.3
    }

    pub const fn output(&self) -> &TemplateFutureOutput<VocabularyRoot> {
        self.4.output()
    }

    pub const fn root_output(&self) -> &TemplateRootOutput<VocabularyRoot> {
        &self.4
    }

    pub fn invoke_requirements(&self) -> &[TemplateFutureRequirement<VocabularyRoot>] {
        &self.5
    }
}

fn resolve_binding_outputs(
    input: &AuthoredInputSignature,
    requirements: Vec<TemplateFutureRequirement<VocabularyRoot>>,
) -> Result<Vec<TemplateFutureRequirement<VocabularyRoot>>, AuthoredNomosError> {
    let mut invokes = Vec::new();
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
            TemplateFuture::Invoke(_) => invokes.push(requirement),
        }
    }
    Ok(invokes)
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
            for requirement in declaration.invoke_requirements() {
                let TemplateFuture::Invoke(target) = requirement.future() else {
                    continue;
                };
                let target = declarations
                    .iter()
                    .find(|candidate| candidate.name() == target)
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
        }
        Ok(Self(declarations))
    }

    pub fn declarations(&self) -> &[AuthoredTransformerDeclaration] {
        &self.0
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
    AuthoredTransformerSet(vec![AuthoredTransformerDeclaration(
        AuthoredTransformerIdentity(name),
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        value,
        output,
        Vec::new(),
    )])
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
    AuthoredTransformerSet(vec![AuthoredTransformerDeclaration(
        AuthoredTransformerIdentity(name),
        MacroKind::Structural(SectionDefault::Newtype),
        AuthoredInputSignature::unit(),
        value,
        output,
        Vec::new(),
    )])
}

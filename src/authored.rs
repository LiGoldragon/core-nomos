//! Phase-stable authored Nomos declarations.
//!
//! Transformer and binding identities are complete translator-issued chains.
//! The result is the one declaration-indexed [`crate::TemplateValue`] substrate;
//! no Logos type has an authored Rust twin in this module.

use std::collections::BTreeSet;

use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use thiserror::Error;

use crate::{
    MacroKind, MetaType, TemplateLanguage, TemplateTerm, TemplateValue, TemplateValueError,
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

/// One authored input position: durable binding identity, then meta-type.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredInputParameter(AuthoredBindingIdentity, MetaType);

impl AuthoredInputParameter {
    pub fn new(binding: AuthoredBindingIdentity, meta: MetaType) -> Self {
        Self(binding, meta)
    }

    pub fn binding(&self) -> &AuthoredBindingIdentity {
        &self.0
    }

    pub const fn meta(&self) -> MetaType {
        self.1
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

    fn contains(&self, binding: &AuthoredBindingIdentity) -> bool {
        self.0
            .iter()
            .any(|parameter| parameter.binding() == binding)
    }
}

/// One authored transformer declaration in durable pre-seal form.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
pub struct AuthoredTransformerDeclaration(
    AuthoredTransformerIdentity,
    MacroKind,
    AuthoredInputSignature,
    TemplateValue<VocabularyRoot>,
);

impl AuthoredTransformerDeclaration {
    /// Validate the generic result against its computed Template(X) declaration,
    /// then refuse every future that references an absent input binding.
    pub fn try_new(
        name: AuthoredTransformerIdentity,
        kind: MacroKind,
        input: AuthoredInputSignature,
        result: TemplateValue<VocabularyRoot>,
        language: &TemplateLanguage<VocabularyRoot>,
    ) -> Result<Self, AuthoredNomosError> {
        language.validate_value(&result)?;
        validate_bindings(&input, &result)?;
        Ok(Self(name, kind, input, result))
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
}

fn validate_bindings(
    input: &AuthoredInputSignature,
    value: &TemplateValue<VocabularyRoot>,
) -> Result<(), AuthoredNomosError> {
    for field in value.fields() {
        validate_term(input, field.term())?;
    }
    Ok(())
}

fn validate_term(
    input: &AuthoredInputSignature,
    term: &TemplateTerm<VocabularyRoot>,
) -> Result<(), AuthoredNomosError> {
    match term {
        TemplateTerm::Future(future) => {
            if let Some(binding) = future.referenced_binding()
                && !input.contains(binding)
            {
                return Err(AuthoredNomosError::UndeclaredBinding {
                    binding: binding.encoded_id().clone(),
                });
            }
        }
        TemplateTerm::Nested(value) => validate_bindings(input, value)?,
        TemplateTerm::Sequence(items) => {
            for item in items {
                validate_term(input, item)?;
            }
        }
        TemplateTerm::Declaration(_)
        | TemplateTerm::Reference(_)
        | TemplateTerm::Literal(_)
        | TemplateTerm::Scalar(_) => {}
    }
    Ok(())
}

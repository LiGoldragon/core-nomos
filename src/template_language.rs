//! The mechanically derived Template(X) language.
//!
//! A source language contributes two addressed inputs: structural grammar
//! records and landing declarations.  [`TemplateLanguage::derive`] walks both
//! inputs together from one addressed root.  It computes the value-or-future
//! landing shape for every semantic position without authoring or emitting a
//! Rust type for any source type or transformer.

use signal_sema_translator::VocabularyRoot;
use structural_codec::{
    AddressedStructuralTable, BorrowedFieldView, DecodeFormId, EncodedConstructorId, EncodedId,
    EncodedTypeId, FieldRole, FieldVisitor, LandingConstructorDeclaration,
    LandingDeclarationCatalog, LandingShape, LanguageDeclaration, LanguageDeclarationError,
    Position, ScalarValue, SharedDescriptor, StableRoleId, StructureRecord,
};

use crate::{AuthoredBindingIdentity, AuthoredTransformerIdentity, NameTransform};

/// The closed future-producing operations admitted by the first Template(X)
/// derivation.
#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum TemplateFutureKind {
    Realize,
    Invoke,
    Splice,
}

/// One future value in a computed template landing position.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum TemplateFuture {
    Realize {
        binding: AuthoredBindingIdentity,
        transform: NameTransform,
    },
    Invoke(AuthoredTransformerIdentity),
    Splice {
        binding: AuthoredBindingIdentity,
    },
}

impl TemplateFuture {
    pub const fn kind(&self) -> TemplateFutureKind {
        match self {
            Self::Realize { .. } => TemplateFutureKind::Realize,
            Self::Invoke(_) => TemplateFutureKind::Invoke,
            Self::Splice { .. } => TemplateFutureKind::Splice,
        }
    }

    pub const fn referenced_binding(&self) -> Option<&AuthoredBindingIdentity> {
        match self {
            Self::Realize { binding, .. } | Self::Splice { binding } => Some(binding),
            Self::Invoke(_) => None,
        }
    }
}

/// The landing shape computed for one semantic source position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemplateLandingShape<Root> {
    /// A vocabulary literal or scalar leaf. These positions are never holes.
    Fixed(LandingShape<Root>),
    /// A declaration or reference value widened to one compatible future.
    ValueOrFuture {
        value: LandingShape<Root>,
        future: TemplateFutureKind,
    },
    /// A nested value whose own addressed declaration is derived recursively.
    Nested(EncodedTypeId<Root>),
    /// A sequence whose literal element is recursively derived and whose item
    /// positions admit invocation or splicing.
    Sequence {
        minimum: u64,
        maximum: Option<u64>,
        element: Box<TemplateLandingShape<Root>>,
        item_futures: Vec<TemplateFutureKind>,
    },
}

impl<Root> TemplateLandingShape<Root> {
    pub fn admits(&self, future: TemplateFutureKind) -> bool {
        match self {
            Self::ValueOrFuture {
                future: admitted, ..
            } => *admitted == future,
            Self::Sequence { item_futures, .. } => item_futures.contains(&future),
            Self::Fixed(_) | Self::Nested(_) => false,
        }
    }
}

/// One structural role in one computed template form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateFieldDeclaration<Root> {
    role: StableRoleId,
    source: SharedDescriptor<Root>,
    landing: Option<TemplateLandingShape<Root>>,
}

impl<Root> TemplateFieldDeclaration<Root> {
    pub const fn role(&self) -> StableRoleId {
        self.role
    }

    pub const fn source(&self) -> &SharedDescriptor<Root> {
        &self.source
    }

    /// `None` identifies textual scaffolding rather than encoded-value data.
    pub const fn landing(&self) -> Option<&TemplateLandingShape<Root>> {
        self.landing.as_ref()
    }
}

/// One accepted source form paired with its computed template landing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateFormDeclaration<Root> {
    identity: Option<DecodeFormId>,
    fields: Vec<TemplateFieldDeclaration<Root>>,
}

impl<Root> TemplateFormDeclaration<Root> {
    /// Decode forms retain their source identity. The canonical encoding form
    /// has no decode-form identity.
    pub const fn identity(&self) -> Option<DecodeFormId> {
        self.identity
    }

    pub fn fields(&self) -> &[TemplateFieldDeclaration<Root>] {
        &self.fields
    }
}

/// Every computed source form for one constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateConstructorDeclaration<Root> {
    constructor: EncodedConstructorId<Root>,
    decode_forms: Vec<TemplateFormDeclaration<Root>>,
    encode_form: TemplateFormDeclaration<Root>,
    landing_fields: Vec<TemplateLandingField<Root>>,
}

impl<Root> TemplateConstructorDeclaration<Root> {
    pub const fn constructor(&self) -> &EncodedConstructorId<Root> {
        &self.constructor
    }

    pub fn decode_forms(&self) -> &[TemplateFormDeclaration<Root>] {
        &self.decode_forms
    }

    pub const fn encode_form(&self) -> &TemplateFormDeclaration<Root> {
        &self.encode_form
    }

    pub fn landing_fields(&self) -> &[TemplateLandingField<Root>] {
        &self.landing_fields
    }
}

/// One semantic field in the computed constructor landing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateLandingField<Root> {
    role: StableRoleId,
    shape: TemplateLandingShape<Root>,
}

impl<Root> TemplateLandingField<Root> {
    pub const fn role(&self) -> StableRoleId {
        self.role
    }

    pub const fn shape(&self) -> &TemplateLandingShape<Root> {
        &self.shape
    }
}

/// Every computed constructor under one addressed source type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateTypeDeclaration<Root> {
    encoded_type: EncodedTypeId<Root>,
    constructors: Vec<TemplateConstructorDeclaration<Root>>,
}

impl<Root> TemplateTypeDeclaration<Root> {
    pub const fn encoded_type(&self) -> &EncodedTypeId<Root> {
        &self.encoded_type
    }

    pub fn constructors(&self) -> &[TemplateConstructorDeclaration<Root>] {
        &self.constructors
    }
}

/// The computed, recursively addressed Template(X) closure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateLanguage<Root> {
    root: EncodedTypeId<Root>,
    types: Vec<TemplateTypeDeclaration<Root>>,
}

impl<Root> TemplateLanguage<Root>
where
    Root: Clone + Ord,
{
    /// Derive one template language from X's grammar and landing declarations.
    ///
    /// The verified closure determines every address visited. There is no
    /// whole-table iteration and no per-X visitor or validator.
    pub fn derive<Record>(
        grammar: &AddressedStructuralTable<Root, Record>,
        landing: &LandingDeclarationCatalog<Root>,
        root: &EncodedTypeId<Root>,
    ) -> Result<Self, TemplateLanguageError<Root>>
    where
        Record: StructureRecord<Root>,
    {
        let declaration = LanguageDeclaration::new(grammar, landing);
        let closure = declaration.verify_root(root)?;
        let mut types = Vec::with_capacity(closure.addressed_types().len());

        for encoded_type in closure.addressed_types() {
            let grammar_entry = grammar.entry(encoded_type).ok_or_else(|| {
                TemplateLanguageError::VerifiedGrammarTypeMissing {
                    encoded_type: encoded_type.clone(),
                }
            })?;
            let landing_type = landing.declaration(encoded_type).ok_or_else(|| {
                TemplateLanguageError::VerifiedLandingTypeMissing {
                    encoded_type: encoded_type.clone(),
                }
            })?;
            let mut constructors = Vec::with_capacity(grammar_entry.constructors().len());

            for codec in grammar_entry.constructors() {
                let landing_constructor = landing_type
                    .constructors()
                    .iter()
                    .find(|candidate| candidate.constructor() == codec.constructor())
                    .ok_or_else(|| TemplateLanguageError::VerifiedConstructorMissing {
                        constructor: codec.constructor().clone(),
                    })?;
                let decode_forms = codec
                    .decode_forms()
                    .iter()
                    .map(|accepted| {
                        derive_form(
                            Some(accepted.identity()),
                            accepted.rule(),
                            landing_constructor,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let encode_form = derive_form(None, codec.encode_form(), landing_constructor)?;
                let landing_fields =
                    derive_landing_fields(codec.encode_form(), landing_constructor)?;
                constructors.push(TemplateConstructorDeclaration {
                    constructor: codec.constructor().clone(),
                    decode_forms,
                    encode_form,
                    landing_fields,
                });
            }

            types.push(TemplateTypeDeclaration {
                encoded_type: encoded_type.clone(),
                constructors,
            });
        }

        Ok(Self {
            root: root.clone(),
            types,
        })
    }

    pub const fn root(&self) -> &EncodedTypeId<Root> {
        &self.root
    }

    pub fn addressed_types(&self) -> &[TemplateTypeDeclaration<Root>] {
        &self.types
    }

    pub fn type_declaration(
        &self,
        expected: &EncodedTypeId<Root>,
    ) -> Option<&TemplateTypeDeclaration<Root>> {
        self.types
            .iter()
            .find(|declaration| declaration.encoded_type() == expected)
    }

    pub fn constructor(
        &self,
        expected: &EncodedConstructorId<Root>,
    ) -> Option<&TemplateConstructorDeclaration<Root>> {
        self.type_declaration(expected.type_id()).and_then(|decl| {
            decl.constructors()
                .iter()
                .find(|constructor| constructor.constructor() == expected)
        })
    }

    /// Validate one declaration-indexed value as this language's root result.
    pub fn validate_value(
        &self,
        value: &TemplateValue<Root>,
    ) -> Result<(), TemplateValueError<Root>> {
        value.validate_as(&self.root, self)
    }
}

fn derive_form<Root, Record>(
    identity: Option<DecodeFormId>,
    record: &Record,
    landing: &LandingConstructorDeclaration<Root>,
) -> Result<TemplateFormDeclaration<Root>, TemplateLanguageError<Root>>
where
    Root: Clone + Ord,
    Record: StructureRecord<Root>,
{
    let fields = collect_descriptors(record)
        .into_iter()
        .map(|(role, source)| {
            let landing_shape = landing
                .fields()
                .iter()
                .find(|field| field.role() == role)
                .map(|field| derive_shape(field.shape(), &source))
                .transpose()?;
            Ok(TemplateFieldDeclaration {
                role,
                source,
                landing: landing_shape,
            })
        })
        .collect::<Result<Vec<_>, TemplateLanguageError<Root>>>()?;
    Ok(TemplateFormDeclaration { identity, fields })
}

fn derive_landing_fields<Root, Record>(
    record: &Record,
    landing: &LandingConstructorDeclaration<Root>,
) -> Result<Vec<TemplateLandingField<Root>>, TemplateLanguageError<Root>>
where
    Root: Clone + Ord,
    Record: StructureRecord<Root>,
{
    let descriptors = collect_descriptors(record);
    landing
        .fields()
        .iter()
        .map(|field| {
            let descriptor = descriptors
                .iter()
                .find(|(role, _)| *role == field.role())
                .map(|(_, descriptor)| descriptor)
                .ok_or_else(|| TemplateLanguageError::VerifiedRoleMissing {
                    constructor: landing.constructor().clone(),
                    role: field.role(),
                })?;
            Ok(TemplateLandingField {
                role: field.role(),
                shape: derive_shape(field.shape(), descriptor)?,
            })
        })
        .collect()
}

fn derive_shape<Root: Clone>(
    landing: &LandingShape<Root>,
    source: &SharedDescriptor<Root>,
) -> Result<TemplateLandingShape<Root>, TemplateLanguageError<Root>> {
    if let SharedDescriptor::Carrier { content, .. } = source {
        return derive_shape(landing, content);
    }
    match (landing, source) {
        (LandingShape::Declaration, SharedDescriptor::Declaration(_)) => {
            Ok(TemplateLandingShape::ValueOrFuture {
                value: LandingShape::Declaration,
                future: TemplateFutureKind::Realize,
            })
        }
        (LandingShape::Reference, SharedDescriptor::Reference(_)) => {
            Ok(TemplateLandingShape::ValueOrFuture {
                value: LandingShape::Reference,
                future: TemplateFutureKind::Invoke,
            })
        }
        (LandingShape::Literal(value), SharedDescriptor::Literal(_)) => Ok(
            TemplateLandingShape::Fixed(LandingShape::Literal(value.clone())),
        ),
        (LandingShape::Scalar(value), SharedDescriptor::Leaf(_)) => Ok(
            TemplateLandingShape::Fixed(LandingShape::Scalar(value.clone())),
        ),
        (LandingShape::Type(target), SharedDescriptor::Delegate { .. }) => {
            Ok(TemplateLandingShape::Nested(target.clone()))
        }
        (
            LandingShape::Sequence {
                minimum,
                maximum,
                element,
            },
            SharedDescriptor::Repeated {
                element: source_element,
                ..
            },
        ) => Ok(TemplateLandingShape::Sequence {
            minimum: *minimum,
            maximum: *maximum,
            element: Box::new(derive_shape(element, source_element)?),
            item_futures: vec![TemplateFutureKind::Invoke, TemplateFutureKind::Splice],
        }),
        _ => Err(TemplateLanguageError::VerifiedShapeDrift),
    }
}

fn collect_descriptors<Root: Clone, Record: StructureRecord<Root>>(
    record: &Record,
) -> Vec<(StableRoleId, SharedDescriptor<Root>)> {
    let mut collector = DescriptorCollector { fields: Vec::new() };
    record.fields().expose(&mut collector);
    collector.fields
}

struct DescriptorCollector<Root> {
    fields: Vec<(StableRoleId, SharedDescriptor<Root>)>,
}

impl<Root: Clone> FieldVisitor<Root> for DescriptorCollector<Root> {
    fn field<Role: FieldRole>(&mut self, position: &Position<Role, Root>) {
        self.fields
            .push((position.role(), position.descriptor().clone()));
    }
}

/// One declaration-indexed template value. The runtime representation is
/// generic over the source language root and has no Rust type per source type.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct TemplateValue<Root> {
    constructor: EncodedConstructorId<Root>,
    fields: Vec<TemplateFieldValue<Root>>,
}

impl<Root: Clone> TemplateValue<Root> {
    pub fn try_new(
        constructor: EncodedConstructorId<Root>,
        mut fields: Vec<TemplateFieldValue<Root>>,
    ) -> Result<Self, TemplateValueError<Root>> {
        fields.sort_by_key(TemplateFieldValue::role);
        for pair in fields.windows(2) {
            if pair[0].role == pair[1].role {
                return Err(TemplateValueError::DuplicateRole {
                    constructor,
                    role: pair[0].role,
                });
            }
        }
        Ok(Self {
            constructor,
            fields,
        })
    }

    pub const fn constructor(&self) -> &EncodedConstructorId<Root> {
        &self.constructor
    }

    pub fn fields(&self) -> &[TemplateFieldValue<Root>] {
        &self.fields
    }

    fn validate_as(
        &self,
        expected_type: &EncodedTypeId<Root>,
        language: &TemplateLanguage<Root>,
    ) -> Result<(), TemplateValueError<Root>>
    where
        Root: Ord,
    {
        if self.constructor.type_id() != expected_type {
            return Err(TemplateValueError::TypeMismatch {
                expected: expected_type.clone(),
                found: self.constructor.type_id().clone(),
            });
        }
        let declaration = language.constructor(&self.constructor).ok_or_else(|| {
            TemplateValueError::UnknownConstructor {
                constructor: self.constructor.clone(),
            }
        })?;

        for expected in declaration.landing_fields() {
            let field = self
                .fields
                .iter()
                .find(|field| field.role == expected.role())
                .ok_or_else(|| TemplateValueError::MissingRole {
                    constructor: self.constructor.clone(),
                    role: expected.role(),
                })?;
            validate_term(
                &field.term,
                expected.shape(),
                &self.constructor,
                expected.role(),
                language,
            )?;
        }
        for field in &self.fields {
            if !declaration
                .landing_fields()
                .iter()
                .any(|expected| expected.role() == field.role)
            {
                return Err(TemplateValueError::ExtraRole {
                    constructor: self.constructor.clone(),
                    role: field.role,
                });
            }
        }
        Ok(())
    }
}

/// One semantic role and its generic template term.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct TemplateFieldValue<Root> {
    role: StableRoleId,
    term: TemplateTerm<Root>,
}

impl<Root> TemplateFieldValue<Root> {
    /// The role must come from a computed declaration; stable roles are
    /// addressing metadata, never authored or emitted field-name identity.
    pub fn new(role: StableRoleId, term: TemplateTerm<Root>) -> Self {
        Self { role, term }
    }

    pub const fn role(&self) -> StableRoleId {
        self.role
    }

    pub const fn term(&self) -> &TemplateTerm<Root> {
        &self.term
    }
}

/// The only runtime landing algebra used for every Template(X) value.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub enum TemplateTerm<Root> {
    Declaration(EncodedId<Root>),
    Reference(EncodedId<Root>),
    Literal(EncodedId<Root>),
    Scalar(ScalarValue),
    Nested(#[rkyv(omit_bounds)] Box<TemplateValue<Root>>),
    Sequence(#[rkyv(omit_bounds)] Vec<TemplateTerm<Root>>),
    Future(TemplateFuture),
}

fn validate_term<Root: Clone + Ord>(
    term: &TemplateTerm<Root>,
    shape: &TemplateLandingShape<Root>,
    constructor: &EncodedConstructorId<Root>,
    role: StableRoleId,
    language: &TemplateLanguage<Root>,
) -> Result<(), TemplateValueError<Root>> {
    match (term, shape) {
        (
            TemplateTerm::Declaration(_),
            TemplateLandingShape::ValueOrFuture {
                value: LandingShape::Declaration,
                ..
            },
        )
        | (
            TemplateTerm::Reference(_),
            TemplateLandingShape::ValueOrFuture {
                value: LandingShape::Reference,
                ..
            },
        ) => Ok(()),
        (
            TemplateTerm::Future(future),
            TemplateLandingShape::ValueOrFuture { future: kind, .. },
        ) if future.kind() == *kind => Ok(()),
        (
            TemplateTerm::Literal(found),
            TemplateLandingShape::Fixed(LandingShape::Literal(expected)),
        ) if found == expected => Ok(()),
        (
            TemplateTerm::Scalar(found),
            TemplateLandingShape::Fixed(LandingShape::Scalar(expected)),
        ) if scalar_kind(found) == expected.clone() => Ok(()),
        (TemplateTerm::Nested(value), TemplateLandingShape::Nested(expected)) => {
            value.validate_as(expected, language)
        }
        (
            TemplateTerm::Sequence(items),
            TemplateLandingShape::Sequence {
                minimum,
                maximum,
                element,
                item_futures,
            },
        ) => {
            let length = items.len() as u64;
            if length < *minimum || maximum.is_some_and(|limit| length > limit) {
                return Err(TemplateValueError::Cardinality {
                    constructor: constructor.clone(),
                    role,
                    minimum: *minimum,
                    maximum: *maximum,
                    found: length,
                });
            }
            for item in items {
                if let TemplateTerm::Future(future) = item
                    && item_futures.contains(&future.kind())
                {
                    continue;
                }
                validate_term(item, element, constructor, role, language)?;
            }
            Ok(())
        }
        (TemplateTerm::Future(future), _) => Err(TemplateValueError::FutureNotAdmitted {
            constructor: constructor.clone(),
            role,
            future: future.kind(),
        }),
        _ => Err(TemplateValueError::TermShape {
            constructor: constructor.clone(),
            role,
        }),
    }
}

fn scalar_kind(value: &ScalarValue) -> structural_codec::LeafCodec {
    match value {
        ScalarValue::Integer(_) => structural_codec::LeafCodec::Integer,
        ScalarValue::Float(_) => structural_codec::LeafCodec::Float,
        ScalarValue::Text(_) => structural_codec::LeafCodec::Text,
        ScalarValue::Boolean(_) => structural_codec::LeafCodec::Boolean,
    }
}

/// Typed failures while deriving the declaration-indexed template language.
#[derive(Clone, Debug, thiserror::Error)]
pub enum TemplateLanguageError<Root> {
    #[error(transparent)]
    Declaration(#[from] LanguageDeclarationError<Root>),
    #[error("verified grammar type disappeared during derivation: {encoded_type:?}")]
    VerifiedGrammarTypeMissing { encoded_type: EncodedTypeId<Root> },
    #[error("verified landing type disappeared during derivation: {encoded_type:?}")]
    VerifiedLandingTypeMissing { encoded_type: EncodedTypeId<Root> },
    #[error("verified constructor disappeared during derivation: {constructor:?}")]
    VerifiedConstructorMissing {
        constructor: EncodedConstructorId<Root>,
    },
    #[error("verified role {role:?} disappeared from {constructor:?} during derivation")]
    VerifiedRoleMissing {
        constructor: EncodedConstructorId<Root>,
        role: StableRoleId,
    },
    #[error("verified grammar/landing shape drifted during derivation")]
    VerifiedShapeDrift,
}

/// Typed failures while constructing or checking a generic template value.
#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum TemplateValueError<Root> {
    #[error("template value repeats role {role:?} under {constructor:?}")]
    DuplicateRole {
        constructor: EncodedConstructorId<Root>,
        role: StableRoleId,
    },
    #[error("template value uses unknown constructor {constructor:?}")]
    UnknownConstructor {
        constructor: EncodedConstructorId<Root>,
    },
    #[error("template value is missing role {role:?} under {constructor:?}")]
    MissingRole {
        constructor: EncodedConstructorId<Root>,
        role: StableRoleId,
    },
    #[error("template value has extra role {role:?} under {constructor:?}")]
    ExtraRole {
        constructor: EncodedConstructorId<Root>,
        role: StableRoleId,
    },
    #[error("template value has type {found:?}; expected {expected:?}")]
    TypeMismatch {
        expected: EncodedTypeId<Root>,
        found: EncodedTypeId<Root>,
    },
    #[error("template value role {role:?} under {constructor:?} has the wrong term shape")]
    TermShape {
        constructor: EncodedConstructorId<Root>,
        role: StableRoleId,
    },
    #[error("template value role {role:?} under {constructor:?} does not admit {future:?}")]
    FutureNotAdmitted {
        constructor: EncodedConstructorId<Root>,
        role: StableRoleId,
        future: TemplateFutureKind,
    },
    #[error(
        "template sequence role {role:?} under {constructor:?} has {found} items; expected {minimum}..{maximum:?}"
    )]
    Cardinality {
        constructor: EncodedConstructorId<Root>,
        role: StableRoleId,
        minimum: u64,
        maximum: Option<u64>,
        found: u64,
    },
}

/// The production root set used by the present Nomos-to-Logos derivation.
pub type LogosTemplateLanguage = TemplateLanguage<VocabularyRoot>;

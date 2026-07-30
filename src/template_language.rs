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

/// The encoded landing value produced by one future.
///
/// This is computed or resolved from declarations. It is never inferred from a
/// transformer spelling, a Logos Rust type, or an evaluator branch.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct TemplateFutureOutput<Root>(LandingShape<Root>);

impl<Root> TemplateFutureOutput<Root> {
    pub fn new(landing: LandingShape<Root>) -> Self {
        Self(landing)
    }

    pub const fn landing(&self) -> &LandingShape<Root> {
        &self.0
    }
}

/// The exact declaration-indexed selector for one transformer's produced
/// fragment.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateRootOutputSelector {
    /// The complete evaluated root value is the produced fragment.
    WholeValue,
    /// A transparent root produces one exact landing field.
    Field(StableRoleId),
}

/// The typed produced-fragment selector and its validated landing shape.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct TemplateRootOutput<Root> {
    selector: TemplateRootOutputSelector,
    output: TemplateFutureOutput<Root>,
}

impl<Root> TemplateRootOutput<Root> {
    pub const fn selector(&self) -> TemplateRootOutputSelector {
        self.selector
    }

    pub const fn output(&self) -> &TemplateFutureOutput<Root> {
        &self.output
    }

    #[cfg(test)]
    pub(crate) fn for_native_admission_test(
        selector: TemplateRootOutputSelector,
        output: TemplateFutureOutput<Root>,
    ) -> Self {
        Self { selector, output }
    }
}

/// One future paired with the output shape required by its computed landing
/// position.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct TemplateFutureRequirement<Root> {
    future: TemplateFuture,
    output: TemplateFutureOutput<Root>,
}

impl<Root> TemplateFutureRequirement<Root> {
    pub const fn future(&self) -> &TemplateFuture {
        &self.future
    }

    pub const fn output(&self) -> &TemplateFutureOutput<Root> {
        &self.output
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
    root_role: StableRoleId,
    fields: Vec<TemplateFieldDeclaration<Root>>,
}

impl<Root> TemplateFormDeclaration<Root> {
    /// Decode forms retain their source identity. The canonical encoding form
    /// has no decode-form identity.
    pub const fn identity(&self) -> Option<DecodeFormId> {
        self.identity
    }

    pub const fn root_role(&self) -> StableRoleId {
        self.root_role
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
        let requirements = self.analyze_value(value)?;
        if let Some(requirement) = requirements.into_iter().next() {
            return Err(TemplateValueError::FutureOutputUnresolved {
                constructor: value.constructor().clone(),
                future: requirement.future,
                expected: requirement.output,
            });
        }
        Ok(())
    }

    /// Validate the declaration-indexed value and return the typed output
    /// requirement for every future it contains.
    ///
    /// Authored Nomos resolves these requirements from input-signature and
    /// within-package transformer declarations before the evaluator can run.
    pub fn analyze_value(
        &self,
        value: &TemplateValue<Root>,
    ) -> Result<Vec<TemplateFutureRequirement<Root>>, TemplateValueError<Root>> {
        let mut requirements = Vec::new();
        value.validate_as(&self.root, self, &mut requirements)?;
        Ok(requirements)
    }

    /// The encoded fragment produced by a transformer whose result uses this
    /// language root.
    ///
    /// Product roots produce one value of their addressed type. Transparent
    /// delimited/item-boundary roots produce the landing carried by their
    /// content role. This is structural metadata, not a language-specific type
    /// switch.
    pub fn root_output(&self) -> Result<TemplateFutureOutput<Root>, TemplateLanguageError<Root>> {
        Ok(self.root_output_contract()?.output)
    }

    /// The exact whole-value or transparent-field selection retained by an
    /// authored declaration for generic invocation substitution.
    pub fn root_output_contract(
        &self,
    ) -> Result<TemplateRootOutput<Root>, TemplateLanguageError<Root>> {
        let declaration = self.type_declaration(&self.root).ok_or_else(|| {
            TemplateLanguageError::VerifiedLandingTypeMissing {
                encoded_type: self.root.clone(),
            }
        })?;
        let mut outputs = declaration.constructors().iter().map(
            |constructor| -> Result<TemplateRootOutput<Root>, TemplateLanguageError<Root>> {
                let root_field = constructor
                    .encode_form()
                    .fields()
                    .iter()
                    .find(|field| field.role() == constructor.encode_form().root_role())
                    .ok_or_else(|| TemplateLanguageError::VerifiedRoleMissing {
                        constructor: constructor.constructor().clone(),
                        role: constructor.encode_form().root_role(),
                    })?;
                let content = match root_field.source() {
                    SharedDescriptor::Delimited { content, .. }
                    | SharedDescriptor::ItemBoundary { content, .. } => Some(*content),
                    _ => None,
                };
                if let Some(content) = content {
                    let landing = constructor
                        .landing_fields()
                        .iter()
                        .find(|field| field.role() == content)
                        .ok_or_else(|| TemplateLanguageError::VerifiedRoleMissing {
                            constructor: constructor.constructor().clone(),
                            role: content,
                        })?;
                    Ok(TemplateRootOutput {
                        selector: TemplateRootOutputSelector::Field(content),
                        output: output_from_template_shape(landing.shape()),
                    })
                } else {
                    Ok(TemplateRootOutput {
                        selector: TemplateRootOutputSelector::WholeValue,
                        output: TemplateFutureOutput::new(LandingShape::Type(self.root.clone())),
                    })
                }
            },
        );
        let first = outputs.next().transpose()?.ok_or_else(|| {
            TemplateLanguageError::RootHasNoConstructor {
                encoded_type: self.root.clone(),
            }
        })?;
        for output in outputs {
            let output = output?;
            if output.output != first.output {
                return Err(TemplateLanguageError::InconsistentRootOutput {
                    encoded_type: self.root.clone(),
                });
            }
            if output.selector != first.selector {
                return Err(TemplateLanguageError::InconsistentRootOutputSelector {
                    encoded_type: self.root.clone(),
                });
            }
        }
        Ok(first)
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
    Ok(TemplateFormDeclaration {
        identity,
        root_role: record.root_role(),
        fields,
    })
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
            Ok(TemplateLandingShape::ValueOrFuture {
                value: LandingShape::Type(target.clone()),
                future: TemplateFutureKind::Realize,
            })
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
        requirements: &mut Vec<TemplateFutureRequirement<Root>>,
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
                requirements,
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
    requirements: &mut Vec<TemplateFutureRequirement<Root>>,
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
            TemplateLandingShape::ValueOrFuture {
                value,
                future: kind,
            },
        ) if future.kind() == *kind => {
            requirements.push(TemplateFutureRequirement {
                future: future.clone(),
                output: TemplateFutureOutput::new(value.clone()),
            });
            Ok(())
        }
        (
            TemplateTerm::Nested(value),
            TemplateLandingShape::ValueOrFuture {
                value: LandingShape::Type(expected),
                ..
            },
        ) => value.validate_as(expected, language, requirements),
        (
            TemplateTerm::Literal(found),
            TemplateLandingShape::Fixed(LandingShape::Literal(expected)),
        ) if found == expected => Ok(()),
        (
            TemplateTerm::Scalar(found),
            TemplateLandingShape::Fixed(LandingShape::Scalar(expected)),
        ) if scalar_kind(found) == expected.clone() => Ok(()),
        (TemplateTerm::Nested(value), TemplateLandingShape::Nested(expected)) => {
            value.validate_as(expected, language, requirements)
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
                    requirements.push(TemplateFutureRequirement {
                        future: future.clone(),
                        output: output_from_template_shape(shape),
                    });
                    continue;
                }
                validate_term(item, element, constructor, role, language, requirements)?;
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

fn output_from_template_shape<Root: Clone>(
    shape: &TemplateLandingShape<Root>,
) -> TemplateFutureOutput<Root> {
    TemplateFutureOutput::new(literal_landing(shape))
}

fn literal_landing<Root: Clone>(shape: &TemplateLandingShape<Root>) -> LandingShape<Root> {
    match shape {
        TemplateLandingShape::Fixed(landing)
        | TemplateLandingShape::ValueOrFuture { value: landing, .. } => landing.clone(),
        TemplateLandingShape::Nested(target) => LandingShape::Type(target.clone()),
        TemplateLandingShape::Sequence {
            minimum,
            maximum,
            element,
            ..
        } => LandingShape::sequence(*minimum, *maximum, literal_landing(element)),
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
    #[error("template root {encoded_type:?} has no constructor")]
    RootHasNoConstructor { encoded_type: EncodedTypeId<Root> },
    #[error("template root {encoded_type:?} has constructors with incompatible outputs")]
    InconsistentRootOutput { encoded_type: EncodedTypeId<Root> },
    #[error("template root {encoded_type:?} has constructors with incompatible output selectors")]
    InconsistentRootOutputSelector { encoded_type: EncodedTypeId<Root> },
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
        "template value under {constructor:?} contains unresolved {future:?} output; expected {expected:?}"
    )]
    FutureOutputUnresolved {
        constructor: EncodedConstructorId<Root>,
        future: TemplateFuture,
        expected: TemplateFutureOutput<Root>,
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

#[cfg(test)]
pub(crate) fn nested_text_value_for_native_admission_test() -> TemplateValue<VocabularyRoot> {
    use encoded_name_table::LocalEncodedId;
    use structural_codec::{LeafCodec, Position};

    #[derive(rkyv::Archive)]
    struct OuterRole;
    impl FieldRole for OuterRole {
        const STABLE_ID: u16 = 31_001;
    }

    #[derive(rkyv::Archive)]
    struct InnerRole;
    impl FieldRole for InnerRole {
        const STABLE_ID: u16 = 31_002;
    }

    fn encoded(chain: &[u16]) -> EncodedId<VocabularyRoot> {
        EncodedId::new(
            VocabularyRoot::Universal,
            chain.iter().copied().map(LocalEncodedId::new).collect(),
        )
        .expect("non-empty native admission test identity")
    }

    let inner_type = EncodedTypeId::new(encoded(&[31, 2]));
    let inner_role = Position::<InnerRole, VocabularyRoot>::try_new(SharedDescriptor::<
        VocabularyRoot,
    >::Leaf(LeafCodec::Text))
    .expect("nonzero inner role")
    .role();
    let inner = TemplateValue::try_new(
        EncodedConstructorId::under(&inner_type, 1),
        vec![TemplateFieldValue::new(
            inner_role,
            TemplateTerm::Scalar(ScalarValue::Text("forbidden".to_owned())),
        )],
    )
    .expect("one inner role");

    let outer_type = EncodedTypeId::new(encoded(&[31, 1]));
    let outer_role = Position::<OuterRole, _>::try_new(SharedDescriptor::Repeated {
        minimum: 0,
        maximum: None,
        element: Box::new(SharedDescriptor::Delegate {
            target: inner_type,
            payload: None,
        }),
    })
    .expect("nonzero outer role")
    .role();
    TemplateValue::try_new(
        EncodedConstructorId::under(&outer_type, 1),
        vec![TemplateFieldValue::new(
            outer_role,
            TemplateTerm::Sequence(vec![TemplateTerm::Nested(Box::new(inner))]),
        )],
    )
    .expect("one outer role")
}

#[cfg(test)]
pub(crate) fn restore_validation_value_for_native_test() -> TemplateValue<VocabularyRoot> {
    use encoded_name_table::LocalEncodedId;
    use structural_codec::{LeafCodec, Position};

    #[derive(rkyv::Archive)]
    struct DeclarationRole;
    impl FieldRole for DeclarationRole {
        const STABLE_ID: u16 = 31_011;
    }

    #[derive(rkyv::Archive)]
    struct BooleanRole;
    impl FieldRole for BooleanRole {
        const STABLE_ID: u16 = 31_012;
    }

    fn encoded(chain: &[u16]) -> EncodedId<VocabularyRoot> {
        EncodedId::new(
            VocabularyRoot::Universal,
            chain.iter().copied().map(LocalEncodedId::new).collect(),
        )
        .expect("non-empty native restore test identity")
    }

    let encoded_type = EncodedTypeId::new(encoded(&[31, 10]));
    let declaration_role = Position::<DeclarationRole, VocabularyRoot>::try_new(
        SharedDescriptor::Leaf(LeafCodec::Boolean),
    )
    .expect("nonzero declaration role")
    .role();
    let boolean_role = Position::<BooleanRole, VocabularyRoot>::try_new(SharedDescriptor::Leaf(
        LeafCodec::Boolean,
    ))
    .expect("nonzero boolean role")
    .role();
    TemplateValue::try_new(
        EncodedConstructorId::under(&encoded_type, 1),
        vec![
            TemplateFieldValue::new(
                declaration_role,
                TemplateTerm::Declaration(encoded(&[31, 11])),
            ),
            TemplateFieldValue::new(
                boolean_role,
                TemplateTerm::Scalar(ScalarValue::Boolean(true)),
            ),
        ],
    )
    .expect("two distinct native restore roles")
}

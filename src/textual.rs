//! The plain-Standard TextualNomos base door.
//!
//! Raw discovery and the shared structural evaluator decode one six-slot Nomos
//! document. The Logos-shaped result positions are not described by a second
//! handwritten grammar: `DerivedTemplateRecord` mechanically lifts the
//! existing Logos records with the fixed value-or-future descriptor algebra.
//! Transformer and binding declarations consume caller-supplied assignments.
//! Every future payload is a `Reference` descriptor and therefore uses the
//! lookup-only half of [`DecodeNameBindings`].

use std::collections::BTreeSet;

use core_logos::{LogosLanguage, LogosRule};
use raw_discovery::{
    BlockTreeDiscoveryConfiguration, BoundaryDiscoveryConfiguration, BoundaryDiscoveryContext,
    BoundaryDiscoveryContextIdentifier, BoundaryDiscoveryTransition, RawProfile,
    SealedTokenProfile, TriggerIdentifier, TriggerSet,
};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::{
    AcceptedDecodeForm, AddressedStructuralTable, AtomDescriptor, BorrowedFieldView,
    ConstructorCodec, ContextualTextualPolicy, DecodeFormId, DecodeNameBindings,
    EncodedConstructorId, EncodedNameResolver, EncodedTypeId, FieldEnd, FieldLink, FieldRole,
    FieldValue, FieldVisitor, LeafCodec, OrderedProduct, OrderedSequence, Position, RuleCoproduct,
    SharedDescriptor, StableRoleId, StructuralEntry, StructuralEvaluator, StructuralRule,
    StructuralRuleView, StructuralValue, StructuralVocabularyIdentity, StructureRecord,
    TableIdentityPayload, TargetLayoutIdentity, TextualRenderingPolicy, UnaryRule,
};

use crate::{
    AuthoredBindingIdentity, AuthoredInputParameter, AuthoredInputSignature, AuthoredNomosError,
    AuthoredTransformerDeclaration, AuthoredTransformerIdentity, AuthoredTransformerSet, MacroKind,
    MetaType, NameTransform, SectionDefault, TemplateFuture, TemplateFutureOutput,
    TemplateLandingShape, TemplateLanguage, TemplateLanguageError, TemplateTerm, TemplateValue,
};

const PARENTHESIS: TriggerIdentifier = TriggerIdentifier::new(0);
const SQUARE: TriggerIdentifier = TriggerIdentifier::new(1);
const BRACE: TriggerIdentifier = TriggerIdentifier::new(2);
const APPLICATION: TriggerIdentifier = TriggerIdentifier::new(3);
const CARRIER: TriggerIdentifier = TriggerIdentifier::new(4);
const WHITESPACE: TriggerIdentifier = TriggerIdentifier::new(5);
const COMMENT: TriggerIdentifier = TriggerIdentifier::new(6);
const ROOT_CONTEXT: BoundaryDiscoveryContextIdentifier = BoundaryDiscoveryContextIdentifier::new(1);
const FORM: DecodeFormId = DecodeFormId::new(1);

/// Translator-issued identities for the fixed Nomos structural types.
#[derive(Clone, Debug)]
pub struct TextualNomosTypeIds {
    pub document: VocabularyEncodedId,
    pub revision: VocabularyEncodedId,
    pub empty_braces: VocabularyEncodedId,
    pub empty_square: VocabularyEncodedId,
    pub transformers: VocabularyEncodedId,
    pub transformer: VocabularyEncodedId,
    pub input_signature: VocabularyEncodedId,
    pub input_parameter: VocabularyEncodedId,
    pub newtype_body: VocabularyEncodedId,
    pub enumeration_body: VocabularyEncodedId,
    pub attributes_body: VocabularyEncodedId,
}

/// Reserved fixed words used by the base-door grammar.
#[derive(Clone, Debug)]
pub struct TextualNomosWords {
    pub named: VocabularyEncodedId,
    pub structural: VocabularyEncodedId,
    pub newtype: VocabularyEncodedId,
    pub enumeration: VocabularyEncodedId,
    pub realize: VocabularyEncodedId,
    pub splice: VocabularyEncodedId,
    pub invoke: VocabularyEncodedId,
}

/// One Nomos input meta-type word paired with its declaration-supplied output.
#[derive(Clone, Debug)]
pub struct TextualNomosMetaType {
    pub word: VocabularyEncodedId,
    pub meta: MetaType,
    pub output: TemplateFutureOutput<VocabularyRoot>,
}

/// A decoded plain-NOTA document and its typed structural mirror.
///
/// The mirror contains encoded data and source-independent identities, never a
/// retained source string. It permits canonical viewing through the same table.
#[derive(Clone, Debug)]
pub struct DecodedNomosDocument {
    revision: i64,
    transformers: AuthoredTransformerSet,
    mirror: StructuralValue<VocabularyRoot>,
}

impl DecodedNomosDocument {
    pub const fn revision(&self) -> i64 {
        self.revision
    }

    pub const fn transformers(&self) -> &AuthoredTransformerSet {
        &self.transformers
    }

    pub const fn structural_value(&self) -> &StructuralValue<VocabularyRoot> {
        &self.mirror
    }
}

/// Typed construction and decoding failures for the TextualNomos base door.
#[derive(Debug, thiserror::Error)]
pub enum TextualNomosError {
    #[error(transparent)]
    Profile(#[from] raw_discovery::TokenProfileError),
    #[error(transparent)]
    Authoring(#[from] structural_codec::AuthoringError),
    #[error(transparent)]
    Table(Box<structural_codec::TableError<VocabularyRoot>>),
    #[error(transparent)]
    Decode(Box<structural_codec::DecodeError<VocabularyRoot>>),
    #[error(transparent)]
    Encode(Box<structural_codec::EncodeError<VocabularyRoot>>),
    #[error(transparent)]
    Template(Box<TemplateLanguageError<VocabularyRoot>>),
    #[error(transparent)]
    TemplateValue(Box<crate::TemplateValueError<VocabularyRoot>>),
    #[error(transparent)]
    Authored(#[from] AuthoredNomosError),
    #[error("{position} identity belongs to {found:?}; expected Universal")]
    WrongRoot {
        position: &'static str,
        found: VocabularyRoot,
    },
    #[error("TextualNomos meta declarations repeat {meta:?}")]
    DuplicateMetaType { meta: MetaType },
    #[error("TextualNomos meta declarations repeat word {word:?}")]
    DuplicateMetaWord { word: VocabularyEncodedId },
    #[error(
        "TextualNomos has {count} meta declarations; the encoded constructor space admits at most {maximum}"
    )]
    TooManyMetaTypes { count: usize, maximum: u16 },
    #[error("decoded structural value is missing typed role {role}")]
    MissingRole { role: &'static str },
    #[error("decoded structural value has the wrong shape at role {role}")]
    WrongFieldShape { role: &'static str },
    #[error("decoded structural value is missing computed landing role {role:?}")]
    MissingComputedRole { role: StableRoleId },
    #[error("decoded transformer selected an unknown constructor {constructor:?}")]
    UnknownTransformerForm {
        constructor: EncodedConstructorId<VocabularyRoot>,
    },
    #[error("decoded input parameter selected an unknown constructor {constructor:?}")]
    UnknownMetaForm {
        constructor: EncodedConstructorId<VocabularyRoot>,
    },
    #[error("template value selected an unknown future keyword {keyword:?}")]
    UnknownFutureKeyword { keyword: VocabularyEncodedId },
    #[error("template future payload is not a lookup-only encoded reference")]
    InvalidFuturePayload,
    #[error("decoded revision is not an integer")]
    InvalidRevision,
}

impl From<structural_codec::TableError<VocabularyRoot>> for TextualNomosError {
    fn from(error: structural_codec::TableError<VocabularyRoot>) -> Self {
        Self::Table(Box::new(error))
    }
}

impl From<structural_codec::DecodeError<VocabularyRoot>> for TextualNomosError {
    fn from(error: structural_codec::DecodeError<VocabularyRoot>) -> Self {
        Self::Decode(Box::new(error))
    }
}

impl From<structural_codec::EncodeError<VocabularyRoot>> for TextualNomosError {
    fn from(error: structural_codec::EncodeError<VocabularyRoot>) -> Self {
        Self::Encode(Box::new(error))
    }
}

impl From<TemplateLanguageError<VocabularyRoot>> for TextualNomosError {
    fn from(error: TemplateLanguageError<VocabularyRoot>) -> Self {
        Self::Template(Box::new(error))
    }
}

impl From<crate::TemplateValueError<VocabularyRoot>> for TextualNomosError {
    fn from(error: crate::TemplateValueError<VocabularyRoot>) -> Self {
        Self::TemplateValue(Box::new(error))
    }
}

#[derive(Clone, Debug)]
struct EncodedTypes {
    document: EncodedTypeId<VocabularyRoot>,
    revision: EncodedTypeId<VocabularyRoot>,
    empty_braces: EncodedTypeId<VocabularyRoot>,
    empty_square: EncodedTypeId<VocabularyRoot>,
    transformers: EncodedTypeId<VocabularyRoot>,
    transformer: EncodedTypeId<VocabularyRoot>,
    input_signature: EncodedTypeId<VocabularyRoot>,
    input_parameter: EncodedTypeId<VocabularyRoot>,
    newtype_body: EncodedTypeId<VocabularyRoot>,
    enumeration_body: EncodedTypeId<VocabularyRoot>,
    attributes_body: EncodedTypeId<VocabularyRoot>,
}

impl TextualNomosTypeIds {
    fn encoded(&self) -> Result<EncodedTypes, TextualNomosError> {
        for (position, encoded_id) in [
            ("document type", &self.document),
            ("revision type", &self.revision),
            ("empty-braces type", &self.empty_braces),
            ("empty-square type", &self.empty_square),
            ("transformers type", &self.transformers),
            ("transformer type", &self.transformer),
            ("input-signature type", &self.input_signature),
            ("input-parameter type", &self.input_parameter),
            ("newtype-body type", &self.newtype_body),
            ("enumeration-body type", &self.enumeration_body),
            ("attributes-body type", &self.attributes_body),
        ] {
            require_universal(position, encoded_id)?;
        }
        Ok(EncodedTypes {
            document: EncodedTypeId::new(self.document.clone()),
            revision: EncodedTypeId::new(self.revision.clone()),
            empty_braces: EncodedTypeId::new(self.empty_braces.clone()),
            empty_square: EncodedTypeId::new(self.empty_square.clone()),
            transformers: EncodedTypeId::new(self.transformers.clone()),
            transformer: EncodedTypeId::new(self.transformer.clone()),
            input_signature: EncodedTypeId::new(self.input_signature.clone()),
            input_parameter: EncodedTypeId::new(self.input_parameter.clone()),
            newtype_body: EncodedTypeId::new(self.newtype_body.clone()),
            enumeration_body: EncodedTypeId::new(self.enumeration_body.clone()),
            attributes_body: EncodedTypeId::new(self.attributes_body.clone()),
        })
    }
}

fn require_universal(
    position: &'static str,
    encoded_id: &VocabularyEncodedId,
) -> Result<(), TextualNomosError> {
    let found = *encoded_id.root_variant();
    if found != VocabularyRoot::Universal {
        return Err(TextualNomosError::WrongRoot { position, found });
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct TransformerForm {
    constructor: EncodedConstructorId<VocabularyRoot>,
    kind: MacroKind,
    template_root: EncodedTypeId<VocabularyRoot>,
}

#[derive(Clone, Debug)]
struct MetaForm {
    constructor: EncodedConstructorId<VocabularyRoot>,
    meta: MetaType,
    output: TemplateFutureOutput<VocabularyRoot>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct FutureSyntax {
    realize: VocabularyEncodedId,
    splice: VocabularyEncodedId,
    invoke: VocabularyEncodedId,
}

/// The one fixed record wrapper used for every addressed type in Template(X).
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct DerivedTemplateRecord<Record> {
    source: Record,
    syntax: FutureSyntax,
}

struct DerivedTemplateView<'record, View> {
    source: View,
    syntax: &'record FutureSyntax,
}

impl<View: BorrowedFieldView<VocabularyRoot>> BorrowedFieldView<VocabularyRoot>
    for DerivedTemplateView<'_, View>
{
    fn expose<Visitor: FieldVisitor<VocabularyRoot>>(&self, visitor: &mut Visitor) {
        let mut lifted = LiftedVisitor {
            target: visitor,
            syntax: self.syntax,
        };
        self.source.expose(&mut lifted);
    }
}

struct LiftedVisitor<'target, 'syntax, Visitor> {
    target: &'target mut Visitor,
    syntax: &'syntax FutureSyntax,
}

impl<Visitor: FieldVisitor<VocabularyRoot>> FieldVisitor<VocabularyRoot>
    for LiftedVisitor<'_, '_, Visitor>
{
    fn field<Role: FieldRole>(&mut self, position: &Position<Role, VocabularyRoot>) {
        let lifted: Position<Role, VocabularyRoot> =
            Position::try_new(lift_descriptor(position.descriptor(), self.syntax))
                .expect("the source record already proved this non-zero stable role");
        self.target.field(&lifted);
    }
}

impl<Record: StructureRecord<VocabularyRoot>> StructureRecord<VocabularyRoot>
    for DerivedTemplateRecord<Record>
{
    type View<'record>
        = DerivedTemplateView<'record, Record::View<'record>>
    where
        Self: 'record;

    fn root_role(&self) -> StableRoleId {
        self.source.root_role()
    }

    fn fields(&self) -> Self::View<'_> {
        DerivedTemplateView {
            source: self.source.fields(),
            syntax: &self.syntax,
        }
    }
}

fn inline_future(keyword: &VocabularyEncodedId) -> SharedDescriptor<VocabularyRoot> {
    SharedDescriptor::InlineApplication {
        operator: APPLICATION,
        head: Box::new(SharedDescriptor::Literal(keyword.clone())),
        payload: Box::new(SharedDescriptor::Reference(AtomDescriptor::any_case())),
    }
}

fn alternation(
    alternatives: impl IntoIterator<Item = SharedDescriptor<VocabularyRoot>>,
) -> SharedDescriptor<VocabularyRoot> {
    let mut unique = Vec::new();
    for alternative in alternatives {
        if !unique.contains(&alternative) {
            unique.push(alternative);
        }
    }
    SharedDescriptor::Alternation(unique)
}

fn lift_descriptor(
    source: &SharedDescriptor<VocabularyRoot>,
    syntax: &FutureSyntax,
) -> SharedDescriptor<VocabularyRoot> {
    let reserved = || {
        vec![
            syntax.realize.clone(),
            syntax.splice.clone(),
            syntax.invoke.clone(),
        ]
    };
    match source {
        SharedDescriptor::Declaration(atom) => alternation([
            inline_future(&syntax.realize),
            SharedDescriptor::DeclarationExcluding {
                atom: atom.clone(),
                excluded: reserved(),
            },
        ]),
        SharedDescriptor::Reference(atom) => alternation([
            inline_future(&syntax.invoke),
            SharedDescriptor::ReferenceExcluding {
                atom: atom.clone(),
                excluded: reserved(),
            },
        ]),
        SharedDescriptor::DeclarationExcluding { atom, excluded } => {
            let mut excluded = excluded.clone();
            excluded.extend(reserved());
            excluded.sort();
            excluded.dedup();
            alternation([
                inline_future(&syntax.realize),
                SharedDescriptor::DeclarationExcluding {
                    atom: atom.clone(),
                    excluded,
                },
            ])
        }
        SharedDescriptor::ReferenceExcluding { atom, excluded } => {
            let mut excluded = excluded.clone();
            excluded.extend(reserved());
            excluded.sort();
            excluded.dedup();
            alternation([
                inline_future(&syntax.invoke),
                SharedDescriptor::ReferenceExcluding {
                    atom: atom.clone(),
                    excluded,
                },
            ])
        }
        SharedDescriptor::Delegate { .. } => {
            alternation([inline_future(&syntax.realize), source.clone()])
        }
        SharedDescriptor::Carrier { carrier, content } => SharedDescriptor::Carrier {
            carrier: *carrier,
            content: Box::new(lift_descriptor(content, syntax)),
        },
        SharedDescriptor::Repeated {
            minimum,
            maximum,
            element,
        } => SharedDescriptor::Repeated {
            minimum: *minimum,
            maximum: *maximum,
            element: Box::new(alternation([
                inline_future(&syntax.invoke),
                inline_future(&syntax.splice),
                lift_descriptor(element, syntax),
            ])),
        },
        SharedDescriptor::InlineApplication {
            operator,
            head,
            payload,
        } => SharedDescriptor::InlineApplication {
            operator: *operator,
            head: Box::new(lift_descriptor(head, syntax)),
            payload: Box::new(lift_descriptor(payload, syntax)),
        },
        SharedDescriptor::Alternation(alternatives) => alternation(
            alternatives
                .iter()
                .map(|alternative| lift_descriptor(alternative, syntax)),
        ),
        SharedDescriptor::Literal(_)
        | SharedDescriptor::Leaf(_)
        | SharedDescriptor::OrderedProduct(_)
        | SharedDescriptor::OrderedSequence(_)
        | SharedDescriptor::Application { .. }
        | SharedDescriptor::Delimited { .. }
        | SharedDescriptor::ItemBoundary { .. } => source.clone(),
    }
}

macro_rules! role {
    ($name:ident, $id:expr) => {
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
        struct $name;

        impl FieldRole for $name {
            const STABLE_ID: u16 = $id;
        }
    };
}

role!(DocumentRoot, 2000);
role!(DocumentRevision, 2001);
role!(DocumentInputs, 2002);
role!(DocumentOutputs, 2003);
role!(DocumentTransformers, 2004);
role!(DocumentSelection, 2005);
role!(DocumentCapsule, 2006);
role!(RevisionRoot, 2010);
role!(RevisionValue, 2011);
role!(DelimitedRoot, 2020);
role!(DelimitedItems, 2021);
role!(TransformerRoot, 2030);
role!(TransformerHeader, 2031);
role!(TransformerBody, 2032);
role!(BodyRoot, 2040);
role!(BodyContent, 2041);
role!(BodyInput, 2042);
role!(BodyResult, 2043);

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct DocumentRecord {
    root: Position<DocumentRoot, VocabularyRoot>,
    revision: Position<DocumentRevision, VocabularyRoot>,
    inputs: Position<DocumentInputs, VocabularyRoot>,
    outputs: Position<DocumentOutputs, VocabularyRoot>,
    transformers: Position<DocumentTransformers, VocabularyRoot>,
    selection: Position<DocumentSelection, VocabularyRoot>,
    capsule: Position<DocumentCapsule, VocabularyRoot>,
}

impl DocumentRecord {
    fn new(types: &EncodedTypes) -> Result<Self, structural_codec::AuthoringError> {
        let root = OrderedProduct::try_new::<DocumentRevision>()?
            .then::<DocumentInputs>()?
            .then::<DocumentOutputs>()?
            .then::<DocumentTransformers>()?
            .then::<DocumentSelection>()?
            .then::<DocumentCapsule>()?;
        let delegate = |target: &EncodedTypeId<VocabularyRoot>| SharedDescriptor::Delegate {
            target: target.clone(),
            payload: None,
        };
        Ok(Self {
            root: Position::try_new(SharedDescriptor::OrderedProduct(root))?,
            revision: Position::try_new(delegate(&types.revision))?,
            inputs: Position::try_new(delegate(&types.empty_square))?,
            outputs: Position::try_new(delegate(&types.empty_square))?,
            transformers: Position::try_new(delegate(&types.transformers))?,
            selection: Position::try_new(delegate(&types.empty_braces))?,
            capsule: Position::try_new(delegate(&types.empty_braces))?,
        })
    }
}

struct DocumentView<'record> {
    record: &'record DocumentRecord,
}

impl BorrowedFieldView<VocabularyRoot> for DocumentView<'_> {
    fn expose<Visitor: FieldVisitor<VocabularyRoot>>(&self, visitor: &mut Visitor) {
        visitor.field(&self.record.root);
        visitor.field(&self.record.revision);
        visitor.field(&self.record.inputs);
        visitor.field(&self.record.outputs);
        visitor.field(&self.record.transformers);
        visitor.field(&self.record.selection);
        visitor.field(&self.record.capsule);
    }
}

impl StructureRecord<VocabularyRoot> for DocumentRecord {
    type View<'record> = DocumentView<'record>;

    fn root_role(&self) -> StableRoleId {
        self.root.role()
    }

    fn fields(&self) -> Self::View<'_> {
        DocumentView { record: self }
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct RevisionRecord {
    root: Position<RevisionRoot, VocabularyRoot>,
    value: Position<RevisionValue, VocabularyRoot>,
}

impl RevisionRecord {
    fn new() -> Result<Self, structural_codec::AuthoringError> {
        let value = Position::try_new(SharedDescriptor::Leaf(LeafCodec::Integer))?;
        Ok(Self {
            root: Position::try_new(SharedDescriptor::Delimited {
                boundary: BRACE,
                content: value.role(),
            })?,
            value,
        })
    }
}

impl StructureRecord<VocabularyRoot> for RevisionRecord {
    type View<'record> = FieldLink<
        'record,
        RevisionRoot,
        VocabularyRoot,
        FieldLink<'record, RevisionValue, VocabularyRoot, FieldEnd>,
    >;

    fn root_role(&self) -> StableRoleId {
        self.root.role()
    }

    fn fields(&self) -> Self::View<'_> {
        FieldLink::new(&self.root, FieldLink::new(&self.value, FieldEnd))
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct DelimitedItemsRecord {
    root: Position<DelimitedRoot, VocabularyRoot>,
    items: Position<DelimitedItems, VocabularyRoot>,
}

impl DelimitedItemsRecord {
    fn new(
        boundary: TriggerIdentifier,
        item: &EncodedTypeId<VocabularyRoot>,
        minimum: u64,
        maximum: Option<u64>,
    ) -> Result<Self, structural_codec::AuthoringError> {
        let items = Position::try_new(SharedDescriptor::Repeated {
            minimum,
            maximum,
            element: Box::new(SharedDescriptor::Delegate {
                target: item.clone(),
                payload: None,
            }),
        })?;
        Ok(Self {
            root: Position::try_new(SharedDescriptor::Delimited {
                boundary,
                content: items.role(),
            })?,
            items,
        })
    }
}

impl StructureRecord<VocabularyRoot> for DelimitedItemsRecord {
    type View<'record> = FieldLink<
        'record,
        DelimitedRoot,
        VocabularyRoot,
        FieldLink<'record, DelimitedItems, VocabularyRoot, FieldEnd>,
    >;

    fn root_role(&self) -> StableRoleId {
        self.root.role()
    }

    fn fields(&self) -> Self::View<'_> {
        FieldLink::new(&self.root, FieldLink::new(&self.items, FieldEnd))
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct TransformerRecord {
    root: Position<TransformerRoot, VocabularyRoot>,
    header: Position<TransformerHeader, VocabularyRoot>,
    body: Position<TransformerBody, VocabularyRoot>,
}

impl TransformerRecord {
    fn new(
        kind: SharedDescriptor<VocabularyRoot>,
        body: &EncodedTypeId<VocabularyRoot>,
    ) -> Result<Self, structural_codec::AuthoringError> {
        let root = OrderedSequence::try_new::<TransformerHeader>()?.then::<TransformerBody>()?;
        Ok(Self {
            root: Position::try_new(SharedDescriptor::OrderedSequence(root))?,
            header: Position::try_new(SharedDescriptor::InlineApplication {
                operator: APPLICATION,
                head: Box::new(SharedDescriptor::Declaration(AtomDescriptor::any_case())),
                payload: Box::new(kind),
            })?,
            body: Position::try_new(SharedDescriptor::Delegate {
                target: body.clone(),
                payload: None,
            })?,
        })
    }
}

impl StructureRecord<VocabularyRoot> for TransformerRecord {
    type View<'record> = FieldLink<
        'record,
        TransformerRoot,
        VocabularyRoot,
        FieldLink<
            'record,
            TransformerHeader,
            VocabularyRoot,
            FieldLink<'record, TransformerBody, VocabularyRoot, FieldEnd>,
        >,
    >;

    fn root_role(&self) -> StableRoleId {
        self.root.role()
    }

    fn fields(&self) -> Self::View<'_> {
        FieldLink::new(
            &self.root,
            FieldLink::new(&self.header, FieldLink::new(&self.body, FieldEnd)),
        )
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct BodyRecord {
    root: Position<BodyRoot, VocabularyRoot>,
    content: Position<BodyContent, VocabularyRoot>,
    input: Position<BodyInput, VocabularyRoot>,
    result: Position<BodyResult, VocabularyRoot>,
}

impl BodyRecord {
    fn new(
        input: &EncodedTypeId<VocabularyRoot>,
        result: &EncodedTypeId<VocabularyRoot>,
    ) -> Result<Self, structural_codec::AuthoringError> {
        let content = OrderedSequence::try_new::<BodyInput>()?.then::<BodyResult>()?;
        let content = Position::try_new(SharedDescriptor::OrderedSequence(content))?;
        Ok(Self {
            root: Position::try_new(SharedDescriptor::Delimited {
                boundary: BRACE,
                content: content.role(),
            })?,
            content,
            input: Position::try_new(SharedDescriptor::Delegate {
                target: input.clone(),
                payload: None,
            })?,
            result: Position::try_new(SharedDescriptor::Delegate {
                target: result.clone(),
                payload: None,
            })?,
        })
    }
}

struct BodyView<'record> {
    record: &'record BodyRecord,
}

impl BorrowedFieldView<VocabularyRoot> for BodyView<'_> {
    fn expose<Visitor: FieldVisitor<VocabularyRoot>>(&self, visitor: &mut Visitor) {
        visitor.field(&self.record.root);
        visitor.field(&self.record.content);
        visitor.field(&self.record.input);
        visitor.field(&self.record.result);
    }
}

impl StructureRecord<VocabularyRoot> for BodyRecord {
    type View<'record> = BodyView<'record>;

    fn root_role(&self) -> StableRoleId {
        self.root.role()
    }

    fn fields(&self) -> Self::View<'_> {
        BodyView { record: self }
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
enum NomosRule {
    Document(DocumentRecord),
    Revision(RevisionRecord),
    Delimited(DelimitedItemsRecord),
    Transformer(TransformerRecord),
    Body(BodyRecord),
    Structural(StructuralRule<VocabularyRoot>),
}

type RevisionView<'record> = FieldLink<
    'record,
    RevisionRoot,
    VocabularyRoot,
    FieldLink<'record, RevisionValue, VocabularyRoot, FieldEnd>,
>;
type DelimitedView<'record> = FieldLink<
    'record,
    DelimitedRoot,
    VocabularyRoot,
    FieldLink<'record, DelimitedItems, VocabularyRoot, FieldEnd>,
>;
type TransformerView<'record> = FieldLink<
    'record,
    TransformerRoot,
    VocabularyRoot,
    FieldLink<
        'record,
        TransformerHeader,
        VocabularyRoot,
        FieldLink<'record, TransformerBody, VocabularyRoot, FieldEnd>,
    >,
>;

enum NomosRuleView<'record> {
    Document(DocumentView<'record>),
    Revision(RevisionView<'record>),
    Delimited(DelimitedView<'record>),
    Transformer(TransformerView<'record>),
    Body(BodyView<'record>),
    Structural(StructuralRuleView<'record, VocabularyRoot>),
}

impl BorrowedFieldView<VocabularyRoot> for NomosRuleView<'_> {
    fn expose<Visitor: FieldVisitor<VocabularyRoot>>(&self, visitor: &mut Visitor) {
        match self {
            Self::Document(view) => view.expose(visitor),
            Self::Revision(view) => view.expose(visitor),
            Self::Delimited(view) => view.expose(visitor),
            Self::Transformer(view) => view.expose(visitor),
            Self::Body(view) => view.expose(visitor),
            Self::Structural(view) => view.expose(visitor),
        }
    }
}

impl StructureRecord<VocabularyRoot> for NomosRule {
    type View<'record> = NomosRuleView<'record>;

    fn root_role(&self) -> StableRoleId {
        match self {
            Self::Document(record) => record.root_role(),
            Self::Revision(record) => record.root_role(),
            Self::Delimited(record) => record.root_role(),
            Self::Transformer(record) => record.root_role(),
            Self::Body(record) => record.root_role(),
            Self::Structural(record) => StructureRecord::root_role(record),
        }
    }

    fn fields(&self) -> Self::View<'_> {
        match self {
            Self::Document(record) => NomosRuleView::Document(record.fields()),
            Self::Revision(record) => NomosRuleView::Revision(record.fields()),
            Self::Delimited(record) => NomosRuleView::Delimited(record.fields()),
            Self::Transformer(record) => NomosRuleView::Transformer(record.fields()),
            Self::Body(record) => NomosRuleView::Body(record.fields()),
            Self::Structural(record) => NomosRuleView::Structural(record.fields()),
        }
    }
}

type TextualNomosRule = RuleCoproduct<NomosRule, DerivedTemplateRecord<LogosRule>>;

/// The sealed Standard-profile TextualNomos base door.
pub struct TextualNomos {
    table: AddressedStructuralTable<VocabularyRoot, TextualNomosRule>,
    document: EncodedTypeId<VocabularyRoot>,
    newtype: TemplateLanguage<VocabularyRoot>,
    enumeration: TemplateLanguage<VocabularyRoot>,
    attributes: TemplateLanguage<VocabularyRoot>,
    transformer_forms: Vec<TransformerForm>,
    meta_forms: Vec<MetaForm>,
    syntax: FutureSyntax,
}

impl TextualNomos {
    pub fn seal(
        logos: &LogosLanguage,
        ids: TextualNomosTypeIds,
        words: TextualNomosWords,
        meta_types: Vec<TextualNomosMetaType>,
    ) -> Result<Self, TextualNomosError> {
        let types = ids.encoded()?;
        for (position, word) in [
            ("Named word", &words.named),
            ("Structural word", &words.structural),
            ("Newtype word", &words.newtype),
            ("Enumeration word", &words.enumeration),
            ("Realize word", &words.realize),
            ("Splice word", &words.splice),
            ("Invoke word", &words.invoke),
        ] {
            require_universal(position, word)?;
        }
        validate_meta_types(&meta_types)?;

        let newtype =
            TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.newtype_type())?;
        let enumeration =
            TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.enumeration_type())?;
        let attributes =
            TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())?;
        let syntax = FutureSyntax {
            realize: words.realize.clone(),
            splice: words.splice.clone(),
            invoke: words.invoke.clone(),
        };

        let profile = RawProfile::standard().seal()?;
        let mut entries = outer_entries(
            &types,
            &words,
            &meta_types,
            logos.newtype_type(),
            logos.enumeration_type(),
            logos.attributes_type(),
        )?;
        entries.extend(lifted_entries(
            logos.grammar(),
            [&newtype, &enumeration, &attributes],
            &syntax,
        ));
        let table = AddressedStructuralTable::seal(
            TableIdentityPayload::new(
                TargetLayoutIdentity::derive(
                    b"core-nomos plain Standard TextualNomos Template(X) v1",
                ),
                profile.identity(),
                StructuralVocabularyIdentity::language(
                    b"core-nomos derived TextualNomos structural vocabulary v1",
                ),
                discovery(),
                TextualRenderingPolicy::new(vec![ContextualTextualPolicy::new(
                    ROOT_CONTEXT,
                    Some(WHITESPACE),
                    Some(CARRIER),
                )]),
                entries,
            ),
            &profile,
        )?;

        let transformer_forms = vec![
            TransformerForm {
                constructor: EncodedConstructorId::under(&types.transformer, 1),
                kind: MacroKind::Structural(SectionDefault::Newtype),
                template_root: logos.newtype_type().clone(),
            },
            TransformerForm {
                constructor: EncodedConstructorId::under(&types.transformer, 2),
                kind: MacroKind::Structural(SectionDefault::Enumeration),
                template_root: logos.enumeration_type().clone(),
            },
            TransformerForm {
                constructor: EncodedConstructorId::under(&types.transformer, 3),
                kind: MacroKind::Named,
                template_root: logos.attributes_type().clone(),
            },
        ];
        let meta_forms = meta_types
            .into_iter()
            .enumerate()
            .map(|(index, declaration)| MetaForm {
                constructor: EncodedConstructorId::under(
                    &types.input_parameter,
                    u16::try_from(index + 1).expect("validated meta declaration count fits u16"),
                ),
                meta: declaration.meta,
                output: declaration.output,
            })
            .collect();

        Ok(Self {
            table,
            document: types.document,
            newtype,
            enumeration,
            attributes,
            transformer_forms,
            meta_forms,
            syntax,
        })
    }

    /// Decode one six-slot document through raw discovery and the shared
    /// structural evaluator, then reify the computed Template(X) values.
    pub fn decode<Bindings: DecodeNameBindings<VocabularyRoot> + ?Sized>(
        &self,
        source: &str,
        bindings: &Bindings,
    ) -> Result<DecodedNomosDocument, TextualNomosError> {
        let mirror =
            StructuralEvaluator::new(&self.table)?.decode_text(&self.document, source, bindings)?;
        let revision = decoded_revision(&mirror)?;
        let mut declarations = Vec::new();
        for transformer in decoded_transformer_values(&mirror)? {
            declarations.push(self.decode_transformer(transformer)?);
        }
        let transformers = AuthoredTransformerSet::try_new(declarations)?;
        Ok(DecodedNomosDocument {
            revision,
            transformers,
            mirror,
        })
    }

    /// Render the retained typed mirror through the same canonical table.
    pub fn view<Resolver: EncodedNameResolver<VocabularyRoot> + ?Sized>(
        &self,
        decoded: &DecodedNomosDocument,
        resolver: &Resolver,
    ) -> Result<String, TextualNomosError> {
        Ok(StructuralEvaluator::new(&self.table)?.encode_text(
            &self.document,
            decoded.structural_value(),
            resolver,
        )?)
    }

    pub fn token_profile(&self) -> &SealedTokenProfile {
        self.table.token_profile()
    }

    fn decode_transformer(
        &self,
        value: &StructuralValue<VocabularyRoot>,
    ) -> Result<AuthoredTransformerDeclaration, TextualNomosError> {
        let form = self
            .transformer_forms
            .iter()
            .find(|form| &form.constructor == value.constructor())
            .ok_or_else(|| TextualNomosError::UnknownTransformerForm {
                constructor: value.constructor().clone(),
            })?;
        let header = field::<TransformerHeader>(value)?;
        let FieldValue::Application { head, .. } = header else {
            return Err(TextualNomosError::WrongFieldShape {
                role: std::any::type_name::<TransformerHeader>(),
            });
        };
        let FieldValue::Declaration(name) = head.as_ref() else {
            return Err(TextualNomosError::WrongFieldShape {
                role: std::any::type_name::<TransformerHeader>(),
            });
        };
        let name = AuthoredTransformerIdentity::try_new(name.encoded_id().clone())?;

        let body = delegated::<TransformerBody>(value)?;
        let input = delegated::<BodyInput>(body)?;
        let input = self.decode_input(input)?;
        let result = delegated::<BodyResult>(body)?;
        let language = self.template(form.template_root.clone()).ok_or_else(|| {
            TextualNomosError::UnknownTransformerForm {
                constructor: value.constructor().clone(),
            }
        })?;
        let result = reify_template(result, language, &self.syntax)?;
        Ok(AuthoredTransformerDeclaration::try_new(
            name, form.kind, input, result, language,
        )?)
    }

    fn decode_input(
        &self,
        value: &StructuralValue<VocabularyRoot>,
    ) -> Result<AuthoredInputSignature, TextualNomosError> {
        let repeated = field::<DelimitedItems>(value)?;
        let FieldValue::Repeated(parameters) = repeated else {
            return Err(TextualNomosError::WrongFieldShape {
                role: std::any::type_name::<DelimitedItems>(),
            });
        };
        let mut decoded = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            let FieldValue::Delegated(parameter) = parameter else {
                return Err(TextualNomosError::WrongFieldShape {
                    role: std::any::type_name::<DelimitedItems>(),
                });
            };
            let meta = self
                .meta_forms
                .iter()
                .find(|form| &form.constructor == parameter.constructor())
                .ok_or_else(|| TextualNomosError::UnknownMetaForm {
                    constructor: parameter.constructor().clone(),
                })?;
            let root = parameter
                .field::<structural_codec::UnaryRoot>()
                .ok_or_else(|| TextualNomosError::MissingRole {
                    role: std::any::type_name::<structural_codec::UnaryRoot>(),
                })?;
            let FieldValue::Application { head, .. } = root else {
                return Err(TextualNomosError::WrongFieldShape {
                    role: std::any::type_name::<structural_codec::UnaryRoot>(),
                });
            };
            let FieldValue::Declaration(binding) = head.as_ref() else {
                return Err(TextualNomosError::WrongFieldShape {
                    role: std::any::type_name::<structural_codec::UnaryRoot>(),
                });
            };
            decoded.push(AuthoredInputParameter::new(
                AuthoredBindingIdentity::try_new(binding.encoded_id().clone())?,
                meta.meta,
                meta.output.clone(),
            ));
        }
        Ok(AuthoredInputSignature::try_new(decoded)?)
    }

    fn template(
        &self,
        root: EncodedTypeId<VocabularyRoot>,
    ) -> Option<&TemplateLanguage<VocabularyRoot>> {
        [&self.newtype, &self.enumeration, &self.attributes]
            .into_iter()
            .find(|language| language.root() == &root)
    }
}

fn validate_meta_types(meta_types: &[TextualNomosMetaType]) -> Result<(), TextualNomosError> {
    if meta_types.len() > usize::from(u16::MAX) {
        return Err(TextualNomosError::TooManyMetaTypes {
            count: meta_types.len(),
            maximum: u16::MAX,
        });
    }
    for declaration in meta_types {
        require_universal("meta-type word", &declaration.word)?;
    }
    for (index, declaration) in meta_types.iter().enumerate() {
        if meta_types[..index]
            .iter()
            .any(|prior| prior.meta == declaration.meta)
        {
            return Err(TextualNomosError::DuplicateMetaType {
                meta: declaration.meta,
            });
        }
        if meta_types[..index]
            .iter()
            .any(|prior| prior.word == declaration.word)
        {
            return Err(TextualNomosError::DuplicateMetaWord {
                word: declaration.word.clone(),
            });
        }
    }
    Ok(())
}

fn outer_entries(
    types: &EncodedTypes,
    words: &TextualNomosWords,
    meta_types: &[TextualNomosMetaType],
    newtype_root: &EncodedTypeId<VocabularyRoot>,
    enumeration_root: &EncodedTypeId<VocabularyRoot>,
    attributes_root: &EncodedTypeId<VocabularyRoot>,
) -> Result<Vec<StructuralEntry<VocabularyRoot, TextualNomosRule>>, TextualNomosError> {
    let structural_kind = |payload: &VocabularyEncodedId| SharedDescriptor::InlineApplication {
        operator: APPLICATION,
        head: Box::new(SharedDescriptor::Literal(words.structural.clone())),
        payload: Box::new(SharedDescriptor::Literal(payload.clone())),
    };
    let transformer_rules = vec![
        (
            1,
            NomosRule::Transformer(TransformerRecord::new(
                structural_kind(&words.newtype),
                &types.newtype_body,
            )?),
        ),
        (
            2,
            NomosRule::Transformer(TransformerRecord::new(
                structural_kind(&words.enumeration),
                &types.enumeration_body,
            )?),
        ),
        (
            3,
            NomosRule::Transformer(TransformerRecord::new(
                SharedDescriptor::Literal(words.named.clone()),
                &types.attributes_body,
            )?),
        ),
    ];
    let meta_rules = meta_types
        .iter()
        .enumerate()
        .map(|(index, declaration)| {
            Ok((
                u16::try_from(index + 1).expect("meta declaration count fits u16"),
                NomosRule::Structural(StructuralRule::Unary(UnaryRule::new(
                    SharedDescriptor::InlineApplication {
                        operator: APPLICATION,
                        head: Box::new(SharedDescriptor::Declaration(AtomDescriptor::any_case())),
                        payload: Box::new(SharedDescriptor::Literal(declaration.word.clone())),
                    },
                )?)),
            ))
        })
        .collect::<Result<Vec<_>, structural_codec::AuthoringError>>()?;
    Ok(vec![
        outer_entry(
            &types.document,
            vec![(1, NomosRule::Document(DocumentRecord::new(types)?))],
        ),
        outer_entry(
            &types.revision,
            vec![(1, NomosRule::Revision(RevisionRecord::new()?))],
        ),
        outer_entry(
            &types.empty_braces,
            vec![(
                1,
                NomosRule::Delimited(DelimitedItemsRecord::new(
                    BRACE,
                    &types.transformer,
                    0,
                    Some(0),
                )?),
            )],
        ),
        outer_entry(
            &types.empty_square,
            vec![(
                1,
                NomosRule::Delimited(DelimitedItemsRecord::new(
                    SQUARE,
                    &types.transformer,
                    0,
                    Some(0),
                )?),
            )],
        ),
        outer_entry(
            &types.transformers,
            vec![(
                1,
                NomosRule::Delimited(DelimitedItemsRecord::new(
                    BRACE,
                    &types.transformer,
                    1,
                    None,
                )?),
            )],
        ),
        outer_entry(&types.transformer, transformer_rules),
        outer_entry(
            &types.input_signature,
            vec![(
                1,
                NomosRule::Delimited(DelimitedItemsRecord::new(
                    PARENTHESIS,
                    &types.input_parameter,
                    0,
                    None,
                )?),
            )],
        ),
        outer_entry(&types.input_parameter, meta_rules),
        outer_entry(
            &types.newtype_body,
            vec![(
                1,
                NomosRule::Body(BodyRecord::new(&types.input_signature, newtype_root)?),
            )],
        ),
        outer_entry(
            &types.enumeration_body,
            vec![(
                1,
                NomosRule::Body(BodyRecord::new(&types.input_signature, enumeration_root)?),
            )],
        ),
        outer_entry(
            &types.attributes_body,
            vec![(
                1,
                NomosRule::Body(BodyRecord::new(&types.input_signature, attributes_root)?),
            )],
        ),
    ])
}

fn outer_entry(
    encoded_type: &EncodedTypeId<VocabularyRoot>,
    rules: Vec<(u16, NomosRule)>,
) -> StructuralEntry<VocabularyRoot, TextualNomosRule> {
    StructuralEntry::new(
        encoded_type.clone(),
        rules
            .into_iter()
            .map(|(local, rule)| {
                let rule = RuleCoproduct::Left(rule);
                ConstructorCodec::new(
                    EncodedConstructorId::under(encoded_type, local),
                    vec![AcceptedDecodeForm::new(FORM, rule.clone())],
                    rule,
                )
            })
            .collect(),
    )
}

fn lifted_entries<'language>(
    grammar: &AddressedStructuralTable<VocabularyRoot, LogosRule>,
    languages: impl IntoIterator<Item = &'language TemplateLanguage<VocabularyRoot>>,
    syntax: &FutureSyntax,
) -> Vec<StructuralEntry<VocabularyRoot, TextualNomosRule>> {
    let mut addressed = BTreeSet::new();
    for language in languages {
        addressed.extend(
            language
                .addressed_types()
                .iter()
                .map(|declaration| declaration.encoded_type().clone()),
        );
    }
    addressed
        .into_iter()
        .filter_map(|encoded_type| grammar.entry(&encoded_type))
        .map(|entry| {
            StructuralEntry::new(
                entry.encoded_type().clone(),
                entry
                    .constructors()
                    .iter()
                    .map(|codec| {
                        let lift = |record: &LogosRule| {
                            RuleCoproduct::Right(DerivedTemplateRecord {
                                source: record.clone(),
                                syntax: syntax.clone(),
                            })
                        };
                        ConstructorCodec::new(
                            codec.constructor().clone(),
                            codec
                                .decode_forms()
                                .iter()
                                .map(|form| {
                                    AcceptedDecodeForm::new(form.identity(), lift(form.rule()))
                                })
                                .collect(),
                            lift(codec.encode_form()),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

fn discovery() -> BlockTreeDiscoveryConfiguration {
    let active = TriggerSet::new(vec![
        PARENTHESIS,
        SQUARE,
        BRACE,
        CARRIER,
        WHITESPACE,
        COMMENT,
    ]);
    BlockTreeDiscoveryConfiguration::new(
        BoundaryDiscoveryConfiguration::new(
            ROOT_CONTEXT,
            vec![BoundaryDiscoveryContext::new(ROOT_CONTEXT, active)],
            vec![
                BoundaryDiscoveryTransition::new(ROOT_CONTEXT, PARENTHESIS, ROOT_CONTEXT),
                BoundaryDiscoveryTransition::new(ROOT_CONTEXT, SQUARE, ROOT_CONTEXT),
                BoundaryDiscoveryTransition::new(ROOT_CONTEXT, BRACE, ROOT_CONTEXT),
            ],
        ),
        vec![],
    )
}

fn field<Role: FieldRole>(
    value: &StructuralValue<VocabularyRoot>,
) -> Result<&FieldValue<VocabularyRoot>, TextualNomosError> {
    value
        .field::<Role>()
        .ok_or_else(|| TextualNomosError::MissingRole {
            role: std::any::type_name::<Role>(),
        })
}

fn delegated<Role: FieldRole>(
    value: &StructuralValue<VocabularyRoot>,
) -> Result<&StructuralValue<VocabularyRoot>, TextualNomosError> {
    let field = field::<Role>(value)?;
    let FieldValue::Delegated(value) = field else {
        return Err(TextualNomosError::WrongFieldShape {
            role: std::any::type_name::<Role>(),
        });
    };
    Ok(value)
}

fn decoded_revision(value: &StructuralValue<VocabularyRoot>) -> Result<i64, TextualNomosError> {
    let revision = delegated::<DocumentRevision>(value)?;
    match field::<RevisionValue>(revision)? {
        FieldValue::Scalar(structural_codec::ScalarValue::Integer(value)) => Ok(*value),
        _ => Err(TextualNomosError::InvalidRevision),
    }
}

fn decoded_transformer_values(
    value: &StructuralValue<VocabularyRoot>,
) -> Result<Vec<&StructuralValue<VocabularyRoot>>, TextualNomosError> {
    let block = delegated::<DocumentTransformers>(value)?;
    let repeated = field::<DelimitedItems>(block)?;
    let FieldValue::Repeated(values) = repeated else {
        return Err(TextualNomosError::WrongFieldShape {
            role: std::any::type_name::<DelimitedItems>(),
        });
    };
    values
        .iter()
        .map(|value| match value {
            FieldValue::Delegated(value) => Ok(value.as_ref()),
            _ => Err(TextualNomosError::WrongFieldShape {
                role: std::any::type_name::<DelimitedItems>(),
            }),
        })
        .collect()
}

fn reify_template(
    value: &StructuralValue<VocabularyRoot>,
    language: &TemplateLanguage<VocabularyRoot>,
    syntax: &FutureSyntax,
) -> Result<TemplateValue<VocabularyRoot>, TextualNomosError> {
    let declaration = language.constructor(value.constructor()).ok_or_else(|| {
        TextualNomosError::UnknownTransformerForm {
            constructor: value.constructor().clone(),
        }
    })?;
    let fields = declaration
        .landing_fields()
        .iter()
        .map(|field| {
            let value = value
                .field_by_role(field.role())
                .ok_or(TextualNomosError::MissingComputedRole { role: field.role() })?;
            Ok(crate::TemplateFieldValue::new(
                field.role(),
                reify_term(value, field.shape(), language, syntax)?,
            ))
        })
        .collect::<Result<Vec<_>, TextualNomosError>>()?;
    Ok(TemplateValue::try_new(value.constructor().clone(), fields)?)
}

fn reify_term(
    value: &FieldValue<VocabularyRoot>,
    shape: &TemplateLandingShape<VocabularyRoot>,
    language: &TemplateLanguage<VocabularyRoot>,
    syntax: &FutureSyntax,
) -> Result<TemplateTerm<VocabularyRoot>, TextualNomosError> {
    match value {
        FieldValue::Declaration(value) => Ok(TemplateTerm::Declaration(value.encoded_id().clone())),
        FieldValue::Reference(value) => Ok(TemplateTerm::Reference(value.encoded_id().clone())),
        FieldValue::Literal(value) => Ok(TemplateTerm::Literal(value.clone())),
        FieldValue::Scalar(value) => Ok(TemplateTerm::Scalar(value.clone())),
        FieldValue::Delegated(value) => Ok(TemplateTerm::Nested(Box::new(reify_template(
            value, language, syntax,
        )?))),
        FieldValue::Repeated(values) => {
            let TemplateLandingShape::Sequence { element, .. } = shape else {
                return Err(TextualNomosError::InvalidFuturePayload);
            };
            Ok(TemplateTerm::Sequence(
                values
                    .iter()
                    .map(|value| reify_term(value, element, language, syntax))
                    .collect::<Result<Vec<_>, _>>()?,
            ))
        }
        FieldValue::Application { head, payload } => {
            reify_future(head, payload, syntax).map(TemplateTerm::Future)
        }
        FieldValue::Delimited(value) | FieldValue::Carrier(value) => {
            reify_term(value, shape, language, syntax)
        }
        FieldValue::OrderedProduct => Err(TextualNomosError::InvalidFuturePayload),
    }
}

fn reify_future(
    head: &FieldValue<VocabularyRoot>,
    payload: &FieldValue<VocabularyRoot>,
    syntax: &FutureSyntax,
) -> Result<TemplateFuture, TextualNomosError> {
    let FieldValue::Literal(keyword) = head else {
        return Err(TextualNomosError::InvalidFuturePayload);
    };
    let FieldValue::Reference(payload) = payload else {
        return Err(TextualNomosError::InvalidFuturePayload);
    };
    if keyword == &syntax.realize {
        return Ok(TemplateFuture::Realize {
            binding: AuthoredBindingIdentity::try_new(payload.encoded_id().clone())?,
            transform: NameTransform::Identity,
        });
    }
    if keyword == &syntax.splice {
        return Ok(TemplateFuture::Splice {
            binding: AuthoredBindingIdentity::try_new(payload.encoded_id().clone())?,
        });
    }
    if keyword == &syntax.invoke {
        return Ok(TemplateFuture::Invoke(
            AuthoredTransformerIdentity::try_new(payload.encoded_id().clone())?,
        ));
    }
    Err(TextualNomosError::UnknownFutureKeyword {
        keyword: keyword.clone(),
    })
}

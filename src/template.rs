//! The result template: logos-encoded-form data with escape nodes. A macro's result
//! is a *quoted* logos skeleton in which specific positions are escapes rather than
//! literals. The escape set is closed — **Realize** and **Splice** — and shared
//! across every position; a position's literal type is fixed by where it sits (a
//! name slot holds an `Identifier`, a type slot a `TypeReference`), so the template
//! stays strongly typed while the escape algebra stays one closed set.
//!
//! Nomos has exactly two escape spellings: `$x` realizes one value and `$@xs`
//! splices one typed vector at a vector-element position. Recursive macro invocation
//! is a separate template surface form, not an escape. This module stores that
//! surface as typed data; evaluation remains entirely stringless.

use core_logos::{Attribute, Field, Generics, TypeReference, Variant, Visibility};
use name_table::Identifier;

use crate::identity::MacroIdentity;

/// A scalar template position: a literal encoded value, or a single-valued escape.
/// Evaluation must produce exactly one `Literal` here; a splice into a scalar is a
/// typed error.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Scalar<Literal> {
    /// A literal Core value, authored against the package's NameTable.
    Literal(Literal),
    /// A single-valued `$x` realize escape.
    Escape(Escape),
}

/// A vector template position: an ordered list of items, each a literal or an
/// escape whose production flattens into the vector. This is the one place a
/// `$@xs` splice belongs.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Sequence<Literal> {
    /// The ordered items.
    pub items: Vec<SequenceItem<Literal>>,
}

impl<Literal> Sequence<Literal> {
    /// A sequence of a single item.
    pub fn of(item: SequenceItem<Literal>) -> Self {
        Self { items: vec![item] }
    }
}

/// One item of a vector template position.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum SequenceItem<Literal> {
    /// A literal element.
    Literal(Literal),
    /// An escape whose production is flattened into the surrounding vector.
    Escape(Escape),
    /// A recursive macro invocation. This is a template surface form, not an
    /// escape primitive. It is meaningful only in the attribute-vector position.
    RecursiveInvoke(MacroIdentity),
}

/// The two primitive Nomos escapes. This enum is intentionally closed: `$x` and
/// `$@xs` are the entire escape set. A recursive macro invocation is represented by
/// [`SequenceItem::RecursiveInvoke`], not by this enum.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Escape {
    /// `$x`: realize exactly one bound value of the expected hole type.
    Realize(Realize),
    /// `$@xs`: flatten exactly one bound typed vector at a vector-element position.
    Splice(Splice),
}

/// The closed identity of an escape primitive. It is used by definition checking and
/// errors instead of string predicates.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscapeKind {
    /// `$x`, which realizes one typed value.
    Realize,
    /// `$@xs`, which concatenates one typed vector.
    Splice,
}

impl Escape {
    /// The primitive kind without inspecting spelling.
    pub fn kind(&self) -> EscapeKind {
        match self {
            Self::Realize(_) => EscapeKind::Realize,
            Self::Splice(_) => EscapeKind::Splice,
        }
    }
}

impl EscapeKind {
    /// The only ruled source spelling for this primitive. Parsing belongs to the
    /// TextualNomos boundary; macro evaluation never reads this spelling.
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::Realize => "$x",
            Self::Splice => "$@xs",
        }
    }
}

/// A typed template boundary checked before a macro definition can be evaluated.
/// Fixed-arity slots do not admit `$@xs`; only vector element positions do.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplatePosition {
    /// A fixed declaration-name slot.
    Name,
    /// A fixed type or enum-payload slot.
    Type,
    /// An attribute-vector element position.
    AttributeElement,
    /// A record field-vector element position.
    FieldElement,
    /// An enum variant-vector element position.
    VariantElement,
}

impl TemplatePosition {
    /// Whether this boundary is a vector element position.
    pub const fn accepts_splice(self) -> bool {
        matches!(
            self,
            Self::AttributeElement | Self::FieldElement | Self::VariantElement
        )
    }
}

/// A `$x` realize escape: which bound value fills one expected typed hole. Name
/// derivation is deliberately absent: it belongs to the NameTable/emission boundary.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Realize {
    /// The bound input this realizes.
    pub binding: BindingRef,
}

/// A reference to a bound input value. The fixture macros need only the top-level
/// input bindings; a splice reads each element's own sub-parts structurally, so no
/// element-local binding reference is required.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingRef {
    /// A top-level input parameter, by its binding name (an identifier in the
    /// package's authoring NameTable).
    Input(Identifier),
}

/// A splice escape: a bound sequence, and the per-element production. `Splice` is
/// concretely exercised in the fixture corpus by struct fields; the element
/// production is closed over the kinds a spliced sequence yields.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Splice {
    /// The bound sequence this expands.
    pub binding: BindingRef,
    /// The per-element production applied to each element.
    pub element: SpliceElement,
}

/// The per-element production of a splice.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum SpliceElement {
    /// Each bound schema field becomes a `EncodedLogos` field: the given visibility,
    /// a name selected by the field-name rule, and its lowered type.
    Field {
        /// The visibility placed on every produced field (schema carries none).
        visibility: Visibility,
        /// How each field's name is selected.
        name_rule: FieldNameRule,
    },
    /// Each bound schema variant becomes a Logos enum variant, preserving its
    /// name and lowering an optional payload to a one-element tuple.
    Variant,
}

/// How a struct macro selects a field's `EncodedLogos` name (deliverable 3: "derived
/// or explicit names per the Field rules").
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldNameRule {
    /// The Field-rule dispatch: an *elided* field is re-derived through the
    /// NameTable/emission boundary into the extended logos table; an explicit field
    /// identifier is preserved.
    /// This is the particular-struct structural default.
    FieldRuleDispatch,
    /// Always request derivation from the field's type at the NameTable/emission
    /// boundary.
    AlwaysDeriveFromType,
    /// Always preserve the schema-stored field name verbatim.
    PreserveSchema,
}

/// A macro's result template, tagged by what fragment it produces. A structural
/// default produces an item; `WireAttributes` produces an attribute vector.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum ResultTemplate {
    /// Produces a single `EncodedItem`.
    Item(ItemTemplate),
    /// Produces an attribute vector (the recursive `Invoke` target).
    Attributes(Sequence<Attribute>),
}

/// An item template — EncodedLogos-shaped, one variant per produced item kind. The
/// enum is closed; the fixture corpus exercises newtypes and structs, and a new
/// item kind is a new variant (the algebra grows by design, no wildcard).
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum ItemTemplate {
    /// A tuple-newtype result.
    Newtype(NewtypeTemplate),
    /// A named-field struct result.
    Struct(StructTemplate),
    /// An enumeration result.
    Enumeration(EnumerationTemplate),
}

/// The newtype result template as data. Visibility is literal; the attribute vector
/// recursively invokes the wire-attributes macro; the name and wrapped type are
/// realized from the input.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NewtypeTemplate {
    /// The item visibility (a literal Core value).
    pub visibility: Visibility,
    /// The attribute preamble — literals and/or a recursive attribute invocation.
    pub attributes: Sequence<Attribute>,
    /// The declared name — realized from the input.
    pub name: Scalar<Identifier>,
    /// The wrapped type — realized from the input's bound type.
    pub wrapped: Scalar<TypeReference>,
}

/// The struct result template. The fields position splices the bound struct fields
/// through the field-name rule; the rest mirrors the newtype template.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct StructTemplate {
    /// The item visibility.
    pub visibility: Visibility,
    /// The attribute preamble.
    pub attributes: Sequence<Attribute>,
    /// The declared name — realized from the input.
    pub name: Scalar<Identifier>,
    /// The generic parameters (literal; empty for the surveyed corpus).
    pub generics: Generics,
    /// The fields — a splice over the bound struct fields.
    pub fields: Sequence<Field>,
}

/// The enum result template. Variants are spliced from the bound EncodedSchema
/// enumeration, while attributes, visibility, and name follow the other item
/// templates.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct EnumerationTemplate {
    pub visibility: Visibility,
    pub attributes: Sequence<Attribute>,
    pub name: Scalar<Identifier>,
    pub generics: Generics,
    pub variants: Sequence<Variant>,
}

/// The enriched generation vocabulary: the schema-derived *support surface* the
/// reference fixtures emit alongside the data declarations — impl blocks (with methods,
/// associated types, and associated consts), functions, consts, const modules, and
/// use imports. Where the per-declaration structural defaults ([`ItemTemplate`])
/// lower one EncodedLogos item per declaration, a [`GenerationClass`] is a whole-schema
/// generator: it reads the schema's newtype catalogue and interface roots
/// ([`core_schema::DeclarationRole`]) and emits an ordered run of EncodedLogos items.
///
/// Each class is closed typed data — no head strings, no text. The schema-derived
/// names, types, and (for the wire stub) transcribed layout values flow from the
/// bound schema when the package is applied ([`crate::MacroPackage::apply_enriched`]);
/// the interpreter that turns a class into EncodedLogos items builds the fixed method
/// and match skeletons directly, exactly as the fixed module prelude
/// ([`crate::ModuleHead`]) authors its stringless EncodedLogos data, keeping every
/// identifier interned into the one continuous logos NameTable.
///
/// The document-order rule the eventual full-file assembly follows is the class
/// order of this enum: the data declarations first, then [`NewtypeErgonomics`],
/// [`InterfaceErgonomics`], the [`WireContract`] vocabulary, the [`WireExchangeCodec`]
/// bodies, the [`WireExchangeEnvelope`] surface, and [`TraceSupport`] — derived from
/// the reference fixture's own block order.
///
/// [`NewtypeErgonomics`]: GenerationClass::NewtypeErgonomics
/// [`InterfaceErgonomics`]: GenerationClass::InterfaceErgonomics
/// [`WireContract`]: GenerationClass::WireContract
/// [`WireExchangeCodec`]: GenerationClass::WireExchangeCodec
/// [`WireExchangeEnvelope`]: GenerationClass::WireExchangeEnvelope
/// [`TraceSupport`]: GenerationClass::TraceSupport
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenerationClass {
    /// Class A — per data-type newtype declaration: the `impl { new / payload /
    /// into_payload }` inherent block and the `From<Inner>` conversion.
    NewtypeErgonomics,
    /// Class B — gated on the interface roots ([`core_schema::DeclarationRole`]
    /// `InterfaceInput` / `InterfaceOutput`): the per-variant constructors that
    /// unwrap newtype payloads, the `From<payload>` conversions, and the cfg-gated
    /// `FromStr` / `Display` impls.
    InterfaceErgonomics,
    /// The **wire contract** — the ordinary-exchange wire vocabulary: the
    /// `short_header` const module, the `SIGNAL_SHORT_HEADER_BYTE_COUNT` byte-count
    /// const, the `SignalFrameError` enum, and the two route enums. The short-header
    /// values are derived from the interface roots' operation positions at generation
    /// time (see `Evaluator::short_header_module`), so the class carries no
    /// selection-time data. (This is the vocabulary the codec speaks; the encode/decode
    /// bodies are the sibling `WireExchangeCodec`.)
    WireContract,
    /// The **wire exchange codec** — the ordinary-exchange encode/decode bodies over
    /// the `WireContract` vocabulary: per interface root an `impl` carrying `route`,
    /// `short_header`, `route_from_short_header`, `encode_signal_frame`, and
    /// `decode_signal_frame`. This retires the empty letter placeholders (former
    /// "classes E/F") for the codec-body work: the stages are named by their content —
    /// the vocabulary and the codec over it.
    WireExchangeCodec,
    /// The **wire exchange envelope** — the ordinary-exchange envelope surface the
    /// ported daemon and clients speak, over the codec: the request root's
    /// `signal_frame::RequestPayload`, `SignalOperationHeads`, and `LogVariant` trait
    /// impls and direct canonical `signal_frame::ExchangeFrame` construction (the
    /// ordinary two-way leg — no `StreamingFrame`, whose subscription envelope waits
    /// on pending psyche rulings);
    /// and the request root's `into_frame` and the reply root's `into_reply_frame`
    /// constructors. Named by its content — the envelope over the codec.
    WireExchangeEnvelope,
    /// Class D — trace support: the `SignalObjectName` / `ObjectName` enums with
    /// their nested-match `name()` bodies and the `TraceEvent` impl.
    TraceSupport,
}

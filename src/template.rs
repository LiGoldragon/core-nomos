//! The result template: logos-encoded-form data with escape nodes. A macro's result
//! is a *quoted* logos skeleton in which specific positions are escapes rather than
//! literals. The escape set is closed — **Realize**, **Invoke**, **Splice** — and
//! shared across every position; a position's literal type is fixed by where it
//! sits (a name slot holds an `Identifier`, a type slot a `TypeReference`), so the
//! template stays strongly typed while the escape algebra stays one closed set.
//!
//! The text spelling of an escape is TextualNomos, a genuinely unsettled question,
//! and is deferred: nothing here parses text. An escape is data.

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
    /// A single-valued escape (`Realize` or `Invoke`).
    Escape(Escape),
}

/// A vector template position: an ordered list of items, each a literal or an
/// escape whose production flattens into the vector. This is the one place a
/// `Splice` (or a multi-valued `Invoke`) belongs.
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
}

/// The closed template escape algebra (nomos-macro-model-v1 §7). Every non-literal
/// template position is exactly one of these three. Closed by design: a fourth
/// escape would be a new variant and a compile error until handled — the psyche
/// ruled name synthesis is *not* a fourth escape but a transform inside `Realize`.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Escape {
    /// Unquote one bound value at this position, optionally through a derived-name
    /// transform. Realizes a bound name (with an optional casing walk) into a name
    /// slot, or a bound type into a type slot.
    Realize(Realize),
    /// Recursively invoke another macro by identity; its produced fragment is
    /// realized (in a scalar slot) or spliced (in a vector slot) in place.
    Invoke(MacroIdentity),
    /// Unquote a bound sequence, expanded element by element into the enclosing
    /// vector.
    Splice(Splice),
}

/// A realize escape: which bound value, and the name transform to apply. For a
/// type binding the transform must be `Identity`; a name binding may carry a
/// derived-name walk.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Realize {
    /// The bound input this realizes.
    pub binding: BindingRef,
    /// The name transform applied when realizing a name.
    pub transform: NameTransform,
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

/// A name transform — the derived-name rule as data, reusing name-table's single
/// home of the walk. This is how "name synthesis reuses the one derived-name rule"
/// stays inside `Realize` instead of becoming a fourth escape.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameTransform {
    /// Realize the bound value verbatim (a name copied, a type lowered).
    Identity,
    /// Derive the `snake_case` field name (name-table `Name::field_name`).
    FieldName,
    /// Derive the `SCREAMING_SNAKE_CASE` constant name (name-table `Name::screaming`).
    Screaming,
    /// Derive the `PascalCase` object spelling (name-table `Name::pascal_case`).
    PascalCase,
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
    /// Each bound schema field becomes a `CoreLogos` field: the given visibility,
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

/// How a struct macro selects a field's `CoreLogos` name (deliverable 3: "derived
/// or explicit names per the Field rules").
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldNameRule {
    /// The Field-rule dispatch: an *elided* field (its schema name equals the
    /// `field_name` of its type) re-derives through name-table's walker into the
    /// extended logos table; an *explicitly-named* field keeps its schema name.
    /// This is the particular-struct structural default.
    FieldRuleDispatch,
    /// Always derive the name from the field's type via the `field_name` walker.
    AlwaysDeriveFromType,
    /// Always preserve the schema-stored field name verbatim.
    PreserveSchema,
}

/// A macro's result template, tagged by what fragment it produces. A structural
/// default produces an item; `WireAttributes` produces an attribute vector.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum ResultTemplate {
    /// Produces a single `CoreItem`.
    Item(ItemTemplate),
    /// Produces an attribute vector (the recursive `Invoke` target).
    Attributes(Sequence<Attribute>),
}

/// An item template — CoreLogos-shaped, one variant per produced item kind. The
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

/// The enum result template. Variants are spliced from the bound CoreSchema
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
/// lower one CoreLogos item per declaration, a [`GenerationClass`] is a whole-schema
/// generator: it reads the schema's newtype catalogue and interface roots
/// ([`core_schema::DeclarationRole`]) and emits an ordered run of CoreLogos items.
///
/// Each class is closed typed data — no head strings, no text. The schema-derived
/// names, types, and (for the wire stub) transcribed layout values flow from the
/// bound schema when the package is applied ([`crate::MacroPackage::apply_enriched`]);
/// the interpreter that turns a class into CoreLogos items builds the fixed method
/// and match skeletons directly, exactly as the fixed module prelude
/// ([`crate::ModuleHead`]) authors its stringless CoreLogos data, keeping every
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
    /// impls; the `Frame` / `FrameBody` / `Request` / `ReplyEnvelope` / `RequestBuilder`
    /// type aliases over `signal_frame::ExchangeFrame` (the ordinary two-way leg — no
    /// `StreamingFrame`, whose subscription envelope waits on pending psyche rulings);
    /// and the request root's `into_frame` and the reply root's `into_reply_frame`
    /// constructors. Named by its content — the envelope over the codec.
    WireExchangeEnvelope,
    /// Class D — trace support: the `SignalObjectName` / `ObjectName` enums with
    /// their nested-match `name()` bodies and the `TraceEvent` impl.
    TraceSupport,
}

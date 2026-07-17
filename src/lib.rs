//! # core-nomos
//!
//! The stringless **Core of Nomos**, the macro/transformation language. A macro is
//! typed data — never text, never Rust — that lowers a stringless `CoreSchema` into
//! stringless `CoreLogos`. This crate realizes the ruling that governs the whole
//! language family: *macros define the entire schema→logos lowering, and the Rust
//! the goldens already emit is the acceptance oracle.*
//!
//! ## What is here (CoreNomos), and what is deferred (TextualNomos)
//!
//! CoreNomos is settled and built here: macros as typed data, the two macro kinds,
//! the stateful-at-rest package, and the engine. **TextualNomos** — the `$` / `<<>>`
//! escape sigils, the meta-type text spellings, Nomos's own delimiters — sits in the
//! psyche's non-rejected review-later pile and is **deferred**. Nothing in this
//! crate parses or prints any Nomos text surface; an escape is a data node
//! ([`Escape`]), and its text spelling is not this crate's concern.
//!
//! ## The two macro kinds
//!
//! - **Named** ([`MacroKind::Named`]) — dispatched by minted [`MacroIdentity`]; an
//!   unknown named invocation is an error. `WireAttributes` is named.
//! - **Structural** ([`MacroKind::Structural`]) — a per-section default selected by
//!   a schema declaration's kind; `WireNewtype` and the particular-struct macro are
//!   structural.
//!
//! ## Stateful at rest
//!
//! A [`MacroPackage`] is a durable, archivable, content-identified value — a
//! loaded-definitions registry as data (a [`MacroIdentity`]-keyed table plus
//! section defaults), carrying its own authoring NameTable sibling excluded from
//! the content identity. Daemon seating is later work; this crate provides the
//! portable package type.
//!
//! ## The closed escape algebra
//!
//! A result template is CoreLogos-shaped data whose non-literal positions are the
//! closed set [`Escape`] = **Realize** / **Invoke** / **Splice**. Name synthesis is
//! not a fourth escape but a [`NameTransform`] inside `Realize`, reusing name-table's
//! single home of the derived-name rule (the psyche's no-fourth-escape ruling).
//!
//! ## The engine
//!
//! [`MacroPackage::apply`] takes a `CoreSchema` and the schema NameTable and returns
//! [`Lowering`]: the `CoreLogos` items and the *extended* logos NameTable — one
//! continuous identifier space in which schema indices are preserved and logos names
//! append. Conversions are typed end to end, outside text.

pub mod definition;
pub mod domain;
pub mod engine;
pub mod error;
pub mod fixtures;
pub mod identity;
pub mod meta;
pub mod package;
pub mod prelude;
pub mod template;

pub use definition::MacroDefinition;
pub use domain::CoreNomosDomain;
pub use engine::Lowering;
pub use error::NomosError;
pub use identity::{MacroIdentity, MacroKind, SectionDefault};
pub use meta::{BoundInput, InputParameter, InputSignature, MetaType, MetaValue};
pub use package::{MacroDefinitions, MacroPackage, PackageRevision};
pub use prelude::{GENERATED_MARKER, ModuleHead};
pub use template::{
    BindingRef, EnumerationTemplate, Escape, FieldNameRule, ItemTemplate, NameTransform,
    NewtypeTemplate, Realize, ResultTemplate, Scalar, Sequence, SequenceItem, Splice,
    SpliceElement, StructTemplate,
};

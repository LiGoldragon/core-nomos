//! # core-nomos
//!
//! The stringless encoded form of Nomos, the macro/transformation language. A macro
//! is typed data — never text, never Rust — that lowers the schema encoded form into
//! the logos encoded form. Macros define the schema-to-logos lowering; generated
//! programs, not rendered-source equality, are the acceptance surface.
//!
//! ## What is here (EncodedNomos), and what is deferred (TextualNomos)
//!
//! EncodedNomos is built here: macros as typed data, the two macro kinds, the
//! stateful-at-rest package, and the engine. **TextualNomos** — including the escape
//! spelling, meta-type text spellings, and delimiters — remains an open design
//! question. Nothing in this crate parses or prints a Nomos text surface; an escape
//! is a data node ([`Escape`]), and its spelling is not this crate's concern.
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
//! A result template is logos-encoded-form data whose non-literal positions are the
//! closed set [`Escape`] = **Realize** / **Invoke** / **Splice**. A [`NameTransform`]
//! is typed intent carried by `Realize`, not a fourth escape; `NameTableBoundary`
//! performs the derived-name work at the NameTable/emission boundary.
//!
//! ## The engine
//!
//! [`MacroPackage::apply`] takes the schema encoded form and its NameTable and
//! returns [`Lowering`]: logos encoded items and the *extended* logos NameTable — one
//! continuous identifier space in which schema indices are preserved and logos names
//! append. Conversions are typed end to end, outside text.

pub mod definition;
pub mod domain;
pub mod engine;
pub mod error;
pub mod fixtures;
pub mod generation;
pub mod identity;
pub mod meta;
mod name_boundary;
pub mod package;
pub mod prelude;
pub mod template;

pub use definition::MacroDefinition;
pub use domain::EncodedNomosDomain;
pub use engine::Lowering;
pub use error::NomosError;
pub use identity::{MacroIdentity, MacroKind, SectionDefault};
pub use meta::{BoundInput, InputParameter, InputSignature, MetaType, MetaValue};
pub use package::{MacroDefinitions, MacroPackage, PackageRevision};
pub use prelude::{GENERATED_MARKER, ModuleHead};
pub use template::{
    BindingRef, EnumerationTemplate, Escape, FieldNameRule, GenerationClass, ItemTemplate,
    NameTransform, NewtypeTemplate, Realize, ResultTemplate, Scalar, Sequence, SequenceItem,
    Splice, SpliceElement, StructTemplate,
};

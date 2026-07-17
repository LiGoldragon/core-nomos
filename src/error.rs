//! The crate-boundary error type.

use content_identity::ArchiveError;
use core_logos::Visibility;
use name_table::{Identifier, NameTableError};
use thiserror::Error;

use crate::identity::{MacroIdentity, SectionDefault};
use crate::meta::MetaType;

/// A failure lowering a `CoreSchema` through a [`MacroPackage`](crate::MacroPackage)
/// into `CoreLogos`. Every variant names the exact structural mismatch, so an
/// unruled input fails loudly rather than producing quietly-wrong logos — the
/// acceptance-oracle discipline made typed.
#[derive(Debug, Clone, Error)]
pub enum NomosError {
    /// A schema declaration of this kind has no structural default macro in the
    /// package. An unknown structural section is an error, never a silent skip.
    #[error("no structural default macro for the {0:?} declaration section")]
    NoStructuralDefault(SectionDefault),

    /// A named invocation named a macro identity absent from the package table.
    /// An unknown named invocation is an error (the ruling).
    #[error("named invocation of {0} is not in the macro table")]
    UnknownMacro(MacroIdentity),

    /// A macro recursively invoked itself along an active call path. Recursive
    /// invocation is required (WireNewtype invokes WireAttributes); an unbounded
    /// cycle is rejected.
    #[error("recursive macro invocation cycle reached {0} while it was already active")]
    RecursionCycle(MacroIdentity),

    /// A template position was filled by an escape whose production does not fit
    /// the position — e.g. a splice into a scalar slot, or an invocation where a
    /// name was expected.
    #[error("template escape does not fit its position: {0}")]
    EscapeShape(&'static str),

    /// An input binding was referenced that the bound input does not carry.
    #[error("template referenced unbound input {0}")]
    UnboundInput(Identifier),

    /// A meta-type could not be filled from the declaration it was applied to —
    /// e.g. a `Type` meta-type against a struct declaration.
    #[error("meta-type {meta:?} cannot bind against this declaration shape")]
    MetaShape { meta: MetaType },

    /// A macro produced a fragment of the wrong kind for where it was invoked —
    /// e.g. a structural default that did not produce an item, or an attribute
    /// invocation that did not produce attributes.
    #[error("macro produced the wrong fragment kind: {0}")]
    FragmentKind(&'static str),

    /// A name transform was applied where its input was not a name — e.g. a field
    /// name derived from a value position.
    #[error("name transform applied to a non-name binding")]
    NameTransformShape,

    /// A field carried a visibility the lowering could not place. Retained as a
    /// typed sibling for the field-rule growth points.
    #[error("field visibility {0:?} is not placeable here")]
    FieldVisibility(Visibility),

    /// A schema type reference could not be lowered into a `CoreLogos` type — a
    /// value application (const generic) has no `TypeReference` home in the
    /// surveyed logos algebra.
    #[error("schema reference cannot lower to a CoreLogos type: {0}")]
    UnsupportedReference(&'static str),

    /// A macro template literal carried a `CoreLogos` type outside the schema-lowering
    /// template vocabulary — a reference or impl-trait type, which belongs to
    /// impl-block signatures, not schema declarations.
    #[error("template type is out of the macro template vocabulary: {0}")]
    UnsupportedTemplateType(&'static str),

    /// A name-table projection failed while resolving or interning a name.
    #[error("name resolution failed: {0}")]
    NameResolution(#[from] NameTableError),

    /// Computing the package's content identity failed at the portable-archive
    /// layer.
    #[error("content identity failed: {0}")]
    ContentIdentity(#[from] ArchiveError),

    /// Projecting the fixed module prelude through the TextualRust codec failed — an
    /// internal invariant break, since the prelude items are fixed and in-subset.
    #[error("module prelude projection failed: {0}")]
    PreludeProjection(#[from] textual_rust::Error),

    /// An enriched generation class could not be built from the schema — e.g. a
    /// generation class that needs interface roots ran against a schema carrying
    /// none, an interface root that was not an enumeration, or an interface root or
    /// operation index that overflows the short-header layout's one-byte field.
    #[error("enriched generation class cannot build from this schema: {0}")]
    Generation(&'static str),
}

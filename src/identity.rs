//! Macro identity and the two settled macro kinds.

use core_schema::CoreType;

/// A macro's minted, stringless identity — a package-local index the macro table
/// is keyed on. It is *minted* (allocated when a macro is registered), not derived
/// from content, so a recursive invocation names a stable target that does not
/// move when an unrelated macro's template changes; and it is stringless, so
/// renaming the macro (a NameTable edit) never touches it. The *package* carries
/// content identity; a macro carries this mint.
///
/// [changed-from-report] The design corpus calls this a "minted identity" without
/// fixing mint-vs-content-hash. A monotonic package mint is the reading most
/// consistent with "a macro table keyed on minted identity": a content hash would
/// be *derived*, and would couple every recursive reference to the invoked macro's
/// bytes. Flagged in ARCHITECTURE.
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
#[rkyv(derive(PartialEq, Eq, PartialOrd, Ord))]
pub struct MacroIdentity(u32);

impl MacroIdentity {
    /// Wrap a raw mint index. Minting through the package is the ordinary way to
    /// obtain one; this exists for reconstruction from stored data.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// The raw mint index.
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for MacroIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "macro {}", self.0)
    }
}

/// The two settled macro kinds. A new class of dispatch would be a new variant —
/// the closed set matches the ruling that Nomos has exactly two.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum MacroKind {
    /// A macro dispatched by explicit identity: it sits in the macro table and is
    /// reached by a named invocation (an explicit reference or a recursive
    /// `Invoke` in another macro's template). An unknown named invocation is an
    /// error. `WireAttributes` is a named macro.
    Named,
    /// A per-section default, selected by a schema declaration's structural kind
    /// rather than by name: an ordinary type declaration in a section lowers via
    /// that section's default macro. `WireNewtype` and the particular-struct macro
    /// are structural.
    Structural(SectionDefault),
}

/// Which schema declaration section a structural macro is the default for. This is
/// the declaration-kind selector, disjoint from `core_schema::CoreType`'s variants
/// only in that it is the *lowering side*'s dispatch key.
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
#[rkyv(derive(PartialEq, Eq, PartialOrd, Ord))]
pub enum SectionDefault {
    /// The default for a newtype declaration section.
    Newtype,
    /// The default for a named-field struct declaration section.
    Struct,
    /// The default for an enum declaration section (a growth point; the fixture
    /// corpus does not exercise it, but the selector is closed and total).
    Enumeration,
}

impl SectionDefault {
    /// Which structural section a schema declaration belongs to — the dispatch
    /// from a declaration's Core kind to the default macro that lowers it.
    /// Exhaustive over `CoreType`, no wildcard, so a new declaration kind is a
    /// compile error until its section is named.
    pub fn of_core_type(value: &CoreType) -> Self {
        match value {
            CoreType::Newtype(_) => Self::Newtype,
            CoreType::Struct(_) => Self::Struct,
            CoreType::Enumeration(_) => Self::Enumeration,
        }
    }
}

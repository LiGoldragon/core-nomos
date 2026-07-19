//! The content-identity domain for the stringless encoded form of Nomos.

use content_identity::{DomainSeparation, HashDomain, LayoutVersion};

/// The layout-versioned hash domain for a [`MacroPackage`](crate::MacroPackage)'s
/// stringless macro data. Like every encoded form in the family, a package's content
/// identity is taken over its portable-archive bytes with the NameTable excluded,
/// so renaming a macro (a NameTable-only edit) is hash-stable and a change to a
/// macro's kind, input signature, or result template moves the identity. The
/// domain carries the layout version in the type, never a hand-remembered suffix.
pub struct EncodedNomosDomain;

impl HashDomain for EncodedNomosDomain {
    fn separation() -> DomainSeparation {
        DomainSeparation::Contextual {
            context: "core-nomos 2026 stringless core of nomos macro package",
            layout: LayoutVersion::new(1),
        }
    }
}

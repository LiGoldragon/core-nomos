//! The legacy caller-issued Nomos Capsule pass-through.

use capsule_content_identity::ContentAddressedHash;
use protos::{Capsule, Nomos};

/// Carry a legacy caller-issued hash and caller-supplied complete NameTree pin.
///
/// This is a pass-through constructor. It does not create a whole-Nomos encoded
/// carrier, derive the hash from a [`MacroPackage`](crate::MacroPackage), compare
/// the hash with component content, inspect or compose the pin, or verify that
/// the pin is complete. In particular, it does not turn the current package
/// identity into a Capsule identity. Module-table composition and the
/// correspondence between this carrier and Nomos content remain unwired. New
/// authored Nomos code uses [`SealedNomosCapsule`](crate::SealedNomosCapsule).
pub fn capsule_from_issued_hash<CompleteNameTreePin>(
    hash: ContentAddressedHash,
    complete_nametree_pin: CompleteNameTreePin,
) -> Capsule<Nomos, CompleteNameTreePin> {
    Capsule::new(hash, complete_nametree_pin)
}

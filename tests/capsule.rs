use capsule_content_identity::ContentAddressedHash;
use protos::{Capsule, CapsuleIdentityVariant, Nomos};

#[derive(Debug, Eq, PartialEq)]
struct OpaqueCompleteNameTreePin([u8; 43]);

#[test]
fn caller_values_are_carried_in_a_nomos_kind_capsule() {
    let pin_bytes = [0xc9; 43];
    let capsule: Capsule<Nomos, OpaqueCompleteNameTreePin> = core_nomos::capsule_from_issued_hash(
        ContentAddressedHash::from_bytes([0x53; 32]),
        OpaqueCompleteNameTreePin(pin_bytes),
    );

    assert_eq!(
        capsule.content_identity().variant(),
        CapsuleIdentityVariant::Nomos
    );
    assert_eq!(capsule.complete_nametree_pin().0, pin_bytes);
}

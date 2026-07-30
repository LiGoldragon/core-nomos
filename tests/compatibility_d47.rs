use core_nomos::{
    AuthenticatedNameTreeProjection, MacroKind, SealedNomosCapsule, SealedNomosPopulation,
    TemplateFuture, TemplateFutureKind,
};
use rkyv::Archive;

const CAPSULE: &[u8] = include_bytes!("goldens/d47_nomos_capsule.bin");
const PROJECTION: &[u8] = include_bytes!("goldens/d47_nomos_projection.bin");

#[test]
fn d47_capsule_and_projection_restore_and_reserialize_byte_exact() {
    let capsule = SealedNomosCapsule::from_archive_bytes(CAPSULE)
        .expect("d47 Capsule restores through current validation");
    let projection = AuthenticatedNameTreeProjection::from_archive_bytes(PROJECTION)
        .expect("d47 projection restores through current validation");
    projection
        .verify_for(&capsule)
        .expect("d47 projection remains bound to the exact Capsule");

    assert_eq!(
        capsule
            .to_archive_bytes()
            .expect("d47 Capsule reserializes"),
        CAPSULE
    );
    assert_eq!(
        projection
            .to_archive_bytes()
            .expect("d47 projection reserializes"),
        PROJECTION
    );
    SealedNomosPopulation::from_archive_parts(CAPSULE, PROJECTION)
        .expect("d47 population restores as one authenticated unit");
}

#[test]
fn appended_future_tags_must_not_grow_the_d47_archive_layout() {
    assert_eq!(std::mem::size_of::<<MacroKind as Archive>::Archived>(), 2);
    assert_eq!(std::mem::align_of::<<MacroKind as Archive>::Archived>(), 1);
    assert_eq!(
        std::mem::size_of::<<TemplateFutureKind as Archive>::Archived>(),
        1
    );
    assert_eq!(
        std::mem::align_of::<<TemplateFutureKind as Archive>::Archived>(),
        1
    );
    assert_eq!(
        std::mem::size_of::<<TemplateFuture as Archive>::Archived>(),
        11
    );
    assert_eq!(
        std::mem::align_of::<<TemplateFuture as Archive>::Archived>(),
        1
    );
}

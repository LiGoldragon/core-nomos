use core_nomos::{
    AuthenticatedNameTreeProjection, AuthoredBindingIdentity, AuthoredTransformerDeclaration,
    AuthoredTransformerIdentity, MacroKind, NameTransform, SealedNomosCapsule,
    SealedNomosPopulation, SectionDefault, TemplateFuture, TemplateFutureKind, TemplateTerm,
};
use encoded_name_table::LocalEncodedId;
use rkyv::Archive;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

const CAPSULE: &[u8] = include_bytes!("goldens/d47_nomos_capsule.bin");
const PROJECTION: &[u8] = include_bytes!("goldens/d47_nomos_projection.bin");
const FUTURE_REALIZE: &[u8] = include_bytes!("goldens/d47_template_future_realize.bin");
const FUTURE_INVOKE: &[u8] = include_bytes!("goldens/d47_template_future_invoke.bin");
const FUTURE_SPLICE: &[u8] = include_bytes!("goldens/d47_template_future_splice.bin");
const MACRO_NAMED: &[u8] = include_bytes!("goldens/d47_macro_kind_named.bin");
const MACRO_STRUCTURAL: &[u8] = include_bytes!("goldens/d47_macro_kind_structural.bin");
const NAMED_INVOKE_DECLARATION: &[u8] = include_bytes!("goldens/d47_named_invoke_declaration.bin");

fn universal(chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("fixture identity is nonempty")
}

fn binding() -> AuthoredBindingIdentity {
    AuthoredBindingIdentity::try_new(universal(&[47, 1, 2])).expect("Universal binding")
}

fn transformer() -> AuthoredTransformerIdentity {
    AuthoredTransformerIdentity::try_new(universal(&[47, 3, 4])).expect("Universal transformer")
}

#[test]
fn d47_capsule_and_projection_restore_and_reserialize_byte_exact() {
    let capsule = SealedNomosCapsule::from_archive_bytes(CAPSULE)
        .expect("d47 Capsule restores through current validation");
    assert_eq!(
        capsule.content_addressed_hash().to_hexadecimal(),
        "d516541a21e3acd3d4af83759a1e68962bd2e914084fd09963b96e02aec4ca52"
    );
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
fn d47_legacy_enum_tags_restore_and_reserialize_byte_exact() {
    let realize = TemplateFuture::Realize {
        binding: binding(),
        transform: NameTransform::PascalCase,
    };
    let invoke = TemplateFuture::Invoke(transformer());
    let splice = TemplateFuture::Splice { binding: binding() };
    let named = MacroKind::Named;
    let structural = MacroKind::Structural(SectionDefault::Enumeration);

    assert_eq!(FUTURE_REALIZE[6], 0, "legacy Realize tag");
    assert_eq!(FUTURE_INVOKE[6], 1, "legacy Invoke tag");
    assert_eq!(FUTURE_SPLICE[6], 2, "legacy Splice tag");
    assert_eq!(MACRO_NAMED[0], 0, "legacy Named tag");
    assert_eq!(MACRO_STRUCTURAL[0], 1, "legacy Structural tag");

    assert_eq!(
        rkyv::from_bytes::<TemplateFuture, rkyv::rancor::Error>(FUTURE_REALIZE)
            .expect("restore legacy Realize"),
        realize
    );
    assert_eq!(
        rkyv::from_bytes::<TemplateFuture, rkyv::rancor::Error>(FUTURE_INVOKE)
            .expect("restore legacy Invoke"),
        invoke
    );
    assert_eq!(
        rkyv::from_bytes::<TemplateFuture, rkyv::rancor::Error>(FUTURE_SPLICE)
            .expect("restore legacy Splice"),
        splice
    );
    assert_eq!(
        rkyv::from_bytes::<MacroKind, rkyv::rancor::Error>(MACRO_NAMED)
            .expect("restore legacy Named"),
        named
    );
    assert_eq!(
        rkyv::from_bytes::<MacroKind, rkyv::rancor::Error>(MACRO_STRUCTURAL)
            .expect("restore legacy Structural"),
        structural
    );

    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&realize)
            .expect("reserialize legacy Realize")
            .as_slice(),
        FUTURE_REALIZE
    );
    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&invoke)
            .expect("reserialize legacy Invoke")
            .as_slice(),
        FUTURE_INVOKE
    );
    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&splice)
            .expect("reserialize legacy Splice")
            .as_slice(),
        FUTURE_SPLICE
    );
    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&named)
            .expect("reserialize legacy Named")
            .as_slice(),
        MACRO_NAMED
    );
    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&structural)
            .expect("reserialize legacy Structural")
            .as_slice(),
        MACRO_STRUCTURAL
    );
}

#[test]
fn d47_named_transformer_carries_a_real_invoke_byte_exact() {
    let declaration = rkyv::from_bytes::<AuthoredTransformerDeclaration, rkyv::rancor::Error>(
        NAMED_INVOKE_DECLARATION,
    )
    .expect("restore legacy Named Invoke declaration");
    assert_eq!(declaration.kind(), MacroKind::Named);
    assert_eq!(declaration.name().encoded_id(), &universal(&[47, 3, 4]));
    let [field] = declaration.result().fields() else {
        panic!("legacy Named Invoke declaration has one output field");
    };
    let TemplateTerm::Sequence(items) = field.term() else {
        panic!("legacy Named Invoke output is a sequence");
    };
    let [TemplateTerm::Future(TemplateFuture::Invoke(target))] = items.as_slice() else {
        panic!("legacy Named Invoke output carries exactly one Invoke");
    };
    assert_eq!(target, declaration.name());
    assert_eq!(
        rkyv::to_bytes::<rkyv::rancor::Error>(&declaration)
            .expect("reserialize legacy Named Invoke declaration")
            .as_slice(),
        NAMED_INVOKE_DECLARATION
    );
}

#[test]
fn appended_tags_must_not_grow_the_d47_archive_layout() {
    assert_eq!(
        std::mem::size_of::<<AuthoredTransformerDeclaration as Archive>::Archived>(),
        71
    );
    assert_eq!(
        std::mem::align_of::<<AuthoredTransformerDeclaration as Archive>::Archived>(),
        1
    );
    assert_eq!(
        std::mem::size_of::<<TemplateTerm<VocabularyRoot> as Archive>::Archived>(),
        12
    );
    assert_eq!(
        std::mem::align_of::<<TemplateTerm<VocabularyRoot> as Archive>::Archived>(),
        1
    );
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

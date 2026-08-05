use core_ethos::{
    WholeEthosAttributes, WholeEthosNewtype, WholeEthosQuality, WholeEthosTypeApplication,
    WholeEthosTypeReference, WholeEthosVisibility, WholeEthosWrappedField,
};
use core_nomos::{ScopeOfDeclarationRecognition, ScopeOfRefusal, ScopeOfTransformer};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

fn id(local: u16) -> VocabularyEncodedId {
    VocabularyEncodedId::new(VocabularyRoot::Universal, vec![LocalEncodedId::new(local)])
        .expect("one local identity")
}

#[test]
fn legacy_scope_of_angle_application_is_a_typed_later_work_refusal() {
    let scope_of = id(1);
    let target = id(2);
    let item = core_ethos::WholeEthosItem::Newtype(WholeEthosNewtype::new(
        target.clone(),
        WholeEthosVisibility::Public,
        WholeEthosAttributes::empty(),
        WholeEthosWrappedField::new(
            WholeEthosVisibility::Private,
            WholeEthosTypeReference::Application(
                WholeEthosTypeApplication::new(
                    WholeEthosQuality::Shape(scope_of.clone()),
                    vec![WholeEthosTypeReference::Identity(id(3))],
                )
                .expect("nonempty application"),
            ),
        ),
    ));
    let transformer =
        ScopeOfTransformer::try_new(scope_of, id(4)).expect("Universal configuration");
    assert!(matches!(
        transformer.recognize(&item),
        Err(ScopeOfRefusal::LegacyScopeOfApplicationUnsupported { target: found }) if found == target
    ));
}

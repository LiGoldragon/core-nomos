use core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosFileKind, WholeEthosHeader,
    WholeEthosItem, WholeEthosNewtype, WholeEthosNexusBody, WholeEthosQuality,
    WholeEthosStreamInitiation, WholeEthosStruct, WholeEthosTypeApplication,
    WholeEthosTypeParameter, WholeEthosTypeReference, WholeEthosVisibility, WholeEthosWrappedField,
};
use core_logos::{WholeLogosItem, WholeLogosTypeReference};
use core_nomos::{SliceOneTransformation, SliceOneTransformationError};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

fn id(local: u16) -> VocabularyEncodedId {
    VocabularyEncodedId::new(VocabularyRoot::Universal, vec![LocalEncodedId::new(local)])
        .expect("one local identity")
}

fn nexus(items: Vec<WholeEthosItem>) -> WholeEthos {
    WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Nexus, 1).expect("supported header"),
        WholeEthosBody::Nexus(WholeEthosNexusBody::new(items, vec![])),
    )
    .expect("valid Nexus document")
}

#[test]
fn lowers_shape_application_with_ordered_arguments_and_named_trait_parameter() {
    let ordered = id(1);
    let vector = id(2);
    let error = id(3);
    let wrapped = WholeEthosTypeReference::Application(
        WholeEthosTypeApplication::new(
            WholeEthosQuality::Shape(vector.clone()),
            vec![
                WholeEthosTypeReference::Parameter(WholeEthosTypeParameter::new(
                    ordered.clone(),
                    WholeEthosQuality::Trait(ordered.clone()),
                )),
                WholeEthosTypeReference::Identity(error.clone()),
            ],
        )
        .expect("nonempty application"),
    );
    let document = nexus(vec![WholeEthosItem::Newtype(WholeEthosNewtype::new(
        id(4),
        WholeEthosVisibility::Public,
        WholeEthosAttributes::empty(),
        WholeEthosWrappedField::new(WholeEthosVisibility::Private, wrapped),
    ))]);

    let logos = SliceOneTransformation::new().lower(&document).expect("strict lowering");
    let [WholeLogosItem::Newtype(newtype)] = logos.items() else { panic!("one newtype") };
    assert_eq!(newtype.type_parameters().len(), 1);
    assert_eq!(newtype.type_parameters()[0].name(), &ordered);
    let WholeLogosTypeReference::Application(application) = newtype.wrapped() else {
        panic!("application retained")
    };
    assert_eq!(application.head(), &vector);
    assert_eq!(application.arguments().len(), 2);
    assert!(matches!(application.arguments()[0], WholeLogosTypeReference::Parameter(ref name) if name == &ordered));
    assert!(matches!(application.arguments()[1], WholeLogosTypeReference::Identity(ref name) if name == &error));
}

#[test]
fn refuses_trait_head_and_parameterized_struct_without_erasing_ontology() {
    let ordered = id(10);
    let invalid_head = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Nexus, 1).expect("supported header"),
        WholeEthosBody::Nexus(WholeEthosNexusBody::new(vec![WholeEthosItem::Newtype(WholeEthosNewtype::new(
        id(11),
        WholeEthosVisibility::Public,
        WholeEthosAttributes::empty(),
        WholeEthosWrappedField::new(
            WholeEthosVisibility::Private,
            WholeEthosTypeReference::Application(
                WholeEthosTypeApplication::new(
                    WholeEthosQuality::Trait(ordered.clone()),
                    vec![WholeEthosTypeReference::Identity(id(12))],
                )
                .expect("nonempty application"),
            ),
        ),
    ))], vec![])),
    );
    assert!(matches!(
        invalid_head,
        Err(core_ethos::WholeEthosArchiveError::TypeApplicationHeadMustBeShape { quality }) if quality == ordered
    ));

    let parameterized_struct = nexus(vec![WholeEthosItem::Struct(
        WholeEthosStruct::new(
            id(13),
            vec![WholeEthosTypeReference::Parameter(WholeEthosTypeParameter::new(
                ordered.clone(),
                WholeEthosQuality::Trait(ordered.clone()),
            ))],
        )
        .expect("nonempty struct"),
    )]);
    assert!(matches!(
        SliceOneTransformation::new().lower(&parameterized_struct),
        Err(SliceOneTransformationError::UnsupportedParameterizedDeclaration { kind: "struct", .. })
    ));
}

#[test]
fn refuses_stream_without_translator_issued_lifecycle_identities() {
    let stream = id(20);
    let document = nexus(vec![WholeEthosItem::StreamInitiation(WholeEthosStreamInitiation {
        stream: stream.clone(),
        query: WholeEthosTypeReference::Identity(id(21)),
        event: WholeEthosTypeReference::Identity(id(22)),
    })]);
    assert!(matches!(
        SliceOneTransformation::new().lower(&document),
        Err(SliceOneTransformationError::StreamLifecycleIdentitiesRequired { stream: found }) if found == stream
    ));
}

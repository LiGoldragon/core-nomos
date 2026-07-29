//! Focused behavior witnesses for the first direct typed transformation.

use core_nomos::SliceOneTransformation;
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use slice_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosEnumeration, WholeEthosItem, WholeEthosNewtype,
    WholeEthosTupleFields, WholeEthosTypeApplication, WholeEthosTypeReference, WholeEthosVariant,
    WholeEthosVariantPayload, WholeEthosVisibility, WholeEthosWrappedField,
};
use slice_core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype, WholeLogosTupleFields,
    WholeLogosTypeApplication, WholeLogosTypeReference, WholeLogosVariant,
    WholeLogosVariantPayload, WholeLogosVisibility,
};

fn encoded(chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("complete test chain")
}

fn ethos_newtype(
    visibility: WholeEthosVisibility,
    name: VocabularyEncodedId,
    wrapped_visibility: WholeEthosVisibility,
    wrapped: WholeEthosTypeReference,
) -> WholeEthosItem {
    WholeEthosItem::Newtype(WholeEthosNewtype::new(
        name,
        visibility,
        WholeEthosAttributes::empty(),
        WholeEthosWrappedField::new(wrapped_visibility, wrapped),
    ))
}

#[test]
fn ordered_newtypes_lower_with_complete_identity_chains_unchanged() {
    let commit_sequence = encoded(&[42, 7, 9]);
    let integer = encoded(&[3]);
    let internal_counter = encoded(&[42, 7, 10]);
    let nested_integer = encoded(&[8, 3]);
    let ethos = WholeEthos::new(vec![
        ethos_newtype(
            WholeEthosVisibility::Public,
            commit_sequence.clone(),
            WholeEthosVisibility::Private,
            WholeEthosTypeReference::Identity(integer.clone()),
        ),
        ethos_newtype(
            WholeEthosVisibility::Private,
            internal_counter.clone(),
            WholeEthosVisibility::Public,
            WholeEthosTypeReference::Identity(nested_integer.clone()),
        ),
    ]);

    let logos = SliceOneTransformation::new().lower(&ethos);

    assert_eq!(
        logos,
        WholeLogos::new(vec![
            WholeLogosItem::Newtype(WholeLogosNewtype::new(
                WholeLogosVisibility::Public,
                commit_sequence,
                WholeLogosVisibility::Private,
                WholeLogosTypeReference::Identity(integer),
            )),
            WholeLogosItem::Newtype(WholeLogosNewtype::new(
                WholeLogosVisibility::Private,
                internal_counter,
                WholeLogosVisibility::Public,
                WholeLogosTypeReference::Identity(nested_integer),
            )),
        ])
    );
}

#[test]
fn enums_and_recursive_applications_lower_exhaustively_without_changing_chains() {
    let vector = encoded(&[4]);
    let integer = encoded(&[3]);
    let vector_integer = WholeEthosTypeReference::Application(WholeEthosTypeApplication::new(
        vector.clone(),
        WholeEthosTypeReference::Identity(integer.clone()),
    ));
    let status = encoded(&[42, 9]);
    let pending = encoded(&[42, 9, 1]);
    let ready = encoded(&[42, 9, 2]);
    let ethos = WholeEthos::new(vec![
        ethos_newtype(
            WholeEthosVisibility::Public,
            encoded(&[42, 8]),
            WholeEthosVisibility::Private,
            vector_integer.clone(),
        ),
        WholeEthosItem::Enumeration(WholeEthosEnumeration::new(
            status.clone(),
            WholeEthosVisibility::Public,
            WholeEthosAttributes::empty(),
            vec![
                WholeEthosVariant::new(
                    pending.clone(),
                    WholeEthosAttributes::empty(),
                    WholeEthosVariantPayload::Unit,
                ),
                WholeEthosVariant::new(
                    ready.clone(),
                    WholeEthosAttributes::empty(),
                    WholeEthosVariantPayload::Tuple(
                        WholeEthosTupleFields::new(vec![
                            WholeEthosTypeReference::Identity(integer.clone()),
                            vector_integer,
                        ])
                        .expect("non-empty tuple"),
                    ),
                ),
            ],
        )),
    ]);

    assert_eq!(
        SliceOneTransformation::new().lower(&ethos),
        WholeLogos::new(vec![
            WholeLogosItem::Newtype(WholeLogosNewtype::new(
                WholeLogosVisibility::Public,
                encoded(&[42, 8]),
                WholeLogosVisibility::Private,
                WholeLogosTypeReference::Application(WholeLogosTypeApplication::new(
                    vector.clone(),
                    WholeLogosTypeReference::Identity(integer.clone()),
                )),
            )),
            WholeLogosItem::Enumeration(WholeLogosEnumeration::new(
                WholeLogosVisibility::Public,
                status,
                vec![
                    WholeLogosVariant::new(pending, WholeLogosVariantPayload::Unit),
                    WholeLogosVariant::new(
                        ready,
                        WholeLogosVariantPayload::Tuple(
                            WholeLogosTupleFields::new(vec![
                                WholeLogosTypeReference::Identity(integer.clone()),
                                WholeLogosTypeReference::Application(
                                    WholeLogosTypeApplication::new(
                                        vector,
                                        WholeLogosTypeReference::Identity(integer),
                                    ),
                                ),
                            ])
                            .expect("non-empty tuple"),
                        ),
                    ),
                ],
            )),
        ])
    );
}

#[test]
fn empty_whole_ethos_lowers_to_empty_whole_logos() {
    assert_eq!(
        SliceOneTransformation::new().lower(&WholeEthos::new(Vec::new())),
        WholeLogos::new(Vec::new())
    );
}

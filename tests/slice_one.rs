//! Focused behavior witnesses for the first direct typed transformation.

use core_nomos::{
    SliceOneTransformation, SliceOneTransformationError, SliceOneVocabularyReferenceMapping,
};
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

fn encoded_for(root: VocabularyRoot, chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        root,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("complete test chain")
}

fn encoded(chain: &[u16]) -> VocabularyEncodedId {
    encoded_for(VocabularyRoot::Universal, chain)
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

#[test]
fn exact_reference_mappings_lower_recursively_without_rebinding_declarations() {
    let universal_vector = encoded(&[4]);
    let universal_integer = encoded(&[3]);
    let rust_vec = encoded_for(VocabularyRoot::Rust, &[4]);
    let rust_u64 = encoded_for(VocabularyRoot::Rust, &[3]);
    let vector_mapping =
        SliceOneVocabularyReferenceMapping::new(universal_vector.clone(), rust_vec.clone())
            .expect("Universal to Rust mapping");
    let integer_mapping =
        SliceOneVocabularyReferenceMapping::new(universal_integer.clone(), rust_u64.clone())
            .expect("Universal to Rust mapping");
    let forward = SliceOneTransformation::with_reference_mappings(vec![
        vector_mapping.clone(),
        integer_mapping.clone(),
    ])
    .expect("distinct mapping sources");
    let reverse =
        SliceOneTransformation::with_reference_mappings(vec![integer_mapping, vector_mapping])
            .expect("distinct mapping sources");
    assert_eq!(forward, reverse);

    let declaration = encoded(&[42, 8]);
    let ethos = WholeEthos::new(vec![ethos_newtype(
        WholeEthosVisibility::Public,
        declaration.clone(),
        WholeEthosVisibility::Private,
        WholeEthosTypeReference::Application(WholeEthosTypeApplication::new(
            universal_vector,
            WholeEthosTypeReference::Application(WholeEthosTypeApplication::new(
                universal_integer.clone(),
                WholeEthosTypeReference::Identity(universal_integer),
            )),
        )),
    )]);
    let logos = forward.lower(&ethos);

    assert_eq!(
        logos,
        WholeLogos::new(vec![WholeLogosItem::Newtype(WholeLogosNewtype::new(
            WholeLogosVisibility::Public,
            declaration,
            WholeLogosVisibility::Private,
            WholeLogosTypeReference::Application(WholeLogosTypeApplication::new(
                rust_vec,
                WholeLogosTypeReference::Application(WholeLogosTypeApplication::new(
                    rust_u64.clone(),
                    WholeLogosTypeReference::Identity(rust_u64),
                )),
            )),
        ))])
    );
    let archive = logos.to_archive_bytes().expect("archive transformed Logos");
    assert_eq!(
        WholeLogos::from_archive_bytes(&archive).expect("restore transformed Logos"),
        logos
    );
}

#[test]
fn invalid_roots_and_duplicate_mapping_sources_refuse_typed() {
    let universal_vector = encoded(&[4]);
    let universal_integer = encoded(&[3]);
    let rust_vec = encoded_for(VocabularyRoot::Rust, &[4]);
    let rust_u64 = encoded_for(VocabularyRoot::Rust, &[3]);

    assert_eq!(
        SliceOneVocabularyReferenceMapping::new(rust_vec.clone(), rust_u64.clone()),
        Err(SliceOneTransformationError::MappingSourceRoot {
            found: VocabularyRoot::Rust,
        })
    );
    assert_eq!(
        SliceOneVocabularyReferenceMapping::new(
            universal_vector.clone(),
            universal_integer.clone(),
        ),
        Err(SliceOneTransformationError::MappingTargetRoot {
            found: VocabularyRoot::Universal,
        })
    );

    let first = SliceOneVocabularyReferenceMapping::new(universal_vector.clone(), rust_vec)
        .expect("first mapping");
    let repeated = SliceOneVocabularyReferenceMapping::new(universal_vector.clone(), rust_u64)
        .expect("repeated source remains individually valid");
    assert_eq!(
        SliceOneTransformation::with_reference_mappings(vec![first, repeated]),
        Err(SliceOneTransformationError::DuplicateMappingSource {
            source: universal_vector,
        })
    );
}

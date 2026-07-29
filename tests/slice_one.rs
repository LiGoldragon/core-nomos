//! Focused behavior witnesses for the first direct typed transformation.

use core_nomos::SliceOneTransformation;
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use slice_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosItem, WholeEthosNewtype, WholeEthosVisibility,
    WholeEthosWrappedField,
};
use slice_core_logos::{WholeLogos, WholeLogosItem, WholeLogosNewtype, WholeLogosVisibility};

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
    wrapped: VocabularyEncodedId,
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
            integer.clone(),
        ),
        ethos_newtype(
            WholeEthosVisibility::Private,
            internal_counter.clone(),
            WholeEthosVisibility::Public,
            nested_integer.clone(),
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
                integer,
            )),
            WholeLogosItem::Newtype(WholeLogosNewtype::new(
                WholeLogosVisibility::Private,
                internal_counter,
                WholeLogosVisibility::Public,
                nested_integer,
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

//! Focused witnesses for the Nexus structural transformation.

use core_nomos::{
    NexusStructuralTransformation, NexusTransformation, NexusTransformationError,
    NexusVocabularyReferenceMapping,
};
use encoded_name_table::LocalEncodedId;
use nexus_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosEnumeration, WholeEthosFileKind,
    WholeEthosHeader, WholeEthosItem, WholeEthosMethod, WholeEthosNexusBody, WholeEthosStruct,
    WholeEthosTrait, WholeEthosTupleFields, WholeEthosTypeReference, WholeEthosVariant,
    WholeEthosVariantPayload, WholeEthosVisibility,
};
use nexus_core_logos::{WholeLogosItem, WholeLogosTypeReference, WholeLogosVariantPayload};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

fn identity(root: VocabularyRoot, local: u16) -> VocabularyEncodedId {
    VocabularyEncodedId::new(root, vec![LocalEncodedId::new(local)])
        .expect("complete fixture identity")
}

fn universal(local: u16) -> VocabularyEncodedId {
    identity(VocabularyRoot::Universal, local)
}

fn reference(local: u16) -> WholeEthosTypeReference {
    WholeEthosTypeReference::Identity(universal(local))
}

fn nexus_document() -> WholeEthos {
    let decision = WholeEthosEnumeration::new(
        universal(10),
        WholeEthosVisibility::Public,
        WholeEthosAttributes,
        vec![
            WholeEthosVariant::new(
                universal(11),
                WholeEthosAttributes,
                WholeEthosVariantPayload::Unit,
            ),
            WholeEthosVariant::new(
                universal(12),
                WholeEthosAttributes,
                WholeEthosVariantPayload::Tuple(
                    WholeEthosTupleFields::new(vec![reference(13)]).expect("one decision payload"),
                ),
            ),
        ],
    )
    .expect("decision variants");
    let context = WholeEthosStruct::new(universal(14), vec![reference(15), reference(16)])
        .expect("context fields");
    let trait_definition = WholeEthosTrait::new(
        universal(20),
        vec![WholeEthosMethod::new(
            universal(21),
            vec![reference(15), reference(14)],
            reference(10),
        )],
    )
    .expect("trait method");
    WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Nexus, 1).expect("Nexus header"),
        WholeEthosBody::Nexus(WholeEthosNexusBody::new(
            vec![
                WholeEthosItem::Enumeration(decision),
                WholeEthosItem::Struct(context),
            ],
            vec![trait_definition],
        )),
    )
    .expect("typed Nexus document")
}

#[test]
fn nexus_traits_lower_first_and_types_remain_plain_without_identity_allocation() {
    let guardian_reason = universal(13);
    let rust_guardian_reason = identity(VocabularyRoot::Rust, 103);
    let transformation = NexusTransformation::with_reference_mappings(vec![
        NexusVocabularyReferenceMapping::new(guardian_reason.clone(), rust_guardian_reason.clone())
            .expect("typed reference mapping"),
    ])
    .expect("unique mapping source");
    let logos = transformation
        .lower(&nexus_document())
        .expect("lower Nexus document");

    let [
        WholeLogosItem::TraitDef(_),
        WholeLogosItem::Enumeration(decision),
        WholeLogosItem::Struct(context),
    ] = logos.items()
    else {
        panic!("traits precede Nexus operand types")
    };
    assert_eq!(context.fields().len(), 2);
    let WholeLogosVariantPayload::Tuple(rejected) = decision.variants()[1].payload() else {
        panic!("Rejected carries one reason")
    };
    assert_eq!(
        rejected.fields(),
        &[WholeLogosTypeReference::Identity(rust_guardian_reason)]
    );
    assert_eq!(
        transformation.reference_mappings()[0].source(),
        &guardian_reason
    );
    let archive = logos.to_archive_bytes().expect("archive Nexus Logos");
    assert_eq!(
        nexus_core_logos::WholeLogos::from_archive_bytes(&archive).expect("restore Nexus Logos"),
        logos
    );
}

#[test]
fn non_nexus_documents_refuse_at_the_typed_boundary() {
    let interface = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Interface, 1).expect("Interface header"),
        WholeEthosBody::Interface(nexus_core_ethos::WholeEthosInterfaceBody::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )),
    )
    .expect("typed Interface document");
    assert_eq!(
        NexusTransformation::new().lower(&interface),
        Err(NexusTransformationError::UnsupportedFileKind {
            found: WholeEthosFileKind::Interface,
        })
    );
}

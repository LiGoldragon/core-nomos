//! Focused witnesses for the Nexus structural transformation.

use core_nomos::{
    InterfaceTypeStructuralTransformation, NexusStructuralTransformation, NexusTransformation,
    NexusTransformationError, NexusVocabularyReferenceMapping,
    TypeDeclarationStructuralTransformation,
};
use encoded_name_table::LocalEncodedId;
use nexus_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosEnumeration, WholeEthosFileKind,
    WholeEthosHeader, WholeEthosInterfaceBody, WholeEthosItem, WholeEthosMethod, WholeEthosNewtype,
    WholeEthosNexusBody, WholeEthosStruct, WholeEthosTrait, WholeEthosTupleFields,
    WholeEthosTypeReference, WholeEthosVariant, WholeEthosVariantPayload, WholeEthosVisibility,
    WholeEthosWrappedField,
};
use nexus_core_logos::{
    WholeLogosItem, WholeLogosTypeAttributes, WholeLogosTypeReference, WholeLogosVariantPayload,
};
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
    assert_eq!(decision.attributes(), WholeLogosTypeAttributes::Plain);
    assert_eq!(context.attributes(), WholeLogosTypeAttributes::Plain);
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
            expected: WholeEthosFileKind::Nexus,
            found: WholeEthosFileKind::Interface,
        })
    );
}

#[test]
fn interface_shared_types_lower_with_wire_attributes_without_membership_projection() {
    let interface = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Interface, 1).expect("Interface header"),
        WholeEthosBody::Interface(WholeEthosInterfaceBody::new(
            vec![WholeEthosNewtype::new(
                universal(30),
                WholeEthosVisibility::Public,
                WholeEthosAttributes,
                WholeEthosWrappedField::new(WholeEthosVisibility::Private, reference(31)),
            )],
            Vec::new(),
            Vec::new(),
            vec![
                WholeEthosItem::Struct(
                    WholeEthosStruct::new(universal(32), vec![reference(31)])
                        .expect("wire struct field"),
                ),
                WholeEthosItem::Enumeration(
                    WholeEthosEnumeration::new(
                        universal(33),
                        WholeEthosVisibility::Public,
                        WholeEthosAttributes,
                        vec![WholeEthosVariant::new(
                            universal(34),
                            WholeEthosAttributes,
                            WholeEthosVariantPayload::Tuple(
                                WholeEthosTupleFields::new(vec![reference(31)])
                                    .expect("single wire payload"),
                            ),
                        )],
                    )
                    .expect("wire enumeration variant"),
                ),
            ],
        )),
    )
    .expect("typed Interface document");

    let logos = NexusTransformation::new()
        .lower_interface_types(&interface)
        .expect("lower only Interface shared types");
    let [
        WholeLogosItem::Struct(structure),
        WholeLogosItem::Enumeration(enumeration),
    ] = logos.items()
    else {
        panic!("only Interface.types is projected in this slice")
    };
    assert_eq!(structure.attributes(), WholeLogosTypeAttributes::Wire);
    assert_eq!(enumeration.attributes(), WholeLogosTypeAttributes::Wire);

    let nexus_core_ethos::WholeEthosBody::Interface(body) = interface.body() else {
        panic!("Interface body")
    };
    assert_eq!(
        NexusTransformation::new()
            .lower_type_declarations(body.types(), WholeLogosTypeAttributes::Wire)
            .expect("project the explicit declaration slice"),
        logos,
    );
}

#[test]
fn nexus_lowering_refuses_multi_field_tuple_payload_without_rewriting() {
    let enumeration = WholeEthosEnumeration::new(
        universal(40),
        WholeEthosVisibility::Public,
        WholeEthosAttributes,
        vec![WholeEthosVariant::new(
            universal(41),
            WholeEthosAttributes,
            WholeEthosVariantPayload::Tuple(
                WholeEthosTupleFields::new(vec![reference(42), reference(43)])
                    .expect("Ethos carrier still represents authored arity"),
            ),
        )],
    )
    .expect("typed enumeration");
    let nexus = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Nexus, 1).expect("Nexus header"),
        WholeEthosBody::Nexus(WholeEthosNexusBody::new(
            vec![WholeEthosItem::Enumeration(enumeration)],
            Vec::new(),
        )),
    )
    .expect("typed Nexus document");

    assert_eq!(
        NexusTransformation::new().lower(&nexus),
        Err(NexusTransformationError::UnsupportedVariantTupleArity { found: 2 })
    );
}

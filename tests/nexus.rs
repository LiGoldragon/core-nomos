//! Focused witnesses for the Nexus structural transformation.

use core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosEnumeration, WholeEthosFileKind,
    WholeEthosHeader, WholeEthosInterfaceBody, WholeEthosItem, WholeEthosNewtype,
    WholeEthosNexusBody, WholeEthosQuality, WholeEthosSemaBody, WholeEthosStreamInitiation,
    WholeEthosStruct, WholeEthosTable, WholeEthosTrait, WholeEthosTupleFields,
    WholeEthosTypeApplication, WholeEthosTypeParameter, WholeEthosTypeReference, WholeEthosVariant,
    WholeEthosVariantPayload, WholeEthosVisibility, WholeEthosWrappedField,
};
use core_logos::{
    WholeLogosItem, WholeLogosTypeAttributes, WholeLogosTypeParameter, WholeLogosTypeReference,
    WholeLogosVariantPayload,
};
use core_nomos::{
    BundleStorageProvenance, ExternalStorageProvenance, InterfaceRoleIdentities,
    InterfaceStructuralTransformation, NexusStructuralTransformation, NexusTransformation,
    NexusTransformationError, NexusVocabularyReferenceMapping, NomosStorageProvenance,
    PreservedSemaFamilyProvenance, SemaStructuralTransformation, StorageProvenanceOwner,
    StreamLifecycleIdentities, TypeDeclarationStructuralTransformation,
};
use encoded_name_table::LocalEncodedId;
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

fn storage_provenance(documents: &[WholeEthos], entries: &[(u16, u8)]) -> BundleStorageProvenance {
    let owner = StorageProvenanceOwner::new(
        "test://published-storage-producer".to_owned(),
        "test-revision".to_owned(),
    )
    .expect("published owner evidence");
    BundleStorageProvenance::from_documents(
        documents.iter().cloned(),
        entries
            .iter()
            .map(|(identity, byte)| {
                ExternalStorageProvenance::new(universal(*identity), [*byte; 32], owner.clone())
                    .expect("Universal storage provenance")
            })
            .collect(),
    )
    .expect("complete bundle storage provenance")
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
    let trait_definition = WholeEthosTrait::new(universal(20));
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
        core_logos::WholeLogos::from_archive_bytes(&archive).expect("restore Nexus Logos"),
        logos
    );
}

#[test]
fn non_nexus_documents_refuse_at_the_typed_boundary() {
    let interface = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Interface, 1).expect("Interface header"),
        WholeEthosBody::Interface(core_ethos::WholeEthosInterfaceBody::new(
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
fn interface_positions_lower_to_wire_types_memberships_and_resolved_stream_lifecycle() {
    let interface = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Interface, 1).expect("Interface header"),
        WholeEthosBody::Interface(WholeEthosInterfaceBody::new(
            vec![WholeEthosNewtype::new(
                universal(30),
                WholeEthosVisibility::Public,
                WholeEthosAttributes,
                WholeEthosWrappedField::new(WholeEthosVisibility::Private, reference(31)),
            )],
            vec![WholeEthosNewtype::new(
                universal(36),
                WholeEthosVisibility::Public,
                WholeEthosAttributes,
                WholeEthosWrappedField::new(WholeEthosVisibility::Private, reference(31)),
            )],
            vec![WholeEthosStruct::new(universal(38), vec![reference(31)]).expect("refusal field")],
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
                WholeEthosItem::StreamInitiation(WholeEthosStreamInitiation {
                    stream: universal(40),
                    query: reference(32),
                    event: reference(37),
                }),
            ],
        )),
    )
    .expect("typed Interface document");

    let roles = InterfaceRoleIdentities::new(universal(60), universal(61), universal(62))
        .expect("distinct Universal roles");
    let outcome = NexusTransformation::new()
        .with_stream_lifecycle_identities(vec![
            StreamLifecycleIdentities::new(
                universal(40),
                universal(63),
                universal(64),
                universal(65),
                universal(66),
                universal(67),
            )
            .expect("distinct stream lifecycle identities"),
        ])
        .expect("one lifecycle assignment")
        .lower_interface(&interface, &roles)
        .expect("lower structural Interface surface");
    let [
        WholeLogosItem::Newtype(input),
        WholeLogosItem::TraitImpl(input_membership),
        WholeLogosItem::Newtype(output),
        WholeLogosItem::TraitImpl(output_membership),
        WholeLogosItem::Struct(refusal),
        WholeLogosItem::TraitImpl(refusal_membership),
        WholeLogosItem::Struct(structure),
        WholeLogosItem::Enumeration(enumeration),
        WholeLogosItem::StreamLifecycle(lifecycle),
    ] = outcome.logos().items()
    else {
        panic!("Interface declaration and membership order")
    };
    for attributes in [
        input.attributes(),
        output.attributes(),
        refusal.attributes(),
        structure.attributes(),
        enumeration.attributes(),
    ] {
        assert_eq!(attributes, WholeLogosTypeAttributes::Wire);
    }
    assert_eq!(
        input_membership.implemented_trait(),
        &WholeLogosTypeReference::Identity(universal(60))
    );
    assert_eq!(
        input_membership.implementing_type(),
        &WholeLogosTypeReference::Identity(universal(30))
    );
    assert_eq!(
        output_membership.implemented_trait(),
        &WholeLogosTypeReference::Identity(universal(61))
    );
    assert_eq!(
        output_membership.implementing_type(),
        &WholeLogosTypeReference::Identity(universal(36))
    );
    assert_eq!(
        refusal_membership.implemented_trait(),
        &WholeLogosTypeReference::Identity(universal(62))
    );
    assert_eq!(
        refusal_membership.implementing_type(),
        &WholeLogosTypeReference::Identity(universal(38))
    );
    assert!(input_membership.associated_type_bindings().is_empty());
    assert!(output_membership.associated_type_bindings().is_empty());
    assert!(refusal_membership.associated_type_bindings().is_empty());
    assert_eq!(structure.attributes(), WholeLogosTypeAttributes::Wire);
    assert_eq!(enumeration.attributes(), WholeLogosTypeAttributes::Wire);
    assert_eq!(lifecycle.stream(), &universal(40));
    assert_eq!(lifecycle.initiation().input(), &universal(63));
    assert_eq!(
        lifecycle.initiation().query(),
        &WholeLogosTypeReference::Identity(universal(32))
    );
    assert_eq!(lifecycle.initiation().success().identity(), &universal(64));
    assert_eq!(
        lifecycle.initiation().success().event(),
        &WholeLogosTypeReference::Identity(universal(37))
    );
    assert_eq!(lifecycle.initiation().refusal(), &universal(65));
    assert_eq!(lifecycle.termination().input(), &universal(66));
    assert_eq!(
        lifecycle.termination().identity(),
        lifecycle.initiation().success().identity()
    );
    assert_eq!(lifecycle.termination().refusal(), &universal(67));

    let core_ethos::WholeEthosBody::Interface(body) = interface.body() else {
        panic!("Interface body")
    };
    let shared_types = NexusTransformation::new()
        .lower_type_declarations(&body.types()[..2], WholeLogosTypeAttributes::Wire)
        .expect("project declaration-only shared type slice");
    assert_eq!(shared_types.items(), &outcome.logos().items()[6..8],);
}

#[test]
fn stream_initiation_refuses_without_caller_authored_lifecycle_identities() {
    let interface = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Interface, 1).expect("Interface header"),
        WholeEthosBody::Interface(WholeEthosInterfaceBody::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![WholeEthosItem::StreamInitiation(
                WholeEthosStreamInitiation {
                    stream: universal(40),
                    query: reference(32),
                    event: reference(37),
                },
            )],
        )),
    )
    .expect("typed Interface document");
    let roles = InterfaceRoleIdentities::new(universal(60), universal(61), universal(62))
        .expect("distinct Universal roles");

    assert_eq!(
        NexusTransformation::new().lower_interface(&interface, &roles),
        Err(NexusTransformationError::MissingStreamLifecycleIdentities {
            stream: universal(40),
        })
    );
}

#[test]
fn interface_role_configuration_refuses_non_universal_and_duplicate_identities() {
    assert_eq!(
        InterfaceRoleIdentities::new(
            identity(VocabularyRoot::Rust, 60),
            universal(61),
            universal(62),
        ),
        Err(NexusTransformationError::InterfaceRoleRoot {
            role: "Input",
            found: VocabularyRoot::Rust,
        })
    );
    assert_eq!(
        InterfaceRoleIdentities::new(universal(60), universal(60), universal(62)),
        Err(NexusTransformationError::DuplicateInterfaceRoleIdentity {
            first_role: "Input",
            second_role: "Output",
            identity: universal(60),
        })
    );
}

#[test]
fn nexus_lowering_retains_nary_type_applications_in_authored_order() {
    let nexus = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Nexus, 1).expect("Nexus header"),
        WholeEthosBody::Nexus(WholeEthosNexusBody::new(
            vec![WholeEthosItem::Newtype(WholeEthosNewtype::new(
                universal(80),
                WholeEthosVisibility::Public,
                WholeEthosAttributes,
                WholeEthosWrappedField::new(
                    WholeEthosVisibility::Private,
                    WholeEthosTypeReference::Application(
                        WholeEthosTypeApplication::new(
                            WholeEthosQuality::Shape(universal(81)),
                            vec![reference(82), reference(83)],
                        )
                        .expect("two authored arguments"),
                    ),
                ),
            ))],
            Vec::new(),
        )),
    )
    .expect("typed Nexus document");

    let logos = NexusTransformation::new()
        .lower(&nexus)
        .expect("retain both application arguments");
    let [WholeLogosItem::Newtype(newtype)] = logos.items() else {
        panic!("n-ary application newtype")
    };
    let WholeLogosTypeReference::Application(application) = newtype.wrapped() else {
        panic!("n-ary application")
    };
    assert_eq!(application.head(), &universal(81));
    assert_eq!(
        application.arguments(),
        &[
            WholeLogosTypeReference::Identity(universal(82)),
            WholeLogosTypeReference::Identity(universal(83)),
        ]
    );
}

#[test]
fn nexus_lowering_retains_picked_up_parameter_names_and_quality_bounds() {
    let sortable = universal(84);
    let left = universal(88);
    let right = universal(89);
    let result = universal(85);
    let error = universal(86);
    let nexus = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Nexus, 1).expect("Nexus header"),
        WholeEthosBody::Nexus(WholeEthosNexusBody::new(
            vec![WholeEthosItem::Newtype(WholeEthosNewtype::new(
                universal(87),
                WholeEthosVisibility::Public,
                WholeEthosAttributes,
                WholeEthosWrappedField::new(
                    WholeEthosVisibility::Private,
                    WholeEthosTypeReference::Application(
                        WholeEthosTypeApplication::new(
                            WholeEthosQuality::Shape(result),
                            vec![
                                WholeEthosTypeReference::Parameter(WholeEthosTypeParameter::new(
                                    left.clone(),
                                    WholeEthosQuality::Trait(sortable.clone()),
                                )),
                                WholeEthosTypeReference::Parameter(WholeEthosTypeParameter::new(
                                    right.clone(),
                                    WholeEthosQuality::Trait(sortable.clone()),
                                )),
                                WholeEthosTypeReference::Identity(error),
                            ],
                        )
                        .expect("Result application"),
                    ),
                ),
            ))],
            Vec::new(),
        )),
    )
    .expect("typed parameterized Nexus document");

    let logos = NexusTransformation::new()
        .lower(&nexus)
        .expect("preserve picked-up parameter");
    let [WholeLogosItem::Newtype(newtype)] = logos.items() else {
        panic!("parameterized newtype")
    };
    assert_eq!(
        newtype.type_parameters(),
        &[
            WholeLogosTypeParameter::new(left.clone(), sortable.clone()),
            WholeLogosTypeParameter::new(right.clone(), sortable),
        ]
    );
    let WholeLogosTypeReference::Application(application) = newtype.wrapped() else {
        panic!("Result application")
    };
    assert_eq!(
        application.arguments()[0],
        WholeLogosTypeReference::Parameter(left)
    );
    assert_eq!(
        application.arguments()[1],
        WholeLogosTypeReference::Parameter(right)
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

#[test]
fn sema_record_types_become_stored_values_and_local_tables_become_typed_specifications() {
    let record = universal(80);
    let key = universal(81);
    let table = universal(82);
    let sema = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Sema, 1).expect("Sema header"),
        WholeEthosBody::Sema(WholeEthosSemaBody::new(
            vec![WholeEthosItem::Struct(
                WholeEthosStruct::new(record.clone(), vec![reference(83)])
                    .expect("stored record field"),
            )],
            vec![WholeEthosTable::new(
                table.clone(),
                WholeEthosTypeReference::Identity(record.clone()),
                WholeEthosTypeReference::Identity(key.clone()),
            )],
        )),
    )
    .expect("typed Sema document");

    let provenance = storage_provenance(std::slice::from_ref(&sema), &[(81, 1), (83, 2)]);
    let outcome = NexusTransformation::new()
        .lower_sema(&sema, &provenance)
        .expect("lower Sema storage declarations");
    let [
        WholeLogosItem::Struct(stored),
        WholeLogosItem::Table(specification),
    ] = outcome.logos().items()
    else {
        panic!("stored record precedes its table specification")
    };
    assert_eq!(stored.attributes(), WholeLogosTypeAttributes::Stored);
    assert_eq!(specification.name(), &table);
    assert_eq!(
        specification.record(),
        &WholeLogosTypeReference::Identity(record),
    );
    assert_eq!(specification.key(), &WholeLogosTypeReference::Identity(key),);
}

#[test]
fn preserved_current_spirit_v14_family_requires_the_catalogue_and_exact_generated_layout() {
    let record = universal(126);
    let key = universal(109);
    let entry = universal(125);
    let table = universal(142);
    let sema = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Sema, 1).expect("Sema header"),
        WholeEthosBody::Sema(WholeEthosSemaBody::new(
            vec![WholeEthosItem::Struct(
                WholeEthosStruct::new(record.clone(), vec![reference(109), reference(125)])
                    .expect("stored record fields"),
            )],
            vec![WholeEthosTable::new(
                table.clone(),
                WholeEthosTypeReference::Identity(record.clone()),
                WholeEthosTypeReference::Identity(key.clone()),
            )],
        )),
    )
    .expect("typed current-v14-shaped Sema document");
    let external = {
        let owner = StorageProvenanceOwner::new(
            "test://current-spirit-v14-layout".to_owned(),
            "test-revision".to_owned(),
        )
        .expect("test external owner");
        vec![
            ExternalStorageProvenance::new(key.clone(), [1; 32], owner.clone())
                .expect("key provenance"),
            ExternalStorageProvenance::new(entry, [2; 32], owner).expect("entry provenance"),
        ]
    };
    let ordinary = BundleStorageProvenance::from_documents(vec![sema.clone()], external.clone())
        .expect("ordinary bundle provenance");
    let record_layout = ordinary
        .storage_fingerprint(&WholeEthosTypeReference::Identity(record.clone()))
        .expect("complete record layout")
        .bytes();
    let key_layout = ordinary
        .storage_fingerprint(&WholeEthosTypeReference::Identity(key.clone()))
        .expect("complete key layout")
        .bytes();
    let proof = PreservedSemaFamilyProvenance::new(
        table.clone(),
        record.clone(),
        key.clone(),
        "records".to_owned(),
        "RecordsFamily".to_owned(),
        [
            169, 167, 27, 203, 113, 158, 12, 113, 89, 93, 195, 166, 134, 208, 34, 40, 178, 38, 203,
            139, 155, 209, 108, 101, 12, 183, 180, 233, 6, 84, 230, 177,
        ],
        "7405eee89e3b1b5b6764eb1a50cbdf467b93c9a7".to_owned(),
        14,
        record_layout,
        key_layout,
    )
    .expect("catalogued v14 adoption proof");
    assert_eq!(proof.source(), "https://github.com/LiGoldragon/spirit");
    let provenance = BundleStorageProvenance::from_documents_with_preserved_families(
        vec![sema.clone()],
        external.clone(),
        vec![proof.clone()],
    )
    .expect("one adopted descriptor");
    let outcome = NexusTransformation::new()
        .lower_sema(&sema, &provenance)
        .expect("matching physical descriptor adoption");
    let [WholeLogosItem::Struct(_), WholeLogosItem::Table(table)] = outcome.logos().items() else {
        panic!("stored record and one table")
    };
    let physical = table
        .preserved_sema_family()
        .expect("validated physical descriptor is retained");
    assert_eq!(physical.table_name(), "records");
    assert_eq!(physical.family_name(), "RecordsFamily");
    assert_eq!(
        table.schema_hash().expect("preserved physical schema hash"),
        physical.schema_hash()
    );

    assert!(matches!(
        PreservedSemaFamilyProvenance::new(
            universal(140),
            record.clone(),
            key.clone(),
            "records".to_owned(),
            "RecordsFamily".to_owned(),
            physical.schema_hash(),
            "7405eee89e3b1b5b6764eb1a50cbdf467b93c9a7".to_owned(),
            14,
            record_layout,
            key_layout,
        ),
        Err(
            NexusTransformationError::PreservedSemaFamilyIdentityMismatch {
                position: "table",
                ..
            }
        )
    ));
    assert!(matches!(
        PreservedSemaFamilyProvenance::new(
            table.name().clone(),
            record.clone(),
            key.clone(),
            "records".to_owned(),
            "RecordsFamily".to_owned(),
            physical.schema_hash(),
            "missing-current-revision".to_owned(),
            14,
            record_layout,
            key_layout,
        ),
        Err(NexusTransformationError::PreservedSemaFamilyRevisionMismatch { .. })
    ));
    let mismatched_layout = PreservedSemaFamilyProvenance::new(
        table.name().clone(),
        record,
        key,
        "records".to_owned(),
        "RecordsFamily".to_owned(),
        physical.schema_hash(),
        "7405eee89e3b1b5b6764eb1a50cbdf467b93c9a7".to_owned(),
        14,
        [0; 32],
        key_layout,
    )
    .expect("layout is checked at lowering against the complete bundle");
    let mismatch = BundleStorageProvenance::from_documents_with_preserved_families(
        vec![sema.clone()],
        external,
        vec![mismatched_layout],
    )
    .expect("layout proof is syntactically complete");
    assert!(matches!(
        NexusTransformation::new().lower_sema(&sema, &mismatch),
        Err(NexusTransformationError::PreservedSemaFamilyLayoutMismatch { .. })
    ));
    assert!(matches!(
        BundleStorageProvenance::from_documents_with_preserved_families(
            vec![sema],
            Vec::new(),
            vec![proof.clone(), proof],
        ),
        Err(NexusTransformationError::DuplicatePreservedSemaFamily { .. })
    ));
}

#[test]
fn sema_schema_hash_tracks_direct_and_transitive_layout_under_stable_identities() {
    let record = universal(110);
    let nested = universal(111);
    let table = universal(112);
    let key = universal(113);
    let document = |nested_item: WholeEthosItem| {
        WholeEthos::new(
            WholeEthosHeader::new(WholeEthosFileKind::Sema, 1).expect("Sema header"),
            WholeEthosBody::Sema(WholeEthosSemaBody::new(
                vec![
                    WholeEthosItem::Struct(
                        WholeEthosStruct::new(record.clone(), vec![reference(111)])
                            .expect("record field"),
                    ),
                    nested_item,
                ],
                vec![WholeEthosTable::new(
                    table.clone(),
                    WholeEthosTypeReference::Identity(record.clone()),
                    WholeEthosTypeReference::Identity(key.clone()),
                )],
            )),
        )
        .expect("typed Sema document")
    };
    let nested_newtype = WholeEthosItem::Newtype(WholeEthosNewtype::new(
        nested.clone(),
        WholeEthosVisibility::Public,
        WholeEthosAttributes,
        WholeEthosWrappedField::new(WholeEthosVisibility::Private, reference(114)),
    ));
    let nested_struct = WholeEthosItem::Struct(
        WholeEthosStruct::new(nested, vec![reference(114), reference(115)])
            .expect("changed nested fields"),
    );
    let original_document = document(nested_newtype);
    let original_provenance = storage_provenance(
        std::slice::from_ref(&original_document),
        &[(113, 3), (114, 4), (115, 5)],
    );
    let original = NexusTransformation::new()
        .lower_sema(&original_document, &original_provenance)
        .expect("original storage graph");
    let changed_document = document(nested_struct);
    let changed_provenance = storage_provenance(
        std::slice::from_ref(&changed_document),
        &[(113, 3), (114, 4), (115, 5)],
    );
    let changed = NexusTransformation::new()
        .lower_sema(&changed_document, &changed_provenance)
        .expect("changed storage graph");
    let schema_hash = |outcome: &core_nomos::SemaTransformationOutcome| {
        let WholeLogosItem::Table(table) = outcome.logos().items().last().expect("table item")
        else {
            panic!("table is final")
        };
        table.schema_hash().expect("portable table schema")
    };

    assert_ne!(schema_hash(&original), schema_hash(&changed));
}

#[test]
fn sema_reachable_external_storage_shape_requires_an_explicit_fingerprint() {
    let sema = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Sema, 1).expect("Sema header"),
        WholeEthosBody::Sema(WholeEthosSemaBody::new(
            vec![WholeEthosItem::Struct(
                WholeEthosStruct::new(universal(120), vec![reference(121)])
                    .expect("stored record field"),
            )],
            vec![WholeEthosTable::new(
                universal(122),
                reference(120),
                reference(123),
            )],
        )),
    )
    .expect("typed Sema document");

    let provenance = storage_provenance(std::slice::from_ref(&sema), &[]);
    assert!(matches!(
        NexusTransformation::new().lower_sema(&sema, &provenance),
        Err(NexusTransformationError::MissingExternalStorageProvenance { identity })
            if identity == universal(121)
    ));
}

#[test]
fn sema_resolves_same_bundle_interface_record_without_external_fingerprint() {
    let primitive = universal(130);
    let identifier = universal(131);
    let entry = universal(132);
    let table = universal(133);
    let interface = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Interface, 1).expect("Interface header"),
        WholeEthosBody::Interface(WholeEthosInterfaceBody::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![
                WholeEthosItem::Newtype(WholeEthosNewtype::new(
                    identifier.clone(),
                    WholeEthosVisibility::Public,
                    WholeEthosAttributes,
                    WholeEthosWrappedField::new(
                        WholeEthosVisibility::Private,
                        WholeEthosTypeReference::Identity(primitive.clone()),
                    ),
                )),
                WholeEthosItem::Struct(
                    WholeEthosStruct::new(entry.clone(), vec![reference(131)])
                        .expect("Interface entry field"),
                ),
            ],
        )),
    )
    .expect("typed Interface document");
    let sema = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Sema, 1).expect("Sema header"),
        WholeEthosBody::Sema(WholeEthosSemaBody::new(
            Vec::new(),
            vec![WholeEthosTable::new(
                table.clone(),
                WholeEthosTypeReference::Identity(entry.clone()),
                WholeEthosTypeReference::Identity(identifier.clone()),
            )],
        )),
    )
    .expect("typed Sema document");

    let forward = storage_provenance(&[interface.clone(), sema.clone()], &[(130, 7)]);
    let reverse = storage_provenance(&[sema.clone(), interface.clone()], &[(130, 7)]);
    let forward_logos = NexusTransformation::new()
        .lower_sema(&sema, &forward)
        .expect("same-bundle Interface record resolves");
    let reverse_logos = NexusTransformation::new()
        .lower_sema(&sema, &reverse)
        .expect("bundle registration order has no effect");

    assert_eq!(forward_logos, reverse_logos);
    let [WholeLogosItem::Table(specification)] = forward_logos.logos().items() else {
        panic!("one imported-record table specification")
    };
    assert_eq!(specification.name(), &table);
    assert_eq!(
        specification.record(),
        &WholeLogosTypeReference::Identity(entry)
    );
    assert_eq!(
        specification.key(),
        &WholeLogosTypeReference::Identity(identifier)
    );
}

#[test]
fn sema_table_refuses_a_record_not_declared_by_the_bundle() {
    let foreign_record = universal(140);
    let table = universal(141);
    let sema = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Sema, 1).expect("Sema header"),
        WholeEthosBody::Sema(WholeEthosSemaBody::new(
            Vec::new(),
            vec![WholeEthosTable::new(
                table.clone(),
                reference(140),
                reference(142),
            )],
        )),
    )
    .expect("typed Sema document");
    let provenance = storage_provenance(std::slice::from_ref(&sema), &[(142, 8)]);

    assert!(matches!(
        NexusTransformation::new().lower_sema(&sema, &provenance),
        Err(NexusTransformationError::SemaTableRecordNotBundleOwned {
            table: refused_table,
            record,
        }) if refused_table == table && record == foreign_record
    ));
}

#[test]
fn sema_bundle_provenance_refuses_cycles_and_duplicate_declarations() {
    let first = universal(150);
    let second = universal(151);
    let table = universal(152);
    let cyclic = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Sema, 1).expect("Sema header"),
        WholeEthosBody::Sema(WholeEthosSemaBody::new(
            vec![
                WholeEthosItem::Struct(
                    WholeEthosStruct::new(first.clone(), vec![reference(151)])
                        .expect("first cyclic field"),
                ),
                WholeEthosItem::Struct(
                    WholeEthosStruct::new(second.clone(), vec![reference(150)])
                        .expect("second cyclic field"),
                ),
            ],
            vec![WholeEthosTable::new(table, reference(150), reference(153))],
        )),
    )
    .expect("typed cyclic Sema document");
    let cyclic_provenance = storage_provenance(std::slice::from_ref(&cyclic), &[(153, 9)]);
    assert!(matches!(
        NexusTransformation::new().lower_sema(&cyclic, &cyclic_provenance),
        Err(NexusTransformationError::CyclicSemaStorageShape { identity }) if identity == first
    ));

    let duplicate = WholeEthos::new(
        WholeEthosHeader::new(WholeEthosFileKind::Interface, 1).expect("Interface header"),
        WholeEthosBody::Interface(WholeEthosInterfaceBody::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![WholeEthosItem::Struct(
                WholeEthosStruct::new(first.clone(), vec![reference(153)])
                    .expect("duplicate declaration field"),
            )],
        )),
    )
    .expect("typed duplicate Interface document");
    assert!(matches!(
        BundleStorageProvenance::from_documents(vec![cyclic, duplicate], Vec::new()),
        Err(NexusTransformationError::DuplicateBundleStorageDeclaration { identity }) if identity == first
    ));
}

#[test]
fn sema_table_refuses_applied_record_and_key_shapes_without_partial_logos() {
    let record = universal(90);
    let table = universal(91);
    let document = |record_reference, key_reference| {
        WholeEthos::new(
            WholeEthosHeader::new(WholeEthosFileKind::Sema, 1).expect("Sema header"),
            WholeEthosBody::Sema(WholeEthosSemaBody::new(
                vec![WholeEthosItem::Struct(
                    WholeEthosStruct::new(record.clone(), vec![reference(92)])
                        .expect("stored record field"),
                )],
                vec![WholeEthosTable::new(
                    table.clone(),
                    record_reference,
                    key_reference,
                )],
            )),
        )
        .expect("typed Sema document")
    };
    let applied = |payload| {
        WholeEthosTypeReference::Application(
            WholeEthosTypeApplication::new(WholeEthosQuality::Shape(universal(93)), vec![payload])
                .expect("one application argument"),
        )
    };

    let applied_record = document(applied(reference(90)), reference(94));
    let applied_record_provenance = storage_provenance(std::slice::from_ref(&applied_record), &[]);
    assert!(matches!(
        NexusTransformation::new().lower_sema(&applied_record, &applied_record_provenance),
        Err(NexusTransformationError::InvalidSemaTableRecordShape { .. })
    ));
    let applied_key = document(reference(90), applied(reference(94)));
    let applied_key_provenance = storage_provenance(std::slice::from_ref(&applied_key), &[]);
    assert!(matches!(
        NexusTransformation::new().lower_sema(&applied_key, &applied_key_provenance),
        Err(NexusTransformationError::InvalidSemaTableKeyShape { .. })
    ));
}

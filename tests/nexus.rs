//! Focused witnesses for the Nexus structural transformation.

use core_nomos::{
    InterfaceRoleIdentities, InterfaceStructuralTransformation, NexusStructuralTransformation,
    NexusTransformation, NexusTransformationError, NexusVocabularyReferenceMapping,
    SemaStorageTypeFingerprintMapping, SemaStructuralTransformation, StreamLifecycleIdentities,
    TypeDeclarationStructuralTransformation,
};
use encoded_name_table::LocalEncodedId;
use nexus_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosEnumeration, WholeEthosFileKind,
    WholeEthosHeader, WholeEthosInterfaceBody, WholeEthosItem, WholeEthosNewtype,
    WholeEthosNexusBody, WholeEthosQuality, WholeEthosSemaBody, WholeEthosStreamInitiation,
    WholeEthosStruct, WholeEthosTable, WholeEthosTrait, WholeEthosTupleFields,
    WholeEthosTypeApplication, WholeEthosTypeParameter, WholeEthosTypeReference, WholeEthosVariant,
    WholeEthosVariantPayload, WholeEthosVisibility, WholeEthosWrappedField,
};
use nexus_core_logos::{
    WholeLogosItem, WholeLogosTypeAttributes, WholeLogosTypeParameter, WholeLogosTypeReference,
    WholeLogosVariantPayload,
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

fn storage_transformation(entries: &[(u16, u8)]) -> NexusTransformation {
    NexusTransformation::new()
        .with_storage_fingerprints(
            entries
                .iter()
                .map(|(identity, byte)| {
                    SemaStorageTypeFingerprintMapping::new(universal(*identity), [*byte; 32])
                        .expect("Universal storage mapping")
                })
                .collect(),
        )
        .expect("unique storage fingerprint sources")
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

    let nexus_core_ethos::WholeEthosBody::Interface(body) = interface.body() else {
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

    let outcome = storage_transformation(&[(81, 1), (83, 2)])
        .lower_sema(&sema)
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
    assert!(outcome.deferred_tables().is_empty());
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
    let transformation = storage_transformation(&[(113, 3), (114, 4), (115, 5)]);
    let original = transformation
        .lower_sema(&document(nested_newtype))
        .expect("original storage graph");
    let changed = transformation
        .lower_sema(&document(nested_struct))
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

    assert!(matches!(
        NexusTransformation::new().lower_sema(&sema),
        Err(NexusTransformationError::MissingSemaStorageFingerprint { identity })
            if identity == universal(121)
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

    assert!(matches!(
        NexusTransformation::new().lower_sema(&document(applied(reference(90)), reference(94))),
        Err(NexusTransformationError::InvalidSemaTableRecordShape { .. })
    ));
    assert!(matches!(
        NexusTransformation::new().lower_sema(&document(reference(90), applied(reference(94)))),
        Err(NexusTransformationError::InvalidSemaTableKeyShape { .. })
    ));
}

use std::collections::BTreeMap;

use core_ethos::bootstrap::*;
use core_logos::{
    WholeLogosItem, WholeLogosTypeAttributes, WholeLogosTypeReference, WholeLogosVariantPayload,
};
use core_nomos::{
    BootstrapSliceOneLowering, BootstrapSliceOneLoweringError, ExternalStorageProvenance,
    StorageProvenanceOwner,
};
use name_table::{EncodedName, TextualName};

const AUTHORITY_PROOF: u64 = 0x006e_6f6d_6f73;

fn id(local: u16) -> EncodedName {
    let mut bytes = [0; 16];
    bytes[..2].copy_from_slice(&local.to_le_bytes());
    EncodedName::from_archive_bytes(bytes)
}

fn record(
    module: &[&str],
    owner: Option<EncodedName>,
    name: &str,
    identity: EncodedName,
) -> TextualMetadataRecord {
    TextualMetadataRecord {
        address: TextualProjectionAddress {
            module_path: module.iter().map(|part| (*part).to_owned()).collect(),
            lexical_owner: owner,
            textual_name: TextualName::new(name),
        },
        encoded_name: identity,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestAuthority(u64);

#[derive(Clone, Debug, Eq, PartialEq)]
struct TestReceipt {
    authority: u64,
    transaction: PreparedBootstrapDraft,
}

impl BootstrapNamingAuthority for TestAuthority {
    type Proof = u64;
    type Receipt = TestReceipt;

    fn authorize(
        &self,
        request: BootstrapNamingAuthorityRequest<'_>,
        proof: &Self::Proof,
    ) -> Option<Self::Receipt> {
        (*proof == AUTHORITY_PROOF).then(|| TestReceipt {
            authority: self.0,
            transaction: request.transaction().clone(),
        })
    }

    fn verify_receipt(
        &self,
        request: BootstrapNamingAuthorityRequest<'_>,
        receipt: &Self::Receipt,
    ) -> bool {
        receipt.authority == self.0 && receipt.transaction == *request.transaction()
    }
}

struct Fixture {
    reader: BootstrapReader<TestAuthority>,
    snapshot: TextualMetadataSnapshot,
}

fn fixture(authority: u64) -> Fixture {
    let prior_specs = [
        (
            1,
            "Interface",
            vec![SchemaRole::FileKind(EthosKind::Interface)],
        ),
        (2, "Nexus", vec![SchemaRole::FileKind(EthosKind::Nexus)]),
        (3, "Sema", vec![SchemaRole::FileKind(EthosKind::Sema)]),
        (
            4,
            "Input",
            vec![SchemaRole::InterfaceRole(InterfaceRole::Input)],
        ),
        (
            5,
            "Output",
            vec![SchemaRole::InterfaceRole(InterfaceRole::Output)],
        ),
        (
            6,
            "Refusal",
            vec![SchemaRole::InterfaceRole(InterfaceRole::Refusal)],
        ),
        (
            15,
            "Stream",
            vec![SchemaRole::InterfaceRole(InterfaceRole::Stream)],
        ),
        (7, "String", vec![SchemaRole::Nominal { persistent: true }]),
        (8, "Integer", vec![SchemaRole::Nominal { persistent: true }]),
        (9, "Boolean", vec![SchemaRole::Nominal { persistent: true }]),
        (10, "Unit", vec![SchemaRole::Nominal { persistent: true }]),
        (11, "Vector", vec![SchemaRole::Shape { arity: 1 }]),
        (12, "Option", vec![SchemaRole::Shape { arity: 1 }]),
        (13, "Map", vec![SchemaRole::Shape { arity: 2 }]),
        (14, "Result", vec![SchemaRole::Shape { arity: 2 }]),
        (17, "Sortable", vec![SchemaRole::Trait]),
    ];
    let mut records = Vec::new();
    let mut schemas = Vec::new();
    let mut order = Vec::new();
    for (local, name, roles) in prior_specs {
        let identity = id(local);
        records.push(record(&["builtin"], None, name, identity));
        schemas.push(IdentitySchema::new(identity, roles).expect("valid prior schema"));
        order.push((identity, vec![0x80, local as u8]));
    }
    let snapshot = TextualMetadataSnapshot::new(records).expect("unique prior metadata");
    let schemas = IdentitySchemaCatalog::new(schemas).expect("unique prior schemas");
    let canonical = CanonicalIdentityOrder::new(order).expect("unique prior ordering");
    let priors = BootstrapPriorVocabulary::new(
        BootstrapPriorIdentities {
            interface_kind: id(1),
            nexus_kind: id(2),
            sema_kind: id(3),
            input_role: id(4),
            output_role: id(5),
            refusal_role: id(6),
            stream_role: id(15),
            string_type: id(7),
            integer_type: id(8),
            boolean_type: id(9),
            unit_type: id(10),
            vector_shape: id(11),
            option_shape: id(12),
            map_shape: id(13),
            result_shape: id(14),
        },
        &schemas,
        &snapshot,
    )
    .expect("valid prior vocabulary");
    let admitted = SemaTransactionAssembler::new();
    let catalog = admitted
        .bootstrap_catalog(
            vec!["app".to_owned()],
            BootstrapCatalogMetadata::new(snapshot.clone(), schemas),
            priors,
            BootstrapVersionPolicy::exact(EthosVersion::new(1, 0, 0)),
            canonical,
        )
        .expect("valid catalog");
    let reader = admitted
        .bootstrap_reader(
            admitted.bootstrap_grammar(id(900), id(901)),
            catalog,
            TestAuthority(authority),
        )
        .expect("valid reader");
    Fixture { reader, snapshot }
}

fn seal(fixture: &Fixture, source: &str) -> PreparedBootstrapTransaction<TestAuthority> {
    let admitted = SemaTransactionAssembler::new();
    let plan = fixture.reader.plan(source).expect("valid bootstrap source");
    let mut records = fixture.snapshot.records().to_vec();
    let mut by_occurrence = BTreeMap::new();
    let mut assignments = Vec::new();
    for (index, declaration) in plan.declarations().iter().enumerate() {
        let identity = id(100 + index as u16);
        by_occurrence.insert(declaration.occurrence(), identity);
        assignments.push(admitted.naming_assignment(
            declaration.occurrence(),
            identity,
            IdentityDisposition::New {
                canonical_bytes: declaration.spelling().as_bytes().to_vec(),
            },
        ));
    }
    for (declaration, assignment) in plan.declarations().iter().zip(&assignments) {
        let owner = match declaration.scope() {
            PlannedScope::Module => None,
            PlannedScope::Enum(owner) | PlannedScope::Trait(owner) => Some(by_occurrence[&owner]),
        };
        records.push(record(
            &["app"],
            owner,
            declaration.spelling(),
            *assignment.encoded_name(),
        ));
    }

    fixture
        .reader
        .seal(
            &plan,
            &admitted
                .naming_assignments(assignments)
                .expect("complete authored assignments"),
            &TextualMetadataTransition::new(
                fixture.snapshot.clone(),
                TextualMetadataSnapshot::new(records).expect("complete metadata transition"),
            ),
            &AUTHORITY_PROOF,
        )
        .expect("sealed transaction")
}

fn spelling<'a>(
    transaction: &'a PreparedBootstrapTransaction<TestAuthority>,
    identity: &EncodedName,
) -> &'a str {
    transaction
        .naming_transition()
        .after()
        .record(identity)
        .expect("identity has metadata")
        .address
        .textual_name
        .as_str()
}

fn external_storage(local: u16, fingerprint: u8) -> ExternalStorageProvenance {
    ExternalStorageProvenance::new(
        id(local),
        [fingerprint; 32],
        StorageProvenanceOwner::new(
            "https://github.com/LiGoldragon/bootstrap-fixture".to_owned(),
            "strict-sema-v1".to_owned(),
        )
        .expect("nonempty fixture owner"),
    )
}

#[test]
fn lowers_canonical_types_recursive_shapes_and_every_variant_payload() {
    let fixture = fixture(1);
    let transaction = seal(
        &fixture,
        "Nexus.{1 0 0}\n[]\n{[] [Gamma.[Unit Unary.Result<String Integer> Product.{Map<String Vector<Option<Integer>>> Boolean}] Alpha.Vector<Option<String>> Beta.{String Result<Vector<Integer> Boolean>}]}",
    );

    let logos = BootstrapSliceOneLowering::new()
        .lower(&fixture.reader, &transaction)
        .expect("supported direct lowering");
    assert_eq!(
        logos
            .items()
            .iter()
            .map(|item| match item {
                WholeLogosItem::Newtype(item) => spelling(&transaction, item.name()),
                WholeLogosItem::Struct(item) => spelling(&transaction, item.name()),
                WholeLogosItem::Enumeration(item) => spelling(&transaction, item.name()),
                _ => panic!("bootstrap type lowering emitted a non-type item"),
            })
            .collect::<Vec<_>>(),
        ["Alpha", "Beta", "Gamma"]
    );

    let WholeLogosItem::Newtype(alpha) = &logos.items()[0] else {
        panic!("Alpha is a newtype")
    };
    let WholeLogosTypeReference::Application(vector) = alpha.wrapped() else {
        panic!("outer Vector application retained")
    };
    assert_eq!(vector.head(), &id(11));
    let WholeLogosTypeReference::Application(option) = &vector.arguments()[0] else {
        panic!("nested Option application retained")
    };
    assert_eq!(option.head(), &id(12));
    assert!(
        matches!(option.arguments(), [WholeLogosTypeReference::Identity(identity)] if identity == &id(7))
    );

    let WholeLogosItem::Struct(beta) = &logos.items()[1] else {
        panic!("Beta is a struct")
    };
    assert_eq!(beta.fields().len(), 2);
    assert!(
        matches!(beta.fields()[0], WholeLogosTypeReference::Identity(ref identity) if identity == &id(7))
    );

    let WholeLogosItem::Enumeration(gamma) = &logos.items()[2] else {
        panic!("Gamma is an enumeration")
    };
    assert_eq!(
        gamma
            .variants()
            .iter()
            .map(|variant| spelling(&transaction, variant.name()))
            .collect::<Vec<_>>(),
        ["Product", "Unary", "Unit"]
    );
    assert!(matches!(
        gamma.variants()[0].payload(),
        WholeLogosVariantPayload::Tuple(fields) if fields.fields().len() == 2
    ));
    assert!(matches!(
        gamma.variants()[1].payload(),
        WholeLogosVariantPayload::Tuple(fields) if fields.fields().len() == 1
    ));
    assert!(matches!(
        gamma.variants()[2].payload(),
        WholeLogosVariantPayload::Unit
    ));
}

#[test]
fn lowers_role_free_interface_support_types() {
    let fixture = fixture(2);
    let transaction = seal(
        &fixture,
        "Interface.{1 0 0}\n[]\n{[] [] [] [] [Signal.Result<Vector<String> Integer>]}",
    );
    let logos = BootstrapSliceOneLowering::new()
        .lower(&fixture.reader, &transaction)
        .expect("role-free Interface types are structural");
    let [WholeLogosItem::Newtype(signal)] = logos.items() else {
        panic!("one Interface support type")
    };
    assert_eq!(spelling(&transaction, signal.name()), "Signal");
}

#[test]
fn lowers_sema_record_types_and_tables_with_explicit_storage_provenance() {
    let fixture = fixture(20);
    let transaction = seal(
        &fixture,
        "Sema.{1 0 0}\n[]\n{[Domain.String StoredRecord.{Domain Integer}] [records.{StoredRecord Domain}]}",
    );
    let external = [external_storage(7, 7), external_storage(8, 8)];

    let logos = BootstrapSliceOneLowering::new()
        .lower_sema(&fixture.reader, &transaction, &external)
        .expect("supported Sema record and table lowering");
    let [
        WholeLogosItem::Newtype(key),
        WholeLogosItem::Struct(record),
        WholeLogosItem::Table(table),
    ] = logos.items()
    else {
        panic!("stored declarations precede their table")
    };
    assert_eq!(spelling(&transaction, key.name()), "Domain");
    assert_eq!(key.attributes(), WholeLogosTypeAttributes::Stored);
    assert_eq!(spelling(&transaction, record.name()), "StoredRecord");
    assert_eq!(record.attributes(), WholeLogosTypeAttributes::Stored);
    assert_eq!(spelling(&transaction, table.name()), "records");
    assert_eq!(
        table.record(),
        &WholeLogosTypeReference::Identity(*record.name())
    );
    assert_eq!(table.key(), &WholeLogosTypeReference::Identity(*key.name()));
    assert_ne!(table.record_storage(), table.key_storage());

    assert_eq!(
        logos,
        BootstrapSliceOneLowering::new()
            .lower_sema(&fixture.reader, &transaction, &external)
            .expect("same verified inputs lower deterministically")
    );
}

#[test]
fn strict_sema_lowering_refuses_missing_or_ambiguous_storage_ownership() {
    let fixture = fixture(21);
    let transaction = seal(
        &fixture,
        "Sema.{1 0 0}\n[]\n{[Domain.String StoredRecord.{Domain Integer}] [records.{StoredRecord Domain}]}",
    );
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower_sema(
            &fixture.reader,
            &transaction,
            &[external_storage(8, 8)]
        ),
        Err(BootstrapSliceOneLoweringError::MissingExternalStorageProvenance { identity })
            if identity == id(7)
    ));
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower_sema(
            &fixture.reader,
            &transaction,
            &[external_storage(7, 7), external_storage(7, 7), external_storage(8, 8)]
        ),
        Err(BootstrapSliceOneLoweringError::DuplicateExternalStorageProvenance { identity })
            if identity == id(7)
    ));
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower_sema(
            &fixture.reader,
            &transaction,
            &[external_storage(100, 1), external_storage(7, 7), external_storage(8, 8)]
        ),
        Err(BootstrapSliceOneLoweringError::StorageProvenanceOwnershipConflict { identity })
            if spelling(&transaction, &identity) == "Domain"
    ));
}

#[test]
fn strict_sema_lowering_refuses_foreign_or_non_newtype_table_relationships() {
    let fixture = fixture(22);
    let external = [external_storage(7, 7), external_storage(8, 8)];

    let foreign_record = seal(
        &fixture,
        "Sema.{1 0 0}\n[]\n{[Domain.String] [records.{String Domain}]}",
    );
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower_sema(
            &fixture.reader,
            &foreign_record,
            &external
        ),
        Err(BootstrapSliceOneLoweringError::SemaTableRecordNotDeclared { record, .. })
            if record == id(7)
    ));

    let foreign_key = seal(
        &fixture,
        "Sema.{1 0 0}\n[]\n{[StoredRecord.String] [records.{StoredRecord String}]}",
    );
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower_sema(&fixture.reader, &foreign_key, &external),
        Err(BootstrapSliceOneLoweringError::SemaTableKeyNotDeclared { key, .. })
            if key == id(7)
    ));

    let product_key = seal(
        &fixture,
        "Sema.{1 0 0}\n[]\n{[Key.{String Integer} StoredRecord.String] [records.{StoredRecord Key}]}",
    );
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower_sema(&fixture.reader, &product_key, &external),
        Err(BootstrapSliceOneLoweringError::SemaTableKeyNotNewtype { key, .. })
            if spelling(&product_key, &key) == "Key"
    ));
}

#[test]
fn strict_sema_lowering_revalidates_kind_and_authority() {
    let primary = fixture(23);
    let interface = seal(&primary, "Interface.{1 0 0}\n[]\n{[] [] [] [] [Thing.String]}");
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower_sema(
            &primary.reader,
            &interface,
            &[external_storage(7, 7)]
        ),
        Err(BootstrapSliceOneLoweringError::ExpectedSema {
            found: EthosKind::Interface
        })
    ));

    let sema = seal(
        &primary,
        "Sema.{1 0 0}\n[]\n{[Domain.String] [records.{Domain Domain}]}",
    );
    let other_authority = fixture(24);
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower_sema(
            &other_authority.reader,
            &sema,
            &[external_storage(7, 7)]
        ),
        Err(BootstrapSliceOneLoweringError::Validation(
            BootstrapReadError::NamingAuthorityReceiptRejected
        ))
    ));
}

#[test]
fn matching_reader_revalidates_the_authority_receipt() {
    let primary = fixture(3);
    let transaction = seal(&primary, "Nexus.{1 0 0}\n[]\n{[] [Thing.String]}");
    let other_authority = fixture(4);
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower(&other_authority.reader, &transaction),
        Err(BootstrapSliceOneLoweringError::Validation(
            BootstrapReadError::NamingAuthorityReceiptRejected
        ))
    ));
}

#[test]
fn refuses_traits_roles_tables_and_requirements_without_erasure() {
    let fixture = fixture(5);

    let transaction = seal(&fixture, "Nexus.{1 0 0}\n[]\n{[Behavior.{}] []}");
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower(&fixture.reader, &transaction),
        Err(BootstrapSliceOneLoweringError::Trait { declaration })
            if spelling(&transaction, &declaration) == "Behavior"
    ));

    let transaction = seal(
        &fixture,
        "Interface.{1 0 0}\n[]\n{[Message.String] [] [] [] []}",
    );
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower(&fixture.reader, &transaction),
        Err(BootstrapSliceOneLoweringError::InterfaceRole {
            role: InterfaceRole::Input,
            target,
        }) if spelling(&transaction, &target) == "Message"
    ));

    let transaction = seal(
        &fixture,
        "Sema.{1 0 0}\n[]\n{[Record.{String Integer}] [Entries.{Record String}]}",
    );
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower(&fixture.reader, &transaction),
        Err(BootstrapSliceOneLoweringError::Table { declaration })
            if spelling(&transaction, &declaration) == "Entries"
    ));

    let transaction = seal(
        &fixture,
        "Nexus.{1 0 0}\n[builtin.[Sortable]]\n{[] [Thing.Vector<Option<«Sortable»>>]}",
    );
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower(&fixture.reader, &transaction),
        Err(BootstrapSliceOneLoweringError::TraitRequirement {
            declaration,
            binder: ParameterBinder::Inferred(_),
            required_traits,
        }) if spelling(&transaction, &declaration) == "Thing" && required_traits == [id(17)]
    ));
}

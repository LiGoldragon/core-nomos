use std::collections::BTreeMap;

use core_ethos::bootstrap::*;
use core_logos::{WholeLogosItem, WholeLogosTypeReference, WholeLogosVariantPayload};
use core_nomos::{BootstrapSliceOneLowering, BootstrapSliceOneLoweringError};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

const AUTHORITY_PROOF: u64 = 0x006e_6f6d_6f73;

fn id(local: u16) -> VocabularyEncodedId {
    VocabularyEncodedId::new(VocabularyRoot::Universal, vec![LocalEncodedId::new(local)])
        .expect("nonempty identity")
}

fn record(
    module: &[&str],
    owner: Option<VocabularyEncodedId>,
    name: &str,
    identity: VocabularyEncodedId,
) -> TextualMetadataRecord {
    TextualMetadataRecord {
        address: TextualProjectionAddress {
            module_path: module.iter().map(|part| (*part).to_owned()).collect(),
            lexical_owner: owner,
            visible_name: name.to_owned(),
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
        (7, "String", vec![SchemaRole::Nominal { persistent: true }]),
        (8, "Integer", vec![SchemaRole::Nominal { persistent: true }]),
        (9, "Boolean", vec![SchemaRole::Nominal { persistent: true }]),
        (10, "Unit", vec![SchemaRole::Nominal { persistent: true }]),
        (11, "Vector", vec![SchemaRole::Shape { arity: 1 }]),
        (12, "Option", vec![SchemaRole::Shape { arity: 1 }]),
        (13, "Map", vec![SchemaRole::Shape { arity: 2 }]),
        (14, "Result", vec![SchemaRole::Shape { arity: 2 }]),
        (
            15,
            "Stream",
            vec![
                SchemaRole::Shape { arity: 1 },
                SchemaRole::Nomos(NomosSchema::StreamInitiation { arity: 2 }),
            ],
        ),
        (16, "StreamIdentity", vec![SchemaRole::Shape { arity: 1 }]),
        (17, "Sortable", vec![SchemaRole::Trait]),
    ];
    let mut records = Vec::new();
    let mut schemas = Vec::new();
    let mut order = Vec::new();
    for (local, name, roles) in prior_specs {
        let identity = id(local);
        records.push(record(&["builtin"], None, name, identity.clone()));
        schemas.push(IdentitySchema::new(identity.clone(), roles).expect("valid prior schema"));
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
            string_type: id(7),
            integer_type: id(8),
            boolean_type: id(9),
            unit_type: id(10),
            vector_shape: id(11),
            option_shape: id(12),
            map_shape: id(13),
            result_shape: id(14),
            stream_nomos: id(15),
            stream_shape: id(15),
            stream_identity_shape: id(16),
        },
        &schemas,
        &snapshot,
    )
    .expect("valid prior vocabulary");
    let catalog = BootstrapCatalog::new(
        vec!["app".to_owned()],
        snapshot.clone(),
        schemas,
        priors,
        BootstrapVersionPolicy::exact(EthosVersion::new(1, 0, 0)),
        canonical,
    )
    .expect("valid catalog");
    let reader = BootstrapReader::build(
        BootstrapGrammarIdentities {
            document: id(900),
            syntax: id(901),
        },
        catalog,
        TestAuthority(authority),
    )
    .expect("valid reader");
    Fixture { reader, snapshot }
}

fn seal(fixture: &Fixture, source: &str) -> PreparedBootstrapTransaction<TestAuthority> {
    let plan = fixture.reader.plan(source).expect("valid bootstrap source");
    let mut records = fixture.snapshot.records().to_vec();
    let mut by_occurrence = BTreeMap::new();
    let mut assignments = Vec::new();
    for (index, declaration) in plan.declarations().iter().enumerate() {
        let identity = id(100 + index as u16);
        by_occurrence.insert(declaration.occurrence(), identity.clone());
        assignments.push(NamingAssignment {
            occurrence: declaration.occurrence(),
            encoded_name: identity,
            disposition: IdentityDisposition::New {
                canonical_bytes: declaration.spelling().as_bytes().to_vec(),
            },
        });
    }
    for (declaration, assignment) in plan.declarations().iter().zip(&assignments) {
        let owner = match declaration.scope() {
            PlannedScope::Module => None,
            PlannedScope::Enum(owner) | PlannedScope::Trait(owner) => {
                Some(by_occurrence[&owner].clone())
            }
        };
        records.push(record(
            &["app"],
            owner,
            declaration.spelling(),
            assignment.encoded_name.clone(),
        ));
    }

    let mut generated = Vec::new();
    let mut next_generated = 500u16;
    for declaration in plan
        .declarations()
        .iter()
        .filter(|declaration| declaration.purpose() == DeclarationPurpose::StreamInitiation)
    {
        let initiation = id(next_generated);
        let termination = id(next_generated + 1);
        next_generated += 2;
        records.extend([
            record(
                &["app"],
                None,
                &format!("Start{}", declaration.spelling()),
                initiation.clone(),
            ),
            record(
                &["app"],
                None,
                &format!("Stop{}", declaration.spelling()),
                termination.clone(),
            ),
        ]);
        generated.push(GeneratedStreamAssignment {
            source: declaration.occurrence(),
            initiation: AssignedIdentity {
                encoded_name: initiation,
                disposition: IdentityDisposition::New {
                    canonical_bytes: format!("start-{}", declaration.spelling()).into_bytes(),
                },
            },
            termination: AssignedIdentity {
                encoded_name: termination,
                disposition: IdentityDisposition::New {
                    canonical_bytes: format!("stop-{}", declaration.spelling()).into_bytes(),
                },
            },
        });
    }

    fixture
        .reader
        .seal(
            &plan,
            &NamingAssignments::new(assignments).expect("complete authored assignments"),
            &GeneratedStreamAssignments::new(generated).expect("complete generated assignments"),
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
    identity: &VocabularyEncodedId,
) -> &'a str {
    &transaction
        .naming_transition()
        .after()
        .record(identity)
        .expect("identity has metadata")
        .address
        .visible_name
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
        "Interface.{1 0 0}\n[]\n{[] [] [] [Signal.Result<Vector<String> Integer>]}",
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
fn refuses_traits_roles_streams_tables_and_requirements_without_erasure() {
    let fixture = fixture(5);

    let transaction = seal(&fixture, "Nexus.{1 0 0}\n[]\n{[Behavior.{}] []}");
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower(&fixture.reader, &transaction),
        Err(BootstrapSliceOneLoweringError::Trait { declaration })
            if spelling(&transaction, &declaration) == "Behavior"
    ));

    let transaction = seal(
        &fixture,
        "Interface.{1 0 0}\n[]\n{[Message.String] [] [] []}",
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
        "Interface.{1 0 0}\n[]\n{[] [] [] [Flow.Stream.(String Integer)]}",
    );
    assert!(matches!(
        BootstrapSliceOneLowering::new().lower(&fixture.reader, &transaction),
        Err(BootstrapSliceOneLoweringError::Stream { declaration })
            if spelling(&transaction, &declaration) == "Flow"
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

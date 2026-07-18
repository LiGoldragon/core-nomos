//! The capstone: the five-language pipeline, byte-exact against the real goldens.
//!
//! schema TEXT -> CoreSchema -> Nomos macros -> CoreLogos -> TextualRust -> Rust.
//! The Rust the goldens already emit is the acceptance oracle: macro-produced logos
//! must lower to it, byte for byte. The golden constants below are transcribed
//! verbatim from `textual-rust`'s provenance fixtures (copied from schema-rust
//! @ 87de872) — the same corpus textual-rust proved 153 items against; here the
//! only new variable is the Nomos lowering.

use core_logos::{
    Attribute, ConfigurationAttribute, ConfigurationPredicate, CoreItem, DeriveGroup, Field,
    Generics, PathNode, Struct, TypeReference, Visibility,
};
use core_nomos::MacroPackage;
use core_schema::fixture::{COMMIT_SEQUENCE, DATABASE_MARKER, STATE_DIGEST};
use core_schema::{
    CoreDeclaration, CoreEnum, CoreField, CoreNewtype, CoreReference, CoreSchema, CoreStruct,
    CoreType, CoreVariant, TextualSchema,
};
use name_table::{Identifier, Name, NameTable};
use structural_codec::ids::ScopedCoreTypeId;
use structural_codec::{Converted, EncodedConversion};
use textual_rust::RustSource;

// ---- the real provenance goldens (verbatim bytes; each item ends in a newline) ----

/// runner_generated.rs — the plain two-node preamble.
const GOLDEN_COMMIT_SEQUENCE_PLAIN: &str = "\
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CommitSequence(Integer);
";

const GOLDEN_STATE_DIGEST_PLAIN: &str = "\
#[rustfmt::skip]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StateDigest(Integer);
";

/// spirit_generated.rs — the standard three-node wire preamble.
const GOLDEN_RECORD_IDENTIFIER_WIRE: &str = "\
#[rustfmt::skip]
#[cfg_attr(
    feature = \"nota-text\",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RecordIdentifier(Integer);
";

const GOLDEN_TOPIC_WIRE: &str = "\
#[rustfmt::skip]
#[cfg_attr(
    feature = \"nota-text\",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Topic(String);
";

const GOLDEN_ENTRY_WIRE: &str = "\
#[rustfmt::skip]
#[cfg_attr(
    feature = \"nota-text\",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub topics: Topics,
    pub kind: Kind,
    pub description: Description,
    pub magnitude: Magnitude,
}
";

const GOLDEN_QUERY_WIRE: &str = "\
#[rustfmt::skip]
#[cfg_attr(
    feature = \"nota-text\",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub topic: Topic,
    pub kind: Kind,
}
";

/// The three-attribute CommitSequence — the psyche's illustrative wire sample (the
/// exact bytes textual-rust's own round-trip asserts).
const SAMPLE_COMMIT_SEQUENCE_WIRE: &str = "\
#[rustfmt::skip]
#[cfg_attr(
    feature = \"nota-text\",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CommitSequence(Integer);
";

/// The psyche's private-field sample (secret_digest is private) — constructed at
/// the logos level, since CoreSchema does not carry field visibility. Sample, not
/// an on-disk golden.
const SAMPLE_DATABASE_MARKER_PRIVATE: &str = "\
#[rustfmt::skip]
#[cfg_attr(
    feature = \"nota-text\",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DatabaseMarker {
    pub commit_sequence: CommitSequence,
    pub state_digest: StateDigest,
    secret_digest: StateDigest,
}
";

// ---- helpers (test-only) ----

/// Intern a name and return its identifier.
fn intern(names: &mut NameTable, name: &str) -> Identifier {
    names.intern(Name::new(name))
}

/// A one-declaration CoreSchema wrapping a decoded declaration value.
fn schema_of(value: CoreType) -> CoreSchema {
    CoreSchema::new(vec![CoreDeclaration::public(value)])
}

/// Decode one schema declaration through TextualSchema, seeding a fresh table.
fn decode(expected: ScopedCoreTypeId, text: &str) -> (CoreType, NameTable) {
    let textual = TextualSchema::fixture().expect("build fixture TextualSchema");
    let mut names = NameTable::new();
    let value = textual
        .decode(expected, text, &mut names)
        .unwrap_or_else(|error| panic!("decode {text}: {error}"));
    (value, names)
}

/// Project one lowered item to Rust text.
fn project(item: &CoreItem, names: &NameTable) -> String {
    RustSource::project_item(item, names)
        .expect("project item")
        .as_str()
        .to_owned()
}

// ---- the pipeline, byte-exact from real schema TEXT to real golden ----

#[test]
fn pipeline_plain_newtype_from_text_matches_runner_golden() {
    // The five languages, end to end, twice — CommitSequence and StateDigest.
    for (expected, text, golden) in [
        (
            COMMIT_SEQUENCE,
            "CommitSequence.{ Integer }",
            GOLDEN_COMMIT_SEQUENCE_PLAIN,
        ),
        (
            STATE_DIGEST,
            "StateDigest.{ Integer }",
            GOLDEN_STATE_DIGEST_PLAIN,
        ),
    ] {
        // schema TEXT -> CoreSchema
        let (value, schema_names) = decode(expected, text);
        let schema = schema_of(value);
        let schema_identity = schema.content_identity().expect("schema identity");

        // CoreSchema -> Nomos macros -> CoreLogos (+ extended NameTable)
        let package = MacroPackage::plain_fixture();
        let lowering = package.apply(&schema, &schema_names).expect("lower");
        let item = &lowering.items[0];
        let logos_identity = item.content_identity().expect("logos identity");

        // CoreLogos -> TextualRust -> Rust, byte-exact against the real golden
        let rust = project(item, &lowering.names);
        assert_eq!(rust, golden, "byte-exact plain lowering of {text}");

        println!(
            "\n[{text}]\n  schema identity: {}\n  logos identity:  {}\n  names: schema {} -> logos {} (appended {})\n{rust}",
            schema_identity.to_hexadecimal(),
            logos_identity.to_hexadecimal(),
            schema_names.len(),
            lowering.names.len(),
            lowering.names.len() - schema_names.len(),
        );
    }
}

#[test]
fn lowering_is_an_encoded_conversion_instance() {
    // The schema→logos lowering stated at the TRAIT level: `MacroPackage` is an
    // `EncodedConversion` whose `convert` yields the SAME logos EncodedForm and the
    // SAME continuous NameTable as the eponymous `apply` — the trait face is the
    // lowering, not a reimplementation. The `convert` signature carries no `&str` /
    // `String`: the conversion is a real type conversion, structurally text-free.
    let (value, schema_names) = decode(COMMIT_SEQUENCE, "CommitSequence.{ Integer }");
    let schema = schema_of(value);
    let package = MacroPackage::plain_fixture();

    let converted: Converted<Vec<CoreItem>> =
        EncodedConversion::convert(&package, &schema, &schema_names).expect("trait convert");
    let lowering = package
        .apply(&schema, &schema_names)
        .expect("inherent apply");

    // Same target EncodedForm and same continuous nametree.
    assert_eq!(
        converted.target, lowering.items,
        "the trait conversion target IS the lowered logos item set"
    );
    assert_eq!(
        converted.names.len(),
        lowering.names.len(),
        "the trait conversion threads the same continuous NameTable"
    );
    // Continuity: the layer conversion only ever EXTENDS the schema nametree.
    assert!(
        converted.names.len() >= schema_names.len(),
        "the continuous nametree crosses the layer, schema indices preserved"
    );

    // End-to-end proof the continuous nametree resolves everything: the trait-produced
    // logos item projects to the same byte-exact golden as the inherent path.
    let rust = project(&converted.target[0], &converted.names);
    assert_eq!(
        rust, GOLDEN_COMMIT_SEQUENCE_PLAIN,
        "the trait-level conversion projects byte-exact Rust"
    );
}

#[test]
fn pipeline_wire_newtype_from_text_is_illustrative_sample() {
    let (value, schema_names) = decode(COMMIT_SEQUENCE, "CommitSequence.{ Integer }");
    let schema = schema_of(value);
    let package = MacroPackage::wire_fixture();
    let lowering = package.apply(&schema, &schema_names).expect("lower");
    let rust = project(&lowering.items[0], &lowering.names);
    assert_eq!(rust, SAMPLE_COMMIT_SEQUENCE_WIRE);
}

// ---- byte-exact against the real on-disk provenance goldens (nomos lowering the
//      only new variable): a newtype with the full three-attribute preamble and a
//      multi-field struct ----

#[test]
fn real_provenance_newtypes_lower_byte_exact() {
    let package = MacroPackage::wire_fixture();
    for (type_name, wrapped, golden) in [
        (
            "RecordIdentifier",
            CoreReference::Integer,
            GOLDEN_RECORD_IDENTIFIER_WIRE,
        ),
        ("Topic", CoreReference::String, GOLDEN_TOPIC_WIRE),
    ] {
        let mut names = NameTable::new();
        let identifier = intern(&mut names, type_name);
        let schema = schema_of(CoreType::Newtype(CoreNewtype::new(identifier, wrapped)));
        let lowering = package.apply(&schema, &names).expect("lower");
        assert_eq!(project(&lowering.items[0], &lowering.names), golden);
    }
}

#[test]
fn real_provenance_structs_lower_byte_exact() {
    let package = MacroPackage::wire_fixture();

    // Entry { topics: Topics, kind: Kind, description: Description, magnitude: Magnitude }.
    // Every field name is the field_name of its type, so the particular-struct
    // default derives each name through name-table's walker.
    let entry = {
        let mut names = NameTable::new();
        let identifier = intern(&mut names, "Entry");
        let fields = [
            ("topics", "Topics"),
            ("kind", "Kind"),
            ("description", "Description"),
            ("magnitude", "Magnitude"),
        ]
        .into_iter()
        .map(|(field_name, type_name)| {
            let field_identifier = intern(&mut names, field_name);
            let type_identifier = intern(&mut names, type_name);
            CoreField::new(field_identifier, CoreReference::Plain(type_identifier))
        })
        .collect();
        let schema = schema_of(CoreType::Struct(CoreStruct::new(identifier, fields)));
        let lowering = package.apply(&schema, &names).expect("lower Entry");
        project(&lowering.items[0], &lowering.names)
    };
    assert_eq!(entry, GOLDEN_ENTRY_WIRE);

    // Query { topic: Topic, kind: Kind }.
    let query = {
        let mut names = NameTable::new();
        let identifier = intern(&mut names, "Query");
        let fields = [("topic", "Topic"), ("kind", "Kind")]
            .into_iter()
            .map(|(field_name, type_name)| {
                let field_identifier = intern(&mut names, field_name);
                let type_identifier = intern(&mut names, type_name);
                CoreField::new(field_identifier, CoreReference::Plain(type_identifier))
            })
            .collect();
        let schema = schema_of(CoreType::Struct(CoreStruct::new(identifier, fields)));
        let lowering = package.apply(&schema, &names).expect("lower Query");
        project(&lowering.items[0], &lowering.names)
    };
    assert_eq!(query, GOLDEN_QUERY_WIRE);
}

// ---- the illustrative sample pair end to end ----

#[test]
fn illustrative_struct_from_schema_text_lowers_and_derives_names() {
    // DatabaseMarker.{ CommitSequence StateDigest secretDigest.StateDigest } from
    // real schema text: two elided (derived) field names, one explicit — the
    // particular-struct default runs the field-name walker on the elided pair and
    // preserves the explicit name. Not an on-disk golden, so no byte-exact claim.
    let (value, schema_names) = decode(
        DATABASE_MARKER,
        "DatabaseMarker.{ CommitSequence StateDigest secretDigest.StateDigest }",
    );
    let schema = schema_of(value);
    let package = MacroPackage::wire_fixture();
    let lowering = package.apply(&schema, &schema_names).expect("lower");
    let rust = project(&lowering.items[0], &lowering.names);
    assert!(rust.contains("pub struct DatabaseMarker {"));
    assert!(rust.contains("pub commit_sequence: CommitSequence,"));
    assert!(rust.contains("pub state_digest: StateDigest,"));
    assert!(rust.contains("secretDigest: StateDigest,"));
    println!("\n[illustrative struct from schema text]\n{rust}");
}

#[test]
fn illustrative_private_field_sample_projects_byte_exact() {
    // The psyche's private-field sample: constructed at the logos level because
    // CoreSchema does not carry field visibility. Sample, not an on-disk golden.
    let mut names = NameTable::new();
    let preamble = wire_preamble(&mut names);
    let name = intern(&mut names, "DatabaseMarker");
    let commit_sequence = intern(&mut names, "CommitSequence");
    let state_digest = intern(&mut names, "StateDigest");
    let fields = vec![
        Field {
            visibility: Visibility::Public,
            name: intern(&mut names, "commit_sequence"),
            type_reference: TypeReference::Path(PathNode {
                segments: vec![commit_sequence],
            }),
        },
        Field {
            visibility: Visibility::Public,
            name: intern(&mut names, "state_digest"),
            type_reference: TypeReference::Path(PathNode {
                segments: vec![state_digest],
            }),
        },
        Field {
            visibility: Visibility::Private,
            name: intern(&mut names, "secret_digest"),
            type_reference: TypeReference::Path(PathNode {
                segments: vec![state_digest],
            }),
        },
    ];
    let item = CoreItem::Struct(Struct {
        visibility: Visibility::Public,
        attributes: preamble,
        name,
        generics: Generics::none(),
        fields,
    });
    assert_eq!(project(&item, &names), SAMPLE_DATABASE_MARKER_PRIVATE);
}

/// The three-node wire preamble as literal core-logos attributes (for the
/// logos-level sample above).
fn wire_preamble(names: &mut NameTable) -> Vec<Attribute> {
    let path = |names: &mut NameTable, segments: &[&str]| PathNode {
        segments: segments.iter().map(|s| intern(names, s)).collect(),
    };
    vec![
        Attribute::ToolPath(path(names, &["rustfmt", "skip"])),
        Attribute::Configuration(ConfigurationAttribute {
            predicate: ConfigurationPredicate::Feature(intern(names, "nota-text")),
            inner: Box::new(Attribute::Derive(DeriveGroup {
                paths: vec![
                    path(names, &["nota", "NotaDecode"]),
                    path(names, &["nota", "NotaDecodeTraced"]),
                    path(names, &["nota", "NotaEncode"]),
                ],
            })),
        }),
        Attribute::Derive(DeriveGroup {
            paths: vec![
                path(names, &["rkyv", "Archive"]),
                path(names, &["rkyv", "Serialize"]),
                path(names, &["rkyv", "Deserialize"]),
                path(names, &["Clone"]),
                path(names, &["Debug"]),
                path(names, &["PartialEq"]),
                path(names, &["Eq"]),
            ],
        }),
    ]
}

// ---- declaration visibility is lowered faithfully (golden-bridge item 2) ----

#[test]
fn declaration_visibility_lowers_faithfully() {
    // The schema declaration's coarse Public/Private is an authoritative API promise
    // and stamps the produced item. A Private declaration projects without `pub`; a
    // Public one keeps it. Same structure, visibility the only difference. (Settled
    // psyche ruling primary-56d1.29: schema visibility is authoritative.)
    let mut names = NameTable::new();
    let identifier = intern(&mut names, "Hidden");
    let value = CoreType::Newtype(CoreNewtype::new(identifier, CoreReference::Integer));
    let package = MacroPackage::plain_fixture();

    let public = CoreSchema::new(vec![CoreDeclaration::new(
        core_schema::Visibility::Public,
        value.clone(),
    )]);
    let public_low = package.apply(&public, &names).expect("lower public");
    let public_rust = project(&public_low.items[0], &public_low.names);
    assert!(
        public_rust.contains("pub struct Hidden(Integer);"),
        "public declaration keeps pub: {public_rust}",
    );

    let private = CoreSchema::new(vec![CoreDeclaration::new(
        core_schema::Visibility::Private,
        value,
    )]);
    let private_low = package.apply(&private, &names).expect("lower private");
    let private_rust = project(&private_low.items[0], &private_low.names);
    assert!(
        private_rust.contains("struct Hidden(Integer);") && !private_rust.contains("pub struct"),
        "private declaration drops pub: {private_rust}",
    );
}

// ---- hash discipline across the whole pipeline ----

#[test]
fn hash_discipline_rename_is_stable_output_changes() {
    let plain = MacroPackage::plain_fixture();
    let build = |type_name: &str| {
        let mut names = NameTable::new();
        let identifier = intern(&mut names, type_name);
        let schema = schema_of(CoreType::Newtype(CoreNewtype::new(
            identifier,
            CoreReference::Integer,
        )));
        (schema, names)
    };

    let (schema_a, names_a) = build("CommitSequence");
    let (schema_b, names_b) = build("CommitLog"); // a pure rename: identical structure

    // The CoreSchema identity is rename-stable (names are not in the pre-image).
    assert_eq!(
        schema_a.content_identity().unwrap(),
        schema_b.content_identity().unwrap(),
        "schema identity must not move under a rename",
    );

    let low_a = plain.apply(&schema_a, &names_a).unwrap();
    let low_b = plain.apply(&schema_b, &names_b).unwrap();

    // The CoreLogos identity is rename-stable too.
    assert_eq!(
        low_a.items[0].content_identity().unwrap(),
        low_b.items[0].content_identity().unwrap(),
        "logos identity must not move under a rename",
    );

    // But the projected Rust text changes — names live only in the projection.
    let rust_a = project(&low_a.items[0], &low_a.names);
    let rust_b = project(&low_b.items[0], &low_b.names);
    assert_ne!(rust_a, rust_b);
    assert!(rust_a.contains("CommitSequence"));
    assert!(rust_b.contains("CommitLog"));
}

#[test]
fn payload_enumerations_do_not_claim_copy() {
    let mut names = NameTable::new();
    let input = intern(&mut names, "Input");
    let record = intern(&mut names, "Record");
    let observe = intern(&mut names, "Observe");
    let value = CoreType::Enumeration(CoreEnum::new(
        input,
        vec![
            CoreVariant::new(record, Some(CoreReference::Integer)),
            CoreVariant::new(observe, None),
        ],
    ));
    let lowering = MacroPackage::wire_fixture()
        .apply(&schema_of(value), &names)
        .expect("lower payload enumeration");
    let rust = project(&lowering.items[0], &lowering.names);
    assert!(
        !rust.contains("    Copy,"),
        "payload enums cannot derive Copy"
    );
}

#[test]
fn continuous_identifier_space_preserves_schema_indices() {
    let (value, schema_names) = decode(COMMIT_SEQUENCE, "CommitSequence.{ Integer }");
    let schema = schema_of(value);
    let lowering = MacroPackage::wire_fixture()
        .apply(&schema, &schema_names)
        .expect("lower");

    // Every schema identifier resolves identically in the extended logos table, and
    // the logos table only grew.
    assert!(lowering.names.len() >= schema_names.len());
    for index in 0..schema_names.len() {
        let identifier = Identifier::new(index as u32);
        assert_eq!(
            schema_names.resolve(identifier).unwrap(),
            lowering.names.resolve(identifier).unwrap(),
            "schema identifier {index} must be stable in the logos extension",
        );
    }
}

// ---- loud errors: named-invocation resolution and cycle rejection ----

#[test]
fn missing_structural_default_errors_loudly() {
    let empty = MacroPackage::new(core_nomos::PackageRevision(1));
    let mut names = NameTable::new();
    let identifier = intern(&mut names, "Anything");
    let schema = schema_of(CoreType::Newtype(CoreNewtype::new(
        identifier,
        CoreReference::Integer,
    )));
    let error = empty.apply(&schema, &names).unwrap_err();
    assert!(
        matches!(error, core_nomos::NomosError::NoStructuralDefault(_)),
        "got {error:?}",
    );
}

#[test]
fn unknown_named_invocation_errors_loudly() {
    use core_nomos::{
        BindingRef, Escape, InputParameter, InputSignature, ItemTemplate, MacroDefinition,
        MacroIdentity, MacroKind, MetaType, NameTransform, NewtypeTemplate, PackageRevision,
        Realize, ResultTemplate, Scalar, SectionDefault, Sequence, SequenceItem,
    };

    let mut package = MacroPackage::new(PackageRevision(1));
    let name_binding = package.author_name("name");
    let type_binding = package.author_name("type");
    let newtype_name = package.author_name("Newtype");
    // The attributes position invokes a macro identity that was never registered.
    package.register(MacroDefinition {
        name: newtype_name,
        kind: MacroKind::Structural(SectionDefault::Newtype),
        input: InputSignature {
            parameters: vec![
                InputParameter {
                    binding: name_binding,
                    meta: MetaType::Name,
                },
                InputParameter {
                    binding: type_binding,
                    meta: MetaType::Type,
                },
            ],
        },
        template: ResultTemplate::Item(ItemTemplate::Newtype(NewtypeTemplate {
            visibility: Visibility::Public,
            attributes: Sequence::of(SequenceItem::Escape(Escape::Invoke(MacroIdentity::new(99)))),
            name: Scalar::Escape(Escape::Realize(Realize {
                binding: BindingRef::Input(name_binding),
                transform: NameTransform::Identity,
            })),
            wrapped: Scalar::Escape(Escape::Realize(Realize {
                binding: BindingRef::Input(type_binding),
                transform: NameTransform::Identity,
            })),
        })),
    });

    let mut names = NameTable::new();
    let identifier = intern(&mut names, "Whatever");
    let schema = schema_of(CoreType::Newtype(CoreNewtype::new(
        identifier,
        CoreReference::Integer,
    )));
    let error = package.apply(&schema, &names).unwrap_err();
    assert!(
        matches!(error, core_nomos::NomosError::UnknownMacro(_)),
        "got {error:?}",
    );
}

#[test]
fn recursive_cycle_is_rejected() {
    use core_nomos::{
        Escape, InputSignature, MacroDefinition, MacroKind, PackageRevision, ResultTemplate,
        Sequence, SequenceItem,
    };

    // A self-invoking attributes macro: its own template invokes itself.
    let mut package = MacroPackage::new(PackageRevision(1));
    let attributes_name = package.author_name("SelfAttributes");
    let self_identity = core_nomos::MacroIdentity::new(0);
    package.register(MacroDefinition {
        name: attributes_name,
        kind: MacroKind::Named,
        input: InputSignature::unit(),
        template: ResultTemplate::Attributes(Sequence::of(SequenceItem::Escape(Escape::Invoke(
            self_identity,
        )))),
    });
    // A newtype default that invokes the self-invoking attributes macro.
    let name_binding = package.author_name("name");
    let type_binding = package.author_name("type");
    let newtype_name = package.author_name("Newtype");
    {
        use core_nomos::{
            BindingRef, InputParameter, ItemTemplate, MetaType, NameTransform, NewtypeTemplate,
            Realize, Scalar, SectionDefault,
        };
        package.register(MacroDefinition {
            name: newtype_name,
            kind: MacroKind::Structural(SectionDefault::Newtype),
            input: InputSignature {
                parameters: vec![
                    InputParameter {
                        binding: name_binding,
                        meta: MetaType::Name,
                    },
                    InputParameter {
                        binding: type_binding,
                        meta: MetaType::Type,
                    },
                ],
            },
            template: ResultTemplate::Item(ItemTemplate::Newtype(NewtypeTemplate {
                visibility: Visibility::Public,
                attributes: Sequence::of(SequenceItem::Escape(Escape::Invoke(self_identity))),
                name: Scalar::Escape(Escape::Realize(Realize {
                    binding: BindingRef::Input(name_binding),
                    transform: NameTransform::Identity,
                })),
                wrapped: Scalar::Escape(Escape::Realize(Realize {
                    binding: BindingRef::Input(type_binding),
                    transform: NameTransform::Identity,
                })),
            })),
        });
    }

    let mut names = NameTable::new();
    let identifier = intern(&mut names, "Whatever");
    let schema = schema_of(CoreType::Newtype(CoreNewtype::new(
        identifier,
        CoreReference::Integer,
    )));
    let error = package.apply(&schema, &names).unwrap_err();
    assert!(
        matches!(error, core_nomos::NomosError::RecursionCycle(_)),
        "got {error:?}",
    );
}

// ---- the package as a content-identified value ----

#[test]
fn package_is_content_identified_and_revisioned() {
    let wire = MacroPackage::wire_fixture();
    let plain = MacroPackage::plain_fixture();

    // Deterministic content identity.
    assert_eq!(
        wire.content_identity().unwrap(),
        MacroPackage::wire_fixture().content_identity().unwrap(),
    );
    // The two packages differ (different preambles), so their identities differ.
    assert_ne!(
        wire.content_identity().unwrap(),
        plain.content_identity().unwrap(),
    );
    // The revision is a truthful, separate surface.
    assert_eq!(wire.revision(), core_nomos::PackageRevision(1));
}

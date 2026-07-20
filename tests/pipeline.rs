//! The capstone for the typed schema-to-logos pipeline.
//!
//! Schema text is decoded, lowered through Nomos, and projected as valid Rust.
//! These focused tests assert structural behavior only; process-level working-program
//! evidence belongs to `language-engine-witness`, which compiles and runs emitted code.

use core_logos::{
    Attribute, ConfigurationAttribute, ConfigurationPredicate, DeriveGroup, EncodedItem, Field,
    Generics, PathNode, Struct, TypeReference, Visibility,
};
use core_nomos::MacroPackage;
use core_schema::fixture::{COMMIT_SEQUENCE, DATABASE_MARKER, STATE_DIGEST};
use core_schema::{
    EncodedDeclaration, EncodedEnum, EncodedField, EncodedNewtype, EncodedReference, EncodedSchema,
    EncodedStruct, EncodedType, EncodedVariant, TextualSchema,
};
use name_table::{Identifier, IdentifierNamespace, Name, NameTable};
use structural_codec::ids::ScopedEncodedTypeId;
use structural_codec::{Converted, EncodedConversion};
use textual_rust::RustSource;

// ---- helpers (test-only) ----

/// Intern a name and return its identifier.
fn intern(names: &mut NameTable, name: &str) -> Identifier {
    names
        .intern(Name::new(name))
        .expect("test names fit Schema slice")
}

/// A one-declaration EncodedSchema wrapping a decoded declaration value.
fn schema_of(value: EncodedType) -> EncodedSchema {
    EncodedSchema::new(vec![EncodedDeclaration::public(value)])
}

/// Decode one schema declaration through TextualSchema, seeding a fresh table.
fn decode(expected: ScopedEncodedTypeId, text: &str) -> (EncodedType, NameTable) {
    let textual = TextualSchema::fixture().expect("build fixture TextualSchema");
    let mut names = NameTable::new(IdentifierNamespace::Schema);
    let value = textual
        .decode(expected, text, &mut names)
        .unwrap_or_else(|error| panic!("decode {text}: {error}"));
    (value, names)
}

/// Project one lowered item to Rust text.
fn project(item: &EncodedItem, names: &NameTable) -> String {
    RustSource::project_item(item, names)
        .expect("project item")
        .as_str()
        .to_owned()
}

// ---- focused schema-to-Rust projection coverage ----

#[test]
fn pipeline_plain_newtypes_from_text_project_as_public_rust_items() {
    for (expected, text, type_name) in [
        (
            COMMIT_SEQUENCE,
            "CommitSequence.{ Integer }",
            "CommitSequence",
        ),
        (STATE_DIGEST, "StateDigest.{ Integer }", "StateDigest"),
    ] {
        let (value, schema_names) = decode(expected, text);
        let schema = schema_of(value);
        let lowering = MacroPackage::plain_fixture()
            .apply(&schema, &schema_names)
            .expect("lower plain declaration");
        assert_eq!(lowering.items.len(), 1, "one declaration produces one item");
        let rust = project(&lowering.items[0], &lowering.names);
        assert!(rust.contains(&format!("pub struct {type_name}")), "{rust}");
    }
}

#[test]
fn lowering_is_an_encoded_conversion_instance() {
    let (value, schema_names) = decode(COMMIT_SEQUENCE, "CommitSequence.{ Integer }");
    let schema = schema_of(value);
    let package = MacroPackage::plain_fixture();

    let converted: Converted<Vec<EncodedItem>> =
        EncodedConversion::convert(&package, &schema, &schema_names).expect("trait convert");
    let lowering = package
        .apply(&schema, &schema_names)
        .expect("inherent apply");

    assert_eq!(converted.target, lowering.items);
    assert_eq!(converted.names.len(), lowering.names.len());
    assert_eq!(
        converted.names.namespace(),
        name_table::IdentifierNamespace::Logos
    );
    assert_eq!(
        converted
            .names
            .resolve(Identifier::Schema(0))
            .expect("Schema slice is composed"),
        schema_names
            .resolve(Identifier::Schema(0))
            .expect("schema name exists"),
    );
    let rust = project(&converted.target[0], &converted.names);
    assert!(rust.contains("pub struct CommitSequence"), "{rust}");
}

#[test]
fn pipeline_wire_newtype_from_text_projects_as_generated_rust() {
    let (value, schema_names) = decode(COMMIT_SEQUENCE, "CommitSequence.{ Integer }");
    let schema = schema_of(value);
    let lowering = MacroPackage::wire_fixture()
        .apply(&schema, &schema_names)
        .expect("lower wire declaration");
    let rust = project(&lowering.items[0], &lowering.names);
    assert!(rust.contains("pub struct CommitSequence(u64);"), "{rust}");
    assert!(rust.contains("rkyv::Archive"), "{rust}");
}

#[test]
fn wire_lowering_projects_public_newtypes_and_structs() {
    let package = MacroPackage::wire_fixture();
    for (type_name, wrapped) in [
        ("RecordIdentifier", EncodedReference::Integer),
        ("Topic", EncodedReference::String),
    ] {
        let mut names = NameTable::new(IdentifierNamespace::Schema);
        let identifier = intern(&mut names, type_name);
        let schema = schema_of(EncodedType::Newtype(EncodedNewtype::new(
            identifier, wrapped,
        )));
        let lowering = package.apply(&schema, &names).expect("lower newtype");
        let rust = project(&lowering.items[0], &lowering.names);
        assert!(rust.contains(&format!("pub struct {type_name}")), "{rust}");
    }

    let mut names = NameTable::new(IdentifierNamespace::Schema);
    let entry = intern(&mut names, "Entry");
    let topics = intern(&mut names, "Topics");
    let kind = intern(&mut names, "Kind");
    let schema = schema_of(EncodedType::Struct(EncodedStruct::new(
        entry,
        vec![
            EncodedField::new(
                intern(&mut names, "topics"),
                EncodedReference::Plain(topics),
            ),
            EncodedField::new(intern(&mut names, "kind"), EncodedReference::Plain(kind)),
        ],
    )));
    let lowering = package.apply(&schema, &names).expect("lower struct");
    let rust = project(&lowering.items[0], &lowering.names);
    assert!(rust.contains("pub struct Entry"), "{rust}");
    assert!(rust.contains("pub topics: Topics"), "{rust}");
    assert!(rust.contains("pub kind: Kind"), "{rust}");
}

// ---- the illustrative sample pair end to end ----

#[test]
fn illustrative_struct_from_schema_text_lowers_and_derives_names() {
    // DatabaseMarker.{ CommitSequence StateDigest StateDigest } from real schema
    // text: field names are illegal everywhere (psyche ruling 2026-07-19), so every
    // field name is derived from its type and the two same-typed StateDigest fields
    // would collide on `state_digest`. The deterministic same-typed-field rule
    // (directed work, 2026-07-19) resolves that collision: a type naming more than one
    // field distinguishes each by the ordinal English word of its position among the
    // same-typed fields — `first_state_digest`, `second_state_digest` — while the
    // singly-used `CommitSequence` keeps its bare `commit_sequence`.
    let (value, schema_names) = decode(
        DATABASE_MARKER,
        "DatabaseMarker.{ CommitSequence StateDigest StateDigest }",
    );
    let schema = schema_of(value);
    let package = MacroPackage::wire_fixture();
    let lowering = package.apply(&schema, &schema_names).expect("lower");
    let rust = project(&lowering.items[0], &lowering.names);
    assert!(rust.contains("pub struct DatabaseMarker {"));
    assert!(rust.contains("pub commit_sequence: CommitSequence,"));
    assert!(rust.contains("pub first_state_digest: StateDigest,"));
    assert!(rust.contains("pub second_state_digest: StateDigest,"));
    // The colliding bare name must not survive: position, via the ordinal rule, tells
    // the two StateDigest fields apart.
    assert!(!rust.contains("pub state_digest: StateDigest,"));
    println!("\n[illustrative struct from schema text]\n{rust}");
}

#[test]
fn illustrative_private_field_sample_preserves_visibility() {
    // The psyche's private-field sample is constructed at the logos level because
    // EncodedSchema does not carry field visibility.
    let mut names = NameTable::new(IdentifierNamespace::Schema);
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
    let item = EncodedItem::Struct(Struct {
        visibility: Visibility::Public,
        attributes: preamble,
        name,
        generics: Generics::none(),
        fields,
    });
    let rust = project(&item, &names);
    assert!(rust.contains("pub struct DatabaseMarker"), "{rust}");
    assert!(rust.contains("secret_digest: StateDigest"), "{rust}");
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

// ---- declaration visibility is lowered faithfully (reference fixture-bridge item 2) ----

#[test]
fn declaration_visibility_lowers_faithfully() {
    // The schema declaration's coarse Public/Private is an authoritative API promise
    // and stamps the produced item. A Private declaration projects without `pub`; a
    // Public one keeps it. Same structure, visibility the only difference. (Settled
    // psyche ruling primary-56d1.29: schema visibility is authoritative.)
    let mut names = NameTable::new(IdentifierNamespace::Schema);
    let identifier = intern(&mut names, "Hidden");
    let value = EncodedType::Newtype(EncodedNewtype::new(identifier, EncodedReference::Integer));
    let package = MacroPackage::plain_fixture();

    let public = EncodedSchema::new(vec![EncodedDeclaration::new(
        core_schema::Visibility::Public,
        value.clone(),
    )]);
    let public_low = package.apply(&public, &names).expect("lower public");
    let public_rust = project(&public_low.items[0], &public_low.names);
    assert!(
        public_rust.contains("pub struct Hidden(u64);"),
        "public declaration keeps pub: {public_rust}",
    );

    let private = EncodedSchema::new(vec![EncodedDeclaration::new(
        core_schema::Visibility::Private,
        value,
    )]);
    let private_low = package.apply(&private, &names).expect("lower private");
    let private_rust = project(&private_low.items[0], &private_low.names);
    assert!(
        private_rust.contains("struct Hidden(u64);") && !private_rust.contains("pub struct"),
        "private declaration drops pub: {private_rust}",
    );
}

// ---- hash discipline across the whole pipeline ----

#[test]
fn hash_discipline_rename_is_stable_output_changes() {
    let plain = MacroPackage::plain_fixture();
    let build = |type_name: &str| {
        let mut names = NameTable::new(IdentifierNamespace::Schema);
        let identifier = intern(&mut names, type_name);
        let schema = schema_of(EncodedType::Newtype(EncodedNewtype::new(
            identifier,
            EncodedReference::Integer,
        )));
        (schema, names)
    };

    let (schema_a, names_a) = build("CommitSequence");
    let (schema_b, names_b) = build("CommitLog"); // a pure rename: identical structure

    // The EncodedSchema identity is rename-stable (names are not in the pre-image).
    assert_eq!(
        schema_a.content_identity().unwrap(),
        schema_b.content_identity().unwrap(),
        "schema identity must not move under a rename",
    );

    let low_a = plain.apply(&schema_a, &names_a).unwrap();
    let low_b = plain.apply(&schema_b, &names_b).unwrap();

    // The EncodedLogos identity is rename-stable too.
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
    let mut names = NameTable::new(IdentifierNamespace::Schema);
    let input = intern(&mut names, "Input");
    let record = intern(&mut names, "Record");
    let observe = intern(&mut names, "Observe");
    let value = EncodedType::Enumeration(EncodedEnum::new(
        input,
        vec![
            EncodedVariant::new(record, Some(EncodedReference::Integer)),
            EncodedVariant::new(observe, None),
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
    assert_eq!(
        lowering.names.namespace(),
        name_table::IdentifierNamespace::Logos
    );
    for index in 0..schema_names.len() {
        let identifier =
            Identifier::Schema(u16::try_from(index).expect("test identifier fits Schema slice"));
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
    let mut names = NameTable::new(IdentifierNamespace::Schema);
    let identifier = intern(&mut names, "Anything");
    let schema = schema_of(EncodedType::Newtype(EncodedNewtype::new(
        identifier,
        EncodedReference::Integer,
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

    let mut names = NameTable::new(IdentifierNamespace::Schema);
    let identifier = intern(&mut names, "Whatever");
    let schema = schema_of(EncodedType::Newtype(EncodedNewtype::new(
        identifier,
        EncodedReference::Integer,
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

    let mut names = NameTable::new(IdentifierNamespace::Schema);
    let identifier = intern(&mut names, "Whatever");
    let schema = schema_of(EncodedType::Newtype(EncodedNewtype::new(
        identifier,
        EncodedReference::Integer,
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

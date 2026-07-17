//! The enriched capstone: spirit-min's whole `CoreSchema` — its data declarations
//! and its two interface roots — lowered through the enriched wire package, with the
//! class-A/B/C/D support surface asserted byte-exact against the frozen
//! `spirit_generated.rs` golden.
//!
//! The golden is `schema-rust`'s own provenance fixture, transcribed verbatim here
//! (the same bytes textual-rust proves its projection against). The only new variable
//! is the Nomos generation: every generated item's projection must be present in the
//! golden byte-for-byte, in document order, per class — so a failure names its class.
//!
//! One golden block is deliberately absent from the generated set: the `TraceEvent`
//! tuple-struct declaration `pub struct TraceEvent(pub ObjectName);`. Its field
//! carries `pub`, and core-logos `Newtype` models no tuple-field visibility, so that
//! one declaration is not byte-exact-projectable from the frozen kernel. The class-D
//! `TraceEvent` *impl* is generated and proven; the struct declaration is the flagged
//! blocker (see the return notes / NON_IDEAL_AGENTS.md).

use core_nomos::MacroPackage;
use core_schema::{
    CoreDeclaration, CoreEnum, CoreField, CoreNewtype, CoreReference, CoreSchema, CoreStruct,
    CoreType, CoreVariant, DeclarationRole, SingleTypeReferenceProjection,
};
use name_table::{Identifier, Name, NameTable};
use textual_rust::RustSource;

// The frozen provenance golden, copied verbatim from schema-rust (the same bytes
// textual-rust proves its projection against), so the byte-exact check is
// self-contained and survives the flake-check sandbox.
const GOLDEN: &str = include_str!("fixtures/spirit_generated.rs");

/// The spirit-min schema built by hand through the crate's declaration path: the ten
/// data declarations in golden order, then the two role-tagged interface roots.
struct SpiritMin {
    schema: CoreSchema,
    names: NameTable,
}

impl SpiritMin {
    fn build() -> Self {
        let mut names = NameTable::new();
        let mut intern = |text: &str| names.intern(Name::new(text));

        // Type names.
        let topic = intern("Topic");
        let topics = intern("Topics");
        let description = intern("Description");
        let summary = intern("Summary");
        let record_identifier = intern("RecordIdentifier");
        let entry = intern("Entry");
        let query = intern("Query");
        let record_set = intern("RecordSet");
        let kind = intern("Kind");
        let magnitude = intern("Magnitude");
        let input = intern("Input");
        let output = intern("Output");

        // Newtypes.
        let topic_decl = newtype(topic, CoreReference::String);
        let topics_decl = newtype(topics, vector(CoreReference::Plain(topic)));
        let description_decl = newtype(description, CoreReference::String);
        let summary_decl = newtype(summary, CoreReference::Plain(description));
        let record_identifier_decl = newtype(record_identifier, CoreReference::Integer);
        let record_set_decl = newtype(record_set, vector(CoreReference::Plain(entry)));

        // Structs — field names are the snake_case of their types (elided/derived).
        let entry_decl = CoreDeclaration::public(CoreType::Struct(CoreStruct::new(
            entry,
            vec![
                CoreField::new(intern("topics"), CoreReference::Plain(topics)),
                CoreField::new(intern("kind"), CoreReference::Plain(kind)),
                CoreField::new(intern("description"), CoreReference::Plain(description)),
                CoreField::new(intern("magnitude"), CoreReference::Plain(magnitude)),
            ],
        )));
        let query_decl = CoreDeclaration::public(CoreType::Struct(CoreStruct::new(
            query,
            vec![
                CoreField::new(intern("topic"), CoreReference::Plain(topic)),
                CoreField::new(intern("kind"), CoreReference::Plain(kind)),
            ],
        )));

        // Unit enums.
        let kind_decl = CoreDeclaration::public(CoreType::Enumeration(CoreEnum::new(
            kind,
            [
                "Decision",
                "Principle",
                "Correction",
                "Clarification",
                "Constraint",
            ]
            .into_iter()
            .map(|name| CoreVariant::new(intern(name), None))
            .collect(),
        )));
        let magnitude_decl = CoreDeclaration::public(CoreType::Enumeration(CoreEnum::new(
            magnitude,
            [
                "Minimum", "VeryLow", "Low", "Medium", "High", "VeryHigh", "Maximum",
            ]
            .into_iter()
            .map(|name| CoreVariant::new(intern(name), None))
            .collect(),
        )));

        // Interface roots (payload-carrying enums, role-tagged).
        let input_decl = CoreDeclaration::interface(
            DeclarationRole::InterfaceInput,
            CoreType::Enumeration(CoreEnum::new(
                input,
                vec![
                    CoreVariant::new(intern("Record"), Some(CoreReference::Plain(entry))),
                    CoreVariant::new(intern("Observe"), Some(CoreReference::Plain(query))),
                ],
            )),
        );
        let output_decl = CoreDeclaration::interface(
            DeclarationRole::InterfaceOutput,
            CoreType::Enumeration(CoreEnum::new(
                output,
                vec![
                    CoreVariant::new(
                        intern("RecordAccepted"),
                        Some(CoreReference::Plain(record_identifier)),
                    ),
                    CoreVariant::new(
                        intern("RecordsObserved"),
                        Some(CoreReference::Plain(record_set)),
                    ),
                ],
            )),
        );

        let schema = CoreSchema::new(vec![
            topic_decl,
            topics_decl,
            description_decl,
            summary_decl,
            record_identifier_decl,
            entry_decl,
            query_decl,
            record_set_decl,
            kind_decl,
            magnitude_decl,
            input_decl,
            output_decl,
        ]);
        Self { schema, names }
    }
}

fn newtype(name: Identifier, reference: CoreReference) -> CoreDeclaration {
    CoreDeclaration::public(CoreType::Newtype(CoreNewtype::new(name, reference)))
}

fn vector(argument: CoreReference) -> CoreReference {
    CoreReference::SingleTypeApplication {
        projection: SingleTypeReferenceProjection::Vector,
        argument: Box::new(argument),
    }
}

/// Project one item to Rust text (trailing newline trimmed, the golden-substring
/// unit).
fn project(item: &core_logos::CoreItem, names: &NameTable) -> String {
    RustSource::project_item(item, names)
        .expect("project item")
        .as_str()
        .trim_end()
        .to_owned()
}

/// The class boundaries in the enriched item run, in document order. Each entry is a
/// class name and the count of items it contributes.
const CLASS_LAYOUT: &[(&str, usize)] = &[
    ("declarations", 12),
    ("A: newtype ergonomics", 12),
    ("B: interface ergonomics", 10),
    ("C: wire contract stub", 4),
    ("D: trace support", 5),
];

#[test]
fn enriched_classes_project_byte_exact_against_the_spirit_golden() {
    let spirit = SpiritMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&spirit.schema, &spirit.names)
        .expect("enriched lowering");

    let expected_total: usize = CLASS_LAYOUT.iter().map(|(_, count)| count).sum();
    assert_eq!(
        lowering.items.len(),
        expected_total,
        "the enriched run emits declarations + A + B + C + D",
    );

    // Every generated item's bytes are present verbatim in the golden, and the whole
    // run appears in strictly increasing golden order — the document-order rule.
    let mut previous_offset = 0usize;
    let mut cursor = 0usize;
    for (class, count) in CLASS_LAYOUT {
        for local in 0..*count {
            let item = &lowering.items[cursor];
            let text = project(item, &lowering.names);
            let offset = GOLDEN.find(&text).unwrap_or_else(|| {
                panic!("[{class}] item {local} is not present verbatim in the golden:\n{text}")
            });
            assert!(
                offset >= previous_offset,
                "[{class}] item {local} is out of document order (offset {offset} < {previous_offset}):\n{text}",
            );
            previous_offset = offset;
            cursor += 1;
        }
    }
}

#[test]
fn class_a_covers_every_data_newtype_in_declaration_order() {
    let spirit = SpiritMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&spirit.schema, &spirit.names)
        .expect("lower");
    // Class A begins after the 12 declarations: six inherent+From pairs.
    let class_a = &lowering.items[12..24];
    let heads = [
        "Topic",
        "Topics",
        "Description",
        "Summary",
        "RecordIdentifier",
        "RecordSet",
    ];
    for (pair, head) in class_a.chunks(2).zip(heads) {
        let inherent = project(&pair[0], &lowering.names);
        let from = project(&pair[1], &lowering.names);
        assert!(
            inherent.contains(&format!("impl {head} {{")),
            "inherent impl for {head}:\n{inherent}",
        );
        assert!(inherent.contains("pub fn new("), "{head} carries new()");
        assert!(
            inherent.contains("pub fn payload("),
            "{head} carries payload()"
        );
        assert!(
            inherent.contains("pub fn into_payload("),
            "{head} carries into_payload()"
        );
        assert!(
            from.contains(&format!("for {head} {{")),
            "From impl for {head}:\n{from}"
        );
        assert!(
            GOLDEN.contains(&inherent),
            "{head} inherent impl byte-exact"
        );
        assert!(GOLDEN.contains(&from), "{head} From impl byte-exact");
    }
}

#[test]
fn the_wire_stub_transcribes_the_short_header_module_byte_exact() {
    let spirit = SpiritMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&spirit.schema, &spirit.names)
        .expect("lower");
    // Class C begins after declarations (12) + class A (12) + class B (10) = 34.
    let module = project(&lowering.items[34], &lowering.names);
    assert!(
        module.starts_with("#[rustfmt::skip]\npub mod short_header {"),
        "{module}"
    );
    assert!(
        GOLDEN.contains(&module),
        "short_header module byte-exact:\n{module}"
    );
    // The psyche-pending .9 values are transcribed verbatim.
    assert!(module.contains("pub const INPUT_RECORD: u64 = 0x0000000000000000;"));
    assert!(module.contains("pub const OUTPUT_RECORDS_OBSERVED: u64 = 0x0101000000000000;"));
}

#[test]
fn an_enriched_selection_on_a_root_less_schema_errors_loudly() {
    // Class B/C/D gate on interface roots; a schema of one plain newtype has none.
    let mut names = NameTable::new();
    let identifier = names.intern(Name::new("Lonely"));
    let schema = CoreSchema::new(vec![newtype(identifier, CoreReference::Integer)]);
    let error = MacroPackage::enriched_fixture()
        .apply_enriched(&schema, &names)
        .expect_err("interface classes must reject a root-less schema");
    assert!(
        matches!(error, core_nomos::NomosError::Generation(_)),
        "got {error:?}",
    );
}

#[test]
fn the_plain_and_wire_fixtures_keep_an_empty_selection() {
    // The enriched selection is additive: the existing packages are unchanged, so
    // apply_enriched on them equals apply (declarations only).
    let spirit = SpiritMin::build();
    for package in [MacroPackage::wire_fixture(), MacroPackage::plain_fixture()] {
        assert!(package.selection().is_empty());
        let enriched = package
            .apply_enriched(&spirit.schema, &spirit.names)
            .expect("lower");
        assert_eq!(enriched.items.len(), 12, "declarations only, no generation");
    }
}

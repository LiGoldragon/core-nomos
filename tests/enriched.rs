//! The enriched capstone: spirit-min's whole `CoreSchema` — its data declarations
//! and its two interface roots — lowered through the enriched wire package: the
//! newtype ergonomics, the interface ergonomics, the wire-contract vocabulary, the
//! wire exchange codec, the wire exchange envelope, and the trace support, in the
//! golden's document order.
//!
//! Two specs meet here. The *faithful* items reproduce the frozen `spirit_generated.rs`
//! golden byte-for-byte, in strictly increasing document order — the declarations, A,
//! B, the `short_header` module, the byte-count const, the route enums, the whole
//! ordinary-leg envelope surface (`RequestPayload`, `SignalOperationHeads`,
//! `LogVariant`, the `ExchangeFrame` aliases, and the `into_frame` / `into_reply_frame`
//! constructors), and the class-D trace surface (including the `pub`-field
//! `TraceEvent(pub ObjectName)` tuple struct). The *codec-shape* items — the
//! `SignalFrameError` enum and the two `impl <Root>` codec blocks — are specified
//! behaviorally: they must project to valid Rust that carries the working
//! `encode_signal_frame` / `decode_signal_frame` bodies and speaks the golden's *wire*
//! (an 8-byte little-endian short header ahead of an rkyv archive), not its source
//! text. The load-bearing round-trip proof lives in the four-process witness, where
//! the emitted crate is compiled.

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
    ("wire contract vocabulary", 5),
    ("wire exchange codec", 2),
    ("wire exchange envelope", 10),
    ("D: trace support", 6),
];

/// The item indices whose bodies are *behaviorally* specified, not byte-copies of the
/// golden: the `SignalFrameError` enum (a leaner unit/tuple-variant error than the
/// golden's struct-variant one) and the two `impl <Root>` codec blocks (whose codec
/// bodies mirror the golden's *wire* — 8-byte little-endian header then rkyv archive —
/// in the modeled statement style, not its source text). Every other enriched item
/// still reproduces the golden verbatim.
const CODEC_SHAPE_ITEMS: &[usize] = &[36, 39, 40];

#[test]
fn the_enriched_run_projects_valid_rust_in_the_expected_class_shape() {
    let spirit = SpiritMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&spirit.schema, &spirit.names)
        .expect("enriched lowering");

    let expected_total: usize = CLASS_LAYOUT.iter().map(|(_, count)| count).sum();
    assert_eq!(
        lowering.items.len(),
        expected_total,
        "the enriched run emits declarations + A + B + the wire contract + the wire \
         exchange codec + D",
    );

    // Every generated item — the codec bodies included — projects to valid Rust:
    // `project_item` runs `syn::parse2` + prettyplease, so an Ok result is proof the
    // emitted tokens parse as a Rust item. This is the working-programs spec at the
    // core-nomos boundary; the four-process witness compiles and round-trips them.
    for (index, item) in lowering.items.iter().enumerate() {
        RustSource::project_item(item, &lowering.names).unwrap_or_else(|error| {
            panic!("item {index} did not project to valid Rust: {error:?}")
        });
    }
}

#[test]
fn the_faithful_items_stay_byte_exact_against_the_spirit_golden() {
    let spirit = SpiritMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&spirit.schema, &spirit.names)
        .expect("enriched lowering");

    // Every item that reproduces the golden verbatim is present in it, in strictly
    // increasing document order — the document-order rule. The behaviorally-specified
    // codec items are skipped (their spec is round-trip, not byte-resemblance).
    let mut previous_offset = 0usize;
    for (index, item) in lowering.items.iter().enumerate() {
        if CODEC_SHAPE_ITEMS.contains(&index) {
            continue;
        }
        let text = project(item, &lowering.names);
        let offset = GOLDEN.find(&text).unwrap_or_else(|| {
            panic!("item {index} is not present verbatim in the golden:\n{text}")
        });
        assert!(
            offset >= previous_offset,
            "item {index} is out of document order (offset {offset} < {previous_offset}):\n{text}",
        );
        previous_offset = offset;
    }
}

#[test]
fn the_wire_exchange_codec_emits_working_encode_decode_bodies() {
    let spirit = SpiritMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&spirit.schema, &spirit.names)
        .expect("enriched lowering");

    // The byte-count const and the SignalFrameError vocabulary the codec speaks.
    let byte_count = project(&lowering.items[35], &lowering.names);
    assert_eq!(
        byte_count,
        "#[rustfmt::skip]\nconst SIGNAL_SHORT_HEADER_BYTE_COUNT: usize = 8;"
    );
    let error = project(&lowering.items[36], &lowering.names);
    for fragment in [
        "pub enum SignalFrameError",
        "ArchiveEncode",
        "ArchiveDecode",
        "FrameTooShort",
        "UnknownHeader(u64)",
        "HeaderMismatch",
    ] {
        assert!(
            error.contains(fragment),
            "SignalFrameError carries {fragment}:\n{error}"
        );
    }

    // The Input codec impl carries every ordinary-leg codec method, and its bodies
    // speak the wire: the 8-byte little-endian short header ahead of an rkyv archive,
    // with the decode header-mismatch and unknown-header guards.
    let input_codec = project(&lowering.items[39], &lowering.names);
    for fragment in [
        "pub fn route(&self) -> InputRoute {",
        "pub fn short_header(&self) -> u64 {",
        "pub fn route_from_short_header(header: u64) -> Result<InputRoute, SignalFrameError> {",
        "_ => Err(SignalFrameError::UnknownHeader(header)),",
        "pub fn encode_signal_frame(&self) -> Result<Vec<u8>, SignalFrameError> {",
        "rkyv::to_bytes::<rkyv::rancor::Error>(self)",
        ".map_err(|_| SignalFrameError::ArchiveEncode)?;",
        "let mut frame = self.short_header().to_le_bytes().to_vec();",
        "frame.extend_from_slice(&archive);",
        "Ok(frame)",
        "Result<(InputRoute, Self), SignalFrameError> {",
        "let header = u64::from_le_bytes(",
        ".get(..SIGNAL_SHORT_HEADER_BYTE_COUNT)",
        ".ok_or(SignalFrameError::FrameTooShort)?",
        "let route = Self::route_from_short_header(header)?;",
        "rkyv::from_bytes::<",
        "&frame[SIGNAL_SHORT_HEADER_BYTE_COUNT..]",
        ".map_err(|_| SignalFrameError::ArchiveDecode)?;",
        ".ok_or(SignalFrameError::HeaderMismatch)?;",
        "Ok((route, value))",
    ] {
        assert!(
            input_codec.contains(fragment),
            "Input codec carries `{fragment}`:\n{input_codec}"
        );
    }

    // The Output codec is the same surface over the reply root.
    let output_codec = project(&lowering.items[40], &lowering.names);
    assert!(output_codec.contains("pub fn decode_signal_frame("));
    assert!(output_codec.contains("Result<(OutputRoute, Self), SignalFrameError> {"));

    // The whole module also projects in one pass (the eventual full-file assembly).
    RustSource::project_module(&lowering.items, &lowering.names).expect("project whole module");
}

#[test]
fn the_wire_exchange_envelope_emits_the_ordinary_leg_surface() {
    let spirit = SpiritMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&spirit.schema, &spirit.names)
        .expect("enriched lowering");

    // The envelope follows the two codec impls: the request root's three trait impls,
    // the five `ExchangeFrame` aliases, then the `into_frame` / `into_reply_frame`
    // constructors — every item byte-exact against the golden (the ordinary two-way
    // leg, never `StreamingFrame`).
    let envelope: Vec<String> = (41..=50)
        .map(|index| project(&lowering.items[index], &lowering.names))
        .collect();
    let expected = [
        "impl signal_frame::RequestPayload for Input {}",
        "impl signal_frame::SignalOperationHeads for Input {",
        "impl signal_frame::LogVariant for Input {",
        "pub type Frame = signal_frame::ExchangeFrame<Input, Output>;",
        "pub type FrameBody = signal_frame::ExchangeFrameBody<Input, Output>;",
        "pub type Request = signal_frame::Request<Input>;",
        "pub type ReplyEnvelope = signal_frame::Reply<Output>;",
        "pub type RequestBuilder = signal_frame::RequestBuilder<Input>;",
        "pub fn into_frame(self, exchange: signal_frame::ExchangeIdentifier) -> Frame {",
        "pub fn into_reply_frame(self, exchange: signal_frame::ExchangeIdentifier) -> Frame {",
    ];
    for (item, fragment) in envelope.iter().zip(expected) {
        assert!(
            item.contains(fragment),
            "envelope item carries `{fragment}`:\n{item}"
        );
        assert!(
            GOLDEN.contains(item.as_str()),
            "envelope item is byte-exact against the golden:\n{item}"
        );
    }

    // The ordinary leg names `ExchangeFrame`, never the streaming envelope, whose
    // `StreamingFrame` / `SubscriptionEvent` surface waits on pending psyche rulings.
    for item in &envelope {
        assert!(
            !item.contains("StreamingFrame") && !item.contains("SubscriptionEvent"),
            "the ordinary leg emits no streaming surface:\n{item}"
        );
    }

    // The struct-variant construction the constructors carry — the `StructLiteral`
    // node in shorthand-field form the envelope adds to the Tier-1 vocabulary.
    let into_frame = &envelope[8];
    assert!(into_frame.contains("FrameBody::Request {"), "{into_frame}");
    assert!(
        into_frame.contains("signal_frame::Request::from_payload(self)"),
        "{into_frame}"
    );
    let into_reply = &envelope[9];
    assert!(into_reply.contains("FrameBody::Reply {"), "{into_reply}");
    assert!(
        into_reply.contains("signal_frame::SubReply::Ok(self)"),
        "{into_reply}"
    );
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
fn the_wire_stub_derives_the_short_header_module_byte_exact() {
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
    // The values are derived from each operation's position —
    // (root_index << 56) | (variant_index << 48) — reproducing schema-rust's legacy
    // byte layout exactly: Input::Record at root 0 / variant 0 is 0x0000000000000000,
    // and Output::RecordsObserved at root 1 / variant 1 is 0x0101000000000000.
    assert!(module.contains("pub const INPUT_RECORD: u64 = 0x0000000000000000;"));
    assert!(module.contains("pub const OUTPUT_RECORDS_OBSERVED: u64 = 0x0101000000000000;"));
}

#[test]
fn class_d_emits_the_pub_field_trace_event_declaration_byte_exact() {
    let spirit = SpiritMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&spirit.schema, &spirit.names)
        .expect("lower");
    // Class D begins after declarations (12) + A (12) + B (10) + wire contract (5) +
    // wire exchange codec (2) + wire exchange envelope (10) = 51. The TraceEvent
    // declaration is its fourth item (index 54), between the ObjectName enum and the
    // impl ObjectName — the last class-D gap the layout-4 tuple-field visibility closes.
    let declaration = project(&lowering.items[54], &lowering.names);
    assert!(
        declaration.ends_with("pub struct TraceEvent(pub ObjectName);"),
        "the class-D declaration is the pub-field TraceEvent tuple struct:\n{declaration}"
    );
    assert!(
        GOLDEN.contains(&declaration),
        "the TraceEvent declaration is byte-exact against the golden:\n{declaration}"
    );
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

/// A second, structurally distinct schema: two interface roots that each carry a
/// single operation. It proves the short-header derivation is schema-general — it
/// emits one constant per operation from the operation's position, not the four
/// spirit-min values a transcription would have carried. This is the interface shape
/// the four-process witness drives as `second-min`.
struct SecondMin {
    schema: CoreSchema,
    names: NameTable,
}

impl SecondMin {
    fn build() -> Self {
        let mut names = NameTable::new();
        let mut intern = |text: &str| names.intern(Name::new(text));

        let weight = intern("Weight");
        let note = intern("Note");
        let priority = intern("Priority");
        let parcel = intern("Parcel");
        let ticket = intern("Ticket");
        let input = intern("Input");
        let output = intern("Output");

        let weight_decl = newtype(weight, CoreReference::Integer);
        let note_decl = newtype(note, CoreReference::String);
        let ticket_decl = newtype(ticket, CoreReference::Integer);
        let priority_decl = CoreDeclaration::public(CoreType::Enumeration(CoreEnum::new(
            priority,
            ["Low", "Normal", "Urgent"]
                .into_iter()
                .map(|name| CoreVariant::new(intern(name), None))
                .collect(),
        )));
        let parcel_decl = CoreDeclaration::public(CoreType::Struct(CoreStruct::new(
            parcel,
            vec![
                CoreField::new(intern("weight"), CoreReference::Plain(weight)),
                CoreField::new(intern("note"), CoreReference::Plain(note)),
                CoreField::new(intern("priority"), CoreReference::Plain(priority)),
            ],
        )));
        let input_decl = CoreDeclaration::interface(
            DeclarationRole::InterfaceInput,
            CoreType::Enumeration(CoreEnum::new(
                input,
                vec![CoreVariant::new(
                    intern("Enqueue"),
                    Some(CoreReference::Plain(parcel)),
                )],
            )),
        );
        let output_decl = CoreDeclaration::interface(
            DeclarationRole::InterfaceOutput,
            CoreType::Enumeration(CoreEnum::new(
                output,
                vec![CoreVariant::new(
                    intern("Enqueued"),
                    Some(CoreReference::Plain(ticket)),
                )],
            )),
        );

        let schema = CoreSchema::new(vec![
            weight_decl,
            note_decl,
            priority_decl,
            parcel_decl,
            ticket_decl,
            input_decl,
            output_decl,
        ]);
        Self { schema, names }
    }
}

#[test]
fn the_wire_stub_derives_two_short_headers_for_single_operation_roots() {
    let second = SecondMin::build();
    let lowering = MacroPackage::enriched_fixture()
        .apply_enriched(&second.schema, &second.names)
        .expect("the enriched package lowers second-min by derivation, not a fixed count");
    let module = lowering
        .items
        .iter()
        .map(|item| project(item, &lowering.names))
        .find(|text| text.contains("pub mod short_header {"))
        .expect("the wire stub emits a short_header module");
    // Two roots, one operation each: Input at root 0 / variant 0 and Output at root 1
    // / variant 0 — exactly two derived constants, where a transcription would have
    // carried spirit-min's four and rejected this schema on the count.
    assert!(
        module.contains("pub const INPUT_ENQUEUE: u64 = 0x0000000000000000;"),
        "{module}"
    );
    assert!(
        module.contains("pub const OUTPUT_ENQUEUED: u64 = 0x0100000000000000;"),
        "{module}"
    );
    assert_eq!(
        module.matches("pub const").count(),
        2,
        "exactly two derived short headers:\n{module}"
    );
}

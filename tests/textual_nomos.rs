//! The Standard-profile Nomos base door is one computed Template(Logos) path.

use std::cell::RefCell;
use std::collections::BTreeMap;

use core_logos::{LogosLanguage, LogosLanguageTypeIds, LogosLanguageWords};
use core_nomos::{
    AuthoredTransformerDeclaration, ItemTemplate, MacroIdentity, MacroPackage, MetaType,
    ResultTemplate, SectionDefault, SequenceItem, TemplateFutureKind, TemplateFutureOutput,
    TemplateLandingShape, TemplateLanguage, TemplateTerm, TextualNomos, TextualNomosMetaType,
    TextualNomosTypeIds, TextualNomosWords,
};
use encoded_name_table::{LocalEncodedId, Name};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::{
    DeclarationAssignment, DecodeNameBindings, EncodedNameResolver, LandingShape, NameOccurrence,
    ResolvedReference,
};

const SOURCE: &str = r#"{1}
[]
[]
{
WireAttributes.Named {
()
[
rustfmt.skip
(|nota-text|).[nota.NotaDecode nota.NotaDecodeTraced nota.NotaEncode]
[rkyv.Archive rkyv.Serialize rkyv.Deserialize Clone Debug PartialEq Eq]
]
}
WireNewtype.Structural.Newtype {
(name.Name wrapped.Type)
Public Invoke.WireAttributes Realize.name Private Realize.wrapped
}
Enumeration.Structural.Enumeration {
(name.Name variants.Variants)
Public Invoke.WireAttributes Realize.name () [Splice.variants]
}
}
{}
{}"#;

fn encoded(chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("fixture identity is non-empty")
}

fn logos() -> LogosLanguage {
    LogosLanguage::seal(
        LogosLanguageTypeIds {
            newtype: encoded(&[1]),
            enumeration: encoded(&[2]),
            visibility: encoded(&[3]),
            attributes: encoded(&[4]),
            attribute: encoded(&[5]),
            path: encoded(&[6]),
            configuration_predicate: encoded(&[7]),
            derive_group: encoded(&[8]),
            generics: encoded(&[9]),
            generic_parameter: encoded(&[10]),
            type_reference: encoded(&[11]),
            variant: encoded(&[12]),
        },
        LogosLanguageWords {
            public: encoded(&[20]),
            private: encoded(&[21]),
        },
    )
    .expect("canonical Logos grammar and landing declarations agree")
}

fn root_constructor(
    language: &TemplateLanguage<VocabularyRoot>,
) -> &core_nomos::TemplateConstructorDeclaration<VocabularyRoot> {
    language
        .type_declaration(language.root())
        .and_then(|declaration| declaration.constructors().first())
        .expect("fixture root constructor")
}

fn field_shape(
    language: &TemplateLanguage<VocabularyRoot>,
    index: usize,
) -> &TemplateLandingShape<VocabularyRoot> {
    root_constructor(language)
        .landing_fields()
        .get(index)
        .map(core_nomos::TemplateLandingField::shape)
        .expect("fixture field")
}

fn literal_landing(shape: &TemplateLandingShape<VocabularyRoot>) -> LandingShape<VocabularyRoot> {
    match shape {
        TemplateLandingShape::Fixed(landing)
        | TemplateLandingShape::ValueOrFuture { value: landing, .. } => landing.clone(),
        TemplateLandingShape::Nested(target) => LandingShape::Type(target.clone()),
        TemplateLandingShape::Sequence {
            minimum,
            maximum,
            element,
            ..
        } => LandingShape::sequence(*minimum, *maximum, literal_landing(element)),
    }
}

fn output(shape: &TemplateLandingShape<VocabularyRoot>) -> TemplateFutureOutput<VocabularyRoot> {
    TemplateFutureOutput::new(literal_landing(shape))
}

fn textual(logos: &LogosLanguage) -> TextualNomos {
    let newtype = TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.newtype_type())
        .expect("newtype Template(Logos)");
    let enumeration =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.enumeration_type())
            .expect("enumeration Template(Logos)");
    TextualNomos::seal(
        logos,
        TextualNomosTypeIds {
            document: encoded(&[100, 1]),
            revision: encoded(&[100, 2]),
            empty_braces: encoded(&[100, 3]),
            empty_square: encoded(&[100, 4]),
            transformers: encoded(&[100, 5]),
            transformer: encoded(&[100, 6]),
            input_signature: encoded(&[100, 7]),
            input_parameter: encoded(&[100, 8]),
            newtype_body: encoded(&[100, 9]),
            enumeration_body: encoded(&[100, 10]),
            attributes_body: encoded(&[100, 11]),
        },
        TextualNomosWords {
            named: encoded(&[101, 1]),
            structural: encoded(&[101, 2]),
            newtype: encoded(&[101, 3]),
            enumeration: encoded(&[101, 4]),
            realize: encoded(&[101, 5]),
            splice: encoded(&[101, 6]),
            invoke: encoded(&[101, 7]),
        },
        vec![
            TextualNomosMetaType {
                word: encoded(&[102, 1]),
                meta: MetaType::Name,
                output: output(field_shape(&newtype, 2)),
            },
            TextualNomosMetaType {
                word: encoded(&[102, 2]),
                meta: MetaType::Type,
                output: output(field_shape(&newtype, 4)),
            },
            TextualNomosMetaType {
                word: encoded(&[102, 3]),
                meta: MetaType::Variants,
                output: output(field_shape(&enumeration, 4)),
            },
        ],
    )
    .expect("TextualNomos table seals")
}

#[derive(Default)]
struct Bindings {
    declarations: BTreeMap<(usize, usize), (String, VocabularyEncodedId)>,
    references: BTreeMap<(usize, usize), (String, VocabularyEncodedId)>,
    spellings: BTreeMap<VocabularyEncodedId, Name>,
    declaration_queries: RefCell<Vec<(String, usize, usize)>>,
    reference_queries: RefCell<Vec<(String, usize, usize)>>,
}

impl Bindings {
    fn spelling(&mut self, encoded_id: &VocabularyEncodedId, spelling: &str) {
        self.spellings
            .insert(encoded_id.clone(), Name::new(spelling));
    }

    fn declaration(
        &mut self,
        source: &str,
        spelling: &str,
        occurrence: usize,
        encoded_id: VocabularyEncodedId,
    ) {
        let (start, end) = token_occurrences(source, spelling)[occurrence];
        self.spelling(&encoded_id, spelling);
        self.declarations
            .insert((start, end), (spelling.to_owned(), encoded_id));
    }

    fn reference(
        &mut self,
        source: &str,
        spelling: &str,
        occurrence: usize,
        encoded_id: VocabularyEncodedId,
    ) {
        let (start, end) = token_occurrences(source, spelling)[occurrence];
        self.spelling(&encoded_id, spelling);
        self.references
            .insert((start, end), (spelling.to_owned(), encoded_id));
    }

    fn all_references(&mut self, source: &str, spelling: &str, encoded_id: VocabularyEncodedId) {
        self.spelling(&encoded_id, spelling);
        for (start, end) in token_occurrences(source, spelling) {
            self.references
                .insert((start, end), (spelling.to_owned(), encoded_id.clone()));
        }
    }

    fn remove_reference(&mut self, source: &str, spelling: &str, occurrence: usize) {
        let bound = token_occurrences(source, spelling)[occurrence];
        self.references.remove(&bound);
    }
}

impl EncodedNameResolver<VocabularyRoot> for Bindings {
    fn resolve(&self, encoded_id: &VocabularyEncodedId) -> Option<&Name> {
        self.spellings.get(encoded_id)
    }
}

impl DecodeNameBindings<VocabularyRoot> for Bindings {
    fn declaration_assignment(
        &self,
        occurrence: NameOccurrence<'_>,
    ) -> Option<DeclarationAssignment<VocabularyRoot>> {
        let start = occurrence.bound().start();
        let end = occurrence.bound().end();
        self.declaration_queries
            .borrow_mut()
            .push((occurrence.spelling().to_owned(), start, end));
        self.declarations
            .get(&(start, end))
            .filter(|(spelling, _)| spelling == occurrence.spelling())
            .map(|(_, encoded_id)| DeclarationAssignment::new(encoded_id.clone()))
    }

    fn reference_resolution(
        &self,
        occurrence: NameOccurrence<'_>,
    ) -> Option<ResolvedReference<VocabularyRoot>> {
        let start = occurrence.bound().start();
        let end = occurrence.bound().end();
        self.reference_queries
            .borrow_mut()
            .push((occurrence.spelling().to_owned(), start, end));
        self.references
            .get(&(start, end))
            .filter(|(spelling, _)| spelling == occurrence.spelling())
            .map(|(_, encoded_id)| ResolvedReference::new(encoded_id.clone()))
    }
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '-')
}

fn token_occurrences(source: &str, spelling: &str) -> Vec<(usize, usize)> {
    source
        .match_indices(spelling)
        .filter_map(|(start, matched)| {
            let end = start + matched.len();
            let before = source[..start].chars().next_back();
            let after = source[end..].chars().next();
            (!before.is_some_and(is_word_character) && !after.is_some_and(is_word_character))
                .then_some((start, end))
        })
        .collect()
}

fn bindings(source: &str) -> Bindings {
    let mut bindings = Bindings::default();
    for (identity, spelling) in [
        (encoded(&[20]), "Public"),
        (encoded(&[21]), "Private"),
        (encoded(&[101, 1]), "Named"),
        (encoded(&[101, 2]), "Structural"),
        (encoded(&[101, 3]), "Newtype"),
        (encoded(&[101, 4]), "Enumeration"),
        (encoded(&[101, 5]), "Realize"),
        (encoded(&[101, 6]), "Splice"),
        (encoded(&[101, 7]), "Invoke"),
        (encoded(&[102, 1]), "Name"),
        (encoded(&[102, 2]), "Type"),
        (encoded(&[102, 3]), "Variants"),
    ] {
        bindings.spelling(&identity, spelling);
    }

    bindings.declaration(source, "WireAttributes", 0, encoded(&[110, 1]));
    bindings.reference(source, "WireAttributes", 1, encoded(&[110, 1]));
    bindings.reference(source, "WireAttributes", 2, encoded(&[110, 1]));
    bindings.declaration(source, "WireNewtype", 0, encoded(&[110, 2]));
    bindings.declaration(source, "Enumeration", 0, encoded(&[110, 3]));

    bindings.declaration(source, "name", 0, encoded(&[110, 2, 1]));
    bindings.reference(source, "name", 1, encoded(&[110, 2, 1]));
    bindings.declaration(source, "name", 2, encoded(&[110, 3, 1]));
    bindings.reference(source, "name", 3, encoded(&[110, 3, 1]));
    bindings.declaration(source, "wrapped", 0, encoded(&[110, 2, 2]));
    bindings.reference(source, "wrapped", 1, encoded(&[110, 2, 2]));
    bindings.declaration(source, "variants", 0, encoded(&[110, 3, 2]));
    bindings.reference(source, "variants", 1, encoded(&[110, 3, 2]));

    for (index, spelling) in [
        "rustfmt",
        "skip",
        "nota-text",
        "nota",
        "NotaDecode",
        "NotaDecodeTraced",
        "NotaEncode",
        "rkyv",
        "Archive",
        "Serialize",
        "Deserialize",
        "Clone",
        "Debug",
        "PartialEq",
        "Eq",
    ]
    .into_iter()
    .enumerate()
    {
        bindings.all_references(source, spelling, encoded(&[120, index as u16 + 1]));
    }
    let carrier = "(|nota-text|)";
    if let Some(start) = source.find(carrier) {
        bindings.references.insert(
            (start, start + carrier.len()),
            ("nota-text".to_owned(), encoded(&[120, 3])),
        );
    }
    bindings
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FragmentCardinality {
    One,
    Sequence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FixtureOracle {
    fragment: FragmentCardinality,
    futures: Vec<TemplateFutureKind>,
    top_level_literals: usize,
}

fn collect_generic_futures(
    term: &TemplateTerm<VocabularyRoot>,
    futures: &mut Vec<TemplateFutureKind>,
) {
    match term {
        TemplateTerm::Future(future) => futures.push(future.kind()),
        TemplateTerm::Nested(value) => {
            for field in value.fields() {
                collect_generic_futures(field.term(), futures);
            }
        }
        TemplateTerm::Sequence(items) => {
            for item in items {
                collect_generic_futures(item, futures);
            }
        }
        TemplateTerm::Declaration(_)
        | TemplateTerm::Reference(_)
        | TemplateTerm::Literal(_)
        | TemplateTerm::Scalar(_) => {}
    }
}

fn generic_oracle(declaration: &AuthoredTransformerDeclaration) -> FixtureOracle {
    let fragment = match declaration.output().landing() {
        LandingShape::Sequence { .. } => FragmentCardinality::Sequence,
        _ => FragmentCardinality::One,
    };
    let mut futures = Vec::new();
    for field in declaration.result().fields() {
        collect_generic_futures(field.term(), &mut futures);
    }
    let top_level_literals = if fragment == FragmentCardinality::Sequence {
        declaration
            .result()
            .fields()
            .first()
            .and_then(|field| match field.term() {
                TemplateTerm::Sequence(items) => Some(
                    items
                        .iter()
                        .filter(|item| !matches!(item, TemplateTerm::Future(_)))
                        .count(),
                ),
                _ => None,
            })
            .unwrap_or_default()
    } else {
        0
    };
    FixtureOracle {
        fragment,
        futures,
        top_level_literals,
    }
}

fn collect_legacy_sequence<Literal>(
    sequence: &core_nomos::Sequence<Literal>,
    futures: &mut Vec<TemplateFutureKind>,
) {
    for item in &sequence.items {
        if let SequenceItem::Escape(escape) = item {
            futures.push(match escape {
                core_nomos::Escape::Realize(_) => TemplateFutureKind::Realize,
                core_nomos::Escape::Invoke(_) => TemplateFutureKind::Invoke,
                core_nomos::Escape::Splice(_) => TemplateFutureKind::Splice,
            });
        }
    }
}

fn legacy_oracle(template: &ResultTemplate) -> FixtureOracle {
    let mut futures = Vec::new();
    let (fragment, top_level_literals) = match template {
        ResultTemplate::Attributes(sequence) => {
            collect_legacy_sequence(sequence, &mut futures);
            (
                FragmentCardinality::Sequence,
                sequence
                    .items
                    .iter()
                    .filter(|item| matches!(item, SequenceItem::Literal(_)))
                    .count(),
            )
        }
        ResultTemplate::Item(ItemTemplate::Newtype(value)) => {
            collect_legacy_sequence(&value.attributes, &mut futures);
            if let core_nomos::Scalar::Escape(escape) = &value.name {
                futures.push(match escape {
                    core_nomos::Escape::Realize(_) => TemplateFutureKind::Realize,
                    core_nomos::Escape::Invoke(_) => TemplateFutureKind::Invoke,
                    core_nomos::Escape::Splice(_) => TemplateFutureKind::Splice,
                });
            }
            if let core_nomos::Scalar::Escape(escape) = &value.wrapped {
                futures.push(match escape {
                    core_nomos::Escape::Realize(_) => TemplateFutureKind::Realize,
                    core_nomos::Escape::Invoke(_) => TemplateFutureKind::Invoke,
                    core_nomos::Escape::Splice(_) => TemplateFutureKind::Splice,
                });
            }
            (FragmentCardinality::One, 0)
        }
        ResultTemplate::Item(ItemTemplate::Enumeration(value)) => {
            collect_legacy_sequence(&value.attributes, &mut futures);
            if let core_nomos::Scalar::Escape(escape) = &value.name {
                futures.push(match escape {
                    core_nomos::Escape::Realize(_) => TemplateFutureKind::Realize,
                    core_nomos::Escape::Invoke(_) => TemplateFutureKind::Invoke,
                    core_nomos::Escape::Splice(_) => TemplateFutureKind::Splice,
                });
            }
            collect_legacy_sequence(&value.variants, &mut futures);
            (FragmentCardinality::One, 0)
        }
        ResultTemplate::Item(ItemTemplate::Struct(_)) => panic!("not a base-door oracle"),
    };
    FixtureOracle {
        fragment,
        futures,
        top_level_literals,
    }
}

#[test]
fn standard_text_decodes_three_computed_shapes_and_round_trips() {
    let logos = logos();
    let textual = textual(&logos);
    let first_bindings = bindings(SOURCE);
    let decoded = textual
        .decode(SOURCE, &first_bindings)
        .expect("typed base-door decode");
    assert_eq!(decoded.revision(), 1);
    assert_eq!(decoded.transformers().declarations().len(), 3);

    let production = MacroPackage::wire_fixture().expect("independent production fixture");
    let expected = [
        production
            .definition(MacroIdentity::new(0))
            .expect("WireAttributes")
            .template
            .clone(),
        production
            .structural_default(SectionDefault::Newtype)
            .and_then(|identity| production.definition(identity))
            .expect("WireNewtype")
            .template
            .clone(),
        production
            .structural_default(SectionDefault::Enumeration)
            .and_then(|identity| production.definition(identity))
            .expect("Enumeration")
            .template
            .clone(),
    ];
    let found: BTreeMap<_, _> = decoded
        .transformers()
        .declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.name().encoded_id().clone(),
                generic_oracle(declaration),
            )
        })
        .collect();
    assert_eq!(
        found.get(&encoded(&[110, 1])),
        Some(&legacy_oracle(&expected[0]))
    );
    assert_eq!(
        found.get(&encoded(&[110, 2])),
        Some(&legacy_oracle(&expected[1]))
    );
    assert_eq!(
        found.get(&encoded(&[110, 3])),
        Some(&legacy_oracle(&expected[2]))
    );

    let viewed = textual
        .view(&decoded, &first_bindings)
        .expect("canonical typed view");
    let second_bindings = bindings(&viewed);
    let restored = textual
        .decode(&viewed, &second_bindings)
        .expect("canonical view decodes");
    assert_eq!(restored.revision(), decoded.revision());
    assert_eq!(restored.transformers(), decoded.transformers());
}

#[test]
fn canonical_realize_tail_forces_ordered_sequence_backtracking() {
    let logos = logos();
    let textual = textual(&logos);
    let assigned = bindings(SOURCE);
    let name_bound = token_occurrences(SOURCE, "name")[1];

    let decoded = textual
        .decode(SOURCE, &assigned)
        .expect("the repeated attributes must yield Realize.name to the required name slot");
    assert_eq!(decoded.transformers().declarations().len(), 3);
    assert!(
        assigned
            .reference_queries
            .borrow()
            .iter()
            .filter(|(_, start, end)| (*start, *end) == name_bound)
            .count()
            >= 2,
        "the canonical fixture must exercise the greedy candidate and the shorter viable tail"
    );
}

#[test]
fn every_future_payload_is_lookup_only_and_missing_lookup_refuses() {
    let logos = logos();
    let textual = textual(&logos);
    let successful = bindings(SOURCE);
    textual
        .decode(SOURCE, &successful)
        .expect("valid lookup-only decode");

    for spelling in ["WireAttributes", "name", "wrapped", "variants"] {
        let reference_bounds: Vec<_> = successful
            .reference_queries
            .borrow()
            .iter()
            .filter(|(queried, _, _)| queried == spelling)
            .map(|(_, start, end)| (*start, *end))
            .collect();
        let declaration_bounds: Vec<_> = successful
            .declaration_queries
            .borrow()
            .iter()
            .filter(|(queried, _, _)| queried == spelling)
            .map(|(_, start, end)| (*start, *end))
            .collect();
        assert!(
            reference_bounds
                .iter()
                .all(|bound| !declaration_bounds.contains(bound)),
            "{spelling} future payload must never request a declaration assignment"
        );
    }

    let mut missing = bindings(SOURCE);
    missing.remove_reference(SOURCE, "WireAttributes", 1);
    assert!(
        textual.decode(SOURCE, &missing).is_err(),
        "Invoke must refuse instead of allocating its target"
    );
    let invoke_bound = token_occurrences(SOURCE, "WireAttributes")[1];
    assert!(
        !missing
            .declaration_queries
            .borrow()
            .iter()
            .any(|(_, start, end)| (*start, *end) == invoke_bound),
        "failed lookup must not fall back to declaration assignment"
    );
}

#[test]
fn malformed_unknown_and_wrong_position_future_forms_refuse() {
    let logos = logos();
    let textual = textual(&logos);
    for invalid in [
        SOURCE.replacen("Realize.name", "Invoke.name", 1),
        SOURCE.replacen("Splice.variants", "Realize.variants", 1),
        SOURCE.replacen("Realize.name", "Mystery.name", 1),
        SOURCE.replacen("Realize.name", "Realize..name", 1),
    ] {
        assert!(
            textual.decode(&invalid, &bindings(&invalid)).is_err(),
            "invalid reserved application must refuse: {invalid}"
        );
    }
}

#[test]
fn base_door_keeps_exactly_the_standard_seven_triggers() {
    let logos = logos();
    let textual = textual(&logos);
    assert_eq!(
        textual.token_profile().root_trigger_set().triggers(),
        &(0..=6)
            .map(raw_discovery::TriggerIdentifier::new)
            .collect::<Vec<_>>()
    );
    assert!(textual.token_profile().bare_character_is_forbidden('$'));
}

#[test]
fn decoder_source_has_one_generic_derivation_and_no_authored_twins() {
    let source = include_str!("../src/textual.rs");
    for forbidden in [
        "AuthoredNewtype",
        "AuthoredEnumeration",
        "AuthoredAttributes",
        "WireNewtypeRecord",
        "WireAttributesRecord",
        "EnumerationTemplateRecord",
    ] {
        assert!(
            !source.contains(forbidden),
            "computed landing shapes must not gain {forbidden}"
        );
    }
    assert!(source.contains("DerivedTemplateRecord"));
    assert!(source.contains("fn lift_descriptor"));
}

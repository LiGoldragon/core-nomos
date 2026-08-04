//! The Standard-profile Nomos base door is one computed Template(Logos) path.

#![allow(dead_code, unused_imports)] // Retained helpers serve the retired strict-carrier witnesses above.

use std::cell::RefCell;
use std::collections::BTreeMap;

use capsule_content_identity::PortableArchive;
use core_ethos::{
    EncodedDeclaration, EncodedEnum, EncodedEthos, EncodedField, EncodedNewtype, EncodedReference,
    EncodedStruct, EncodedType, EncodedVariant, WholeEthos, WholeEthosAttributes,
    WholeEthosEnumeration, WholeEthosItem, WholeEthosNewtype, WholeEthosTupleFields,
    WholeEthosTypeApplication, WholeEthosTypeReference, WholeEthosVariant,
    WholeEthosVariantPayload, WholeEthosVisibility, WholeEthosWrappedField,
};
use core_logos::{
    Attribute, ConfigurationPredicate, DeriveGroup, Generics, LogosLanguage, LogosLanguageTypeIds,
    LogosLanguageWords, Visibility,
};
use core_nomos::{
    AuthoredBindingIdentity, AuthoredInputParameter, AuthoredInputSignature, AuthoredNomosError,
    AuthoredTransformerDeclaration, AuthoredTransformerIdentity, AuthoredTransformerSet,
    BindingRef, Escape, FieldNameRule, GenerationClass, ItemTemplate, MacroKind, MacroPackage,
    MetaType, NameTransform, NativeAuthoredEvaluator, NativeEvaluatedLogos, NativeEvaluatedTerm,
    NativeLogosNameTree, NativeLogosPopulation, NativeNameTreeBoundary, NativeNameTreePlan,
    NativeNameTreeRefusal, NativeReferenceMapping, NativeReferenceUniverse, ResultTemplate, Scalar,
    SectionDefault, Sequence, SequenceItem, SpliceElement, TemplateFieldValue, TemplateFuture,
    TemplateFutureOutput, TemplateLandingShape, TemplateLanguage, TemplateRootOutputSelector,
    TemplateTerm, TemplateValue, TextualNomos, TextualNomosMetaType, TextualNomosTypeIds,
    TextualNomosWords,
};
use encoded_name_table::{LocalEncodedId, Name};
use name_table::{IdentifierNamespace, Name as LegacyName, NameTable};
use protos::EncodedPopulation;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::{
    DeclarationAssignment, DecodeNameBindings, EncodedConstructorId, EncodedNameResolver,
    EncodedTypeId, LandingShape, NameOccurrence, ResolvedReference,
};
use textual_rust::RustSource;

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
EnumerationAttributes.Named {
()
[
rustfmt.skip
(|nota-text|).[nota.NotaDecode nota.NotaDecodeTraced nota.NotaEncode]
[rkyv.Archive rkyv.Serialize rkyv.Deserialize Clone Copy Debug PartialEq Eq]
]
}
WireNewtype.Structural.Newtype {
(name.Name type.Type)
Public Invoke.WireAttributes Realize.name Private Realize.type
}
ParticularStruct.Structural.Struct {
(name.Name fields.Fields)
Public Invoke.WireAttributes Realize.name () [Splice.fields]
}
Enumeration.Structural.Enumeration {
(name.Name variants.Variants)
Public Invoke.EnumerationAttributes Realize.name () [Splice.variants]
}
}
{}
{}"#;

const RECURSIVE_SOURCE: &str = r#"{1}
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
EnumerationAttributes.Named {
()
[
rustfmt.skip
(|nota-text|).[nota.NotaDecode nota.NotaDecodeTraced nota.NotaEncode]
[rkyv.Archive rkyv.Serialize rkyv.Deserialize Clone Copy Debug PartialEq Eq]
]
}
WireNewtype.Structural.Newtype {
(name.Name type.Type)
Public Invoke.WireAttributes Realize.name Private Realize.type
}
ParticularStruct.Structural.Struct {
(name.Name fields.Fields)
Public Invoke.WireAttributes Realize.name () [Splice.fields]
}
ScopeOfStep.Recursive.Enumeration {
(variant.Name source.Variants children.Variants)
[
Invoke.ScopeOfStep
Splice.children
InsertAt.children 0 rustfmt.skip
[Clone]
]
}
Enumeration.Structural.Enumeration {
(name.Name variants.Variants)
Public Invoke.ScopeOfStep Realize.name () [Splice.variants]
}
}
{}
{}"#;

const RECURSIVE_CANONICAL: &str = "{1} [] [] {WireAttributes.Named {() [rustfmt.skip (|nota-text|).[nota.NotaDecode nota.NotaDecodeTraced nota.NotaEncode] [rkyv.Archive rkyv.Serialize rkyv.Deserialize Clone Debug PartialEq Eq]]} EnumerationAttributes.Named {() [rustfmt.skip (|nota-text|).[nota.NotaDecode nota.NotaDecodeTraced nota.NotaEncode] [rkyv.Archive rkyv.Serialize rkyv.Deserialize Clone Copy Debug PartialEq Eq]]} WireNewtype.Structural.Newtype {(name.Name type.Type) Public Invoke.WireAttributes Realize.name Private Realize.type} ParticularStruct.Structural.Struct {(name.Name fields.Fields) Public Invoke.WireAttributes Realize.name () [Splice.fields]} ScopeOfStep.Recursive.Enumeration {(variant.Name source.Variants children.Variants) [Invoke.ScopeOfStep Splice.children InsertAt.children 0 rustfmt.skip [Clone]]} Enumeration.Structural.Enumeration {(name.Name variants.Variants) Public Invoke.ScopeOfStep Realize.name () [Splice.variants]}} {} {}";

fn encoded(chain: &[u16]) -> VocabularyEncodedId {
    encoded_at(VocabularyRoot::Universal, chain)
}

fn encoded_at(root: VocabularyRoot, chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        root,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("fixture identity is non-empty")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FixtureCompleteNameTree;

#[derive(Clone, Debug)]
struct FixtureNameTreePlan;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct FixtureLogosNameTree(usize);

impl NativeNameTreeBoundary for FixtureCompleteNameTree {
    type LogosNameTree = FixtureLogosNameTree;
    type Plan = FixtureNameTreePlan;

    fn authenticated_plan(&self) -> Result<Self::Plan, NativeNameTreeRefusal> {
        Ok(FixtureNameTreePlan)
    }
}

impl NativeNameTreePlan for FixtureNameTreePlan {
    type LogosNameTree = FixtureLogosNameTree;

    fn realize_declaration(
        &self,
        source: &VocabularyEncodedId,
        transform: NameTransform,
    ) -> Result<VocabularyEncodedId, NativeNameTreeRefusal> {
        if transform == NameTransform::Identity {
            return Ok(source.clone());
        }
        let suffix = match transform {
            NameTransform::Identity => unreachable!(),
            NameTransform::FieldName => 1,
            NameTransform::Screaming => 2,
            NameTransform::PascalCase => 3,
        };
        let mut chain = source.chain().to_vec();
        chain.push(LocalEncodedId::new(suffix));
        VocabularyEncodedId::new(*source.root_variant(), chain).map_err(|_| {
            NativeNameTreeRefusal::CannotRealize {
                identity: source.clone(),
                transform,
            }
        })
    }

    fn logos_name_tree(
        &self,
        evaluated: &NativeEvaluatedLogos,
    ) -> Result<Self::LogosNameTree, NativeNameTreeRefusal> {
        Ok(FixtureLogosNameTree(evaluated.values().len()))
    }
}

impl NativeLogosNameTree for FixtureLogosNameTree {
    fn validate_for(&self, evaluated: &NativeEvaluatedLogos) -> Result<(), NativeNameTreeRefusal> {
        if self.0 == evaluated.values().len() {
            Ok(())
        } else {
            Err(NativeNameTreeRefusal::CannotConstructLogosTree)
        }
    }
}

fn collect_template_references(
    value: &TemplateValue<VocabularyRoot>,
    references: &mut Vec<VocabularyEncodedId>,
) {
    for field in value.fields() {
        match field.term() {
            TemplateTerm::Reference(identity) => references.push(identity.clone()),
            TemplateTerm::Nested(value) => collect_template_references(value, references),
            TemplateTerm::Sequence(items) => {
                for item in items {
                    collect_template_term_references(item, references);
                }
            }
            TemplateTerm::Declaration(_)
            | TemplateTerm::Literal(_)
            | TemplateTerm::Scalar(_)
            | TemplateTerm::Future(_) => {}
        }
    }
}

fn collect_template_term_references(
    term: &TemplateTerm<VocabularyRoot>,
    references: &mut Vec<VocabularyEncodedId>,
) {
    match term {
        TemplateTerm::Reference(identity) => references.push(identity.clone()),
        TemplateTerm::Nested(value) => collect_template_references(value, references),
        TemplateTerm::Sequence(items) => {
            for item in items {
                collect_template_term_references(item, references);
            }
        }
        TemplateTerm::Declaration(_)
        | TemplateTerm::Literal(_)
        | TemplateTerm::Scalar(_)
        | TemplateTerm::Future(_) => {}
    }
}

fn collect_native_references(
    value: &core_nomos::NativeEvaluatedValue,
    references: &mut Vec<VocabularyEncodedId>,
) {
    for field in value.fields() {
        collect_native_term_references(field.term(), references);
    }
}

fn collect_native_term_references(
    term: &NativeEvaluatedTerm,
    references: &mut Vec<VocabularyEncodedId>,
) {
    match term {
        NativeEvaluatedTerm::Reference(identity) => references.push(identity.clone()),
        NativeEvaluatedTerm::Nested(value) => collect_native_references(value, references),
        NativeEvaluatedTerm::Sequence(items) => {
            for item in items {
                collect_native_term_references(item, references);
            }
        }
        NativeEvaluatedTerm::Declaration(_)
        | NativeEvaluatedTerm::Literal(_)
        | NativeEvaluatedTerm::Scalar(_)
        | NativeEvaluatedTerm::TypeReference(_)
        | NativeEvaluatedTerm::Variant(_) => {}
    }
}

fn transformer_chain(spelling: &str) -> Option<&'static [u16]> {
    match spelling {
        "WireAttributes" => Some(&[110, 1]),
        "EnumerationAttributes" => Some(&[110, 2]),
        "WireNewtype" => Some(&[110, 3]),
        "ParticularStruct" => Some(&[110, 4]),
        "Enumeration" => Some(&[110, 5]),
        "ScopeOfStep" => Some(&[110, 6]),
        _ => None,
    }
}

fn binding_chain(transformer: &str, spelling: &str) -> Option<&'static [u16]> {
    match (transformer, spelling) {
        ("WireNewtype", "name") => Some(&[110, 3, 1]),
        ("WireNewtype", "type") => Some(&[110, 3, 2]),
        ("ParticularStruct", "name") => Some(&[110, 4, 1]),
        ("ParticularStruct", "fields") => Some(&[110, 4, 2]),
        ("Enumeration", "name") => Some(&[110, 5, 1]),
        ("Enumeration", "variants") => Some(&[110, 5, 2]),
        ("ScopeOfStep", "variant") => Some(&[110, 6, 1]),
        ("ScopeOfStep", "source") => Some(&[110, 6, 2]),
        ("ScopeOfStep", "children") => Some(&[110, 6, 3]),
        _ => None,
    }
}

struct RecursiveBindingFixture {
    spelling: &'static str,
    reference_count: usize,
}

fn logos() -> LogosLanguage {
    LogosLanguage::seal(
        LogosLanguageTypeIds {
            newtype: encoded(&[1]),
            structure: encoded(&[13]),
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
            field: encoded(&[14]),
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
    let structure = TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.struct_type())
        .expect("struct Template(Logos)");
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
            struct_body: encoded(&[100, 12]),
            enumeration_body: encoded(&[100, 10]),
            attributes_body: encoded(&[100, 11]),
        },
        TextualNomosWords {
            named: encoded(&[101, 1]),
            structural: encoded(&[101, 2]),
            recursive: encoded(&[101, 9]),
            newtype: encoded(&[101, 3]),
            structure: encoded(&[101, 8]),
            enumeration: encoded(&[101, 4]),
            realize: encoded(&[101, 5]),
            splice: encoded(&[101, 6]),
            invoke: encoded(&[101, 7]),
            insert_at: encoded(&[101, 10]),
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
            TextualNomosMetaType {
                word: encoded(&[102, 4]),
                meta: MetaType::Fields,
                output: output(field_shape(&structure, 4)),
            },
        ],
    )
    .expect("TextualNomos table seals")
}

#[test]
fn direct_typed_name_table_constructor_checks_identity_and_table_spelling_uniqueness() {
    let first = encoded(&[220, 1]);
    let second = encoded(&[220, 2]);
    let names = core_nomos::NomosNameTable::try_from_entries(vec![
        (second.clone(), "Second".into()),
        (first.clone(), "First".into()),
    ])
    .expect("typed direct NameTree");
    assert_eq!(
        names
            .entries()
            .map(|(identity, _)| identity)
            .collect::<Vec<_>>(),
        vec![&first, &second]
    );
    assert!(
        core_nomos::NomosNameTable::try_from_entries(vec![
            (first.clone(), "Same".into()),
            (second, "Same".into()),
        ])
        .is_err()
    );
    assert!(
        core_nomos::NomosNameTable::try_from_entries(vec![
            (first.clone(), "First".into()),
            (first, "Again".into()),
        ])
        .is_err()
    );
    assert!(
        core_nomos::NomosNameTable::try_from_entries(vec![(
            encoded_at(VocabularyRoot::Rust, &[220, 3]),
            "Rust".into(),
        )])
        .is_err()
    );
    assert!(
        core_nomos::NomosNameTable::try_from_entries(vec![(encoded(&[220, 4]), String::new(),)])
            .is_err()
    );
    assert!(
        core_nomos::NomosNameTable::try_from_entries(vec![
            (encoded(&[221, 1]), "Shared".into()),
            (encoded(&[222, 1]), "Shared".into()),
        ])
        .is_ok()
    );
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
        (encoded(&[101, 8]), "Struct"),
        (encoded(&[101, 4]), "Enumeration"),
        (encoded(&[101, 5]), "Realize"),
        (encoded(&[101, 6]), "Splice"),
        (encoded(&[101, 7]), "Invoke"),
        (encoded(&[101, 9]), "Recursive"),
        (encoded(&[101, 10]), "InsertAt"),
        (encoded(&[102, 1]), "Name"),
        (encoded(&[102, 2]), "Type"),
        (encoded(&[102, 3]), "Variants"),
        (encoded(&[102, 4]), "Fields"),
    ] {
        bindings.spelling(&identity, spelling);
    }

    for spelling in [
        "WireAttributes",
        "EnumerationAttributes",
        "WireNewtype",
        "ParticularStruct",
        "Enumeration",
        "ScopeOfStep",
    ] {
        if token_occurrences(source, spelling).is_empty() {
            continue;
        }
        let identity = encoded(transformer_chain(spelling).expect("fixture transformer crosswalk"));
        let occurrences = token_occurrences(source, spelling);
        let declaration = occurrences
            .iter()
            .position(|(_, end)| {
                [".Named", ".Structural", ".Recursive"]
                    .iter()
                    .any(|suffix| source[*end..].starts_with(suffix))
            })
            .expect("transformer declaration occurrence");
        bindings.declaration(source, spelling, declaration, identity.clone());
        for (occurrence, (start, _)) in occurrences.into_iter().enumerate() {
            if source[..start].ends_with("Invoke.") {
                bindings.reference(source, spelling, occurrence, identity.clone());
            }
        }
    }

    for (transformer, spelling, declaration, reference) in [
        ("WireNewtype", "name", 0, 1),
        ("ParticularStruct", "name", 2, 3),
        ("Enumeration", "name", 4, 5),
        ("WireNewtype", "type", 0, 1),
        ("ParticularStruct", "fields", 0, 1),
        ("Enumeration", "variants", 0, 1),
    ] {
        let identity =
            encoded(binding_chain(transformer, spelling).expect("fixture binding crosswalk"));
        bindings.declaration(source, spelling, declaration, identity.clone());
        bindings.reference(source, spelling, reference, identity);
    }

    if !token_occurrences(source, "ScopeOfStep").is_empty() {
        for fixture in [
            RecursiveBindingFixture {
                spelling: "variant",
                reference_count: 0,
            },
            RecursiveBindingFixture {
                spelling: "source",
                reference_count: 0,
            },
            RecursiveBindingFixture {
                spelling: "children",
                reference_count: 2,
            },
        ] {
            let identity =
                encoded(binding_chain("ScopeOfStep", fixture.spelling).expect("recursive binding"));
            bindings.declaration(source, fixture.spelling, 0, identity.clone());
            for occurrence in 1..=fixture.reference_count {
                bindings.reference(source, fixture.spelling, occurrence, identity.clone());
            }
        }
    }

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
        "Copy",
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
    for (start, _) in source.match_indices(carrier) {
        bindings.references.insert(
            (start, start + carrier.len()),
            ("nota-text".to_owned(), encoded(&[120, 3])),
        );
    }
    bindings
}

fn rebind_occurrence(
    assigned: &mut Bindings,
    source: &str,
    spelling: &str,
    occurrence: usize,
    identity: VocabularyEncodedId,
) {
    let bound = token_occurrences(source, spelling)[occurrence];
    let assignment = assigned
        .declarations
        .get_mut(&bound)
        .or_else(|| assigned.references.get_mut(&bound))
        .expect("fixture occurrence has a declaration or reference assignment");
    assignment.1 = identity;
}

fn swap_wire_newtype_binding_identities(assigned: &mut Bindings, source: &str) {
    for occurrence in 0..2 {
        rebind_occurrence(assigned, source, "name", occurrence, encoded(&[110, 3, 2]));
        rebind_occurrence(assigned, source, "type", occurrence, encoded(&[110, 3, 1]));
    }
    assigned.spelling(&encoded(&[110, 3, 2]), "name");
    assigned.spelling(&encoded(&[110, 3, 1]), "type");
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SplicePosition {
    StructFields,
    EnumerationVariants,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FixtureAdapterError {
    MissingName(String),
    Unsupported {
        transformer: String,
        path: String,
        value: String,
    },
    SpliceElement {
        transformer: String,
        path: String,
        expected: String,
        found: String,
    },
}

struct FixtureAdapter<'fixture> {
    legacy: &'fixture MacroPackage,
    bindings: &'fixture Bindings,
    newtype: TemplateLanguage<VocabularyRoot>,
    structure: TemplateLanguage<VocabularyRoot>,
    enumeration: TemplateLanguage<VocabularyRoot>,
    attributes: TemplateLanguage<VocabularyRoot>,
}

impl<'fixture> FixtureAdapter<'fixture> {
    fn new(
        legacy: &'fixture MacroPackage,
        bindings: &'fixture Bindings,
        logos: &LogosLanguage,
    ) -> Self {
        Self {
            legacy,
            bindings,
            newtype: TemplateLanguage::derive(
                logos.grammar(),
                logos.landing(),
                logos.newtype_type(),
            )
            .expect("newtype language"),
            structure: TemplateLanguage::derive(
                logos.grammar(),
                logos.landing(),
                logos.struct_type(),
            )
            .expect("struct language"),
            enumeration: TemplateLanguage::derive(
                logos.grammar(),
                logos.landing(),
                logos.enumeration_type(),
            )
            .expect("enumeration language"),
            attributes: TemplateLanguage::derive(
                logos.grammar(),
                logos.landing(),
                logos.attributes_type(),
            )
            .expect("attributes language"),
        }
    }

    fn authored_set(&self) -> Result<AuthoredTransformerSet, FixtureAdapterError> {
        let declarations = self
            .legacy
            .definitions()
            .macros
            .values()
            .map(|definition| self.declaration(definition))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AuthoredTransformerSet::try_new(declarations)
            .expect("the independent fixture has distinct, coherent transformers"))
    }

    fn declaration(
        &self,
        definition: &core_nomos::MacroDefinition,
    ) -> Result<AuthoredTransformerDeclaration, FixtureAdapterError> {
        let transformer = self.legacy_name(definition.name)?;
        let language = self.language(definition.kind);
        let parameters = definition
            .input
            .parameters
            .iter()
            .map(|parameter| {
                Ok(AuthoredInputParameter::new(
                    self.binding_identity(&transformer, parameter.binding)?,
                    parameter.meta,
                    self.parameter_output(definition.kind, parameter.meta),
                ))
            })
            .collect::<Result<Vec<_>, FixtureAdapterError>>()?;
        let input = AuthoredInputSignature::try_new(parameters)
            .expect("the independent fixture has distinct bindings");
        let result = self.template(&transformer, &definition.template)?;
        Ok(AuthoredTransformerDeclaration::try_new(
            self.transformer_identity(&transformer)?,
            definition.kind,
            input,
            result,
            language,
        )
        .expect("the exhaustive legacy conversion inhabits Template(Logos)"))
    }

    fn language(&self, kind: MacroKind) -> &TemplateLanguage<VocabularyRoot> {
        match kind {
            MacroKind::Named => &self.attributes,
            MacroKind::Structural(SectionDefault::Newtype) => &self.newtype,
            MacroKind::Structural(SectionDefault::Struct) => &self.structure,
            MacroKind::Structural(SectionDefault::Enumeration) => &self.enumeration,
            MacroKind::Recursive {
                section: SectionDefault::Enumeration,
            } => &self.attributes,
            MacroKind::Recursive { section } => {
                panic!("unsupported recursive fixture section {section:?}")
            }
        }
    }

    fn parameter_output(
        &self,
        kind: MacroKind,
        meta: MetaType,
    ) -> TemplateFutureOutput<VocabularyRoot> {
        let shape = match (kind, meta) {
            (MacroKind::Structural(SectionDefault::Newtype), MetaType::Name) => {
                field_shape(&self.newtype, 2)
            }
            (MacroKind::Structural(SectionDefault::Newtype), MetaType::Type) => {
                field_shape(&self.newtype, 4)
            }
            (MacroKind::Structural(SectionDefault::Struct), MetaType::Name) => {
                field_shape(&self.structure, 2)
            }
            (MacroKind::Structural(SectionDefault::Struct), MetaType::Fields) => {
                field_shape(&self.structure, 4)
            }
            (MacroKind::Structural(SectionDefault::Enumeration), MetaType::Name) => {
                field_shape(&self.enumeration, 2)
            }
            (MacroKind::Structural(SectionDefault::Enumeration), MetaType::Variants) => {
                field_shape(&self.enumeration, 4)
            }
            _ => panic!("legacy fixture input meta-type belongs to its structural section"),
        };
        output(shape)
    }

    fn template(
        &self,
        transformer: &str,
        template: &ResultTemplate,
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        match template {
            ResultTemplate::Attributes(sequence) => self.value(
                &self.attributes,
                4,
                1,
                vec![TemplateTerm::Sequence(self.attribute_sequence(
                    transformer,
                    "attributes",
                    sequence,
                )?)],
            ),
            ResultTemplate::Item(ItemTemplate::Newtype(value)) => self.value(
                &self.newtype,
                1,
                1,
                vec![
                    TemplateTerm::Nested(Box::new(self.visibility(
                        transformer,
                        "newtype.visibility",
                        value.visibility.clone(),
                    )?)),
                    TemplateTerm::Sequence(self.attribute_sequence(
                        transformer,
                        "newtype.attributes",
                        &value.attributes,
                    )?),
                    self.name_scalar(transformer, "newtype.name", &value.name)?,
                    TemplateTerm::Nested(Box::new(self.visibility(
                        transformer,
                        "newtype.wrapped_visibility",
                        Visibility::Private,
                    )?)),
                    self.type_scalar(transformer, "newtype.wrapped", &value.wrapped)?,
                ],
            ),
            ResultTemplate::Item(ItemTemplate::Struct(value)) => self.value(
                &self.structure,
                13,
                1,
                vec![
                    TemplateTerm::Nested(Box::new(self.visibility(
                        transformer,
                        "struct.visibility",
                        value.visibility.clone(),
                    )?)),
                    TemplateTerm::Sequence(self.attribute_sequence(
                        transformer,
                        "struct.attributes",
                        &value.attributes,
                    )?),
                    self.name_scalar(transformer, "struct.name", &value.name)?,
                    TemplateTerm::Nested(Box::new(self.generics(
                        transformer,
                        "struct.generics",
                        &value.generics,
                    )?)),
                    self.field_sequence(
                        transformer,
                        "struct.fields",
                        &value.fields,
                        SplicePosition::StructFields,
                    )?,
                ],
            ),
            ResultTemplate::Item(ItemTemplate::Enumeration(value)) => self.value(
                &self.enumeration,
                2,
                1,
                vec![
                    TemplateTerm::Nested(Box::new(self.visibility(
                        transformer,
                        "enumeration.visibility",
                        value.visibility.clone(),
                    )?)),
                    TemplateTerm::Sequence(self.attribute_sequence(
                        transformer,
                        "enumeration.attributes",
                        &value.attributes,
                    )?),
                    self.name_scalar(transformer, "enumeration.name", &value.name)?,
                    TemplateTerm::Nested(Box::new(self.generics(
                        transformer,
                        "enumeration.generics",
                        &value.generics,
                    )?)),
                    self.variant_sequence(
                        transformer,
                        "enumeration.variants",
                        &value.variants,
                        SplicePosition::EnumerationVariants,
                    )?,
                ],
            ),
        }
    }

    fn value(
        &self,
        language: &TemplateLanguage<VocabularyRoot>,
        type_local: u16,
        constructor_local: u16,
        terms: Vec<TemplateTerm<VocabularyRoot>>,
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        let encoded_type = EncodedTypeId::new(encoded(&[type_local]));
        let constructor = EncodedConstructorId::under(&encoded_type, constructor_local);
        let declaration =
            language
                .constructor(&constructor)
                .ok_or_else(|| FixtureAdapterError::Unsupported {
                    transformer: "<grammar>".to_owned(),
                    path: format!("type[{type_local}].constructor[{constructor_local}]"),
                    value: "missing from Template(Logos) closure".to_owned(),
                })?;
        if terms.len() != declaration.landing_fields().len() {
            return Err(FixtureAdapterError::Unsupported {
                transformer: "<grammar>".to_owned(),
                path: format!("type[{type_local}].constructor[{constructor_local}]"),
                value: format!(
                    "{} terms for {} landing fields",
                    terms.len(),
                    declaration.landing_fields().len()
                ),
            });
        }
        let fields = declaration
            .landing_fields()
            .iter()
            .zip(terms)
            .map(|(field, term)| TemplateFieldValue::new(field.role(), term))
            .collect();
        Ok(TemplateValue::try_new(constructor, fields).expect("landing roles are distinct"))
    }

    fn visibility(
        &self,
        transformer: &str,
        path: &str,
        visibility: Visibility,
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        let (constructor, spelling) = match visibility {
            Visibility::Public => (1, "Public"),
            Visibility::Private => (2, "Private"),
            other => {
                return Err(FixtureAdapterError::Unsupported {
                    transformer: transformer.to_owned(),
                    path: path.to_owned(),
                    value: format!("{other:?}"),
                });
            }
        };
        self.value(
            self.language_for_nested(3),
            3,
            constructor,
            vec![TemplateTerm::Literal(self.fixed_id(spelling)?)],
        )
    }

    fn attribute_sequence(
        &self,
        transformer: &str,
        path: &str,
        sequence: &Sequence<Attribute>,
    ) -> Result<Vec<TemplateTerm<VocabularyRoot>>, FixtureAdapterError> {
        sequence
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| match item {
                SequenceItem::Literal(attribute) => Ok(TemplateTerm::Nested(Box::new(
                    self.attribute(transformer, &format!("{path}[{index}]"), attribute)?,
                ))),
                SequenceItem::Escape(escape) => Ok(TemplateTerm::Future(self.future(
                    transformer,
                    &format!("{path}[{index}]"),
                    escape,
                    None,
                )?)),
            })
            .collect()
    }

    fn attribute(
        &self,
        transformer: &str,
        path: &str,
        attribute: &Attribute,
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        match attribute {
            Attribute::Derive(group) => self.value(
                self.language_for_nested(5),
                5,
                1,
                vec![TemplateTerm::Nested(Box::new(self.derive_group(
                    transformer,
                    &format!("{path}.derive"),
                    group,
                )?))],
            ),
            Attribute::Configuration(configuration) => {
                let Attribute::Derive(group) = configuration.inner.as_ref() else {
                    return Err(FixtureAdapterError::Unsupported {
                        transformer: transformer.to_owned(),
                        path: format!("{path}.inner"),
                        value: format!("{:?}", configuration.inner),
                    });
                };
                self.value(
                    self.language_for_nested(5),
                    5,
                    2,
                    vec![
                        TemplateTerm::Nested(Box::new(self.configuration_predicate(
                            transformer,
                            &format!("{path}.predicate"),
                            &configuration.predicate,
                        )?)),
                        TemplateTerm::Nested(Box::new(self.derive_group(
                            transformer,
                            &format!("{path}.inner.derive"),
                            group,
                        )?)),
                    ],
                )
            }
            Attribute::ToolPath(path_value) => {
                let (head, tail) = path_value.segments.split_first().ok_or_else(|| {
                    FixtureAdapterError::Unsupported {
                        transformer: transformer.to_owned(),
                        path: path.to_owned(),
                        value: "empty tool path".to_owned(),
                    }
                })?;
                self.value(
                    self.language_for_nested(5),
                    5,
                    3,
                    vec![
                        TemplateTerm::Reference(self.fixed_identifier(*head)?),
                        TemplateTerm::Nested(Box::new(self.path(
                            transformer,
                            &format!("{path}.tail"),
                            tail,
                        )?)),
                    ],
                )
            }
            other => Err(FixtureAdapterError::Unsupported {
                transformer: transformer.to_owned(),
                path: path.to_owned(),
                value: format!("{other:?}"),
            }),
        }
    }

    fn configuration_predicate(
        &self,
        transformer: &str,
        path: &str,
        predicate: &ConfigurationPredicate,
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        match predicate {
            ConfigurationPredicate::Feature(feature) => self.value(
                self.language_for_nested(7),
                7,
                1,
                vec![TemplateTerm::Reference(self.fixed_identifier(*feature)?)],
            ),
        }
        .map_err(|error| self.at(error, transformer, path))
    }

    fn derive_group(
        &self,
        transformer: &str,
        path: &str,
        group: &DeriveGroup,
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        self.value(
            self.language_for_nested(8),
            8,
            1,
            vec![TemplateTerm::Sequence(
                group
                    .paths
                    .iter()
                    .enumerate()
                    .map(|(index, path_value)| {
                        Ok(TemplateTerm::Nested(Box::new(self.path(
                            transformer,
                            &format!("{path}[{index}]"),
                            &path_value.segments,
                        )?)))
                    })
                    .collect::<Result<Vec<_>, FixtureAdapterError>>()?,
            )],
        )
    }

    fn path(
        &self,
        transformer: &str,
        path: &str,
        segments: &[name_table::Identifier],
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        let (head, tail) =
            segments
                .split_first()
                .ok_or_else(|| FixtureAdapterError::Unsupported {
                    transformer: transformer.to_owned(),
                    path: path.to_owned(),
                    value: "empty path".to_owned(),
                })?;
        if tail.is_empty() {
            self.value(
                self.language_for_nested(6),
                6,
                1,
                vec![TemplateTerm::Reference(self.fixed_identifier(*head)?)],
            )
        } else {
            self.value(
                self.language_for_nested(6),
                6,
                2,
                vec![
                    TemplateTerm::Reference(self.fixed_identifier(*head)?),
                    TemplateTerm::Nested(Box::new(self.path(
                        transformer,
                        &format!("{path}.tail"),
                        tail,
                    )?)),
                ],
            )
        }
    }

    fn generics(
        &self,
        transformer: &str,
        path: &str,
        generics: &Generics,
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        if !generics.parameters.is_empty() {
            return Err(FixtureAdapterError::Unsupported {
                transformer: transformer.to_owned(),
                path: path.to_owned(),
                value: format!("{:?}", generics.parameters),
            });
        }
        self.value(
            self.language_for_nested(9),
            9,
            1,
            vec![TemplateTerm::Sequence(Vec::new())],
        )
    }

    fn name_scalar(
        &self,
        transformer: &str,
        path: &str,
        scalar: &Scalar<name_table::Identifier>,
    ) -> Result<TemplateTerm<VocabularyRoot>, FixtureAdapterError> {
        match scalar {
            Scalar::Literal(identifier) => Ok(TemplateTerm::Declaration(
                self.fixed_identifier(*identifier)?,
            )),
            Scalar::Escape(escape) => Ok(TemplateTerm::Future(self.future(
                transformer,
                path,
                escape,
                None,
            )?)),
        }
    }

    fn type_scalar(
        &self,
        transformer: &str,
        path: &str,
        scalar: &Scalar<core_logos::TypeReference>,
    ) -> Result<TemplateTerm<VocabularyRoot>, FixtureAdapterError> {
        match scalar {
            Scalar::Literal(reference) => Ok(TemplateTerm::Nested(Box::new(self.type_reference(
                transformer,
                path,
                reference,
            )?))),
            Scalar::Escape(escape) => Ok(TemplateTerm::Future(self.future(
                transformer,
                path,
                escape,
                None,
            )?)),
        }
    }

    fn type_reference(
        &self,
        transformer: &str,
        path: &str,
        reference: &core_logos::TypeReference,
    ) -> Result<TemplateValue<VocabularyRoot>, FixtureAdapterError> {
        match reference {
            core_logos::TypeReference::Path(path_value) => self.value(
                self.language_for_nested(11),
                11,
                1,
                vec![TemplateTerm::Nested(Box::new(self.path(
                    transformer,
                    path,
                    &path_value.segments,
                )?))],
            ),
            other => Err(FixtureAdapterError::Unsupported {
                transformer: transformer.to_owned(),
                path: path.to_owned(),
                value: format!("{other:?}"),
            }),
        }
    }

    fn field_sequence(
        &self,
        transformer: &str,
        path: &str,
        sequence: &Sequence<core_logos::Field>,
        position: SplicePosition,
    ) -> Result<TemplateTerm<VocabularyRoot>, FixtureAdapterError> {
        let items = sequence
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| match item {
                SequenceItem::Literal(field) => Ok(TemplateTerm::Nested(Box::new(self.value(
                    &self.structure,
                    14,
                    1,
                    vec![
                        TemplateTerm::Nested(Box::new(self.visibility(
                            transformer,
                            &format!("{path}[{index}].visibility"),
                            field.visibility.clone(),
                        )?)),
                        TemplateTerm::Declaration(self.fixed_identifier(field.name)?),
                        TemplateTerm::Nested(Box::new(self.type_reference(
                            transformer,
                            &format!("{path}[{index}].type"),
                            &field.type_reference,
                        )?)),
                    ],
                )?))),
                SequenceItem::Escape(escape) => Ok(TemplateTerm::Future(self.future(
                    transformer,
                    &format!("{path}[{index}]"),
                    escape,
                    Some(position),
                )?)),
            })
            .collect::<Result<Vec<_>, FixtureAdapterError>>()?;
        Ok(TemplateTerm::Sequence(items))
    }

    fn variant_sequence(
        &self,
        transformer: &str,
        path: &str,
        sequence: &Sequence<core_logos::Variant>,
        position: SplicePosition,
    ) -> Result<TemplateTerm<VocabularyRoot>, FixtureAdapterError> {
        let items = sequence
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| match item {
                SequenceItem::Literal(variant) => {
                    if !matches!(variant.payload, core_logos::VariantPayload::Unit) {
                        return Err(FixtureAdapterError::Unsupported {
                            transformer: transformer.to_owned(),
                            path: format!("{path}[{index}].payload"),
                            value: format!("{:?}", variant.payload),
                        });
                    }
                    Ok(TemplateTerm::Nested(Box::new(self.value(
                        &self.enumeration,
                        12,
                        1,
                        vec![TemplateTerm::Declaration(
                            self.fixed_identifier(variant.name)?,
                        )],
                    )?)))
                }
                SequenceItem::Escape(escape) => Ok(TemplateTerm::Future(self.future(
                    transformer,
                    &format!("{path}[{index}]"),
                    escape,
                    Some(position),
                )?)),
            })
            .collect::<Result<Vec<_>, FixtureAdapterError>>()?;
        Ok(TemplateTerm::Sequence(items))
    }

    fn future(
        &self,
        transformer: &str,
        path: &str,
        escape: &Escape,
        splice_position: Option<SplicePosition>,
    ) -> Result<TemplateFuture, FixtureAdapterError> {
        match escape {
            Escape::Realize(realize) => Ok(TemplateFuture::Realize {
                binding: self.binding_ref(transformer, realize.binding)?,
                transform: realize.transform,
            }),
            Escape::Invoke(target) => {
                let definition = self.legacy.definition(*target).ok_or_else(|| {
                    FixtureAdapterError::Unsupported {
                        transformer: transformer.to_owned(),
                        path: path.to_owned(),
                        value: format!("missing Invoke target {target:?}"),
                    }
                })?;
                Ok(TemplateFuture::Invoke(self.transformer_identity(
                    &self.legacy_name(definition.name)?,
                )?))
            }
            Escape::Splice(splice) => {
                let expected = match splice_position {
                    Some(SplicePosition::StructFields) => SpliceElement::Field {
                        visibility: Visibility::Public,
                        name_rule: FieldNameRule::FieldRuleDispatch,
                    },
                    Some(SplicePosition::EnumerationVariants) => SpliceElement::Variant,
                    None => {
                        return Err(FixtureAdapterError::Unsupported {
                            transformer: transformer.to_owned(),
                            path: path.to_owned(),
                            value: "Splice in a non-sequence fixture position".to_owned(),
                        });
                    }
                };
                if splice.element != expected {
                    return Err(FixtureAdapterError::SpliceElement {
                        transformer: transformer.to_owned(),
                        path: path.to_owned(),
                        expected: format!("{expected:?}"),
                        found: format!("{:?}", splice.element),
                    });
                }
                Ok(TemplateFuture::Splice {
                    binding: self.binding_ref(transformer, splice.binding)?,
                })
            }
        }
    }

    fn binding_ref(
        &self,
        transformer: &str,
        binding: BindingRef,
    ) -> Result<AuthoredBindingIdentity, FixtureAdapterError> {
        match binding {
            BindingRef::Input(identifier) => self.binding_identity(transformer, identifier),
        }
    }

    fn transformer_identity(
        &self,
        spelling: &str,
    ) -> Result<AuthoredTransformerIdentity, FixtureAdapterError> {
        let chain = transformer_chain(spelling)
            .ok_or_else(|| FixtureAdapterError::MissingName(spelling.to_owned()))?;
        Ok(AuthoredTransformerIdentity::try_new(encoded(chain)).expect("Universal fixture chain"))
    }

    fn binding_identity(
        &self,
        transformer: &str,
        identifier: name_table::Identifier,
    ) -> Result<AuthoredBindingIdentity, FixtureAdapterError> {
        let spelling = self.legacy_name(identifier)?;
        let chain = binding_chain(transformer, &spelling)
            .ok_or_else(|| FixtureAdapterError::MissingName(format!("{transformer}.{spelling}")))?;
        Ok(AuthoredBindingIdentity::try_new(encoded(chain)).expect("Universal fixture chain"))
    }

    fn fixed_identifier(
        &self,
        identifier: name_table::Identifier,
    ) -> Result<VocabularyEncodedId, FixtureAdapterError> {
        self.fixed_id(&self.legacy_name(identifier)?)
    }

    fn fixed_id(&self, spelling: &str) -> Result<VocabularyEncodedId, FixtureAdapterError> {
        self.bindings
            .spellings
            .iter()
            .find(|(_, candidate)| candidate.as_str() == spelling)
            .map(|(encoded_id, _)| encoded_id.clone())
            .ok_or_else(|| FixtureAdapterError::MissingName(spelling.to_owned()))
    }

    fn legacy_name(
        &self,
        identifier: name_table::Identifier,
    ) -> Result<String, FixtureAdapterError> {
        self.legacy
            .names()
            .resolve(identifier)
            .map(|name| name.as_str().to_owned())
            .map_err(|_| FixtureAdapterError::MissingName(format!("{identifier:?}")))
    }

    fn language_for_nested(&self, type_local: u16) -> &TemplateLanguage<VocabularyRoot> {
        for language in [
            &self.newtype,
            &self.structure,
            &self.enumeration,
            &self.attributes,
        ] {
            if language
                .type_declaration(&EncodedTypeId::new(encoded(&[type_local])))
                .is_some()
            {
                return language;
            }
        }
        panic!("nested Logos type belongs to every required closure")
    }

    fn at(&self, error: FixtureAdapterError, transformer: &str, path: &str) -> FixtureAdapterError {
        match error {
            FixtureAdapterError::Unsupported { value, .. } => FixtureAdapterError::Unsupported {
                transformer: transformer.to_owned(),
                path: path.to_owned(),
                value,
            },
            other => other,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EquivalenceMismatch {
    transformer: String,
    dimension: &'static str,
    path: Vec<String>,
    expected: String,
    found: String,
}

fn mismatch(
    transformer: &str,
    dimension: &'static str,
    path: &[String],
    expected: impl std::fmt::Debug,
    found: impl std::fmt::Debug,
) -> EquivalenceMismatch {
    EquivalenceMismatch {
        transformer: transformer.to_owned(),
        dimension,
        path: path.to_vec(),
        expected: format!("{expected:?}"),
        found: format!("{found:?}"),
    }
}

fn compare_sets(
    expected: &AuthoredTransformerSet,
    found: &AuthoredTransformerSet,
    names: &Bindings,
) -> Result<(), EquivalenceMismatch> {
    if expected.declarations().len() != found.declarations().len() {
        return Err(mismatch(
            "<package>",
            "transformer-count",
            &[],
            expected.declarations().len(),
            found.declarations().len(),
        ));
    }
    for (index, (expected, found)) in expected
        .declarations()
        .iter()
        .zip(found.declarations())
        .enumerate()
    {
        let transformer = names
            .spellings
            .get(expected.name().encoded_id())
            .map(Name::as_str)
            .unwrap_or("<unknown>");
        let mut path = vec![format!("declaration[{index}]")];
        if expected.name() != found.name() {
            return Err(mismatch(
                transformer,
                "transformer-identity",
                &path,
                expected.name(),
                found.name(),
            ));
        }
        if expected.kind() != found.kind() {
            return Err(mismatch(
                transformer,
                "macro-kind",
                &path,
                expected.kind(),
                found.kind(),
            ));
        }
        let expected_parameters = expected.input().parameters();
        let found_parameters = found.input().parameters();
        if expected_parameters.len() != found_parameters.len() {
            return Err(mismatch(
                transformer,
                "parameter-count",
                &path,
                expected_parameters.len(),
                found_parameters.len(),
            ));
        }
        for (parameter_index, (expected, found)) in
            expected_parameters.iter().zip(found_parameters).enumerate()
        {
            path.push(format!("input[{parameter_index}]"));
            if expected.binding() != found.binding() {
                return Err(mismatch(
                    transformer,
                    "parameter-binding",
                    &path,
                    expected.binding(),
                    found.binding(),
                ));
            }
            if expected.meta() != found.meta() {
                return Err(mismatch(
                    transformer,
                    "parameter-meta",
                    &path,
                    expected.meta(),
                    found.meta(),
                ));
            }
            if expected.output() != found.output() {
                return Err(mismatch(
                    transformer,
                    "parameter-output",
                    &path,
                    expected.output(),
                    found.output(),
                ));
            }
            path.pop();
        }
        if expected.output() != found.output() {
            return Err(mismatch(
                transformer,
                "fragment-kind",
                &path,
                expected.output(),
                found.output(),
            ));
        }
        path.push("result".to_owned());
        compare_value(transformer, &path, expected.result(), found.result())?;
        path.pop();
        if expected.future_requirements() != found.future_requirements() {
            return Err(mismatch(
                transformer,
                "invoke-requirements",
                &path,
                expected.future_requirements(),
                found.future_requirements(),
            ));
        }
    }
    Ok(())
}

fn compare_value(
    transformer: &str,
    path: &[String],
    expected: &TemplateValue<VocabularyRoot>,
    found: &TemplateValue<VocabularyRoot>,
) -> Result<(), EquivalenceMismatch> {
    if expected.constructor() != found.constructor() {
        return Err(mismatch(
            transformer,
            "constructor",
            path,
            expected.constructor(),
            found.constructor(),
        ));
    }
    if expected.fields().len() != found.fields().len() {
        return Err(mismatch(
            transformer,
            "role-count",
            path,
            expected.fields().len(),
            found.fields().len(),
        ));
    }
    for (index, (expected, found)) in expected.fields().iter().zip(found.fields()).enumerate() {
        let mut nested = path.to_vec();
        nested.push(format!("field[{index}]::{:?}", expected.role()));
        if expected.role() != found.role() {
            return Err(mismatch(
                transformer,
                "stable-role",
                &nested,
                expected.role(),
                found.role(),
            ));
        }
        compare_term(transformer, &nested, expected.term(), found.term())?;
    }
    Ok(())
}

fn compare_term(
    transformer: &str,
    path: &[String],
    expected: &TemplateTerm<VocabularyRoot>,
    found: &TemplateTerm<VocabularyRoot>,
) -> Result<(), EquivalenceMismatch> {
    match (expected, found) {
        (TemplateTerm::Nested(expected), TemplateTerm::Nested(found)) => {
            compare_value(transformer, path, expected, found)
        }
        (TemplateTerm::Sequence(expected), TemplateTerm::Sequence(found)) => {
            if expected.len() != found.len() {
                return Err(mismatch(
                    transformer,
                    "sequence-length",
                    path,
                    expected.len(),
                    found.len(),
                ));
            }
            for (index, (expected, found)) in expected.iter().zip(found).enumerate() {
                let mut nested = path.to_vec();
                nested.push(format!("item[{index}]"));
                compare_term(transformer, &nested, expected, found)?;
            }
            Ok(())
        }
        (
            TemplateTerm::Future(TemplateFuture::Realize {
                binding: expected_binding,
                transform: expected_transform,
            }),
            TemplateTerm::Future(TemplateFuture::Realize {
                binding: found_binding,
                transform: found_transform,
            }),
        ) if expected_binding == found_binding && expected_transform != found_transform => {
            Err(mismatch(
                transformer,
                "name-transform",
                path,
                expected_transform,
                found_transform,
            ))
        }
        (
            TemplateTerm::Future(TemplateFuture::Invoke(expected)),
            TemplateTerm::Future(TemplateFuture::Invoke(found)),
        ) if expected != found => Err(mismatch(
            transformer,
            "invoke-target",
            path,
            expected,
            found,
        )),
        (TemplateTerm::Scalar(structural_codec::ScalarValue::Text(expected)), _)
        | (_, TemplateTerm::Scalar(structural_codec::ScalarValue::Text(expected))) => {
            Err(mismatch(
                transformer,
                "text-scalar-refusal",
                path,
                "<no text scalar in encoded fixture form>",
                expected,
            ))
        }
        _ if expected == found => Ok(()),
        _ => Err(mismatch(transformer, "term", path, expected, found)),
    }
}

fn compare_selection(
    expected: &[GenerationClass],
    found: &[GenerationClass],
) -> Result<(), EquivalenceMismatch> {
    if expected.len() != found.len() {
        return Err(mismatch(
            "<selection>",
            "generation-class-count",
            &[],
            expected.len(),
            found.len(),
        ));
    }
    for (index, (expected, found)) in expected.iter().zip(found).enumerate() {
        if expected != found {
            return Err(mismatch(
                "<selection>",
                "generation-class-order",
                &[format!("selection[{index}]")],
                expected,
                found,
            ));
        }
    }
    Ok(())
}

fn structurally_proven_legacy_evaluator_sanity(
    authored: &AuthoredTransformerSet,
    legacy: &MacroPackage,
    assigned: &Bindings,
    logos: &LogosLanguage,
) -> Result<MacroPackage, EquivalenceMismatch> {
    let expected = FixtureAdapter::new(legacy, assigned, logos)
        .authored_set()
        .expect("the fixture adapter is exhaustive for the wire package");
    compare_sets(&expected, authored, assigned)?;
    // This is deliberately not an independent authored evaluator. Exact reversible
    // structural proof admits the retained legacy fixture as a test-only executor.
    Ok(legacy.clone())
}

fn shared_lowering_corpus() -> (EncodedEthos, NameTable) {
    let mut names = NameTable::new(IdentifierNamespace::Schema);
    let mut intern = |spelling: &str| {
        names
            .intern(LegacyName::new(spelling))
            .expect("shared corpus name")
    };
    let count = intern("Count");
    let record = intern("Record");
    let value = intern("value");
    let input = intern("Input");
    let set = intern("Set");
    let observe = intern("Observe");
    let declarations = vec![
        EncodedDeclaration::public(EncodedType::Newtype(EncodedNewtype::new(
            count,
            EncodedReference::Integer,
        ))),
        EncodedDeclaration::public(EncodedType::Struct(EncodedStruct::new(
            record,
            vec![EncodedField::new(value, EncodedReference::Plain(count))],
        ))),
        EncodedDeclaration::public(EncodedType::Enumeration(EncodedEnum::new(
            input,
            vec![
                EncodedVariant::new(set, Some(EncodedReference::Plain(record))),
                EncodedVariant::new(observe, None),
            ],
        ))),
    ];
    (EncodedEthos::new(declarations), names)
}

fn replace_declaration(
    original: &AuthoredTransformerSet,
    index: usize,
    replacement: AuthoredTransformerDeclaration,
) -> AuthoredTransformerSet {
    let mut declarations = original.declarations().to_vec();
    declarations[index] = replacement;
    AuthoredTransformerSet::try_new(declarations).expect("isolated mutation remains coherent")
}

fn mutate_first_realize_transform(
    value: &TemplateValue<VocabularyRoot>,
    replacement: NameTransform,
) -> TemplateValue<VocabularyRoot> {
    fn term(
        source: &TemplateTerm<VocabularyRoot>,
        replacement: NameTransform,
        changed: &mut bool,
    ) -> TemplateTerm<VocabularyRoot> {
        match source {
            TemplateTerm::Nested(value) => {
                TemplateTerm::Nested(Box::new(value_with_transform(value, replacement, changed)))
            }
            TemplateTerm::Sequence(values) => TemplateTerm::Sequence(
                values
                    .iter()
                    .map(|value| term(value, replacement, changed))
                    .collect(),
            ),
            TemplateTerm::Future(TemplateFuture::Realize { binding, .. }) if !*changed => {
                *changed = true;
                TemplateTerm::Future(TemplateFuture::Realize {
                    binding: binding.clone(),
                    transform: replacement,
                })
            }
            other => other.clone(),
        }
    }

    fn value_with_transform(
        value: &TemplateValue<VocabularyRoot>,
        replacement: NameTransform,
        changed: &mut bool,
    ) -> TemplateValue<VocabularyRoot> {
        TemplateValue::try_new(
            value.constructor().clone(),
            value
                .fields()
                .iter()
                .map(|field| {
                    TemplateFieldValue::new(field.role(), term(field.term(), replacement, changed))
                })
                .collect(),
        )
        .expect("role-preserving test mutation")
    }

    let mut changed = false;
    let mutated = value_with_transform(value, replacement, &mut changed);
    assert!(
        changed,
        "fixture contains one realized name before its type"
    );
    mutated
}

#[cfg(any())] // Superseded base-door fixture uses the retired structural form.
#[test]
fn standard_text_is_structurally_equivalent_to_all_five_legacy_transformers() {
    let logos = logos();
    let textual = textual(&logos);
    let first_bindings = bindings(SOURCE);
    let decoded = textual
        .decode(SOURCE, &first_bindings)
        .expect("typed base-door decode");
    assert_eq!(decoded.revision(), 1);
    assert_eq!(decoded.transformers().declarations().len(), 5);

    let production = MacroPackage::wire_fixture().expect("independent production fixture");
    assert_eq!(decoded.revision(), i64::from(production.revision().0));
    let expected = FixtureAdapter::new(&production, &first_bindings, &logos)
        .authored_set()
        .expect("the legacy fixture has a lossless authored representation");
    compare_sets(&expected, decoded.transformers(), &first_bindings)
        .expect("every structural dimension must match");
    assert_eq!(&expected, decoded.transformers());

    for (section, expected_transformer) in [
        (SectionDefault::Newtype, encoded(&[110, 3])),
        (SectionDefault::Struct, encoded(&[110, 4])),
        (SectionDefault::Enumeration, encoded(&[110, 5])),
    ] {
        let legacy = production
            .structural_default(section)
            .and_then(|identity| production.definition(identity))
            .expect("wire fixture has every structural default");
        let spelling = production
            .names()
            .resolve(legacy.name)
            .expect("fixture name resolves")
            .as_str();
        let authored = decoded
            .transformers()
            .declarations()
            .iter()
            .find(|declaration| declaration.name().encoded_id() == &expected_transformer)
            .expect("authored structural default exists");
        assert_eq!(
            authored.kind(),
            MacroKind::Structural(section),
            "{spelling}"
        );
    }

    let name_bindings = [
        encoded(&[110, 3, 1]),
        encoded(&[110, 4, 1]),
        encoded(&[110, 5, 1]),
    ];
    assert_eq!(
        name_bindings
            .iter()
            .map(|identity| identity.chain().last().copied())
            .collect::<Vec<_>>(),
        vec![Some(LocalEncodedId::new(1)); 3]
    );
    assert_eq!(
        name_bindings
            .iter()
            .map(|identity| identity.chain()[..2].to_vec())
            .collect::<Vec<_>>(),
        vec![
            vec![LocalEncodedId::new(110), LocalEncodedId::new(3)],
            vec![LocalEncodedId::new(110), LocalEncodedId::new(4)],
            vec![LocalEncodedId::new(110), LocalEncodedId::new(5)],
        ]
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

// Retired: this fixture relies on the superseded unsectioned WholeEthos and
// legacy per-section evaluator semantics.
#[cfg(any())]
#[test]
fn recursive_text_round_trips_and_evaluates_children_before_the_parent() {
    let logos = logos();
    let textual = textual(&logos);
    let assigned = bindings(RECURSIVE_SOURCE);
    let decoded = textual
        .decode(RECURSIVE_SOURCE, &assigned)
        .expect("recursive and InsertAt syntax decode");
    assert_eq!(decoded.transformers().declarations().len(), 6);
    let recursive = decoded
        .transformers()
        .declarations()
        .iter()
        .find(|declaration| declaration.name().encoded_id() == &encoded(&[110, 6]))
        .expect("recursive declaration");
    assert_eq!(
        recursive.kind(),
        MacroKind::Recursive {
            section: SectionDefault::Enumeration
        }
    );

    let viewed = textual
        .view(&decoded, &assigned)
        .expect("canonical recursive view");
    assert_eq!(viewed, RECURSIVE_CANONICAL);
    let restored = textual
        .decode(&viewed, &bindings(&viewed))
        .expect("canonical recursive view decodes");
    assert_eq!(restored.transformers(), decoded.transformers());

    let grandchild = WholeEthosEnumeration::new(
        encoded(&[310, 3]),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        vec![WholeEthosVariant::new(
            encoded(&[310, 3, 1]),
            WholeEthosAttributes::empty(),
            WholeEthosVariantPayload::Unit,
        )],
    );
    let child = WholeEthosEnumeration::new(
        encoded(&[310, 2]),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        vec![
            WholeEthosVariant::new(
                encoded(&[310, 2, 1]),
                WholeEthosAttributes::empty(),
                WholeEthosVariantPayload::Unit,
            ),
            WholeEthosVariant::new(
                encoded(&[310, 2, 2]),
                WholeEthosAttributes::empty(),
                WholeEthosVariantPayload::Tuple(
                    WholeEthosTupleFields::new(vec![WholeEthosTypeReference::Identity(
                        grandchild.name().clone(),
                    )])
                    .expect("one grandchild edge"),
                ),
            ),
        ],
    );
    let leaf_newtype = WholeEthosNewtype::new(
        encoded(&[310, 4]),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        WholeEthosWrappedField::new(
            WholeEthosVisibility::Private,
            WholeEthosTypeReference::Identity(encoded(&[399, 2])),
        ),
    );
    let root = WholeEthosEnumeration::new(
        encoded(&[310, 1]),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        vec![
            WholeEthosVariant::new(
                encoded(&[310, 1, 1]),
                WholeEthosAttributes::empty(),
                WholeEthosVariantPayload::Tuple(
                    WholeEthosTupleFields::new(vec![
                        WholeEthosTypeReference::Identity(child.name().clone()),
                        WholeEthosTypeReference::Identity(leaf_newtype.name().clone()),
                        WholeEthosTypeReference::Identity(encoded(&[399, 1])),
                    ])
                    .expect("child edge, newtype leaf, and absent leaf"),
                ),
            ),
            WholeEthosVariant::new(
                encoded(&[310, 1, 2]),
                WholeEthosAttributes::empty(),
                WholeEthosVariantPayload::Unit,
            ),
        ],
    );
    let input = EncodedPopulation::new(
        WholeEthos::new(vec![
            WholeEthosItem::Enumeration(root),
            WholeEthosItem::Enumeration(child),
            WholeEthosItem::Enumeration(grandchild),
            WholeEthosItem::Newtype(leaf_newtype),
        ]),
        FixtureCompleteNameTree,
    );
    let evaluator = NativeAuthoredEvaluator::try_new(
        decoded.transformers(),
        NativeReferenceUniverse::identity(),
    )
    .expect("recursive package admission");
    let transformed = evaluator.transform(&input).expect("recursive transform");
    let NativeEvaluatedTerm::Sequence(attributes) =
        transformed.population().encoded_form().values()[0].fields()[1].term()
    else {
        panic!("enumeration attributes are a sequence")
    };
    let constructors = attributes
        .iter()
        .map(|attribute| {
            let NativeEvaluatedTerm::Nested(attribute) = attribute else {
                panic!("recursive attributes contain only attribute values")
            };
            attribute.constructor().local()
        })
        .collect::<Vec<_>>();
    assert_eq!(constructors, vec![3, 3, 1, 3, 3, 1, 1, 1, 3, 1]);

    let empty_root = EncodedPopulation::new(
        WholeEthos::new(vec![WholeEthosItem::Enumeration(
            WholeEthosEnumeration::new(
                encoded(&[310, 5]),
                WholeEthosVisibility::Private,
                WholeEthosAttributes::empty(),
                Vec::new(),
            ),
        )]),
        FixtureCompleteNameTree,
    );
    let empty = evaluator
        .transform(&empty_root)
        .expect("zero-variant recursive root");
    assert!(matches!(
        empty.population().encoded_form().values()[0].fields()[1].term(),
        NativeEvaluatedTerm::Sequence(items) if items.is_empty()
    ));
}

// Retired with the same legacy whole-document evaluator carrier.
#[cfg(any())]
#[test]
fn recursive_preflight_refuses_applications_sharing_and_cycles() {
    let logos = logos();
    let textual = textual(&logos);
    let assigned = bindings(RECURSIVE_SOURCE);
    let decoded = textual
        .decode(RECURSIVE_SOURCE, &assigned)
        .expect("recursive package");
    let evaluator = NativeAuthoredEvaluator::try_new(
        decoded.transformers(),
        NativeReferenceUniverse::identity(),
    )
    .expect("recursive package admission");

    let application_source = EncodedPopulation::new(
        WholeEthos::new(vec![WholeEthosItem::Enumeration(
            WholeEthosEnumeration::new(
                encoded(&[320, 1]),
                WholeEthosVisibility::Private,
                WholeEthosAttributes::empty(),
                vec![WholeEthosVariant::new(
                    encoded(&[320, 1, 1]),
                    WholeEthosAttributes::empty(),
                    WholeEthosVariantPayload::Tuple(
                        WholeEthosTupleFields::new(vec![WholeEthosTypeReference::Application(
                            WholeEthosTypeApplication::new(
                                encoded(&[399, 10]),
                                WholeEthosTypeReference::Identity(encoded(&[399, 11])),
                            ),
                        )])
                        .expect("one application edge"),
                    ),
                )],
            ),
        )]),
        FixtureCompleteNameTree,
    );
    assert!(matches!(
        evaluator.transform(&application_source),
        Err(core_nomos::NativeEvaluationError::RecursiveSourceApplication { .. })
    ));

    let shared_child = WholeEthosEnumeration::new(
        encoded(&[321, 2]),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        vec![WholeEthosVariant::new(
            encoded(&[321, 2, 1]),
            WholeEthosAttributes::empty(),
            WholeEthosVariantPayload::Unit,
        )],
    );
    let shared_parent = WholeEthosEnumeration::new(
        encoded(&[321, 1]),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        vec![WholeEthosVariant::new(
            encoded(&[321, 1, 1]),
            WholeEthosAttributes::empty(),
            WholeEthosVariantPayload::Tuple(
                WholeEthosTupleFields::new(vec![
                    WholeEthosTypeReference::Identity(shared_child.name().clone()),
                    WholeEthosTypeReference::Identity(shared_child.name().clone()),
                ])
                .expect("two edges to the same child"),
            ),
        )],
    );
    let sharing_source = EncodedPopulation::new(
        WholeEthos::new(vec![
            WholeEthosItem::Enumeration(shared_parent),
            WholeEthosItem::Enumeration(shared_child),
        ]),
        FixtureCompleteNameTree,
    );
    assert!(matches!(
        evaluator.transform(&sharing_source),
        Err(core_nomos::NativeEvaluationError::RecursiveSourceSharing { .. })
    ));

    let first_identity = encoded(&[322, 1]);
    let second_identity = encoded(&[322, 2]);
    let first = WholeEthosEnumeration::new(
        first_identity.clone(),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        vec![WholeEthosVariant::new(
            encoded(&[322, 1, 1]),
            WholeEthosAttributes::empty(),
            WholeEthosVariantPayload::Tuple(
                WholeEthosTupleFields::new(vec![WholeEthosTypeReference::Identity(
                    second_identity.clone(),
                )])
                .expect("first cycle edge"),
            ),
        )],
    );
    let second = WholeEthosEnumeration::new(
        second_identity,
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        vec![WholeEthosVariant::new(
            encoded(&[322, 2, 1]),
            WholeEthosAttributes::empty(),
            WholeEthosVariantPayload::Tuple(
                WholeEthosTupleFields::new(vec![WholeEthosTypeReference::Identity(first_identity)])
                    .expect("second cycle edge"),
            ),
        )],
    );
    let cycle_source = EncodedPopulation::new(
        WholeEthos::new(vec![
            WholeEthosItem::Enumeration(first),
            WholeEthosItem::Enumeration(second),
        ]),
        FixtureCompleteNameTree,
    );
    assert!(matches!(
        evaluator.transform(&cycle_source),
        Err(core_nomos::NativeEvaluationError::RecursiveSourceCycle { .. })
    ));
}

// Retired: it asserts the removed legacy evaluator is a valid shared corpus.
#[cfg(any())]
#[test]
fn structural_proof_admits_a_legacy_evaluator_sanity_on_a_shared_corpus() {
    let logos = logos();
    let assigned = bindings(SOURCE);
    let decoded = textual(&logos)
        .decode(SOURCE, &assigned)
        .expect("typed authored fixture");
    let legacy = MacroPackage::wire_fixture().expect("legacy fixture");
    let reified = structurally_proven_legacy_evaluator_sanity(
        decoded.transformers(),
        &legacy,
        &assigned,
        &logos,
    )
    .expect("exhaustive structural equivalence admits test-only legacy evaluation");
    let (ethos, names) = shared_lowering_corpus();
    let expected = legacy.apply(&ethos, &names).expect("legacy lowering");
    let found = reified.apply(&ethos, &names).expect("reified lowering");
    let project = |lowering: &core_nomos::Lowering| {
        lowering
            .items
            .iter()
            .map(|item| {
                RustSource::project_item(item, &lowering.names)
                    .expect("project shared corpus item")
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(project(&expected), project(&found));
}

// Retired: native evaluation now admits only its explicit Nexus boundary.
#[cfg(any())]
#[test]
fn native_evaluator_substitutes_authored_futures_over_complete_populations() {
    let logos = logos();
    let assigned = bindings(SOURCE);
    let legacy = MacroPackage::wire_fixture().expect("independent authoring oracle");
    let adapter = FixtureAdapter::new(&legacy, &assigned, &logos);
    let authored = adapter
        .authored_set()
        .expect("five-transformer authored set");

    assert!(matches!(
        authored.declarations()[0].root_output().selector(),
        TemplateRootOutputSelector::Field(_)
    ));
    assert!(matches!(
        authored.declarations()[2].root_output().selector(),
        TemplateRootOutputSelector::WholeValue
    ));
    let authored_bytes = authored
        .to_archive_bytes()
        .expect("archive selector-bearing authored set");
    let restored = AuthoredTransformerSet::from_archive_bytes(&authored_bytes)
        .expect("restore selector-bearing authored set");
    assert_eq!(restored, authored);

    let integer = encoded(&[200, 1]);
    let vector = encoded(&[200, 2]);
    let rust_integer = encoded_at(VocabularyRoot::Rust, &[20, 1]);
    let rust_vector = encoded_at(VocabularyRoot::Rust, &[20, 2]);
    let mut authored_references = Vec::new();
    collect_template_references(
        authored.declarations()[0].result(),
        &mut authored_references,
    );
    let fixed_source = authored_references[0].clone();
    let unlisted_source = authored_references
        .iter()
        .find(|identity| **identity != fixed_source)
        .expect("attribute corpus has multiple exact references")
        .clone();
    let fixed_target = encoded_at(VocabularyRoot::Rust, &[20, 3]);
    let references = NativeReferenceUniverse::try_new(vec![
        NativeReferenceMapping::try_new(integer.clone(), rust_integer.clone())
            .expect("Universal-to-Rust integer mapping"),
        NativeReferenceMapping::try_new(vector.clone(), rust_vector.clone())
            .expect("Universal-to-Rust vector mapping"),
        NativeReferenceMapping::try_new(fixed_source.clone(), fixed_target.clone())
            .expect("authored fixed reference mapping"),
    ])
    .expect("distinct complete mapping sources");
    let evaluator = NativeAuthoredEvaluator::try_new(&restored, references.clone())
        .expect("admit native package");

    let input_newtype = WholeEthosItem::Newtype(WholeEthosNewtype::new(
        encoded(&[300, 1]),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        WholeEthosWrappedField::new(
            WholeEthosVisibility::Public,
            WholeEthosTypeReference::Application(WholeEthosTypeApplication::new(
                vector,
                WholeEthosTypeReference::Identity(integer.clone()),
            )),
        ),
    ));
    let input_enumeration = WholeEthosItem::Enumeration(WholeEthosEnumeration::new(
        encoded(&[300, 2]),
        WholeEthosVisibility::Private,
        WholeEthosAttributes::empty(),
        vec![
            WholeEthosVariant::new(
                encoded(&[300, 2, 1]),
                WholeEthosAttributes::empty(),
                WholeEthosVariantPayload::Unit,
            ),
            WholeEthosVariant::new(
                encoded(&[300, 2, 2]),
                WholeEthosAttributes::empty(),
                WholeEthosVariantPayload::Tuple(
                    WholeEthosTupleFields::new(vec![WholeEthosTypeReference::Identity(integer)])
                        .expect("non-empty tuple"),
                ),
            ),
        ],
    ));
    let input = EncodedPopulation::new(
        WholeEthos::new(vec![input_newtype, input_enumeration]),
        FixtureCompleteNameTree,
    );
    let transformed = evaluator.transform(&input).expect("native transform");

    assert_eq!(
        transformed.population().name_tree(),
        &FixtureLogosNameTree(2)
    );
    let values = transformed.population().encoded_form().values();
    assert_eq!(values.len(), 2);
    let mut evaluated_references = Vec::new();
    collect_native_references(&values[0], &mut evaluated_references);
    assert!(evaluated_references.contains(&fixed_target));
    assert!(!evaluated_references.contains(&fixed_source));
    assert!(evaluated_references.contains(&unlisted_source));
    assert!(matches!(
        values[0].fields()[2].term(),
        NativeEvaluatedTerm::Declaration(identity)
            if identity == &encoded(&[300, 1])
                && identity.root_variant() == &VocabularyRoot::Universal
    ));

    let NativeEvaluatedTerm::Sequence(attributes) = values[0].fields()[1].term() else {
        panic!("invoked attributes land as their actual sequence")
    };
    assert_eq!(attributes.len(), 3);
    assert!(
        attributes
            .iter()
            .all(|attribute| matches!(attribute, NativeEvaluatedTerm::Nested(_)))
    );

    let [whole_newtype, whole_enumeration] = transformed.whole_projection().items() else {
        panic!("bounded projection retains source item order")
    };
    let core_logos::WholeLogosItem::Newtype(whole_newtype) = whole_newtype else {
        panic!("first projection is a newtype")
    };
    assert_eq!(
        whole_newtype.visibility(),
        &core_logos::WholeLogosVisibility::Public
    );
    assert_eq!(
        whole_newtype.wrapped_visibility(),
        &core_logos::WholeLogosVisibility::Private
    );
    let core_logos::WholeLogosTypeReference::Application(application) = whole_newtype.wrapped()
    else {
        panic!("recursive mapped application")
    };
    assert_eq!(application.head(), &rust_vector);
    assert_eq!(
        application.payload(),
        &core_logos::WholeLogosTypeReference::Identity(rust_integer.clone())
    );

    let core_logos::WholeLogosItem::Enumeration(whole_enumeration) = whole_enumeration else {
        panic!("second projection is an enumeration")
    };
    assert_eq!(
        whole_enumeration.visibility(),
        &core_logos::WholeLogosVisibility::Public
    );
    let core_logos::WholeLogosVariantPayload::Tuple(tuple) =
        whole_enumeration.variants()[1].payload()
    else {
        panic!("tuple variant is retained")
    };
    assert_eq!(
        tuple.fields(),
        &[core_logos::WholeLogosTypeReference::Identity(rust_integer)]
    );

    // Authored visibility literals are authoritative even where po2.5's
    // SliceOne reference preserved the input visibility.
    // `[to-be-reviewed-by-psyche]`
    assert_ne!(
        transformed.whole_projection(),
        &core_nomos::SliceOneTransformation::new().lower(input.encoded_form())
    );

    let visibility_changed_input = EncodedPopulation::new(
        WholeEthos::new(vec![WholeEthosItem::Newtype(WholeEthosNewtype::new(
            encoded(&[300, 1]),
            WholeEthosVisibility::Public,
            WholeEthosAttributes::empty(),
            WholeEthosWrappedField::new(
                WholeEthosVisibility::Private,
                WholeEthosTypeReference::Application(WholeEthosTypeApplication::new(
                    encoded(&[200, 2]),
                    WholeEthosTypeReference::Identity(encoded(&[200, 1])),
                )),
            ),
        ))]),
        FixtureCompleteNameTree,
    );
    let visibility_changed = evaluator
        .transform(&visibility_changed_input)
        .expect("input visibility remains outside the authored signature");
    let core_logos::WholeLogosItem::Newtype(visibility_unchanged) =
        &visibility_changed.whole_projection().items()[0]
    else {
        panic!("visibility witness remains a newtype")
    };
    assert_eq!(visibility_unchanged, whole_newtype);

    let original_newtype = &restored.declarations()[2];
    let mut changed_fields = original_newtype.result().fields().to_vec();
    changed_fields[0] = TemplateFieldValue::new(
        changed_fields[0].role(),
        TemplateTerm::Nested(Box::new(
            adapter
                .visibility(
                    "WireNewtype",
                    "newtype.visibility",
                    core_logos::Visibility::Private,
                )
                .expect("private visibility value"),
        )),
    );
    let changed_result = TemplateValue::try_new(
        original_newtype.result().constructor().clone(),
        changed_fields,
    )
    .expect("stable roles remain distinct");
    let changed_newtype = AuthoredTransformerDeclaration::try_new(
        original_newtype.name().clone(),
        original_newtype.kind(),
        original_newtype.input().clone(),
        changed_result,
        &adapter.newtype,
    )
    .expect("authored visibility mutation remains well typed");
    let changed_package = replace_declaration(&restored, 2, changed_newtype);
    let changed_evaluator = NativeAuthoredEvaluator::try_new(&changed_package, references)
        .expect("mutated authored package remains admissible");
    let authored_visibility_changed = changed_evaluator
        .transform(&input)
        .expect("authored literal mutation transforms");
    let core_logos::WholeLogosItem::Newtype(changed_newtype) =
        &authored_visibility_changed.whole_projection().items()[0]
    else {
        panic!("first changed projection is a newtype")
    };
    assert_eq!(
        changed_newtype.visibility(),
        &core_logos::WholeLogosVisibility::Private
    );

    let output_bytes = transformed
        .population()
        .to_archive_bytes()
        .expect("archive native Logos population");
    let output = NativeLogosPopulation::<FixtureLogosNameTree>::from_archive_bytes(
        &output_bytes,
        &evaluator,
    )
    .expect("validate and restore native Logos population");
    assert_eq!(output, *transformed.population());
}

#[test]
fn native_admission_rejects_cycles_and_invoke_targets_that_require_input() {
    let logos = logos();
    let assigned = bindings(SOURCE);
    let legacy = MacroPackage::wire_fixture().expect("authoring oracle");
    let adapter = FixtureAdapter::new(&legacy, &assigned, &logos);
    let authored = adapter.authored_set().expect("authored set");
    let first = &authored.declarations()[0];
    let second = &authored.declarations()[1];

    let invoke_result = |target: AuthoredTransformerIdentity| {
        adapter
            .value(
                &adapter.attributes,
                4,
                1,
                vec![TemplateTerm::Sequence(vec![TemplateTerm::Future(
                    TemplateFuture::Invoke(target),
                )])],
            )
            .expect("Invoke inhabits the transparent attributes root")
    };
    let first_cycle = AuthoredTransformerDeclaration::try_new(
        first.name().clone(),
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        invoke_result(second.name().clone()),
        &adapter.attributes,
    )
    .expect("first cycle edge has a compatible output");
    let second_cycle = AuthoredTransformerDeclaration::try_new(
        second.name().clone(),
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        invoke_result(first.name().clone()),
        &adapter.attributes,
    )
    .expect("second cycle edge has a compatible output");
    let cycle = AuthoredTransformerSet::try_new(vec![first_cycle, second_cycle])
        .expect("authored set resolves both lookup-only targets");
    assert!(matches!(
        NativeAuthoredEvaluator::try_new(&cycle, NativeReferenceUniverse::identity()),
        Err(core_nomos::NativeEvaluationError::InvocationCycle { .. })
    ));

    let non_unit_target = AuthoredTransformerDeclaration::try_new(
        first.name().clone(),
        MacroKind::Named,
        AuthoredInputSignature::try_new(vec![
            authored.declarations()[2].input().parameters()[0].clone(),
        ])
        .expect("one distinct parameter"),
        first.result().clone(),
        &adapter.attributes,
    )
    .expect("unused input does not change the target output");
    let invoker = AuthoredTransformerDeclaration::try_new(
        second.name().clone(),
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        invoke_result(first.name().clone()),
        &adapter.attributes,
    )
    .expect("invocation output remains compatible");
    let non_unit = AuthoredTransformerSet::try_new(vec![non_unit_target, invoker])
        .expect("lookup/output validation is independent of evaluator admission");
    assert!(matches!(
        NativeAuthoredEvaluator::try_new(&non_unit, NativeReferenceUniverse::identity()),
        Err(core_nomos::NativeEvaluationError::InvocationRequiresInput { .. })
    ));
}

#[test]
fn enriched_generation_selection_stays_exact_and_outside_the_authored_seal() {
    let wire = MacroPackage::wire_fixture().expect("wire fixture");
    let enriched = MacroPackage::enriched_fixture().expect("enriched fixture");
    let expected = [
        GenerationClass::NewtypeErgonomics,
        GenerationClass::InterfaceErgonomics,
        GenerationClass::WireContract,
        GenerationClass::WireExchangeCodec,
        GenerationClass::WireExchangeEnvelope,
        GenerationClass::TraceSupport,
    ];
    assert_eq!(wire.definitions(), enriched.definitions());
    assert!(wire.selection().is_empty());
    compare_selection(&expected, enriched.selection()).expect("exact enriched selection");

    let mut reordered = expected;
    reordered.swap(0, 1);
    let mismatch = compare_selection(enriched.selection(), &reordered)
        .expect_err("GenerationClass order is semantic");
    assert_eq!(mismatch.transformer, "<selection>");
    assert_eq!(mismatch.dimension, "generation-class-order");
    assert_eq!(mismatch.path, vec!["selection[0]"]);
}

#[test]
fn isolated_kind_meta_fragment_and_transform_mutations_are_localized() {
    let logos = logos();
    let assigned = bindings(SOURCE);
    let legacy = MacroPackage::wire_fixture().expect("wire fixture");
    let adapter = FixtureAdapter::new(&legacy, &assigned, &logos);
    let expected = adapter.authored_set().expect("legacy adapter");
    let original = &expected.declarations()[2];

    let replacement = AuthoredTransformerDeclaration::try_new(
        original.name().clone(),
        MacroKind::Named,
        original.input().clone(),
        original.result().clone(),
        &adapter.newtype,
    )
    .expect("MacroKind is independent typed declaration data");
    let found = replace_declaration(&expected, 2, replacement);
    let mismatch = compare_sets(&expected, &found, &assigned).expect_err("kind mutation");
    assert_eq!(mismatch.transformer, "WireNewtype");
    assert_eq!(mismatch.dimension, "macro-kind");
    assert_eq!(mismatch.path, vec!["declaration[2]"]);

    let mut parameters = original.input().parameters().to_vec();
    parameters[0] = AuthoredInputParameter::new(
        parameters[0].binding().clone(),
        MetaType::Type,
        parameters[0].output().clone(),
    );
    let replacement = AuthoredTransformerDeclaration::try_new(
        original.name().clone(),
        original.kind(),
        AuthoredInputSignature::try_new(parameters).expect("distinct bindings"),
        original.result().clone(),
        &adapter.newtype,
    )
    .expect("the output remains landing-compatible despite the semantic meta mutation");
    let found = replace_declaration(&expected, 2, replacement);
    let mismatch = compare_sets(&expected, &found, &assigned).expect_err("meta mutation");
    assert_eq!(mismatch.transformer, "WireNewtype");
    assert_eq!(mismatch.dimension, "parameter-meta");
    assert_eq!(
        mismatch.path,
        vec!["declaration[2]".to_owned(), "input[0]".to_owned()]
    );

    let replacement = AuthoredTransformerDeclaration::try_new(
        original.name().clone(),
        original.kind(),
        original.input().clone(),
        expected.declarations()[0].result().clone(),
        &adapter.attributes,
    )
    .expect("the alternate fragment is independently typed");
    let found = replace_declaration(&expected, 2, replacement);
    let mismatch = compare_sets(&expected, &found, &assigned).expect_err("fragment mutation");
    assert_eq!(mismatch.transformer, "WireNewtype");
    assert_eq!(mismatch.dimension, "fragment-kind");
    assert_eq!(mismatch.path, vec!["declaration[2]"]);

    let replacement = AuthoredTransformerDeclaration::try_new(
        original.name().clone(),
        original.kind(),
        original.input().clone(),
        mutate_first_realize_transform(original.result(), NameTransform::FieldName),
        &adapter.newtype,
    )
    .expect("a name transform preserves the name landing");
    let found = replace_declaration(&expected, 2, replacement);
    let mismatch = compare_sets(&expected, &found, &assigned).expect_err("transform mutation");
    assert_eq!(mismatch.transformer, "WireNewtype");
    assert_eq!(mismatch.dimension, "name-transform");
    assert!(
        mismatch
            .path
            .iter()
            .any(|segment| segment.contains("field["))
    );
}

#[cfg(any())] // Superseded fixture predates the adjacent-angle structural form.
#[test]
fn declaration_set_reordering_preserves_structural_equivalence() {
    const WIRE: &str = r#"WireAttributes.Named {
()
[
rustfmt.skip
(|nota-text|).[nota.NotaDecode nota.NotaDecodeTraced nota.NotaEncode]
[rkyv.Archive rkyv.Serialize rkyv.Deserialize Clone Debug PartialEq Eq]
]
}
"#;
    const ENUMERATION: &str = r#"EnumerationAttributes.Named {
()
[
rustfmt.skip
(|nota-text|).[nota.NotaDecode nota.NotaDecodeTraced nota.NotaEncode]
[rkyv.Archive rkyv.Serialize rkyv.Deserialize Clone Copy Debug PartialEq Eq]
]
}
"#;
    let reordered = SOURCE
        .replacen(WIRE, "__WIRE_ATTRIBUTES_BLOCK__\n", 1)
        .replacen(ENUMERATION, WIRE, 1)
        .replacen("__WIRE_ATTRIBUTES_BLOCK__\n", ENUMERATION, 1);
    let logos = logos();
    let assigned = bindings(&reordered);
    let decoded = textual(&logos)
        .decode(&reordered, &assigned)
        .expect("declaration-set order is not semantic");
    let legacy = MacroPackage::wire_fixture().expect("wire fixture");
    let expected = FixtureAdapter::new(&legacy, &assigned, &logos)
        .authored_set()
        .expect("legacy adapter");
    compare_sets(&expected, decoded.transformers(), &assigned)
        .expect("canonical declaration sorting preserves the same set");
}

#[cfg(any())] // Superseded fixture predates the adjacent-angle structural form.
#[test]
fn ordered_dimensions_and_identity_ancestry_have_negative_witnesses() {
    let logos = logos();
    let legacy = MacroPackage::wire_fixture().expect("wire fixture");

    let parameter_reordered = SOURCE.replacen("(name.Name type.Type)", "(type.Type name.Name)", 1);
    let assigned = bindings(&parameter_reordered);
    let found = textual(&logos)
        .decode(&parameter_reordered, &assigned)
        .expect("the reordered signature remains independently well typed");
    let expected = FixtureAdapter::new(&legacy, &assigned, &logos)
        .authored_set()
        .expect("legacy adapter");
    let mismatch = compare_sets(&expected, found.transformers(), &assigned)
        .expect_err("parameter order is semantic");
    assert_eq!(mismatch.transformer, "WireNewtype");
    assert_eq!(mismatch.dimension, "parameter-binding");
    assert!(mismatch.path.iter().any(|segment| segment == "input[0]"));

    let sequence_reordered = SOURCE.replacen(
        "rkyv.Deserialize Clone Debug PartialEq Eq",
        "rkyv.Deserialize Debug Clone PartialEq Eq",
        1,
    );
    let assigned = bindings(&sequence_reordered);
    let found = textual(&logos)
        .decode(&sequence_reordered, &assigned)
        .expect("the reordered derive group remains independently well typed");
    let expected = FixtureAdapter::new(&legacy, &assigned, &logos)
        .authored_set()
        .expect("legacy adapter");
    let mismatch = compare_sets(&expected, found.transformers(), &assigned)
        .expect_err("derive sequence order is semantic");
    assert_eq!(mismatch.transformer, "WireAttributes");
    assert_eq!(mismatch.dimension, "term");
    assert!(
        mismatch
            .path
            .iter()
            .any(|segment| segment.starts_with("item["))
    );

    let wrong_invoke = SOURCE.replacen("Invoke.EnumerationAttributes", "Invoke.WireAttributes", 1);
    let assigned = bindings(&wrong_invoke);
    let found = textual(&logos)
        .decode(&wrong_invoke, &assigned)
        .expect("the alternate attribute target has the same landing output");
    let expected = FixtureAdapter::new(&legacy, &assigned, &logos)
        .authored_set()
        .expect("legacy adapter");
    let mismatch = compare_sets(&expected, found.transformers(), &assigned)
        .expect_err("Enumeration must invoke EnumerationAttributes");
    assert_eq!(mismatch.transformer, "Enumeration");
    assert_eq!(mismatch.dimension, "invoke-target");

    let mut assigned = bindings(SOURCE);
    swap_wire_newtype_binding_identities(&mut assigned, SOURCE);
    let found = textual(&logos)
        .decode(SOURCE, &assigned)
        .expect("coherent binding swaps are well typed but semantically distinct");
    let expected = FixtureAdapter::new(&legacy, &assigned, &logos)
        .authored_set()
        .expect("legacy adapter retains the prescribed identities");
    let mismatch = compare_sets(&expected, found.transformers(), &assigned)
        .expect_err("name/type binding identity swaps must not collide");
    assert_eq!(mismatch.transformer, "WireNewtype");
    assert_eq!(mismatch.dimension, "parameter-binding");
    assert_eq!(
        mismatch.path,
        vec!["declaration[2]".to_owned(), "input[0]".to_owned()]
    );

    let mut assigned = bindings(SOURCE);
    rebind_occurrence(&mut assigned, SOURCE, "Enumeration", 0, encoded(&[110, 6]));
    assigned.spelling(&encoded(&[110, 6]), "Enumeration");
    let found = textual(&logos)
        .decode(SOURCE, &assigned)
        .expect("a changed complete chain remains a valid Universal identity");
    let expected = FixtureAdapter::new(&legacy, &assigned, &logos)
        .authored_set()
        .expect("legacy adapter retains the prescribed chain");
    let mismatch = compare_sets(&expected, found.transformers(), &assigned)
        .expect_err("changing one identity ancestry segment is semantic");
    assert_eq!(mismatch.transformer, "Enumeration");
    assert_eq!(mismatch.dimension, "transformer-identity");
}

#[test]
fn legacy_only_splice_semantics_are_checked_explicitly() {
    let logos = logos();
    let assigned = bindings(SOURCE);
    let legacy = MacroPackage::wire_fixture().expect("wire fixture");
    let definition = legacy
        .structural_default(SectionDefault::Struct)
        .and_then(|identity| legacy.definition(identity))
        .expect("structural struct default");
    let mut template = definition.template.clone();
    let ResultTemplate::Item(ItemTemplate::Struct(structure)) = &mut template else {
        panic!("fixture default is a struct template");
    };
    let SequenceItem::Escape(Escape::Splice(splice)) = &mut structure.fields.items[0] else {
        panic!("fixture fields are a splice");
    };
    splice.element = SpliceElement::Field {
        visibility: Visibility::Private,
        name_rule: FieldNameRule::PreserveEthos,
    };
    let error = FixtureAdapter::new(&legacy, &assigned, &logos)
        .template("ParticularStruct", &template)
        .expect_err("the authored Splice must not erase legacy element semantics");
    assert!(matches!(
        error,
        FixtureAdapterError::SpliceElement {
            transformer,
            path,
            ..
        } if transformer == "ParticularStruct" && path == "struct.fields[0]"
    ));
}

#[test]
fn duplicate_identities_and_text_scalars_refuse_with_typed_diagnostics() {
    let logos = logos();
    let assigned = bindings(SOURCE);
    let legacy = MacroPackage::wire_fixture().expect("wire fixture");
    let expected = FixtureAdapter::new(&legacy, &assigned, &logos)
        .authored_set()
        .expect("legacy adapter");

    let duplicate = expected.declarations()[0].clone();
    let error = AuthoredTransformerSet::try_new(vec![duplicate.clone(), duplicate])
        .expect_err("duplicate transformer identities refuse");
    assert!(matches!(
        error,
        AuthoredNomosError::DuplicateTransformer { transformer }
            if transformer == encoded(&[110, 1])
    ));

    let duplicate = expected.declarations()[2].input().parameters()[0].clone();
    let error = AuthoredInputSignature::try_new(vec![duplicate.clone(), duplicate])
        .expect_err("duplicate binding identities refuse");
    assert!(matches!(
        error,
        AuthoredNomosError::DuplicateBinding { binding }
            if binding == encoded(&[110, 3, 1])
    ));

    let text = TemplateTerm::Scalar(structural_codec::ScalarValue::Text(
        "forbidden normalization".to_owned(),
    ));
    let error = compare_term("WireAttributes", &["result".to_owned()], &text, &text)
        .expect_err("text scalar normalization is not an encoded equivalence");
    assert_eq!(error.dimension, "text-scalar-refusal");
}

#[cfg(any())] // Superseded fixture predates the adjacent-angle structural form.
#[test]
fn canonical_realize_tail_forces_ordered_sequence_backtracking() {
    let logos = logos();
    let textual = textual(&logos);
    let assigned = bindings(SOURCE);
    let name_bound = token_occurrences(SOURCE, "name")[1];

    let decoded = textual
        .decode(SOURCE, &assigned)
        .expect("the repeated attributes must yield Realize.name to the required name slot");
    assert_eq!(decoded.transformers().declarations().len(), 5);
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

#[cfg(any())] // Superseded fixture predates the adjacent-angle structural form.
#[test]
fn every_future_payload_is_lookup_only_and_missing_lookup_refuses() {
    let logos = logos();
    let textual = textual(&logos);
    let successful = bindings(SOURCE);
    textual
        .decode(SOURCE, &successful)
        .expect("valid lookup-only decode");

    for spelling in [
        "WireAttributes",
        "EnumerationAttributes",
        "name",
        "type",
        "fields",
        "variants",
    ] {
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

#[cfg(any())] // Superseded fixture asserts retired InsertAt source positioning.
#[test]
fn malformed_insert_at_forms_refuse_before_authored_evaluation() {
    let logos = logos();
    let textual = textual(&logos);

    let missing_boundary = RECURSIVE_SOURCE.replacen("InsertAt.children 0", "InsertAt.children", 1);
    assert!(
        textual
            .decode(&missing_boundary, &bindings(&missing_boundary))
            .is_err(),
        "InsertAt without its integer boundary must refuse"
    );

    let noninteger_boundary =
        RECURSIVE_SOURCE.replacen("InsertAt.children 0", "InsertAt.children rustfmt.skip", 1);
    assert!(
        textual
            .decode(&noninteger_boundary, &bindings(&noninteger_boundary))
            .is_err(),
        "InsertAt with a noninteger boundary must refuse"
    );

    let missing_value = RECURSIVE_SOURCE.replacen(
        "[\nInvoke.ScopeOfStep\nSplice.children\nInsertAt.children 0 rustfmt.skip\n[Clone]\n]",
        "[\nInvoke.ScopeOfStep\nSplice.children\n[Clone]\nInsertAt.children 0\n]",
        1,
    );
    assert!(matches!(
        textual.decode(&missing_value, &bindings(&missing_value)),
        Err(core_nomos::TextualNomosError::InsertAtMissingValue)
    ));

    let wrong_position =
        RECURSIVE_SOURCE.replacen("Public Invoke.ScopeOfStep", "InsertAt.variants 0", 1);
    assert!(
        textual
            .decode(&wrong_position, &bindings(&wrong_position))
            .is_err(),
        "InsertAt outside a repeated sequence must refuse"
    );
}

#[cfg(any())] // The current grammar owns the eighth angle trigger.
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
    assert!(source.contains("SharedDescriptor::AdjacentSequence(_)"));
}

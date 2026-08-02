//! Typed future-output compatibility is proved before evaluator entry.

use core_logos::{LogosLanguage, LogosLanguageTypeIds, LogosLanguageWords};
use core_nomos::{
    AuthoredBindingIdentity, AuthoredInputParameter, AuthoredInputSignature, AuthoredNomosError,
    AuthoredTransformerDeclaration, AuthoredTransformerIdentity, AuthoredTransformerSet,
    ItemTemplate, MacroIdentity, MacroKind, MacroPackage, MetaType, NameTransform, ResultTemplate,
    SectionDefault, SequenceItem, TemplateFieldValue, TemplateFuture, TemplateFutureKind,
    TemplateFutureOutput, TemplateLandingShape, TemplateLanguage, TemplateTerm, TemplateValue,
};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::{LandingShape, ScalarValue, StableRoleId};

const ATTRIBUTES_FIELD: usize = 1;
const NAME_FIELD: usize = 2;
const NEWTYPE_WRAPPED_FIELD: usize = 4;
const ENUMERATION_VARIANTS_FIELD: usize = 4;

fn encoded(chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("non-empty fixture identity")
}

fn transformer(chain: &[u16]) -> AuthoredTransformerIdentity {
    AuthoredTransformerIdentity::try_new(encoded(chain)).expect("Universal transformer")
}

fn binding(chain: &[u16]) -> AuthoredBindingIdentity {
    AuthoredBindingIdentity::try_new(encoded(chain)).expect("Universal binding")
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
    .expect("source grammar and landing declarations agree")
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

fn root_constructor(
    language: &TemplateLanguage<VocabularyRoot>,
) -> &core_nomos::TemplateConstructorDeclaration<VocabularyRoot> {
    language
        .type_declaration(language.root())
        .and_then(|declaration| declaration.constructors().first())
        .expect("root constructor")
}

fn field_shape(
    language: &TemplateLanguage<VocabularyRoot>,
    index: usize,
) -> &TemplateLandingShape<VocabularyRoot> {
    root_constructor(language)
        .landing_fields()
        .get(index)
        .map(core_nomos::TemplateLandingField::shape)
        .expect("root carries fixture role")
}

fn field_role(language: &TemplateLanguage<VocabularyRoot>, index: usize) -> StableRoleId {
    root_constructor(language)
        .landing_fields()
        .get(index)
        .map(core_nomos::TemplateLandingField::role)
        .expect("root carries fixture role")
}

fn literal_value(
    constructor: &structural_codec::EncodedConstructorId<VocabularyRoot>,
    language: &TemplateLanguage<VocabularyRoot>,
) -> TemplateValue<VocabularyRoot> {
    let declaration = language
        .constructor(constructor)
        .expect("addressed constructor");
    let fields = declaration
        .landing_fields()
        .iter()
        .map(|field| TemplateFieldValue::new(field.role(), literal_term(field.shape(), language)))
        .collect();
    TemplateValue::try_new(constructor.clone(), fields).expect("computed roles are unique")
}

fn literal_term(
    shape: &TemplateLandingShape<VocabularyRoot>,
    language: &TemplateLanguage<VocabularyRoot>,
) -> TemplateTerm<VocabularyRoot> {
    match shape {
        TemplateLandingShape::Fixed(LandingShape::Literal(value)) => {
            TemplateTerm::Literal(value.clone())
        }
        TemplateLandingShape::Fixed(LandingShape::Scalar(codec)) => {
            let scalar = match codec {
                structural_codec::LeafCodec::Integer => ScalarValue::Integer(0),
                structural_codec::LeafCodec::Float => ScalarValue::Float(0.0),
                structural_codec::LeafCodec::Text
                | structural_codec::LeafCodec::PipeText
                | structural_codec::LeafCodec::Foreign(_) => ScalarValue::Text(String::new()),
                structural_codec::LeafCodec::Boolean => ScalarValue::Boolean(false),
            };
            TemplateTerm::Scalar(scalar)
        }
        TemplateLandingShape::ValueOrFuture { value, .. } => match value {
            LandingShape::Declaration => TemplateTerm::Declaration(encoded(&[90, 1])),
            LandingShape::Reference => TemplateTerm::Reference(encoded(&[90, 2])),
            LandingShape::Type(target) => {
                let constructor = language
                    .type_declaration(target)
                    .and_then(|declaration| declaration.constructors().first())
                    .expect("addressed nested constructor");
                TemplateTerm::Nested(Box::new(literal_value(constructor.constructor(), language)))
            }
            LandingShape::Literal(_) | LandingShape::Scalar(_) | LandingShape::Sequence { .. } => {
                panic!("single value position")
            }
        },
        TemplateLandingShape::Nested(target) => {
            let constructor = language
                .type_declaration(target)
                .and_then(|declaration| declaration.constructors().first())
                .expect("addressed nested constructor");
            TemplateTerm::Nested(Box::new(literal_value(constructor.constructor(), language)))
        }
        TemplateLandingShape::Sequence { .. } => TemplateTerm::Sequence(Vec::new()),
        TemplateLandingShape::Fixed(
            LandingShape::Declaration
            | LandingShape::Reference
            | LandingShape::Type(_)
            | LandingShape::Sequence { .. },
        ) => panic!("term-producing landing is never fixed"),
    }
}

fn root_value(
    language: &TemplateLanguage<VocabularyRoot>,
    mut replacement: impl FnMut(
        StableRoleId,
        &TemplateLandingShape<VocabularyRoot>,
    ) -> Option<TemplateTerm<VocabularyRoot>>,
) -> TemplateValue<VocabularyRoot> {
    let constructor = root_constructor(language);
    let fields = constructor
        .landing_fields()
        .iter()
        .map(|field| {
            TemplateFieldValue::new(
                field.role(),
                replacement(field.role(), field.shape())
                    .unwrap_or_else(|| literal_term(field.shape(), language)),
            )
        })
        .collect();
    TemplateValue::try_new(constructor.constructor().clone(), fields)
        .expect("computed roles are unique")
}

fn attributes_value(
    language: &TemplateLanguage<VocabularyRoot>,
    item: Option<TemplateTerm<VocabularyRoot>>,
    literal_count: usize,
) -> TemplateValue<VocabularyRoot> {
    root_value(language, |_role, shape| {
        let TemplateLandingShape::Sequence { element, .. } = shape else {
            panic!("attributes root is a transparent sequence")
        };
        let items = if let Some(item) = item.clone() {
            vec![item]
        } else {
            (0..literal_count)
                .map(|_| literal_term(element, language))
                .collect()
        };
        Some(TemplateTerm::Sequence(items))
    })
}

fn authored_parameter(
    binding: AuthoredBindingIdentity,
    meta: MetaType,
    language: &TemplateLanguage<VocabularyRoot>,
    field: usize,
) -> AuthoredInputParameter {
    AuthoredInputParameter::new(binding, meta, output(field_shape(language, field)))
}

struct FixtureDeclarations {
    attributes: AuthoredTransformerDeclaration,
    newtype: AuthoredTransformerDeclaration,
    enumeration: AuthoredTransformerDeclaration,
    attributes_identity: AuthoredTransformerIdentity,
    newtype_identity: AuthoredTransformerIdentity,
    enumeration_identity: AuthoredTransformerIdentity,
}

fn valid_fixture_declarations(
    attributes: &TemplateLanguage<VocabularyRoot>,
    newtype: &TemplateLanguage<VocabularyRoot>,
    enumeration: &TemplateLanguage<VocabularyRoot>,
) -> FixtureDeclarations {
    let attributes_identity = transformer(&[40, 1]);
    let newtype_identity = transformer(&[40, 2]);
    let enumeration_identity = transformer(&[40, 3]);
    let name = binding(&[50, 1]);
    let wrapped = binding(&[50, 2]);
    let variants = binding(&[50, 3]);
    let newtype_attributes_role = field_role(newtype, ATTRIBUTES_FIELD);
    let newtype_name_role = field_role(newtype, NAME_FIELD);
    let newtype_wrapped_role = field_role(newtype, NEWTYPE_WRAPPED_FIELD);
    let enumeration_attributes_role = field_role(enumeration, ATTRIBUTES_FIELD);
    let enumeration_name_role = field_role(enumeration, NAME_FIELD);
    let enumeration_variants_role = field_role(enumeration, ENUMERATION_VARIANTS_FIELD);

    let attributes_declaration = AuthoredTransformerDeclaration::try_new(
        attributes_identity.clone(),
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        attributes_value(attributes, None, 3),
        attributes,
    )
    .expect("literal attributes declaration");

    let newtype_value = root_value(newtype, |role, _shape| {
        if role == newtype_attributes_role {
            Some(TemplateTerm::Sequence(vec![TemplateTerm::Future(
                TemplateFuture::Invoke(attributes_identity.clone()),
            )]))
        } else if role == newtype_name_role {
            Some(TemplateTerm::Future(TemplateFuture::Realize {
                binding: name.clone(),
                transform: NameTransform::Identity,
            }))
        } else if role == newtype_wrapped_role {
            Some(TemplateTerm::Future(TemplateFuture::Realize {
                binding: wrapped.clone(),
                transform: NameTransform::Identity,
            }))
        } else {
            None
        }
    });
    let newtype_declaration = AuthoredTransformerDeclaration::try_new(
        newtype_identity.clone(),
        MacroKind::Structural(SectionDefault::Newtype),
        AuthoredInputSignature::try_new(vec![
            authored_parameter(name.clone(), MetaType::Name, newtype, NAME_FIELD),
            authored_parameter(wrapped, MetaType::Type, newtype, NEWTYPE_WRAPPED_FIELD),
        ])
        .expect("distinct bindings"),
        newtype_value,
        newtype,
    )
    .expect("typed newtype futures");

    let enumeration_value = root_value(enumeration, |role, _shape| {
        if role == enumeration_attributes_role {
            Some(TemplateTerm::Sequence(vec![TemplateTerm::Future(
                TemplateFuture::Invoke(attributes_identity.clone()),
            )]))
        } else if role == enumeration_name_role {
            Some(TemplateTerm::Future(TemplateFuture::Realize {
                binding: name.clone(),
                transform: NameTransform::Identity,
            }))
        } else if role == enumeration_variants_role {
            Some(TemplateTerm::Sequence(vec![TemplateTerm::Future(
                TemplateFuture::Splice {
                    binding: variants.clone(),
                },
            )]))
        } else {
            None
        }
    });
    let enumeration_declaration = AuthoredTransformerDeclaration::try_new(
        enumeration_identity.clone(),
        MacroKind::Structural(SectionDefault::Enumeration),
        AuthoredInputSignature::try_new(vec![
            authored_parameter(name, MetaType::Name, enumeration, NAME_FIELD),
            authored_parameter(
                variants,
                MetaType::Variants,
                enumeration,
                ENUMERATION_VARIANTS_FIELD,
            ),
        ])
        .expect("distinct bindings"),
        enumeration_value,
        enumeration,
    )
    .expect("typed enumeration futures");

    FixtureDeclarations {
        attributes: attributes_declaration,
        newtype: newtype_declaration,
        enumeration: enumeration_declaration,
        attributes_identity,
        newtype_identity,
        enumeration_identity,
    }
}

#[test]
fn valid_invoke_attributes_and_splice_variants_seal_before_evaluation() {
    let logos = logos();
    let attributes =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("attributes language");
    let newtype = TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.newtype_type())
        .expect("newtype language");
    let enumeration =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.enumeration_type())
            .expect("enumeration language");
    let fixture = valid_fixture_declarations(&attributes, &newtype, &enumeration);
    let sealed = AuthoredTransformerSet::try_new(vec![
        fixture.attributes,
        fixture.newtype,
        fixture.enumeration,
    ])
    .expect("all future outputs inhabit their computed landings");
    assert_eq!(sealed.declarations().len(), 3);
}

#[test]
fn splice_variants_into_attributes_refuses_during_declaration_load() {
    let logos = logos();
    let attributes =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("attributes language");
    let enumeration =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.enumeration_type())
            .expect("enumeration language");
    let variants = binding(&[60, 1]);
    let result = attributes_value(
        &attributes,
        Some(TemplateTerm::Future(TemplateFuture::Splice {
            binding: variants.clone(),
        })),
        0,
    );
    let expected = attributes.root_output().expect("attributes output");
    let found = output(field_shape(&enumeration, ENUMERATION_VARIANTS_FIELD));

    assert_eq!(
        AuthoredTransformerDeclaration::try_new(
            transformer(&[60, 2]),
            MacroKind::Named,
            AuthoredInputSignature::try_new(vec![AuthoredInputParameter::new(
                variants,
                MetaType::Variants,
                found.clone(),
            )])
            .expect("one binding"),
            result,
            &attributes,
        ),
        Err(AuthoredNomosError::FutureOutputMismatch {
            future: TemplateFutureKind::Splice,
            expected,
            found,
        })
    );
}

#[test]
fn invoke_attributes_into_variants_refuses_when_package_targets_resolve() {
    let logos = logos();
    let attributes =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("attributes language");
    let enumeration =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.enumeration_type())
            .expect("enumeration language");
    let attributes_identity = transformer(&[70, 1]);
    let attributes_declaration = AuthoredTransformerDeclaration::try_new(
        attributes_identity.clone(),
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        attributes_value(&attributes, None, 0),
        &attributes,
    )
    .expect("attributes target");
    let variants_role = field_role(&enumeration, ENUMERATION_VARIANTS_FIELD);
    let result = root_value(&enumeration, |role, _shape| {
        (role == variants_role).then(|| {
            TemplateTerm::Sequence(vec![TemplateTerm::Future(TemplateFuture::Invoke(
                attributes_identity.clone(),
            ))])
        })
    });
    let caller = AuthoredTransformerDeclaration::try_new(
        transformer(&[70, 2]),
        MacroKind::Structural(SectionDefault::Enumeration),
        AuthoredInputSignature::unit(),
        result,
        &enumeration,
    )
    .expect("invoke target type resolves only with the complete set");
    let expected = output(field_shape(&enumeration, ENUMERATION_VARIANTS_FIELD));
    let found = attributes.root_output().expect("attributes output");

    assert_eq!(
        AuthoredTransformerSet::try_new(vec![attributes_declaration, caller]),
        Err(AuthoredNomosError::FutureOutputMismatch {
            future: TemplateFutureKind::Invoke,
            expected,
            found,
        })
    );
}

#[test]
fn realize_type_into_name_refuses_during_declaration_load() {
    let logos = logos();
    let newtype = TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.newtype_type())
        .expect("newtype language");
    let mismatched = binding(&[80, 1]);
    let name_role = field_role(&newtype, NAME_FIELD);
    let result = root_value(&newtype, |role, _shape| {
        (role == name_role).then(|| {
            TemplateTerm::Future(TemplateFuture::Realize {
                binding: mismatched.clone(),
                transform: NameTransform::Identity,
            })
        })
    });
    let expected = output(field_shape(&newtype, NAME_FIELD));
    let found = output(field_shape(&newtype, NEWTYPE_WRAPPED_FIELD));

    assert_eq!(
        AuthoredTransformerDeclaration::try_new(
            transformer(&[80, 2]),
            MacroKind::Structural(SectionDefault::Newtype),
            AuthoredInputSignature::try_new(vec![AuthoredInputParameter::new(
                mismatched,
                MetaType::Type,
                found.clone(),
            )])
            .expect("one binding"),
            result,
            &newtype,
        ),
        Err(AuthoredNomosError::FutureOutputMismatch {
            future: TemplateFutureKind::Realize,
            expected,
            found,
        })
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FragmentCardinality {
    One,
    Sequence,
}

#[derive(Debug, Eq, PartialEq)]
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
            let kind = match escape {
                core_nomos::Escape::Realize(_) => TemplateFutureKind::Realize,
                core_nomos::Escape::Invoke(_) => TemplateFutureKind::Invoke,
                core_nomos::Escape::Splice(_) => TemplateFutureKind::Splice,
            };
            futures.push(kind);
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
        ResultTemplate::Item(ItemTemplate::Struct(_)) => panic!("not one of the three oracles"),
    };
    FixtureOracle {
        fragment,
        futures,
        top_level_literals,
    }
}

#[test]
fn three_generic_skeletons_match_independent_production_fixture_oracles() {
    let logos = logos();
    let attributes =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("attributes language");
    let newtype = TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.newtype_type())
        .expect("newtype language");
    let enumeration =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.enumeration_type())
            .expect("enumeration language");
    let fixture = valid_fixture_declarations(&attributes, &newtype, &enumeration);
    let production = MacroPackage::wire_fixture().expect("existing Rust fixture");

    let production_attributes = production
        .definition(MacroIdentity::new(0))
        .expect("WireAttributes fixture");
    let production_newtype = production
        .structural_default(SectionDefault::Newtype)
        .and_then(|identity| production.definition(identity))
        .expect("WireNewtype fixture");
    let production_enumeration = production
        .structural_default(SectionDefault::Enumeration)
        .and_then(|identity| production.definition(identity))
        .expect("Enumeration fixture");

    assert_eq!(
        generic_oracle(&fixture.attributes),
        legacy_oracle(&production_attributes.template)
    );
    assert_eq!(
        generic_oracle(&fixture.newtype),
        legacy_oracle(&production_newtype.template)
    );
    assert_eq!(
        generic_oracle(&fixture.enumeration),
        legacy_oracle(&production_enumeration.template)
    );

    assert_eq!(fixture.attributes.name(), &fixture.attributes_identity);
    assert_eq!(fixture.newtype.name(), &fixture.newtype_identity);
    assert_eq!(fixture.enumeration.name(), &fixture.enumeration_identity);
}

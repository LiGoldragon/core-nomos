//! Template(Logos) is computed from one grammar/declaration pair for all roots.

use core_logos::{LogosLanguage, LogosLanguageTypeIds, LogosLanguageWords};
use core_nomos::{
    TemplateFieldValue, TemplateFutureOutput, TemplateLandingShape, TemplateLanguage, TemplateTerm,
    TemplateValue, TemplateValueError,
};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::{EncodedConstructorId, LandingShape, ScalarValue};

fn encoded(chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("non-empty fixture identity")
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

fn generic_value(
    constructor: &EncodedConstructorId<VocabularyRoot>,
    language: &TemplateLanguage<VocabularyRoot>,
) -> TemplateValue<VocabularyRoot> {
    let declaration = language
        .constructor(constructor)
        .expect("addressed computed constructor");
    let fields = declaration
        .landing_fields()
        .iter()
        .map(|field| TemplateFieldValue::new(field.role(), generic_term(field.shape(), language)))
        .collect();
    TemplateValue::try_new(constructor.clone(), fields).expect("declaration supplies unique roles")
}

fn generic_term(
    shape: &TemplateLandingShape<VocabularyRoot>,
    language: &TemplateLanguage<VocabularyRoot>,
) -> TemplateTerm<VocabularyRoot> {
    match shape {
        TemplateLandingShape::Fixed(LandingShape::Literal(value)) => {
            TemplateTerm::Literal(value.clone())
        }
        TemplateLandingShape::Fixed(LandingShape::Scalar(codec)) => {
            let value = match codec {
                structural_codec::LeafCodec::Integer => ScalarValue::Integer(0),
                structural_codec::LeafCodec::Float => ScalarValue::Float(0.0),
                structural_codec::LeafCodec::Text
                | structural_codec::LeafCodec::PipeText
                | structural_codec::LeafCodec::Foreign(_) => ScalarValue::Text(String::new()),
                structural_codec::LeafCodec::Boolean => ScalarValue::Boolean(false),
            };
            TemplateTerm::Scalar(value)
        }
        TemplateLandingShape::Fixed(
            LandingShape::Declaration
            | LandingShape::Reference
            | LandingShape::Type(_)
            | LandingShape::Sequence { .. },
        ) => panic!("derivation never leaves a term-producing landing fixed"),
        TemplateLandingShape::ValueOrFuture { value, .. } => match value {
            LandingShape::Declaration => TemplateTerm::Declaration(encoded(&[30, 1])),
            LandingShape::Reference => TemplateTerm::Reference(encoded(&[30, 2])),
            LandingShape::Type(target) => {
                let constructor = language
                    .type_declaration(target)
                    .and_then(|declaration| declaration.constructors().first())
                    .expect("recursively addressed nested constructor");
                TemplateTerm::Nested(Box::new(generic_value(constructor.constructor(), language)))
            }
            LandingShape::Literal(_) | LandingShape::Scalar(_) | LandingShape::Sequence { .. } => {
                panic!("value-or-future carries a single term-producing landing")
            }
        },
        TemplateLandingShape::Nested(target) => {
            let constructor = language
                .type_declaration(target)
                .and_then(|declaration| declaration.constructors().first())
                .expect("recursively addressed nested constructor");
            TemplateTerm::Nested(Box::new(generic_value(constructor.constructor(), language)))
        }
        TemplateLandingShape::Sequence {
            minimum, element, ..
        } => TemplateTerm::Sequence(
            (0..*minimum)
                .map(|_| generic_term(element, language))
                .collect(),
        ),
    }
}

#[test]
fn three_roots_use_one_derivation_and_one_runtime_value_algebra() {
    let logos = logos();
    let roots = [
        logos.newtype_type(),
        logos.enumeration_type(),
        logos.attributes_type(),
    ];
    for root in roots {
        let template = TemplateLanguage::derive(logos.grammar(), logos.landing(), root)
            .expect("derive addressed Template(Logos) closure");
        let constructor = template
            .type_declaration(root)
            .and_then(|declaration| declaration.constructors().first())
            .expect("root constructor");
        let value = generic_value(constructor.constructor(), &template);
        template
            .validate_value(&value)
            .expect("one generic validator accepts the declaration-indexed value");
        assert!(template.addressed_types().len() > 1);
    }
}

#[test]
fn computed_roots_publish_generic_fragment_outputs() {
    let logos = logos();
    let attributes =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("derive attributes template");
    let newtype = TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.newtype_type())
        .expect("derive newtype template");
    assert!(matches!(
        attributes
            .root_output()
            .expect("transparent sequence output")
            .landing(),
        LandingShape::Sequence { .. }
    ));
    assert_eq!(
        newtype.root_output().expect("addressed value output"),
        TemplateFutureOutput::new(LandingShape::Type(logos.newtype_type().clone()))
    );
}

#[test]
fn root_validation_refuses_a_value_from_the_addressed_nested_closure() {
    let logos = logos();
    let template = TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.newtype_type())
        .expect("derive newtype template");
    let nested = template
        .addressed_types()
        .iter()
        .find(|declaration| declaration.encoded_type() != template.root())
        .and_then(|declaration| declaration.constructors().first())
        .expect("addressed nested constructor");
    let value = generic_value(nested.constructor(), &template);

    assert_eq!(
        template.validate_value(&value),
        Err(TemplateValueError::TypeMismatch {
            expected: template.root().clone(),
            found: nested.constructor().type_id().clone(),
        })
    );
}

#[test]
fn derivation_source_contains_no_per_type_twin_or_generated_rust() {
    let source = include_str!("../src/template_language.rs");
    for forbidden in [
        "AuthoredNewtype",
        "AuthoredEnumeration",
        "AuthoredAttribute",
        "struct TemplateNewtype",
        "struct TemplateEnumeration",
        "struct TemplateAttribute",
        "quote!",
        "proc_macro",
    ] {
        assert!(
            !source.contains(forbidden),
            "fixed derivation must not contain {forbidden}"
        );
    }
}

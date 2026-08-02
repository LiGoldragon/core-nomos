//! Durable witnesses for Nomos-owned authored state over generic Template(X).

use core_logos::{LogosLanguage, LogosLanguageTypeIds, LogosLanguageWords};
use core_nomos::{
    AuthoredBindingIdentity, AuthoredIdentityPosition, AuthoredInputParameter,
    AuthoredInputSignature, AuthoredNomosError, AuthoredTransformerDeclaration,
    AuthoredTransformerIdentity, MacroKind, MetaType, TemplateFieldValue, TemplateFuture,
    TemplateFutureOutput, TemplateLanguage, TemplateTerm, TemplateValue,
};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::LandingShape;

fn encoded(root: VocabularyRoot, chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        root,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("complete test chain")
}

fn universal(chain: &[u16]) -> VocabularyEncodedId {
    encoded(VocabularyRoot::Universal, chain)
}

fn rust(chain: &[u16]) -> VocabularyEncodedId {
    encoded(VocabularyRoot::Rust, chain)
}

fn transformer(chain: &[u16]) -> AuthoredTransformerIdentity {
    AuthoredTransformerIdentity::try_new(universal(chain)).expect("Universal transformer")
}

fn binding(chain: &[u16]) -> AuthoredBindingIdentity {
    AuthoredBindingIdentity::try_new(universal(chain)).expect("Universal binding")
}

fn logos() -> LogosLanguage {
    LogosLanguage::seal(
        LogosLanguageTypeIds {
            newtype: universal(&[1]),
            structure: universal(&[13]),
            enumeration: universal(&[2]),
            visibility: universal(&[3]),
            attributes: universal(&[4]),
            attribute: universal(&[5]),
            path: universal(&[6]),
            configuration_predicate: universal(&[7]),
            derive_group: universal(&[8]),
            generics: universal(&[9]),
            generic_parameter: universal(&[10]),
            type_reference: universal(&[11]),
            field: universal(&[14]),
            variant: universal(&[12]),
        },
        LogosLanguageWords {
            public: universal(&[20]),
            private: universal(&[21]),
        },
    )
    .expect("Logos source declaration")
}

fn attributes_result(
    template: &TemplateLanguage<VocabularyRoot>,
    items: Vec<TemplateTerm<VocabularyRoot>>,
) -> TemplateValue<VocabularyRoot> {
    let constructor = template
        .type_declaration(template.root())
        .and_then(|declaration| declaration.constructors().first())
        .expect("attributes constructor");
    let field = constructor
        .landing_fields()
        .first()
        .expect("attributes sequence field");
    TemplateValue::try_new(
        constructor.constructor().clone(),
        vec![TemplateFieldValue::new(
            field.role(),
            TemplateTerm::Sequence(items),
        )],
    )
    .expect("unique computed role")
}

#[test]
fn declaration_archives_one_generic_template_value_with_complete_identities() {
    let logos = logos();
    let template =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("derive Template(Logos)");
    let name = transformer(&[40, 7, 2]);
    let declaration = AuthoredTransformerDeclaration::try_new(
        name.clone(),
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        attributes_result(&template, Vec::new()),
        &template,
    )
    .expect("generic result fits its computed declaration");

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&declaration).expect("archive declaration");
    let restored = rkyv::from_bytes::<AuthoredTransformerDeclaration, rkyv::rancor::Error>(&bytes)
        .expect("restore declaration");
    assert_eq!(restored, declaration);
    assert_eq!(restored.name().encoded_id(), name.encoded_id());
}

#[test]
fn generic_binding_walk_refuses_an_undeclared_splice_before_construction() {
    let logos = logos();
    let template =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("derive Template(Logos)");
    let absent = binding(&[40, 8, 9]);
    let result = attributes_result(
        &template,
        vec![TemplateTerm::Future(TemplateFuture::Splice {
            binding: absent.clone(),
        })],
    );
    assert_eq!(
        AuthoredTransformerDeclaration::try_new(
            transformer(&[40, 8]),
            MacroKind::Named,
            AuthoredInputSignature::unit(),
            result,
            &template,
        ),
        Err(AuthoredNomosError::UndeclaredBinding {
            binding: absent.encoded_id().clone(),
        })
    );
}

#[test]
fn authored_identity_and_duplicate_binding_failures_remain_typed() {
    assert_eq!(
        AuthoredTransformerIdentity::try_new(rust(&[1])),
        Err(AuthoredNomosError::WrongRoot {
            position: AuthoredIdentityPosition::Transformer,
            found: VocabularyRoot::Rust,
        })
    );
    assert_eq!(
        AuthoredBindingIdentity::try_new(rust(&[2])),
        Err(AuthoredNomosError::WrongRoot {
            position: AuthoredIdentityPosition::Binding,
            found: VocabularyRoot::Rust,
        })
    );

    let repeated = binding(&[40, 8, 1]);
    assert_eq!(
        AuthoredInputSignature::try_new(vec![
            AuthoredInputParameter::new(
                repeated.clone(),
                MetaType::Name,
                TemplateFutureOutput::new(LandingShape::Declaration),
            ),
            AuthoredInputParameter::new(
                repeated.clone(),
                MetaType::Type,
                TemplateFutureOutput::new(LandingShape::Reference),
            ),
        ]),
        Err(AuthoredNomosError::DuplicateBinding {
            binding: repeated.encoded_id().clone(),
        })
    );
}

#[test]
fn authored_module_contains_no_parallel_logos_universe() {
    let source = include_str!("../src/authored.rs");
    for forbidden in [
        "AuthoredAttribute",
        "AuthoredPath",
        "AuthoredTypeReference",
        "AuthoredNewtypeSkeleton",
        "AuthoredEnumerationSkeleton",
        "AuthoredStructSkeleton",
        "AuthoredVisibility",
        "name_table::Identifier",
        "String>",
        "&str",
    ] {
        assert!(
            !source.contains(forbidden),
            "authored state must not contain {forbidden}"
        );
    }
}

use std::fs;
use std::path::{Path, PathBuf};

use core_logos::{LogosLanguage, LogosLanguageTypeIds, LogosLanguageWords};
use core_nomos::{
    AuthoredBindingIdentity, AuthoredInputSignature, AuthoredTransformerDeclaration,
    AuthoredTransformerIdentity, MacroKind, NameTransform, SectionDefault, TemplateFieldValue,
    TemplateFuture, TemplateLanguage, TemplateTerm, TemplateValue,
};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

fn golden_root() -> PathBuf {
    PathBuf::from(
        std::env::var_os("FREEZE_D47_CORE").expect("FREEZE_D47_CORE names the core-nomos checkout"),
    )
    .join("tests/goldens")
}

fn universal(chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("fixture identity is nonempty")
}

fn binding() -> AuthoredBindingIdentity {
    AuthoredBindingIdentity::try_new(universal(&[47, 1, 2])).expect("Universal binding")
}

fn transformer() -> AuthoredTransformerIdentity {
    AuthoredTransformerIdentity::try_new(universal(&[47, 3, 4])).expect("Universal transformer")
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

fn named_invoke_declaration() -> AuthoredTransformerDeclaration {
    let logos = logos();
    let template =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("derive Template(Logos)");
    let constructor = template
        .type_declaration(template.root())
        .and_then(|declaration| declaration.constructors().first())
        .expect("attributes constructor");
    let field = constructor
        .landing_fields()
        .first()
        .expect("attributes sequence field");
    let name = transformer();
    let result = TemplateValue::try_new(
        constructor.constructor().clone(),
        vec![TemplateFieldValue::new(
            field.role(),
            TemplateTerm::Sequence(vec![TemplateTerm::Future(TemplateFuture::Invoke(
                name.clone(),
            ))]),
        )],
    )
    .expect("unique computed role");
    AuthoredTransformerDeclaration::try_new(
        name,
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        result,
        &template,
    )
    .expect("Named declaration carries one typed Invoke")
}

fn write(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).expect("write d47 legacy enum fixture");
}

#[test]
#[ignore = "one-shot d47 legacy enum fixture generator"]
fn freeze_d47_legacy_enum_serializations() {
    let root = golden_root();
    write(
        &root.join("d47_template_future_realize.bin"),
        &rkyv::to_bytes::<rkyv::rancor::Error>(&TemplateFuture::Realize {
            binding: binding(),
            transform: NameTransform::PascalCase,
        })
        .expect("archive legacy Realize"),
    );
    write(
        &root.join("d47_template_future_invoke.bin"),
        &rkyv::to_bytes::<rkyv::rancor::Error>(&TemplateFuture::Invoke(transformer()))
            .expect("archive legacy Invoke"),
    );
    write(
        &root.join("d47_template_future_splice.bin"),
        &rkyv::to_bytes::<rkyv::rancor::Error>(&TemplateFuture::Splice { binding: binding() })
            .expect("archive legacy Splice"),
    );
    write(
        &root.join("d47_macro_kind_named.bin"),
        &rkyv::to_bytes::<rkyv::rancor::Error>(&MacroKind::Named)
            .expect("archive legacy Named kind"),
    );
    write(
        &root.join("d47_macro_kind_structural.bin"),
        &rkyv::to_bytes::<rkyv::rancor::Error>(&MacroKind::Structural(SectionDefault::Enumeration))
            .expect("archive legacy Structural kind"),
    );
    write(
        &root.join("d47_named_invoke_declaration.bin"),
        &rkyv::to_bytes::<rkyv::rancor::Error>(&named_invoke_declaration())
            .expect("archive legacy Named Invoke declaration"),
    );
}

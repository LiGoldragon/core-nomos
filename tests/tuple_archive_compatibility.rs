use std::mem::{align_of, size_of};

use core_nomos::{
    AuthoredBindingIdentity, AuthoredInputParameter, AuthoredInputSignature,
    AuthoredTransformerDeclaration, AuthoredTransformerIdentity, MacroKind, MetaType,
    TemplateFutureOutput, TemplateFutureRequirement, TemplateLanguage, TemplateRootOutput,
    TemplateValue,
};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::LandingShape;
use textual_core_logos::{LogosLanguage, LogosLanguageTypeIds, LogosLanguageWords};

const NAMED_INVOKE_DECLARATION: &[u8] = include_bytes!("goldens/d47_named_invoke_declaration.bin");

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NamedAuthoredInputParameter {
    binding: AuthoredBindingIdentity,
    meta: MetaType,
    output: TemplateFutureOutput<VocabularyRoot>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
pub struct NamedAuthoredTransformerDeclaration {
    name: AuthoredTransformerIdentity,
    kind: MacroKind,
    input: AuthoredInputSignature,
    result: TemplateValue<VocabularyRoot>,
    output: TemplateRootOutput<VocabularyRoot>,
    future_requirements: Vec<TemplateFutureRequirement<VocabularyRoot>>,
}

macro_rules! assert_archive_compatible {
    ($production_type:ty, $named_type:ty, $production:expr, $named:expr) => {{
        let production = $production;
        let named = $named;
        let production_bytes =
            rkyv::to_bytes::<rkyv::rancor::Error>(&production).expect("archive production carrier");
        let named_bytes =
            rkyv::to_bytes::<rkyv::rancor::Error>(&named).expect("archive named-field mirror");

        assert_eq!(
            production_bytes.as_slice(),
            named_bytes.as_slice(),
            "tuple and named-field carriers must emit identical bytes",
        );
        assert_eq!(
            size_of::<rkyv::Archived<$production_type>>(),
            size_of::<rkyv::Archived<$named_type>>(),
            "archived sizes must match",
        );
        assert_eq!(
            align_of::<rkyv::Archived<$production_type>>(),
            align_of::<rkyv::Archived<$named_type>>(),
            "archived alignments must match",
        );

        let _: &rkyv::Archived<$production_type> =
            rkyv::access::<rkyv::Archived<$production_type>, rkyv::rancor::Error>(&named_bytes)
                .expect("access named bytes through production archived layout");
        let _: &rkyv::Archived<$named_type> =
            rkyv::access::<rkyv::Archived<$named_type>, rkyv::rancor::Error>(&production_bytes)
                .expect("access production bytes through named archived layout");

        let production_from_named =
            rkyv::from_bytes::<$production_type, rkyv::rancor::Error>(&named_bytes)
                .expect("restore production carrier from named bytes");
        let named_from_production =
            rkyv::from_bytes::<$named_type, rkyv::rancor::Error>(&production_bytes)
                .expect("restore named carrier from production bytes");
        assert_eq!(production_from_named, production);
        assert_eq!(named_from_production, named);

        assert_eq!(
            rkyv::to_bytes::<rkyv::rancor::Error>(&production_from_named)
                .expect("reserialize production carrier restored from named bytes")
                .as_slice(),
            named_bytes.as_slice(),
        );
        assert_eq!(
            rkyv::to_bytes::<rkyv::rancor::Error>(&named_from_production)
                .expect("reserialize named carrier restored from production bytes")
                .as_slice(),
            production_bytes.as_slice(),
        );
    }};
}

fn universal(chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("fixture identity is nonempty")
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

// Trait exception — the proper trait cannot be determined: this function is an
// entry point whose contract is supplied by Rust's test harness.
#[test]
fn named_fields_preserve_authored_archive_carriers() {
    let legacy = rkyv::from_bytes::<AuthoredTransformerDeclaration, rkyv::rancor::Error>(
        NAMED_INVOKE_DECLARATION,
    )
    .expect("restore d47 declaration fixture");
    let logos = logos();
    let language =
        TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.attributes_type())
            .expect("derive Template(Logos attributes)");
    let parameter = AuthoredInputParameter::new(
        AuthoredBindingIdentity::try_new(universal(&[61, 3, 5])).expect("Universal input binding"),
        MetaType::Name,
        TemplateFutureOutput::new(LandingShape::Declaration),
    );
    let input = AuthoredInputSignature::try_new(vec![parameter.clone()])
        .expect("one distinct input binding");
    let declaration = AuthoredTransformerDeclaration::try_new(
        legacy.name().clone(),
        legacy.kind(),
        input,
        legacy.result().clone(),
        &language,
    )
    .expect("meaningful declaration retains its Invoke requirement");

    assert!(!declaration.input().parameters().is_empty());
    assert!(!declaration.future_requirements().is_empty());

    assert_archive_compatible!(
        AuthoredInputParameter,
        NamedAuthoredInputParameter,
        parameter.clone(),
        NamedAuthoredInputParameter {
            binding: parameter.binding().clone(),
            meta: parameter.meta(),
            output: parameter.output().clone(),
        }
    );
    assert_archive_compatible!(
        AuthoredTransformerDeclaration,
        NamedAuthoredTransformerDeclaration,
        declaration.clone(),
        NamedAuthoredTransformerDeclaration {
            name: declaration.name().clone(),
            kind: declaration.kind(),
            input: declaration.input().clone(),
            result: declaration.result().clone(),
            output: declaration.root_output().clone(),
            future_requirements: declaration.future_requirements().to_vec(),
        }
    );
}

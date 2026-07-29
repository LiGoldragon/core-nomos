//! Durable witnesses for the authored-to-sealed Nomos phase boundary.

use core_nomos::{
    AuthoredAttribute, AuthoredBindingIdentity, AuthoredBindingRef, AuthoredConfigurationAttribute,
    AuthoredConfigurationPredicate, AuthoredDeriveGroup, AuthoredEscape, AuthoredIdentityPosition,
    AuthoredInputParameter, AuthoredInputSignature, AuthoredItemSkeleton, AuthoredNewtypeSkeleton,
    AuthoredNomosError, AuthoredPath, AuthoredRealize, AuthoredResultSkeleton, AuthoredScalar,
    AuthoredSequence, AuthoredSequenceItem, AuthoredTransformerDeclaration,
    AuthoredTransformerIdentity, AuthoredVisibility, MacroKind, MetaType, NameTransform,
    SectionDefault,
};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

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

fn input_ref(binding: &AuthoredBindingIdentity) -> AuthoredBindingRef {
    AuthoredBindingRef::Input(binding.clone())
}

#[test]
fn wire_newtype_retains_complete_declaration_binding_and_invoke_chains() {
    let wire_newtype = transformer(&[40, 7, 1]);
    let wire_attributes = transformer(&[40, 7, 2]);
    let name = binding(&[40, 7, 1, 1]);
    let wrapped = binding(&[40, 7, 1, 2]);
    let input = AuthoredInputSignature::try_new(vec![
        AuthoredInputParameter::new(name.clone(), MetaType::Name),
        AuthoredInputParameter::new(wrapped.clone(), MetaType::Type),
    ])
    .expect("distinct positional bindings");
    let declaration = AuthoredTransformerDeclaration::try_new(
        wire_newtype.clone(),
        MacroKind::Structural(SectionDefault::Newtype),
        input,
        AuthoredResultSkeleton::Item(AuthoredItemSkeleton::Newtype(AuthoredNewtypeSkeleton::new(
            AuthoredVisibility::Public,
            AuthoredSequence::of(AuthoredSequenceItem::Escape(AuthoredEscape::Invoke(
                wire_attributes.clone(),
            ))),
            AuthoredScalar::Escape(AuthoredEscape::Realize(AuthoredRealize::new(
                input_ref(&name),
                NameTransform::Identity,
            ))),
            AuthoredScalar::Escape(AuthoredEscape::Realize(AuthoredRealize::new(
                input_ref(&wrapped),
                NameTransform::Identity,
            ))),
        ))),
    )
    .expect("all typed escapes bind declared inputs");

    let bytes =
        rkyv::to_bytes::<rkyv::rancor::Error>(&declaration).expect("archive authored declaration");
    let restored = rkyv::from_bytes::<AuthoredTransformerDeclaration, rkyv::rancor::Error>(&bytes)
        .expect("restore authored declaration");
    assert_eq!(restored, declaration);
    assert_eq!(
        restored.name().encoded_id(),
        wire_newtype.encoded_id(),
        "the declaration keeps its complete module chain"
    );

    let AuthoredResultSkeleton::Item(AuthoredItemSkeleton::Newtype(skeleton)) = restored.result()
    else {
        panic!("newtype skeleton");
    };
    let [AuthoredSequenceItem::Escape(AuthoredEscape::Invoke(target))] =
        skeleton.attributes().items()
    else {
        panic!("one durable attributes invocation");
    };
    assert_eq!(
        target.encoded_id(),
        wire_attributes.encoded_id(),
        "Invoke retains durable target identity before package-local rebinding"
    );
    assert_eq!(
        restored.input().parameters()[0]
            .binding()
            .encoded_id()
            .chain(),
        &[
            LocalEncodedId::new(40),
            LocalEncodedId::new(7),
            LocalEncodedId::new(1),
            LocalEncodedId::new(1)
        ]
    );
}

#[test]
fn wire_attributes_literals_round_trip_as_full_chain_typed_logos_data() {
    let rustfmt_skip =
        AuthoredPath::try_new(vec![rust(&[30, 1]), rust(&[30, 2])]).expect("tool path");
    let nota_feature = universal(&[41, 3]);
    let nota_derives = AuthoredDeriveGroup::new(
        [&[31, 1][..], &[31, 2][..], &[31, 3][..]]
            .into_iter()
            .map(|chain| AuthoredPath::try_new(vec![rust(chain)]).expect("derive path"))
            .collect(),
    );
    let rkyv_derives = AuthoredDeriveGroup::new(
        [
            &[32, 1][..],
            &[32, 2][..],
            &[32, 3][..],
            &[32, 4][..],
            &[32, 5][..],
            &[32, 6][..],
            &[32, 7][..],
        ]
        .into_iter()
        .map(|chain| AuthoredPath::try_new(vec![rust(chain)]).expect("derive path"))
        .collect(),
    );
    let declaration = AuthoredTransformerDeclaration::try_new(
        transformer(&[40, 7, 2]),
        MacroKind::Named,
        AuthoredInputSignature::unit(),
        AuthoredResultSkeleton::Attributes(AuthoredSequence::new(vec![
            AuthoredSequenceItem::Literal(AuthoredAttribute::ToolPath(rustfmt_skip)),
            AuthoredSequenceItem::Literal(AuthoredAttribute::Configuration(
                AuthoredConfigurationAttribute::new(
                    AuthoredConfigurationPredicate::Feature(nota_feature.clone()),
                    AuthoredAttribute::Derive(nota_derives),
                ),
            )),
            AuthoredSequenceItem::Literal(AuthoredAttribute::Derive(rkyv_derives)),
        ])),
    )
    .expect("literal attribute skeleton has no input bindings");

    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&declaration)
        .expect("archive attributes declaration");
    let restored = rkyv::from_bytes::<AuthoredTransformerDeclaration, rkyv::rancor::Error>(&bytes)
        .expect("restore attributes declaration");
    assert_eq!(restored, declaration);

    let AuthoredResultSkeleton::Attributes(attributes) = restored.result() else {
        panic!("attribute-vector result");
    };
    let AuthoredSequenceItem::Literal(AuthoredAttribute::Configuration(configuration)) =
        &attributes.items()[1]
    else {
        panic!("conditional derive");
    };
    let AuthoredConfigurationPredicate::Feature(feature) = configuration.predicate();
    assert_eq!(
        feature, &nota_feature,
        "feature identity remains a complete Universal chain"
    );
}

#[test]
fn authored_identity_and_binding_failures_are_typed_and_pre_mutation() {
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
            AuthoredInputParameter::new(repeated.clone(), MetaType::Name),
            AuthoredInputParameter::new(repeated.clone(), MetaType::Type),
        ]),
        Err(AuthoredNomosError::DuplicateBinding {
            binding: repeated.encoded_id().clone(),
        })
    );

    let absent = binding(&[40, 8, 9]);
    let result =
        AuthoredResultSkeleton::Item(AuthoredItemSkeleton::Newtype(AuthoredNewtypeSkeleton::new(
            AuthoredVisibility::Public,
            AuthoredSequence::new(Vec::new()),
            AuthoredScalar::Escape(AuthoredEscape::Realize(AuthoredRealize::new(
                input_ref(&absent),
                NameTransform::Identity,
            ))),
            AuthoredScalar::Escape(AuthoredEscape::Realize(AuthoredRealize::new(
                input_ref(&absent),
                NameTransform::Identity,
            ))),
        )));
    assert_eq!(
        AuthoredTransformerDeclaration::try_new(
            transformer(&[40, 8]),
            MacroKind::Structural(SectionDefault::Newtype),
            AuthoredInputSignature::unit(),
            result,
        ),
        Err(AuthoredNomosError::UndeclaredBinding {
            binding: absent.encoded_id().clone(),
        })
    );
}

#[test]
fn authored_carrier_has_no_flat_or_spelling_bearing_identity_surface() {
    let source = include_str!("../src/authored.rs");
    for forbidden in [
        "name_table::Identifier",
        "IdentifierNamespace",
        "NameTable",
        "String>",
        "String,",
        "&str",
    ] {
        assert!(
            !source.contains(forbidden),
            "authored stage must not contain {forbidden}"
        );
    }
}

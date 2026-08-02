//! Focused witnesses for the non-recursive ScopeOf pre-gate slice.

use core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosEnumeration, WholeEthosItem, WholeEthosNewtype,
    WholeEthosTupleFields, WholeEthosTypeApplication, WholeEthosTypeReference, WholeEthosVariant,
    WholeEthosVariantPayload, WholeEthosVisibility, WholeEthosWrappedField,
};
use core_logos::WholeLogosVisibility;
use core_nomos::{
    ScopeOfDeclarationContract, ScopeOfDeclarationRecognition, ScopeOfGate,
    ScopeOfGateObservations, ScopeOfLogosRealization, ScopeOfNomosPlanning,
    ScopeOfNomosVariantPayload, ScopeOfRefusal, ScopeOfSourceResolution, ScopeOfTransformer,
};
use encoded_name_table::LocalEncodedId;
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

fn encoded(chain: &[u16]) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        chain.iter().copied().map(LocalEncodedId::new).collect(),
    )
    .expect("fixture identities are complete")
}

#[derive(Clone)]
struct ScopeOfFixture {
    scope_of: VocabularyEncodedId,
    domain_scope: VocabularyEncodedId,
    domain: VocabularyEncodedId,
    all: VocabularyEncodedId,
    technology: VocabularyEncodedId,
    technology_domain: VocabularyEncodedId,
    ethos: WholeEthos,
}

impl ScopeOfFixture {
    fn new() -> Self {
        let scope_of = encoded(&[7]);
        let domain_scope = encoded(&[42, 3]);
        let domain = encoded(&[42, 1]);
        let all = encoded(&[42, 1, 1]);
        let technology = encoded(&[42, 1, 2]);
        let technology_domain = encoded(&[42, 2]);
        let domain_enumeration = WholeEthosEnumeration::new(
            domain.clone(),
            WholeEthosVisibility::Public,
            WholeEthosAttributes::empty(),
            vec![
                WholeEthosVariant::new(
                    all.clone(),
                    WholeEthosAttributes::empty(),
                    WholeEthosVariantPayload::Unit,
                ),
                WholeEthosVariant::new(
                    technology.clone(),
                    WholeEthosAttributes::empty(),
                    WholeEthosVariantPayload::Tuple(
                        WholeEthosTupleFields::new(vec![WholeEthosTypeReference::Identity(
                            technology_domain.clone(),
                        )])
                        .expect("one positional child"),
                    ),
                ),
            ],
        );
        let declaration = WholeEthosNewtype::new(
            domain_scope.clone(),
            WholeEthosVisibility::Public,
            WholeEthosAttributes::empty(),
            WholeEthosWrappedField::new(
                WholeEthosVisibility::Private,
                WholeEthosTypeReference::Application(WholeEthosTypeApplication::new(
                    scope_of.clone(),
                    WholeEthosTypeReference::Identity(domain.clone()),
                )),
            ),
        );
        Self {
            scope_of,
            domain_scope,
            domain,
            all,
            technology,
            technology_domain,
            ethos: WholeEthos::new(vec![
                WholeEthosItem::Enumeration(domain_enumeration),
                WholeEthosItem::Newtype(declaration),
            ]),
        }
    }

    fn transformer(&self) -> ScopeOfTransformer {
        ScopeOfTransformer::try_new(self.scope_of.clone(), self.all.clone())
            .expect("Universal fixture configuration")
    }
}

#[test]
fn exact_typed_application_is_recognized_without_spelling() {
    let fixture = ScopeOfFixture::new();
    let transformer = fixture.transformer();
    let items = fixture.ethos.items();

    assert_eq!(
        transformer
            .recognize(&items[0])
            .expect("unrelated enumeration is admissible"),
        None
    );
    let declaration = transformer
        .recognize(&items[1])
        .expect("typed application is well formed")
        .expect("exact ScopeOf application");
    assert_eq!(declaration.target(), &fixture.domain_scope);
    assert_eq!(declaration.source(), &fixture.domain);

    let different_head = ScopeOfTransformer::try_new(encoded(&[8]), fixture.all.clone())
        .expect("Universal alternate head");
    assert_eq!(
        different_head
            .recognize(&items[1])
            .expect("different typed head is unrelated"),
        None
    );
}

#[test]
fn source_resolution_is_exact_and_requires_an_enumeration() {
    let fixture = ScopeOfFixture::new();
    let transformer = fixture.transformer();
    let declaration = transformer
        .recognize(&fixture.ethos.items()[1])
        .expect("typed declaration")
        .expect("ScopeOf declaration");
    let source = transformer
        .resolve(&fixture.ethos, &declaration)
        .expect("exact source enumeration");

    assert_eq!(source.name(), &fixture.domain);
    assert_eq!(source.variants().len(), 2);
}

#[test]
fn root_plan_preserves_order_and_records_only_unresolved_dependencies() {
    let fixture = ScopeOfFixture::new();
    let transformer = fixture.transformer();
    let declaration = transformer
        .recognize(&fixture.ethos.items()[1])
        .expect("typed declaration")
        .expect("ScopeOf declaration");
    let source = transformer
        .resolve(&fixture.ethos, &declaration)
        .expect("source enumeration");
    let plan = transformer
        .plan_root(&declaration, source)
        .expect("one-level plan");

    assert_eq!(plan.visibility(), &WholeLogosVisibility::Public);
    assert_eq!(plan.name(), &fixture.domain_scope);
    assert_eq!(plan.variants().len(), 2);
    assert_eq!(plan.variants()[0].name().source_variant(), &fixture.all);
    assert_eq!(
        plan.variants()[0].payload(),
        &ScopeOfNomosVariantPayload::Unit
    );
    assert_eq!(
        plan.variants()[1].name().source_variant(),
        &fixture.technology
    );
    assert_eq!(
        plan.variants()[1].payload(),
        &ScopeOfNomosVariantPayload::Child {
            source_domain: fixture.technology_domain.clone(),
        }
    );
}

#[test]
fn identity_and_recursion_gates_are_typed_and_concrete_logos_refuses() {
    let fixture = ScopeOfFixture::new();
    let transformer = fixture.transformer();
    let declaration = transformer
        .recognize(&fixture.ethos.items()[1])
        .expect("typed declaration")
        .expect("ScopeOf declaration");
    let source = transformer
        .resolve(&fixture.ethos, &declaration)
        .expect("source enumeration");
    let plan = transformer
        .plan_root(&declaration, source)
        .expect("one-level plan");

    assert_eq!(
        plan.gates(),
        vec![
            ScopeOfGate::GeneratedOutputIdentity {
                source_variant: fixture.all.clone(),
            },
            ScopeOfGate::GeneratedOutputIdentity {
                source_variant: fixture.technology.clone(),
            },
            ScopeOfGate::RecursiveDescent {
                source_domain: fixture.technology_domain.clone(),
            },
        ]
    );
    assert_eq!(
        transformer.realize(&plan),
        Err(ScopeOfRefusal::GeneratedOutputIdentityRequired {
            source_variant: fixture.all,
        })
    );
}

#[test]
fn root_all_is_required_without_injection() {
    let fixture = ScopeOfFixture::new();
    let transformer = ScopeOfTransformer::try_new(fixture.scope_of.clone(), encoded(&[42, 1, 99]))
        .expect("Universal absent All configuration");
    let declaration = transformer
        .recognize(&fixture.ethos.items()[1])
        .expect("typed declaration")
        .expect("ScopeOf declaration");
    let source = transformer
        .resolve(&fixture.ethos, &declaration)
        .expect("source enumeration");

    assert_eq!(
        transformer.plan_root(&declaration, source),
        Err(ScopeOfRefusal::RootAllMissing {
            source_identity: fixture.domain,
        })
    );
}

#[test]
fn matching_head_with_application_operand_refuses_typed() {
    let fixture = ScopeOfFixture::new();
    let transformer = fixture.transformer();
    let malformed = WholeEthosItem::Newtype(WholeEthosNewtype::new(
        fixture.domain_scope.clone(),
        WholeEthosVisibility::Public,
        WholeEthosAttributes::empty(),
        WholeEthosWrappedField::new(
            WholeEthosVisibility::Private,
            WholeEthosTypeReference::Application(WholeEthosTypeApplication::new(
                fixture.scope_of,
                WholeEthosTypeReference::Application(WholeEthosTypeApplication::new(
                    encoded(&[9]),
                    WholeEthosTypeReference::Identity(fixture.domain),
                )),
            )),
        ),
    ));

    assert_eq!(
        transformer.recognize(&malformed),
        Err(ScopeOfRefusal::SourceOperandNotIdentity {
            target: fixture.domain_scope,
        })
    );
}

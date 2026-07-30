use std::collections::BTreeMap;
use std::sync::Arc;

use core_logos::{LogosLanguage, LogosLanguageTypeIds, LogosLanguageWords};
use core_nomos::{
    LoadedNomosDocument, LoadedNomosPopulation, MetaType, NameTreeProjectionVersion,
    NomosLoadError, NomosModulePath, SealedNomosPopulation, TemplateFutureOutput,
    TemplateLandingShape, TemplateLanguage, TextualNomos, TextualNomosMetaType,
    TextualNomosTypeIds, TextualNomosWords,
};
use encoded_name_table::{LocalEncodedId, Name, OperationKey};
use sema_translator::{DispatchOutcome, Runtime, StaticAuthorizationPolicy};
use signal_sema_translator::{
    AuthorityCapability, AuthorityOperation, AuthorityReply, AuthorityRequest,
    AuthorityRequestDigest, AuthorityRole, AuthorizationClaim, CommittedReceipt, DatabaseMarker,
    NoWriteFailure, PrincipalId, ReadOperation, Rename, RenameCommitReceipt, SealCommitReceipt,
    VocabularyEncodedId, VocabularyRoot, VocabularyTableAddress, WritePrecondition,
};
use structural_codec::{EncodedNameResolver, LandingShape};

const PRINCIPAL: PrincipalId = PrincipalId::new([23; 32]);
const SOURCE: &str = r#"{1}
[]
[]
{
WireAttributes.Named {
()
[]
}
WireNewtype.Structural.Newtype {
(name.Name wrapped.Type)
Public Invoke.WireAttributes Realize.name Private Realize.wrapped
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

fn textual(logos: &LogosLanguage) -> TextualNomos {
    let newtype = TemplateLanguage::derive(logos.grammar(), logos.landing(), logos.newtype_type())
        .expect("newtype Template(Logos)");
    let constructor = newtype
        .type_declaration(newtype.root())
        .and_then(|declaration| declaration.constructors().first())
        .expect("fixture newtype constructor");
    let field_output = |index: usize| {
        let shape = constructor
            .landing_fields()
            .get(index)
            .expect("fixture landing field")
            .shape();
        TemplateFutureOutput::new(literal_landing(shape))
    };

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
            newtype: encoded(&[101, 3]),
            structure: encoded(&[101, 8]),
            enumeration: encoded(&[101, 4]),
            realize: encoded(&[101, 5]),
            splice: encoded(&[101, 6]),
            invoke: encoded(&[101, 7]),
        },
        vec![
            TextualNomosMetaType {
                word: encoded(&[102, 1]),
                meta: MetaType::Name,
                output: field_output(2),
            },
            TextualNomosMetaType {
                word: encoded(&[102, 2]),
                meta: MetaType::Type,
                output: field_output(4),
            },
        ],
    )
    .expect("TextualNomos table seals")
}

struct FixedNames(BTreeMap<VocabularyEncodedId, Name>);

impl FixedNames {
    fn new() -> Self {
        Self(
            [
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
            ]
            .into_iter()
            .map(|(identity, spelling)| (identity, Name::new(spelling)))
            .collect(),
        )
    }
}

impl EncodedNameResolver<VocabularyRoot> for FixedNames {
    fn resolve(&self, encoded_id: &VocabularyEncodedId) -> Option<&Name> {
        self.0.get(encoded_id)
    }
}

fn operation_key(value: u8) -> [u8; 32] {
    [value; 32]
}

fn authorization(operation: &AuthorityOperation) -> AuthorizationClaim {
    let (role, capability) = match operation {
        AuthorityOperation::SealUniversal(_) => (
            AuthorityRole::UniversalAuthor,
            AuthorityCapability::SealUniversal,
        ),
        AuthorityOperation::Rename(_) => (
            AuthorityRole::UniversalMaintainer,
            AuthorityCapability::Rename,
        ),
        AuthorityOperation::Read(_) => (AuthorityRole::Reader, AuthorityCapability::Read),
        AuthorityOperation::PublishRustVocabulary(_) => (
            AuthorityRole::RustVocabularyPublisher,
            AuthorityCapability::PublishRustVocabulary,
        ),
    };
    AuthorizationClaim {
        principal: PRINCIPAL,
        role,
        capability,
    }
}

async fn request(runtime: &Runtime, operation: AuthorityOperation) -> DispatchOutcome {
    runtime
        .request(
            PRINCIPAL,
            AuthorityRequest {
                authorization: authorization(&operation),
                operation,
            },
        )
        .await
        .expect("authority request reaches the sole writer")
}

async fn current(runtime: &Runtime) -> DatabaseMarker {
    match request(runtime, AuthorityOperation::Read(ReadOperation::Current))
        .await
        .reply
    {
        AuthorityReply::Current(current) => current.database_marker,
        other => panic!("expected current authority, got {other:?}"),
    }
}

async fn current_snapshot(
    runtime: &Runtime,
    address: VocabularyTableAddress,
) -> signal_sema_translator::ObservedSnapshot {
    match request(
        runtime,
        AuthorityOperation::Read(ReadOperation::CurrentSnapshot { address }),
    )
    .await
    .reply
    {
        AuthorityReply::Snapshot(snapshot) => snapshot,
        other => panic!("expected current table snapshot, got {other:?}"),
    }
}

fn expected(marker: DatabaseMarker) -> WritePrecondition {
    WritePrecondition {
        database_marker: marker,
        table_generations: Vec::new(),
    }
}

async fn submit_plan(runtime: &Runtime, planned: &core_nomos::PlannedNomosLoad) -> DispatchOutcome {
    request(
        runtime,
        AuthorityOperation::SealUniversal(planned.request().clone()),
    )
    .await
}

async fn durable_receipt(runtime: &Runtime, operation_key: OperationKey) -> AuthorityReply {
    request(
        runtime,
        AuthorityOperation::Read(ReadOperation::CommittedReceipt { operation_key }),
    )
    .await
    .reply
}

fn seal_receipt(outcome: &DispatchOutcome) -> &SealCommitReceipt {
    match &outcome.reply {
        AuthorityReply::Committed(CommittedReceipt::SealUniversal(receipt)) => receipt,
        other => panic!("expected committed Nomos allocation, got {other:?}"),
    }
}

fn recovered_seal_receipt(reply: &AuthorityReply) -> &SealCommitReceipt {
    match reply {
        AuthorityReply::Receipt(CommittedReceipt::SealUniversal(receipt)) => receipt,
        other => panic!("expected recovered durable Nomos allocation, got {other:?}"),
    }
}

fn resolved_id(
    receipt: &SealCommitReceipt,
    modules: &[&str],
    spelling: &str,
) -> VocabularyEncodedId {
    receipt
        .name_table
        .declarations()
        .iter()
        .find(|resolved| {
            resolved.path().spelling().as_str() == spelling
                && resolved
                    .path()
                    .table()
                    .modules()
                    .iter()
                    .map(Name::as_str)
                    .eq(modules.iter().copied())
        })
        .unwrap_or_else(|| panic!("missing declaration {modules:?}/{spelling}"))
        .encoded_id()
        .clone()
}

fn transformer_id(loaded: &LoadedNomosDocument, spelling: &str) -> VocabularyEncodedId {
    loaded
        .transformers()
        .declarations()
        .iter()
        .find(|declaration| {
            loaded.names().spelling(declaration.name().encoded_id()) == Some(spelling)
        })
        .expect("loaded transformer spelling")
        .name()
        .encoded_id()
        .clone()
}

async fn open_runtime(directory: &tempfile::TempDir) -> Runtime {
    Runtime::open(
        &directory.path().join("nomos-authority.sema"),
        Arc::new(StaticAuthorizationPolicy::new().grant_all(PRINCIPAL)),
    )
    .await
    .expect("authority runtime opens")
}

#[tokio::test]
async fn one_seal_allocates_nested_nomos_chains_and_materializes_only_from_its_receipt() {
    let directory = tempfile::tempdir().expect("temporary authority directory");
    let runtime = open_runtime(&directory).await;
    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();
    let planned = textual
        .plan_load(
            SOURCE,
            &fixed,
            NomosModulePath::try_from_spellings(["fixture"]).expect("module path"),
            operation_key(1),
            expected(current(&runtime).await),
        )
        .expect("allocation-free structural plan");

    assert_eq!(planned.request().declarations.len(), 1);
    assert_eq!(planned.request().references.len(), 3);
    let outcome = submit_plan(&runtime, &planned).await;
    let receipt = seal_receipt(&outcome);
    let replay = submit_plan(&runtime, &planned).await;
    assert_eq!(seal_receipt(&replay), receipt);
    let durable = durable_receipt(&runtime, planned.request().operation_key).await;
    assert_eq!(recovered_seal_receipt(&durable), receipt);
    let loaded = textual
        .complete_load(&planned, &durable, &fixed)
        .expect("durable receipt-backed immutable decode");

    let module = resolved_id(receipt, &[], "fixture");
    let attributes = resolved_id(receipt, &["fixture"], "WireAttributes");
    let newtype = resolved_id(receipt, &["fixture"], "WireNewtype");
    let name = resolved_id(receipt, &["fixture", "WireNewtype"], "name");
    let wrapped = resolved_id(receipt, &["fixture", "WireNewtype"], "wrapped");
    assert_eq!(module.chain().len(), 1);
    assert_eq!(attributes.chain().len(), 2);
    assert_eq!(newtype.chain().len(), 2);
    assert_eq!(name.chain().len(), 3);
    assert_eq!(wrapped.chain().len(), 3);
    assert_eq!(name.chain()[..2], newtype.chain()[..]);
    assert_eq!(wrapped.chain()[..2], newtype.chain()[..]);
    assert_eq!(transformer_id(&loaded, "WireNewtype"), newtype);
    let viewed = textual
        .view(loaded.decoded(), loaded.names())
        .expect("loaded sibling renders every encoded name");
    assert!(viewed.contains("WireAttributes.Named"));
    assert!(viewed.contains("WireNewtype.Structural.Newtype"));
    assert!(viewed.contains("Invoke.WireAttributes"));
    assert!(viewed.contains("Realize.wrapped"));
    let archived =
        rkyv::to_bytes::<rkyv::rancor::Error>(loaded.names()).expect("archive name sibling");
    let restored = rkyv::from_bytes::<core_nomos::NomosNameTable, rkyv::rancor::Error>(&archived)
        .expect("restore name sibling");
    assert_eq!(restored, *loaded.names());

    runtime.shutdown().await.expect("runtime shuts down");
    let runtime = open_runtime(&directory).await;
    let recovered = durable_receipt(&runtime, planned.request().operation_key).await;
    assert_eq!(recovered, durable);
    let recovered_loaded = textual
        .complete_load(&planned, &recovered, &fixed)
        .expect("restart-recovered durable receipt materializes");
    assert_eq!(
        recovered_loaded
            .population()
            .content_identity()
            .expect("content identity"),
        loaded
            .population()
            .content_identity()
            .expect("content identity")
    );
    runtime
        .shutdown()
        .await
        .expect("restarted runtime shuts down");
}

#[tokio::test]
async fn text_edit_remints_while_operational_rename_changes_only_the_name_sibling() {
    let directory = tempfile::tempdir().expect("temporary authority directory");
    let runtime = open_runtime(&directory).await;
    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();
    let initial_plan = textual
        .plan_load(
            SOURCE,
            &fixed,
            NomosModulePath::try_from_spellings(["fixture"]).expect("module path"),
            operation_key(2),
            expected(current(&runtime).await),
        )
        .expect("initial plan");
    let initial_outcome = submit_plan(&runtime, &initial_plan).await;
    let initial_receipt = seal_receipt(&initial_outcome);
    let initial_durable = durable_receipt(&runtime, initial_plan.request().operation_key).await;
    let mut initial = textual
        .complete_load(&initial_plan, &initial_durable, &fixed)
        .expect("initial load");
    let initial_transformer = transformer_id(&initial, "WireNewtype");
    let wrapped = resolved_id(initial_receipt, &["fixture", "WireNewtype"], "wrapped");
    let sealed_before = initial
        .population()
        .seal(NameTreeProjectionVersion::initial())
        .expect("receipt-validated population seals");
    let capsule_bytes_before = sealed_before
        .capsule()
        .to_archive_bytes()
        .expect("archive immutable Capsule");
    let invalid_chain_refused = (0..capsule_bytes_before.len().saturating_sub(3)).any(|offset| {
        let mut mutation = capsule_bytes_before.clone();
        mutation[offset..offset + 4].fill(0);
        matches!(
            core_nomos::SealedNomosCapsule::from_archive_bytes(&mutation),
            Err(core_nomos::NomosSealError::InvalidEncodedChain { .. })
        )
    });
    assert!(
        invalid_chain_refused,
        "an empty encoded-chain archive mutation must reach typed refusal"
    );

    let edited_source = SOURCE.replace("WireNewtype", "WireWrapped");
    let edited_plan = textual
        .plan_load(
            &edited_source,
            &fixed,
            NomosModulePath::try_from_spellings(["fixture"]).expect("module path"),
            operation_key(3),
            expected(current(&runtime).await),
        )
        .expect("text-edit plan");
    let edited_outcome = submit_plan(&runtime, &edited_plan).await;
    let edited_receipt = seal_receipt(&edited_outcome);
    let edited_durable = durable_receipt(&runtime, edited_plan.request().operation_key).await;
    assert_eq!(recovered_seal_receipt(&edited_durable), edited_receipt);
    let edited = textual
        .complete_load(&edited_plan, &edited_durable, &fixed)
        .expect("text-edit load");
    let edited_transformer = transformer_id(&edited, "WireWrapped");
    assert_ne!(initial_transformer, edited_transformer);
    let edited_sealed = edited
        .population()
        .seal(NameTreeProjectionVersion::initial())
        .expect("changed authorship seals");
    assert_ne!(
        sealed_before.capsule().content_identity(),
        edited_sealed.capsule().content_identity()
    );
    let owning_snapshot = current_snapshot(&runtime, initial_transformer.owning_table()).await;
    assert!(
        owning_snapshot
            .snapshot
            .entries()
            .iter()
            .any(|spelling| spelling.as_str() == "WireNewtype")
    );
    assert!(
        owning_snapshot
            .snapshot
            .entries()
            .iter()
            .any(|spelling| spelling.as_str() == "WireWrapped")
    );

    let renamed = request(
        &runtime,
        AuthorityOperation::Rename(Rename {
            operation_key: OperationKey::new(operation_key(4)),
            expected: expected(current(&runtime).await),
            target: wrapped.clone(),
            new_spelling: Name::new("inner"),
        }),
    )
    .await;
    let rename_receipt: &RenameCommitReceipt = match &renamed.reply {
        AuthorityReply::Committed(CommittedReceipt::Rename(receipt)) => receipt,
        other => panic!("expected committed operational rename, got {other:?}"),
    };
    initial
        .apply_rename(rename_receipt)
        .expect("loaded sibling accepts the committed rename");
    let sealed_after = initial
        .population()
        .advance_projection(&sealed_before)
        .expect("rename advances projection only");
    assert_eq!(initial.names().spelling(&wrapped), Some("inner"));
    assert_eq!(
        initial
            .population()
            .content_identity()
            .expect("content identity"),
        sealed_before.capsule().content_identity()
    );
    let binding_after = initial
        .transformers()
        .declarations()
        .iter()
        .find(|declaration| declaration.name().encoded_id() == &initial_transformer)
        .expect("initial transformer")
        .input()
        .parameters()
        .iter()
        .find(|parameter| parameter.binding().encoded_id() == &wrapped)
        .expect("renamed binding");
    assert_eq!(binding_after.binding().encoded_id(), &wrapped);
    assert_eq!(
        sealed_after.capsule().content_identity(),
        sealed_before.capsule().content_identity()
    );
    assert_eq!(
        sealed_after
            .capsule()
            .to_archive_bytes()
            .expect("archive renamed Capsule"),
        capsule_bytes_before
    );
    assert_eq!(
        sealed_after.projection().version(),
        NameTreeProjectionVersion::new(1)
    );
    assert_ne!(
        sealed_after.projection().integrity_bytes(),
        sealed_before.projection().integrity_bytes()
    );
    let rendered = sealed_after
        .projection()
        .render_chain(&wrapped)
        .expect("projection renders every ancestor");
    assert_eq!(rendered, ["fixture", "WireNewtype", "inner"]);
    assert_eq!(
        sealed_after
            .projection()
            .resolve_chain(VocabularyRoot::Universal, &rendered),
        Some(wrapped.clone())
    );

    let projected_population = LoadedNomosPopulation::from_typed(
        initial.transformers().clone(),
        sealed_after.projection().to_name_table(),
    );
    let resealed = projected_population
        .seal(sealed_after.projection().version())
        .expect("rendered projection reseals");
    assert_eq!(resealed, sealed_after);
    let restored = SealedNomosPopulation::from_archive_parts(
        &sealed_after
            .capsule()
            .to_archive_bytes()
            .expect("archive Capsule"),
        &sealed_after
            .projection()
            .to_archive_bytes()
            .expect("archive projection"),
    )
    .expect("restore independently persisted seal parts");
    assert_eq!(restored, sealed_after);

    let ancestor_plan = textual
        .plan_load(
            SOURCE,
            &fixed,
            NomosModulePath::try_from_spellings(["other"]).expect("alternate module path"),
            operation_key(5),
            expected(current(&runtime).await),
        )
        .expect("ancestor-edit plan");
    let ancestor_outcome = submit_plan(&runtime, &ancestor_plan).await;
    let ancestor_durable = durable_receipt(&runtime, ancestor_plan.request().operation_key).await;
    let ancestor = textual
        .complete_load(&ancestor_plan, &ancestor_durable, &fixed)
        .expect("ancestor-edit load");
    assert!(matches!(
        ancestor_outcome.reply,
        AuthorityReply::Committed(CommittedReceipt::SealUniversal(_))
    ));
    let ancestor_sealed = ancestor
        .population()
        .seal(NameTreeProjectionVersion::initial())
        .expect("ancestor-edited population seals");
    assert_ne!(
        ancestor_sealed.capsule().content_identity(),
        sealed_before.capsule().content_identity()
    );

    runtime.shutdown().await.expect("runtime shuts down");
}

#[tokio::test]
async fn unresolved_lookup_and_wrong_receipt_refuse_without_a_loaded_document() {
    let directory = tempfile::tempdir().expect("temporary authority directory");
    let runtime = open_runtime(&directory).await;
    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();
    let valid = textual
        .plan_load(
            SOURCE,
            &fixed,
            NomosModulePath::try_from_spellings(["fixture"]).expect("module path"),
            operation_key(5),
            expected(current(&runtime).await),
        )
        .expect("valid plan");
    let valid_outcome = submit_plan(&runtime, &valid).await;
    let valid_receipt = seal_receipt(&valid_outcome);
    let valid_durable = durable_receipt(&runtime, valid.request().operation_key).await;
    assert_eq!(recovered_seal_receipt(&valid_durable), valid_receipt);

    let wrong_receipt_plan = textual
        .plan_load(
            SOURCE,
            &fixed,
            NomosModulePath::try_from_spellings(["fixture"]).expect("module path"),
            operation_key(6),
            expected(current(&runtime).await),
        )
        .expect("second plan");
    assert!(matches!(
        textual.complete_load(&wrong_receipt_plan, &valid_durable, &fixed),
        Err(NomosLoadError::ReceiptOperationKeyMismatch)
    ));

    let unresolved_source = SOURCE.replace("Invoke.WireAttributes", "Invoke.Missing");
    let before = current(&runtime).await;
    let unresolved = textual
        .plan_load(
            &unresolved_source,
            &fixed,
            NomosModulePath::try_from_spellings(["fixture"]).expect("module path"),
            operation_key(7),
            expected(before),
        )
        .expect("lookup-only reference remains allocation-free in the plan");
    let refused = submit_plan(&runtime, &unresolved).await;
    assert!(matches!(
        refused.reply,
        AuthorityReply::Rejected(NoWriteFailure::UnresolvedReference { .. })
    ));
    assert_eq!(current(&runtime).await, before);

    runtime.shutdown().await.expect("runtime shuts down");
}

#[tokio::test]
async fn materialization_refuses_non_durable_digest_marker_path_and_multiplicity_substitution() {
    let directory = tempfile::tempdir().expect("temporary authority directory");
    let runtime = open_runtime(&directory).await;
    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();
    let planned = textual
        .plan_load(
            SOURCE,
            &fixed,
            NomosModulePath::try_from_spellings(["fixture"]).expect("module path"),
            operation_key(8),
            expected(current(&runtime).await),
        )
        .expect("valid plan");
    let committed = submit_plan(&runtime, &planned).await;
    assert!(matches!(
        textual.complete_load(&planned, &committed.reply, &fixed),
        Err(NomosLoadError::ReceiptNotDurableSeal)
    ));
    let durable = durable_receipt(&runtime, planned.request().operation_key).await;
    textual
        .complete_load(&planned, &durable, &fixed)
        .expect("exact durable receipt succeeds");
    let expected_digest = planned
        .request()
        .canonical_request_digest()
        .expect("canonical plan digest");

    let mut wrong_digest = durable.clone();
    let AuthorityReply::Receipt(CommittedReceipt::SealUniversal(receipt)) = &mut wrong_digest
    else {
        unreachable!()
    };
    receipt.request_digest = AuthorityRequestDigest::new([0xD1; 32]);
    assert!(matches!(
        textual.complete_load(&planned, &wrong_digest, &fixed),
        Err(NomosLoadError::ReceiptRequestDigestMismatch { expected, found })
            if expected == expected_digest && found == AuthorityRequestDigest::new([0xD1; 32])
    ));

    let mut wrong_marker = durable.clone();
    let AuthorityReply::Receipt(CommittedReceipt::SealUniversal(receipt)) = &mut wrong_marker
    else {
        unreachable!()
    };
    receipt.database_marker = receipt
        .database_marker
        .checked_successor()
        .expect("fixture marker has a future successor");
    assert!(matches!(
        textual.complete_load(&planned, &wrong_marker, &fixed),
        Err(NomosLoadError::ReceiptDatabaseMarkerMismatch { expected, found })
            if expected == planned.request().expected.database_marker.checked_successor().unwrap()
                && found == recovered_seal_receipt(&wrong_marker).database_marker
    ));

    let substituted_directory = tempfile::tempdir().expect("substituted authority directory");
    let substituted_runtime = open_runtime(&substituted_directory).await;
    let substituted_source = SOURCE.replace("WireAttributes", "WireDecorations");
    let substituted_plan = textual
        .plan_load(
            &substituted_source,
            &fixed,
            NomosModulePath::try_from_spellings(["fixture"]).expect("module path"),
            operation_key(8),
            expected(current(&substituted_runtime).await),
        )
        .expect("substituted path plan");
    submit_plan(&substituted_runtime, &substituted_plan).await;
    let mut substituted = durable_receipt(
        &substituted_runtime,
        substituted_plan.request().operation_key,
    )
    .await;
    let AuthorityReply::Receipt(CommittedReceipt::SealUniversal(receipt)) = &mut substituted else {
        unreachable!()
    };
    receipt.request_digest = expected_digest;
    assert!(matches!(
        textual.complete_load(&planned, &substituted, &fixed),
        Err(NomosLoadError::ReceiptDeclarationMismatch { .. })
    ));

    let duplicate_directory = tempfile::tempdir().expect("duplicate authority directory");
    let duplicate_runtime = open_runtime(&duplicate_directory).await;
    let mut duplicate_request = planned.request().clone();
    duplicate_request
        .references
        .push(duplicate_request.references[0].clone());
    let duplicate_operation_key = duplicate_request.operation_key;
    let duplicate_outcome = request(
        &duplicate_runtime,
        AuthorityOperation::SealUniversal(duplicate_request),
    )
    .await;
    assert!(matches!(
        duplicate_outcome.reply,
        AuthorityReply::Committed(CommittedReceipt::SealUniversal(_))
    ));
    let mut duplicate = durable_receipt(&duplicate_runtime, duplicate_operation_key).await;
    let AuthorityReply::Receipt(CommittedReceipt::SealUniversal(receipt)) = &mut duplicate else {
        unreachable!()
    };
    receipt.request_digest = expected_digest;
    assert!(matches!(
        textual.complete_load(&planned, &duplicate, &fixed),
        Err(NomosLoadError::ReceiptReferenceMismatch {
            expected: 3,
            found: 4
        })
    ));

    duplicate_runtime
        .shutdown()
        .await
        .expect("duplicate runtime shuts down");
    substituted_runtime
        .shutdown()
        .await
        .expect("substituted runtime shuts down");
    runtime.shutdown().await.expect("runtime shuts down");
}

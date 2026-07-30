use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::sync::Arc;

use core_logos::{LogosLanguage, LogosLanguageTypeIds, LogosLanguageWords};
use core_nomos::{
    AuthoredTransformerSet, LoadedNomosPopulation, MetaType, NameTreeProjectionVersion,
    NomosFileManifest, NomosManifestFile, NomosManifestLoadError, NomosModulePath, NomosSourcePath,
    TemplateFutureOutput, TemplateLandingShape, TemplateLanguage, TextualNomos,
    TextualNomosMetaType, TextualNomosTypeIds, TextualNomosWords,
};
use encoded_name_table::{LocalEncodedId, Name, OperationKey};
use sema_translator::{DispatchOutcome, Runtime, StaticAuthorizationPolicy};
use signal_sema_translator::{
    AuthorityCapability, AuthorityOperation, AuthorityReply, AuthorityRequest, AuthorityRole,
    AuthorizationClaim, CommittedReceipt, DatabaseMarker, NoWriteFailure, PrincipalId,
    ReadOperation, SealCommitReceipt, VocabularyEncodedId, VocabularyRoot, WritePrecondition,
};
use structural_codec::{EncodedNameResolver, LandingShape};

const PRINCIPAL: PrincipalId = PrincipalId::new([24; 32]);
const ATTRIBUTES: &str = r#"{1}
[]
[]
{
WireAttributes.Named {
()
[]
}
}
{}
{}"#;
const NEWTYPE: &str = r#"{1}
[]
[]
{
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
                (encoded(&[101, 9]), "Recursive"),
                (encoded(&[101, 10]), "InsertAt"),
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

fn source_path(path: &str) -> NomosSourcePath {
    NomosSourcePath::try_new(path).expect("valid relative Nomos path")
}

fn module() -> NomosModulePath {
    NomosModulePath::try_from_spellings(["fixture"]).expect("module path")
}

fn manifest(reverse: bool) -> NomosFileManifest {
    let attributes = NomosManifestFile(source_path("attributes.nomos"), module(), vec![]);
    let entry = NomosManifestFile(
        source_path("entry.nomos"),
        module(),
        vec![source_path("attributes.nomos")],
    );
    let files = if reverse {
        vec![entry, attributes]
    } else {
        vec![attributes, entry]
    };
    NomosFileManifest(source_path("entry.nomos"), files)
}

fn write_sources(directory: &tempfile::TempDir) {
    fs::write(directory.path().join("attributes.nomos"), ATTRIBUTES)
        .expect("attributes fixture writes");
    fs::write(directory.path().join("entry.nomos"), NEWTYPE).expect("entry fixture writes");
}

fn authorization(operation: &AuthorityOperation) -> AuthorizationClaim {
    let (role, capability) = match operation {
        AuthorityOperation::SealUniversal(_) => (
            AuthorityRole::UniversalAuthor,
            AuthorityCapability::SealUniversal,
        ),
        AuthorityOperation::Read(_) => (AuthorityRole::Reader, AuthorityCapability::Read),
        AuthorityOperation::Rename(_) => (
            AuthorityRole::UniversalMaintainer,
            AuthorityCapability::Rename,
        ),
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

fn expected(marker: DatabaseMarker) -> WritePrecondition {
    WritePrecondition {
        database_marker: marker,
        table_generations: Vec::new(),
    }
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
        other => panic!("expected committed manifest allocation, got {other:?}"),
    }
}

async fn open_runtime(directory: &tempfile::TempDir) -> Runtime {
    Runtime::open(
        &directory.path().join("nomos-manifest.sema"),
        Arc::new(StaticAuthorizationPolicy::new().grant_all(PRINCIPAL)),
    )
    .await
    .expect("authority runtime opens")
}

// [not-understood-by-psyche, Entry 7, NomosTrainAddendum-2026-07-30]
// A previously committed transformer is external to a later manifest package,
// even when both manifests use the same authored module.
#[tokio::test]
async fn invoke_target_from_prior_manifest_refuses_before_any_new_authority_write() {
    let source_directory = tempfile::tempdir().expect("temporary source directory");
    write_sources(&source_directory);
    let authority_directory = tempfile::tempdir().expect("authority directory");
    let runtime = open_runtime(&authority_directory).await;
    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();

    let seed_manifest = NomosFileManifest(
        source_path("attributes.nomos"),
        vec![NomosManifestFile(
            source_path("attributes.nomos"),
            module(),
            vec![],
        )],
    );
    let seed = textual
        .plan_file_population(
            source_directory.path(),
            &seed_manifest,
            &fixed,
            [36; 32],
            expected(current(&runtime).await),
        )
        .expect("seed package plans");
    let committed = request(
        &runtime,
        AuthorityOperation::SealUniversal(seed.request().clone()),
    )
    .await;
    seal_receipt(&committed);

    let before = current(&runtime).await;
    let external_invoke_manifest = NomosFileManifest(
        source_path("entry.nomos"),
        vec![NomosManifestFile(
            source_path("entry.nomos"),
            module(),
            vec![],
        )],
    );
    assert!(matches!(
        textual.plan_file_population(
            source_directory.path(),
            &external_invoke_manifest,
            &fixed,
            [37; 32],
            expected(before)
        ),
        Err(NomosManifestLoadError::ExternalInvoke(modules, spelling))
            if modules == vec![Name::new("fixture")]
                && spelling == Name::new("WireAttributes")
    ));
    assert_eq!(current(&runtime).await, before);
    assert!(matches!(
        durable_receipt(&runtime, OperationKey::new([37; 32])).await,
        AuthorityReply::Rejected(NoWriteFailure::UnknownCommittedReceipt {
            operation_key
        }) if operation_key == OperationKey::new([37; 32])
    ));

    runtime.shutdown().await.expect("runtime shuts down");
}

#[tokio::test]
async fn multi_file_population_is_one_canonical_request_and_source_neutral_shape() {
    let source_directory = tempfile::tempdir().expect("temporary source directory");
    write_sources(&source_directory);
    let first_authority = tempfile::tempdir().expect("first authority directory");
    let first_runtime = open_runtime(&first_authority).await;
    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();
    let before = current(&first_runtime).await;
    let planned = textual
        .plan_file_population(
            source_directory.path(),
            &manifest(false),
            &fixed,
            [31; 32],
            expected(before),
        )
        .expect("complete dependency graph plans");
    let reversed = textual
        .plan_file_population(
            source_directory.path(),
            &manifest(true),
            &fixed,
            [31; 32],
            expected(before),
        )
        .expect("reordered dependency graph plans");

    assert_eq!(planned.request(), reversed.request());
    assert_eq!(
        planned
            .request()
            .canonical_request_digest()
            .expect("canonical digest"),
        reversed
            .request()
            .canonical_request_digest()
            .expect("canonical digest")
    );
    assert_eq!(planned.request().declarations.len(), 1);
    assert_eq!(planned.request().references.len(), 3);

    let committed = request(
        &first_runtime,
        AuthorityOperation::SealUniversal(planned.request().clone()),
    )
    .await;
    let first_receipt = seal_receipt(&committed);
    let durable = durable_receipt(&first_runtime, planned.request().operation_key).await;
    let population = textual
        .complete_file_population(&planned, &durable, &fixed)
        .expect("durable receipt materializes the complete graph");
    let spellings = population
        .transformers()
        .declarations()
        .iter()
        .map(|declaration| {
            population
                .names()
                .spelling(declaration.name().encoded_id())
                .expect("transformer spelling")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(spellings, BTreeSet::from(["WireAttributes", "WireNewtype"]));

    let direct = LoadedNomosPopulation::from_typed(
        AuthoredTransformerSet::try_new(population.transformers().declarations().to_vec())
            .expect("already typed direct population"),
        population.names().clone(),
    );
    assert_eq!(direct, population);
    let canonical_seal = population
        .seal(NameTreeProjectionVersion::initial())
        .expect("canonical population seals");
    let mut reversed_declarations = population.transformers().declarations().to_vec();
    reversed_declarations.reverse();
    let construction_order_permutation = LoadedNomosPopulation::from_typed(
        AuthoredTransformerSet::try_new(reversed_declarations)
            .expect("permuted declarations canonicalize"),
        population.names().clone(),
    )
    .seal(NameTreeProjectionVersion::initial())
    .expect("permuted construction seals");
    assert_eq!(construction_order_permutation, canonical_seal);

    let second_authority = tempfile::tempdir().expect("second authority directory");
    let second_runtime = open_runtime(&second_authority).await;
    let committed_reversed = request(
        &second_runtime,
        AuthorityOperation::SealUniversal(reversed.request().clone()),
    )
    .await;
    let second_receipt = seal_receipt(&committed_reversed);
    assert_eq!(
        first_receipt.name_table.declarations(),
        second_receipt.name_table.declarations()
    );
    assert_eq!(
        first_receipt.name_table.references(),
        second_receipt.name_table.references()
    );
    let reversed_durable = durable_receipt(&second_runtime, reversed.request().operation_key).await;
    let reversed_population = textual
        .complete_file_population(&reversed, &reversed_durable, &fixed)
        .expect("reversed traversal materializes");
    assert_eq!(
        reversed_population
            .seal(NameTreeProjectionVersion::initial())
            .expect("reversed traversal seals"),
        canonical_seal
    );

    first_runtime
        .shutdown()
        .await
        .expect("first runtime shuts down");
    second_runtime
        .shutdown()
        .await
        .expect("second runtime shuts down");
}

#[tokio::test]
async fn missing_and_cyclic_graphs_refuse_before_any_authority_write() {
    let source_directory = tempfile::tempdir().expect("temporary source directory");
    write_sources(&source_directory);
    let authority_directory = tempfile::tempdir().expect("authority directory");
    let runtime = open_runtime(&authority_directory).await;
    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();
    let before = current(&runtime).await;

    let missing = NomosFileManifest(
        source_path("entry.nomos"),
        vec![NomosManifestFile(
            source_path("entry.nomos"),
            module(),
            vec![source_path("missing.nomos")],
        )],
    );
    assert!(matches!(
        textual.plan_file_population(
            source_directory.path(),
            &missing,
            &fixed,
            [32; 32],
            expected(before)
        ),
        Err(NomosManifestLoadError::MissingDependency(_, _))
    ));
    assert_eq!(current(&runtime).await, before);

    let cyclic = NomosFileManifest(
        source_path("entry.nomos"),
        vec![
            NomosManifestFile(
                source_path("entry.nomos"),
                module(),
                vec![source_path("attributes.nomos")],
            ),
            NomosManifestFile(
                source_path("attributes.nomos"),
                module(),
                vec![source_path("entry.nomos")],
            ),
        ],
    );
    assert!(matches!(
        textual.plan_file_population(
            source_directory.path(),
            &cyclic,
            &fixed,
            [33; 32],
            expected(before)
        ),
        Err(NomosManifestLoadError::DependencyCycle(_))
    ));
    assert_eq!(current(&runtime).await, before);

    runtime.shutdown().await.expect("runtime shuts down");
}

#[cfg(unix)]
#[test]
fn lexical_traversal_and_symlink_escape_refuse_before_a_plan_exists() {
    use std::os::unix::fs::symlink;

    assert!(matches!(
        NomosSourcePath::try_new("../escape.nomos"),
        Err(NomosManifestLoadError::InvalidSourcePath(_))
    ));

    let source_directory = tempfile::tempdir().expect("temporary source directory");
    let outside_directory = tempfile::tempdir().expect("outside source directory");
    fs::write(outside_directory.path().join("outside.nomos"), ATTRIBUTES)
        .expect("outside source writes");
    symlink(
        outside_directory.path().join("outside.nomos"),
        source_directory.path().join("entry.nomos"),
    )
    .expect("escape symlink creates");

    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();
    let manifest = NomosFileManifest(
        source_path("entry.nomos"),
        vec![NomosManifestFile(
            source_path("entry.nomos"),
            module(),
            vec![],
        )],
    );
    assert!(matches!(
        textual.plan_file_population(
            source_directory.path(),
            &manifest,
            &fixed,
            [35; 32],
            expected(DatabaseMarker::new(0, 0))
        ),
        Err(NomosManifestLoadError::SourceEscapesRoot(path))
            if path == source_path("entry.nomos")
    ));
}

#[tokio::test]
async fn duplicate_spelling_across_files_is_a_distinct_atomic_redefinition() {
    let source_directory = tempfile::tempdir().expect("temporary source directory");
    fs::write(source_directory.path().join("first.nomos"), ATTRIBUTES)
        .expect("first duplicate writes");
    fs::write(source_directory.path().join("second.nomos"), ATTRIBUTES)
        .expect("second duplicate writes");
    let manifest = NomosFileManifest(
        source_path("second.nomos"),
        vec![
            NomosManifestFile(source_path("first.nomos"), module(), vec![]),
            NomosManifestFile(
                source_path("second.nomos"),
                module(),
                vec![source_path("first.nomos")],
            ),
        ],
    );
    let authority_directory = tempfile::tempdir().expect("authority directory");
    let runtime = open_runtime(&authority_directory).await;
    let logos = logos();
    let textual = textual(&logos);
    let fixed = FixedNames::new();
    let before = current(&runtime).await;
    let planned = textual
        .plan_file_population(
            source_directory.path(),
            &manifest,
            &fixed,
            [34; 32],
            expected(before),
        )
        .expect("duplicate declarations remain visible to the authority");
    let refused = request(
        &runtime,
        AuthorityOperation::SealUniversal(planned.request().clone()),
    )
    .await;
    assert!(matches!(
        refused.reply,
        AuthorityReply::Rejected(NoWriteFailure::Redefinition { spelling, .. })
            if spelling == Name::new("WireAttributes")
    ));
    assert_eq!(current(&runtime).await, before);

    runtime.shutdown().await.expect("runtime shuts down");
}

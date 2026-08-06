//! Identity-preserving structural lowering for Nexus and Interface documents.
//!
//! Nexus traits are emitted before their operand types, following the trait-first
//! ontology discipline. Declarations retain their translator-issued identities;
//! only exact caller-supplied reference mappings may cross into Rust vocabulary.
//! Nexus declarations remain plain. Interface declarations use the canonical
//! `WireAttributes` policy and acquire universal Input, Output, or Refusal
//! membership from their body position. Strict stream initiations lower into a
//! complete archiveable lifecycle contract; this transformer never retains a
//! deferred stream outcome.

use std::collections::{BTreeMap, BTreeSet};

use capsule_content_identity::IdentityHasher;
use core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosEnumeration, WholeEthosFileKind,
    WholeEthosItem, WholeEthosNewtype, WholeEthosQuality, WholeEthosSemaTableKey,
    WholeEthosStreamInitiation, WholeEthosStruct, WholeEthosTrait, WholeEthosTypeApplication,
    WholeEthosTypeParameter, WholeEthosTypeReference, WholeEthosVariant, WholeEthosVariantPayload,
    WholeEthosVisibility,
};
use core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype,
    WholeLogosPreservedSemaFamily, WholeLogosSemaTableKey, WholeLogosStorageFingerprint,
    WholeLogosStreamHandle, WholeLogosStreamInitiation, WholeLogosStreamLifecycle,
    WholeLogosStreamTermination, WholeLogosStruct, WholeLogosTable, WholeLogosTraitDef,
    WholeLogosTraitImpl, WholeLogosTupleFields, WholeLogosTypeApplication,
    WholeLogosTypeAttributes, WholeLogosTypeParameter, WholeLogosTypeReference, WholeLogosVariant,
    WholeLogosVariantPayload, WholeLogosVisibility,
};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

const CURRENT_SPIRIT_V14_SOURCE: &str = "https://github.com/LiGoldragon/spirit";
const CURRENT_SPIRIT_V14_REVISION: &str = "7405eee89e3b1b5b6764eb1a50cbdf467b93c9a7";
const CURRENT_SPIRIT_V14_STORE_SCHEMA: u64 = 14;

/// The Nexus document-to-Logos structural contract.
pub trait NexusStructuralTransformation {
    /// Lower one typed Nexus document without allocating or deriving identities.
    fn lower(&self, ethos: &WholeEthos) -> Result<WholeLogos, NexusTransformationError>;
}

/// The deliberately narrow Interface shared-type structural contract.
pub trait InterfaceTypeStructuralTransformation {
    /// Lower only `Interface.types` with canonical wire emission attributes.
    /// Input, Output, and Refusal positions are not projected by this slice.
    fn lower_interface_types(
        &self,
        ethos: &WholeEthos,
    ) -> Result<WholeLogos, NexusTransformationError>;
}

/// The complete presently structural Interface document-to-Logos contract.
pub trait InterfaceStructuralTransformation {
    /// Lower positional declarations, their universal role memberships, and
    /// each authored stream initiation into its resolved lifecycle contract.
    fn lower_interface(
        &self,
        ethos: &WholeEthos,
        roles: &InterfaceRoleIdentities,
    ) -> Result<InterfaceTransformationOutcome, NexusTransformationError>;
}

/// The Sema record/table document-to-Logos structural contract.
pub trait SemaStructuralTransformation {
    /// Lower stored record declarations and every table whose record shape is
    /// registered in the complete authored bundle. Malformed, unknown, or
    /// foreign-owned record shapes refuse the whole projection.
    fn lower_sema(
        &self,
        ethos: &WholeEthos,
        provenance: &dyn NomosStorageProvenance,
    ) -> Result<SemaTransformationOutcome, NexusTransformationError>;
}

/// Exact revision-bearing owner evidence for an externally archived storage
/// type. The owner/revision records provenance without changing the stable
/// storage-shape bytes: an ABI-preserving producer repin is not a new layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProvenanceOwner {
    source: String,
    revision: String,
}

impl StorageProvenanceOwner {
    /// Bind one published producer source and immutable revision.
    pub fn new(source: String, revision: String) -> Result<Self, NexusTransformationError> {
        if source.is_empty() {
            return Err(NexusTransformationError::StorageProvenanceOwnerEmpty { field: "source" });
        }
        if revision.is_empty() {
            return Err(NexusTransformationError::StorageProvenanceOwnerEmpty {
                field: "revision",
            });
        }
        Ok(Self { source, revision })
    }

    /// Published producer that owns this archived storage contract.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Immutable producer revision that supplied the contract.
    pub fn revision(&self) -> &str {
        &self.revision
    }
}

/// Provenance for one non-bundle storage type. The identity, exact archive
/// fingerprint, and published owner revision travel together; no unlabelled
/// fingerprint map can cross the Nomos boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalStorageProvenance {
    identity: VocabularyEncodedId,
    fingerprint: WholeLogosStorageFingerprint,
    owner: StorageProvenanceOwner,
    successor: Option<ExternalStorageSuccessorEvidence>,
}

/// The complete set of archive-ABI predicates discharged before an external
/// producer successor may retain an already-published storage fingerprint.
///
/// This is deliberately a closed record: a successor may not omit a check or
/// claim only a source-level rename. Each predicate must have been proven for
/// the sealed evidence revision before Nomos will carry the adoption record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchiveAbiEquivalenceChecks {
    layout: bool,
    variant_order: bool,
    discriminants: bool,
    size: bool,
    alignment: bool,
    archive_bytes: bool,
}

impl ArchiveAbiEquivalenceChecks {
    /// Construct a complete archive-ABI proof claim, refusing every partial
    /// or failed predicate.
    pub fn new(
        layout: bool,
        variant_order: bool,
        discriminants: bool,
        size: bool,
        alignment: bool,
        archive_bytes: bool,
    ) -> Result<Self, NexusTransformationError> {
        for (check, passed) in [
            ("layout", layout),
            ("variant order", variant_order),
            ("discriminants", discriminants),
            ("size", size),
            ("alignment", alignment),
            ("archive bytes", archive_bytes),
        ] {
            if !passed {
                return Err(NexusTransformationError::ArchiveAbiCheckNotProven { check });
            }
        }
        Ok(Self {
            layout,
            variant_order,
            discriminants,
            size,
            alignment,
            archive_bytes,
        })
    }

    /// Whether the proof includes the exact generated declaration layout.
    pub const fn layout(&self) -> bool {
        self.layout
    }

    /// Whether the proof includes exact enum variant ordering.
    pub const fn variant_order(&self) -> bool {
        self.variant_order
    }

    /// Whether the proof includes exact enum discriminants.
    pub const fn discriminants(&self) -> bool {
        self.discriminants
    }

    /// Whether the proof includes native type sizes.
    pub const fn size(&self) -> bool {
        self.size
    }

    /// Whether the proof includes native type alignment.
    pub const fn alignment(&self) -> bool {
        self.alignment
    }

    /// Whether the proof includes archived wire bytes.
    pub const fn archive_bytes(&self) -> bool {
        self.archive_bytes
    }
}

/// Sealed adoption evidence for an archive-ABI-equivalent external producer
/// successor. It keeps the physical v14 owner and the currently compiled
/// owner simultaneously, without introducing a second decoder or storage
/// descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalStorageSuccessorEvidence {
    physical_owner: StorageProvenanceOwner,
    compiled_owner: StorageProvenanceOwner,
    type_identities: Vec<VocabularyEncodedId>,
    proof_digest: [u8; 32],
    evidence_revision: String,
    checks: ArchiveAbiEquivalenceChecks,
}

impl ExternalStorageSuccessorEvidence {
    /// Bind a fully checked successor proof to every declared external type
    /// whose v14 physical layout it retains.
    pub fn new(
        physical_owner: StorageProvenanceOwner,
        compiled_owner: StorageProvenanceOwner,
        mut type_identities: Vec<VocabularyEncodedId>,
        proof_digest: [u8; 32],
        evidence_revision: String,
        checks: ArchiveAbiEquivalenceChecks,
    ) -> Result<Self, NexusTransformationError> {
        if physical_owner == compiled_owner {
            return Err(NexusTransformationError::ExternalStorageSuccessorUnchanged);
        }
        if evidence_revision.is_empty() {
            return Err(NexusTransformationError::ExternalStorageEvidenceRevisionEmpty);
        }
        if proof_digest == [0; 32] {
            return Err(NexusTransformationError::ExternalStorageEvidenceDigestEmpty);
        }
        if type_identities.is_empty() {
            return Err(NexusTransformationError::ExternalStorageSuccessorTypesEmpty);
        }
        for identity in &type_identities {
            if identity.root_variant() != &VocabularyRoot::Universal {
                return Err(NexusTransformationError::StorageProvenanceIdentityRoot {
                    found: *identity.root_variant(),
                });
            }
        }
        type_identities.sort();
        for adjacent in type_identities.windows(2) {
            if adjacent[0] == adjacent[1] {
                return Err(
                    NexusTransformationError::ExternalStorageSuccessorTypeDuplicate {
                        identity: adjacent[0].clone(),
                    },
                );
            }
        }
        Ok(Self {
            physical_owner,
            compiled_owner,
            type_identities,
            proof_digest,
            evidence_revision,
            checks,
        })
    }

    /// Published revision that originated the still-frozen physical layout.
    pub const fn physical_owner(&self) -> &StorageProvenanceOwner {
        &self.physical_owner
    }

    /// Published revision that is compiled now.
    pub const fn compiled_owner(&self) -> &StorageProvenanceOwner {
        &self.compiled_owner
    }

    /// Every Universal type identity covered by the one successor proof.
    pub fn type_identities(&self) -> &[VocabularyEncodedId] {
        &self.type_identities
    }

    /// Digest of the immutable archive-ABI proof material.
    pub const fn proof_digest(&self) -> [u8; 32] {
        self.proof_digest
    }

    /// Immutable revision containing the proof material.
    pub fn evidence_revision(&self) -> &str {
        &self.evidence_revision
    }

    /// Complete proof predicates; partial evidence is never representable.
    pub const fn checks(&self) -> ArchiveAbiEquivalenceChecks {
        self.checks
    }
}

/// One catalogued current-Spirit-v14 physical family adopted by a fresh
/// semantic Sema table. Unlike external type evidence, this does not make a
/// second storage implementation available: it only proves that one declared
/// semantic table may retain one exact existing descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreservedSemaFamilyProvenance {
    table: VocabularyEncodedId,
    record_archive_type: VocabularyEncodedId,
    key_archive_type: VocabularyEncodedId,
    record_layout: WholeLogosStorageFingerprint,
    key_layout: WholeLogosStorageFingerprint,
    source_spirit_revision: String,
    store_schema: u64,
    physical: WholeLogosPreservedSemaFamily,
}

impl PreservedSemaFamilyProvenance {
    /// Admit exactly one known Spirit-v14 physical descriptor only when every
    /// semantic identity, archive layout, source revision, and store schema
    /// has been supplied. The physical name/family/hash are checked against a
    /// sealed catalogue rather than accepted as an arbitrary override.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        table: VocabularyEncodedId,
        record_archive_type: VocabularyEncodedId,
        key_archive_type: VocabularyEncodedId,
        physical_table_name: String,
        physical_family_name: String,
        physical_schema_hash: [u8; 32],
        source_spirit_revision: String,
        store_schema: u64,
        record_layout: [u8; 32],
        key_layout: [u8; 32],
    ) -> Result<Self, NexusTransformationError> {
        for (position, identity) in [
            ("table", &table),
            ("record archive type", &record_archive_type),
            ("key archive type", &key_archive_type),
        ] {
            if identity.root_variant() != &VocabularyRoot::Universal {
                return Err(NexusTransformationError::PreservedSemaFamilyIdentityRoot {
                    position,
                    found: *identity.root_variant(),
                });
            }
        }
        let catalogue = current_spirit_v14_family(&physical_table_name, &physical_family_name)
            .ok_or_else(|| NexusTransformationError::UnknownPreservedSemaFamily {
                table_name: physical_table_name.clone(),
                family_name: physical_family_name.clone(),
            })?;
        if physical_schema_hash != catalogue.schema_hash {
            return Err(
                NexusTransformationError::PreservedSemaFamilySchemaHashMismatch {
                    family_name: physical_family_name,
                    found: physical_schema_hash,
                    expected: catalogue.schema_hash,
                },
            );
        }
        for (position, found, expected) in [
            ("table", &table, catalogue.table),
            (
                "record archive type",
                &record_archive_type,
                catalogue.record,
            ),
            ("key archive type", &key_archive_type, catalogue.key),
        ] {
            if found != &current_spirit_v14_identity(expected) {
                return Err(
                    NexusTransformationError::PreservedSemaFamilyIdentityMismatch {
                        position,
                        found: found.clone(),
                        expected: current_spirit_v14_identity(expected),
                    },
                );
            }
        }
        if source_spirit_revision != CURRENT_SPIRIT_V14_REVISION {
            return Err(
                NexusTransformationError::PreservedSemaFamilyRevisionMismatch {
                    found: source_spirit_revision,
                    expected: CURRENT_SPIRIT_V14_REVISION,
                },
            );
        }
        if store_schema != CURRENT_SPIRIT_V14_STORE_SCHEMA {
            return Err(
                NexusTransformationError::PreservedSemaFamilyStoreSchemaMismatch {
                    found: store_schema,
                    expected: CURRENT_SPIRIT_V14_STORE_SCHEMA,
                },
            );
        }
        Ok(Self {
            table,
            record_archive_type,
            key_archive_type,
            record_layout: WholeLogosStorageFingerprint::new(record_layout),
            key_layout: WholeLogosStorageFingerprint::new(key_layout),
            source_spirit_revision,
            store_schema,
            physical: WholeLogosPreservedSemaFamily::new(
                physical_table_name,
                physical_family_name,
                physical_schema_hash,
            ),
        })
    }

    /// Fresh semantic Sema table receiving the physical descriptor.
    pub const fn table(&self) -> &VocabularyEncodedId {
        &self.table
    }

    /// Semantic identity of the record archive type.
    pub const fn record_archive_type(&self) -> &VocabularyEncodedId {
        &self.record_archive_type
    }

    /// Semantic identity of the key archive type.
    pub const fn key_archive_type(&self) -> &VocabularyEncodedId {
        &self.key_archive_type
    }

    /// Exact immutable source of the preserved physical descriptor.
    pub const fn source(&self) -> &'static str {
        CURRENT_SPIRIT_V14_SOURCE
    }

    /// Immutable current-Spirit-v14 revision cited by this proof.
    pub fn source_spirit_revision(&self) -> &str {
        &self.source_spirit_revision
    }

    /// Store schema accepted by this purpose-built adoption path.
    pub const fn store_schema(&self) -> u64 {
        self.store_schema
    }

    /// Exact physical descriptor retained on successful lowering.
    pub const fn physical(&self) -> &WholeLogosPreservedSemaFamily {
        &self.physical
    }

    fn validate_table_layout(
        &self,
        record: &VocabularyEncodedId,
        key: &VocabularyEncodedId,
        record_layout: WholeLogosStorageFingerprint,
        key_layout: WholeLogosStorageFingerprint,
    ) -> Result<WholeLogosPreservedSemaFamily, NexusTransformationError> {
        if record != &self.record_archive_type || key != &self.key_archive_type {
            return Err(
                NexusTransformationError::PreservedSemaFamilyTableTypeMismatch {
                    table: self.table.clone(),
                },
            );
        }
        if record_layout != self.record_layout || key_layout != self.key_layout {
            return Err(
                NexusTransformationError::PreservedSemaFamilyLayoutMismatch {
                    table: self.table.clone(),
                },
            );
        }
        Ok(self.physical.clone())
    }
}

struct CurrentSpiritV14Family {
    table: u16,
    record: u16,
    key: u16,
    schema_hash: [u8; 32],
}

fn current_spirit_v14_family(
    table_name: &str,
    family_name: &str,
) -> Option<CurrentSpiritV14Family> {
    match (table_name, family_name) {
        ("records", "RecordsFamily") => Some(CurrentSpiritV14Family {
            table: 142,
            record: 126,
            key: 109,
            schema_hash: [
                169, 167, 27, 203, 113, 158, 12, 113, 89, 93, 195, 166, 134, 208, 34, 40, 178, 38,
                203, 139, 155, 209, 108, 101, 12, 183, 180, 233, 6, 84, 230, 177,
            ],
        }),
        ("migrations", "MigrationsFamily") => Some(CurrentSpiritV14Family {
            table: 141,
            record: 84,
            key: 121,
            schema_hash: [
                230, 253, 154, 216, 87, 227, 13, 141, 82, 16, 203, 108, 170, 143, 69, 87, 143, 191,
                234, 25, 90, 168, 75, 182, 238, 134, 0, 229, 158, 24, 20, 143,
            ],
        }),
        _ => None,
    }
}

fn current_spirit_v14_identity(local: u16) -> VocabularyEncodedId {
    VocabularyEncodedId::new(
        VocabularyRoot::Universal,
        vec![encoded_name_table::LocalEncodedId::new(local)],
    )
    .expect("one sealed current-Spirit-v14 identity")
}

impl ExternalStorageProvenance {
    /// Bind one Universal external type to evidence from its owning producer.
    pub fn new(
        identity: VocabularyEncodedId,
        fingerprint: [u8; 32],
        owner: StorageProvenanceOwner,
    ) -> Result<Self, NexusTransformationError> {
        if identity.root_variant() != &VocabularyRoot::Universal {
            return Err(NexusTransformationError::StorageProvenanceIdentityRoot {
                found: *identity.root_variant(),
            });
        }
        Ok(Self {
            identity,
            fingerprint: WholeLogosStorageFingerprint::new(fingerprint),
            owner,
            successor: None,
        })
    }

    /// Bind a compiled external producer to its different physical-layout
    /// origin only after one complete archive-ABI successor proof has been
    /// supplied. The currently compiled owner is never inferred from a
    /// fingerprint or a package alias.
    pub fn with_successor(
        identity: VocabularyEncodedId,
        fingerprint: [u8; 32],
        owner: StorageProvenanceOwner,
        successor: ExternalStorageSuccessorEvidence,
    ) -> Result<Self, NexusTransformationError> {
        if successor.compiled_owner() != &owner {
            return Err(
                NexusTransformationError::ExternalStorageSuccessorOwnerMismatch {
                    configured: owner,
                    evidence: successor.compiled_owner().clone(),
                },
            );
        }
        if !successor.type_identities().contains(&identity) {
            return Err(
                NexusTransformationError::ExternalStorageSuccessorIdentityMissing { identity },
            );
        }
        Ok(Self {
            identity,
            fingerprint: WholeLogosStorageFingerprint::new(fingerprint),
            owner,
            successor: Some(successor),
        })
    }

    /// The imported Ethos identity covered by this evidence.
    pub const fn identity(&self) -> &VocabularyEncodedId {
        &self.identity
    }

    /// Exact archived storage-shape evidence supplied by its owner.
    pub const fn fingerprint(&self) -> WholeLogosStorageFingerprint {
        self.fingerprint
    }

    /// Published owner and immutable revision for the evidence.
    pub const fn owner(&self) -> &StorageProvenanceOwner {
        &self.owner
    }

    /// Sealed physical-to-current producer evidence, when the compiled owner
    /// is an archive-ABI-equivalent successor rather than the physical origin.
    pub const fn successor(&self) -> Option<&ExternalStorageSuccessorEvidence> {
        self.successor.as_ref()
    }
}

/// Typed storage-shape resolution owned by Nomos. Bundle declarations receive
/// the same recursive local algorithm wherever they are imported; only absent
/// identities may use explicit external producer provenance.
pub trait NomosStorageProvenance {
    /// Resolve one complete concrete reference into its deterministic storage
    /// shape or refuse the missing/foreign/cyclic ownership boundary.
    fn storage_fingerprint(
        &self,
        reference: &WholeEthosTypeReference,
    ) -> Result<WholeLogosStorageFingerprint, NexusTransformationError>;

    /// Whether a type identity is a declaration in the pre-registered bundle.
    fn declares(&self, identity: &VocabularyEncodedId) -> bool;

    /// Resolve the exact bundle declaration which owns a source-declared
    /// table-key archive contract.  This is separate from storage-shape
    /// evidence: a key must never gain lookup semantics from an external
    /// fingerprint or a generated Rust spelling.
    fn sema_table_key_archive_identity(
        &self,
        table: &VocabularyEncodedId,
        key: &WholeEthosSemaTableKey,
    ) -> Result<VocabularyEncodedId, NexusTransformationError>;

    /// Return the one physical descriptor proven for this semantic table, or
    /// refuse a cited proof whose table types or complete layouts no longer
    /// match. Absence keeps ordinary fresh-table generation intact.
    fn preserved_sema_family(
        &self,
        table: &VocabularyEncodedId,
        record: &VocabularyEncodedId,
        key: &VocabularyEncodedId,
        record_layout: WholeLogosStorageFingerprint,
        key_layout: WholeLogosStorageFingerprint,
    ) -> Result<Option<WholeLogosPreservedSemaFamily>, NexusTransformationError>;
}

/// Pre-registered declarations from the entire authored Interface/Nexus/Sema
/// bundle and exact provenance for the genuinely external leaves it reaches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleStorageProvenance {
    declarations: BTreeMap<VocabularyEncodedId, WholeEthosItem>,
    external: Vec<ExternalStorageProvenance>,
    preserved_sema_families: Vec<PreservedSemaFamilyProvenance>,
}

impl BundleStorageProvenance {
    /// Register all value declarations before lowering any root. Duplicate
    /// Universal identities and bundle/external ownership conflicts are typed
    /// refusals rather than order-sensitive overwrite behavior.
    pub fn from_documents(
        documents: impl IntoIterator<Item = WholeEthos>,
        external: Vec<ExternalStorageProvenance>,
    ) -> Result<Self, NexusTransformationError> {
        Self::from_documents_with_preserved_families(documents, external, Vec::new())
    }

    /// Register the full bundle plus the only admissible existing Spirit-v14
    /// physical descriptors. Duplicate semantic tables and all mismatches are
    /// rejected before any root is lowered.
    pub fn from_documents_with_preserved_families(
        documents: impl IntoIterator<Item = WholeEthos>,
        mut external: Vec<ExternalStorageProvenance>,
        mut preserved_sema_families: Vec<PreservedSemaFamilyProvenance>,
    ) -> Result<Self, NexusTransformationError> {
        let mut declarations = BTreeMap::new();
        for document in documents {
            Self::register_document(&mut declarations, document)?;
        }
        external.sort_by(|left, right| left.identity().cmp(right.identity()));
        for adjacent in external.windows(2) {
            if adjacent[0].identity() == adjacent[1].identity() {
                return Err(
                    NexusTransformationError::DuplicateExternalStorageProvenance {
                        identity: adjacent[0].identity().clone(),
                    },
                );
            }
        }
        for provenance in &external {
            if declarations.contains_key(provenance.identity()) {
                return Err(
                    NexusTransformationError::StorageProvenanceOwnershipConflict {
                        identity: provenance.identity().clone(),
                    },
                );
            }
        }
        preserved_sema_families.sort_by(|left, right| left.table().cmp(right.table()));
        for adjacent in preserved_sema_families.windows(2) {
            if adjacent[0].table() == adjacent[1].table() {
                return Err(NexusTransformationError::DuplicatePreservedSemaFamily {
                    table: adjacent[0].table().clone(),
                });
            }
        }
        Ok(Self {
            declarations,
            external,
            preserved_sema_families,
        })
    }

    /// Canonically ordered external provenance in this bundle boundary.
    pub fn external(&self) -> &[ExternalStorageProvenance] {
        &self.external
    }

    fn register_document(
        declarations: &mut BTreeMap<VocabularyEncodedId, WholeEthosItem>,
        document: WholeEthos,
    ) -> Result<(), NexusTransformationError> {
        let mut register = |item: WholeEthosItem| {
            let Some(identity) = storage_declaration_identity(&item).cloned() else {
                return Ok(());
            };
            if declarations.insert(identity.clone(), item).is_some() {
                return Err(
                    NexusTransformationError::DuplicateBundleStorageDeclaration { identity },
                );
            }
            Ok(())
        };
        match document.body() {
            WholeEthosBody::Interface(body) => {
                for input in body.inputs() {
                    register(WholeEthosItem::Newtype(input.clone()))?;
                }
                for output in body.outputs() {
                    register(WholeEthosItem::Newtype(output.clone()))?;
                }
                for refusal in body.refusals() {
                    register(WholeEthosItem::Struct(refusal.clone()))?;
                }
                for item in body.types() {
                    register(item.clone())?;
                }
            }
            WholeEthosBody::Nexus(body) => {
                for item in body.types() {
                    register(item.clone())?;
                }
            }
            WholeEthosBody::Sema(body) => {
                for item in body.record_types() {
                    register(item.clone())?;
                }
            }
        }
        Ok(())
    }

    fn external_fingerprint(
        &self,
        identity: &VocabularyEncodedId,
    ) -> Result<WholeLogosStorageFingerprint, NexusTransformationError> {
        self.external
            .binary_search_by(|entry| entry.identity().cmp(identity))
            .map(|index| self.external[index].fingerprint())
            .map_err(
                |_| NexusTransformationError::MissingExternalStorageProvenance {
                    identity: identity.clone(),
                },
            )
    }

    fn local_storage_fingerprint(
        &self,
        identity: &VocabularyEncodedId,
        visiting: &mut BTreeSet<VocabularyEncodedId>,
    ) -> Result<WholeLogosStorageFingerprint, NexusTransformationError> {
        if !visiting.insert(identity.clone()) {
            return Err(NexusTransformationError::CyclicSemaStorageShape {
                identity: identity.clone(),
            });
        }
        let declaration = self
            .declarations
            .get(identity)
            .expect("bundle declaration checked before local storage fingerprint");
        let result = match declaration {
            WholeEthosItem::Newtype(newtype) => {
                let wrapped =
                    self.storage_fingerprint_inner(newtype.wrapped_field().reference(), visiting)?;
                let mut hasher = storage_shape_hasher(b"newtype");
                update_identity(&mut hasher, identity);
                hasher.update_length_prefixed(&wrapped.bytes());
                WholeLogosStorageFingerprint::new(hasher.finalize_bytes())
            }
            WholeEthosItem::Struct(structure) => {
                let mut hasher = storage_shape_hasher(b"struct");
                update_identity(&mut hasher, identity);
                update_count(&mut hasher, structure.fields().len());
                for field in structure.fields() {
                    let field = self.storage_fingerprint_inner(field, visiting)?;
                    hasher.update_length_prefixed(&field.bytes());
                }
                WholeLogosStorageFingerprint::new(hasher.finalize_bytes())
            }
            WholeEthosItem::Enumeration(enumeration) => {
                let mut hasher = storage_shape_hasher(b"enumeration");
                update_identity(&mut hasher, identity);
                update_count(&mut hasher, enumeration.variants().len());
                for variant in enumeration.variants() {
                    update_identity(&mut hasher, variant.name());
                    match variant.payload() {
                        WholeEthosVariantPayload::Unit => {
                            hasher.update_length_prefixed(b"unit");
                        }
                        WholeEthosVariantPayload::Tuple(fields) => {
                            hasher.update_length_prefixed(b"tuple");
                            update_count(&mut hasher, fields.fields().len());
                            for field in fields.fields() {
                                let field = self.storage_fingerprint_inner(field, visiting)?;
                                hasher.update_length_prefixed(&field.bytes());
                            }
                        }
                    }
                }
                WholeLogosStorageFingerprint::new(hasher.finalize_bytes())
            }
            WholeEthosItem::StreamInitiation(initiation) => {
                return Err(NexusTransformationError::InvalidSemaRecordDeclaration {
                    identity: initiation.stream.clone(),
                });
            }
        };
        visiting.remove(identity);
        Ok(result)
    }

    fn storage_fingerprint_inner(
        &self,
        reference: &WholeEthosTypeReference,
        visiting: &mut BTreeSet<VocabularyEncodedId>,
    ) -> Result<WholeLogosStorageFingerprint, NexusTransformationError> {
        match reference {
            WholeEthosTypeReference::Identity(identity) => {
                if self.declarations.contains_key(identity) {
                    self.local_storage_fingerprint(identity, visiting)
                } else {
                    self.external_fingerprint(identity)
                }
            }
            WholeEthosTypeReference::Parameter(parameter) => {
                Err(NexusTransformationError::UnresolvedTypeParameter {
                    name: parameter.name().clone(),
                })
            }
            WholeEthosTypeReference::Application(application) => {
                let WholeEthosQuality::Shape(application_head) = application.head() else {
                    return Err(NexusTransformationError::TypeApplicationHeadMustBeShape {
                        quality: application.head().identity().clone(),
                    });
                };
                let head = self.external_fingerprint(application_head)?;
                let mut hasher = storage_shape_hasher(b"application");
                update_identity(&mut hasher, application_head);
                hasher.update_length_prefixed(&head.bytes());
                update_count(&mut hasher, application.arguments().len());
                for argument in application.arguments() {
                    let argument = self.storage_fingerprint_inner(argument, visiting)?;
                    hasher.update_length_prefixed(&argument.bytes());
                }
                Ok(WholeLogosStorageFingerprint::new(hasher.finalize_bytes()))
            }
        }
    }
}

impl NomosStorageProvenance for BundleStorageProvenance {
    fn storage_fingerprint(
        &self,
        reference: &WholeEthosTypeReference,
    ) -> Result<WholeLogosStorageFingerprint, NexusTransformationError> {
        self.storage_fingerprint_inner(reference, &mut BTreeSet::new())
    }

    fn declares(&self, identity: &VocabularyEncodedId) -> bool {
        self.declarations.contains_key(identity)
    }

    fn sema_table_key_archive_identity(
        &self,
        table: &VocabularyEncodedId,
        key: &WholeEthosSemaTableKey,
    ) -> Result<VocabularyEncodedId, NexusTransformationError> {
        let WholeEthosTypeReference::Identity(identity) = key.archive_type() else {
            return Err(NexusTransformationError::InvalidSemaTableKeyShape {
                table: table.clone(),
            });
        };
        let declaration = self.declarations.get(identity).ok_or_else(|| {
            NexusTransformationError::SemaTableKeyNotBundleOwned {
                table: table.clone(),
                key: identity.clone(),
            }
        })?;
        if !matches!(declaration, WholeEthosItem::Newtype(_)) {
            return Err(NexusTransformationError::SemaTableKeyNotNewtype {
                table: table.clone(),
                key: identity.clone(),
            });
        }
        Ok(identity.clone())
    }

    fn preserved_sema_family(
        &self,
        table: &VocabularyEncodedId,
        record: &VocabularyEncodedId,
        key: &VocabularyEncodedId,
        record_layout: WholeLogosStorageFingerprint,
        key_layout: WholeLogosStorageFingerprint,
    ) -> Result<Option<WholeLogosPreservedSemaFamily>, NexusTransformationError> {
        match self
            .preserved_sema_families
            .binary_search_by(|entry| entry.table().cmp(table))
        {
            Ok(index) => self.preserved_sema_families[index]
                .validate_table_layout(record, key, record_layout, key_layout)
                .map(Some),
            Err(_) => Ok(None),
        }
    }
}

/// The three universal marker-trait identities assigned by Interface position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceRoleIdentities {
    input: VocabularyEncodedId,
    output: VocabularyEncodedId,
    refusal: VocabularyEncodedId,
}

// Trait exception — too trivial: validated construction and read-only access
// for one role-identity configuration record.
impl InterfaceRoleIdentities {
    /// Validate distinct universal identities for the three positional roles.
    pub fn new(
        input: VocabularyEncodedId,
        output: VocabularyEncodedId,
        refusal: VocabularyEncodedId,
    ) -> Result<Self, NexusTransformationError> {
        Self::validate_role("Input", &input)?;
        Self::validate_role("Output", &output)?;
        Self::validate_role("Refusal", &refusal)?;
        Self::validate_distinct("Input", &input, "Output", &output)?;
        Self::validate_distinct("Input", &input, "Refusal", &refusal)?;
        Self::validate_distinct("Output", &output, "Refusal", &refusal)?;
        Ok(Self {
            input,
            output,
            refusal,
        })
    }

    /// Universal Input trait identity.
    pub const fn input(&self) -> &VocabularyEncodedId {
        &self.input
    }

    /// Universal Output trait identity.
    pub const fn output(&self) -> &VocabularyEncodedId {
        &self.output
    }

    /// Universal Refusal trait identity.
    pub const fn refusal(&self) -> &VocabularyEncodedId {
        &self.refusal
    }

    fn validate_role(
        role: &'static str,
        identity: &VocabularyEncodedId,
    ) -> Result<(), NexusTransformationError> {
        if identity.root_variant() != &VocabularyRoot::Universal {
            return Err(NexusTransformationError::InterfaceRoleRoot {
                role,
                found: *identity.root_variant(),
            });
        }
        Ok(())
    }

    fn validate_distinct(
        first_role: &'static str,
        first: &VocabularyEncodedId,
        second_role: &'static str,
        second: &VocabularyEncodedId,
    ) -> Result<(), NexusTransformationError> {
        if first == second {
            return Err(NexusTransformationError::DuplicateInterfaceRoleIdentity {
                first_role,
                second_role,
                identity: first.clone(),
            });
        }
        Ok(())
    }
}

/// Fully lowered Interface Logos.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceTransformationOutcome {
    logos: WholeLogos,
}

// Trait exception — too trivial: read-only outcome ergonomics.
impl InterfaceTransformationOutcome {
    /// Structurally projected Interface Logos.
    pub const fn logos(&self) -> &WholeLogos {
        &self.logos
    }
}

/// Sema Logos plus valid tables whose imported record shape is not generated
/// by this document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemaTransformationOutcome {
    logos: WholeLogos,
}

// Trait exception — too trivial: read-only outcome ergonomics.
impl SemaTransformationOutcome {
    /// Stored record declarations followed by their complete table set.
    pub const fn logos(&self) -> &WholeLogos {
        &self.logos
    }
}

/// File-kind-neutral projection of currently supported type declarations.
///
/// The caller owns section meaning and must account separately for any
/// constructs it does not pass through this boundary.
pub trait TypeDeclarationStructuralTransformation {
    /// Lower ordinary newtype, struct, and enumeration declarations with one
    /// explicit canonical emission policy.
    fn lower_type_declarations(
        &self,
        items: &[WholeEthosItem],
        attributes: WholeLogosTypeAttributes,
    ) -> Result<WholeLogos, NexusTransformationError>;
}

/// Exact, allocation-free Nexus lowering data.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NexusTransformation {
    reference_mappings: Vec<NexusVocabularyReferenceMapping>,
    stream_lifecycle_identities: Vec<StreamLifecycleIdentities>,
}

// Trait exception — too trivial: constructor and read-only data ergonomics for
// the NexusStructuralTransformation implementation.
impl NexusTransformation {
    /// Construct an identity-reference Nexus transformation.
    pub const fn new() -> Self {
        Self {
            reference_mappings: Vec::new(),
            stream_lifecycle_identities: Vec::new(),
        }
    }

    /// Construct with exact, canonically ordered reference mappings.
    pub fn with_reference_mappings(
        mut reference_mappings: Vec<NexusVocabularyReferenceMapping>,
    ) -> Result<Self, NexusTransformationError> {
        reference_mappings.sort_by(|left, right| left.source().cmp(right.source()));
        for adjacent in reference_mappings.windows(2) {
            if adjacent[0].source() == adjacent[1].source() {
                return Err(NexusTransformationError::DuplicateMappingSource {
                    identity: adjacent[0].source().clone(),
                });
            }
        }
        Ok(Self {
            reference_mappings,
            stream_lifecycle_identities: Vec::new(),
        })
    }

    /// Canonically ordered exact reference mappings.
    pub fn reference_mappings(&self) -> &[NexusVocabularyReferenceMapping] {
        &self.reference_mappings
    }

    /// Attach caller-authored generated identities for each strict stream
    /// lifecycle. This transformer selects and carries these identities but
    /// never allocates or derives them.
    pub fn with_stream_lifecycle_identities(
        mut self,
        mut stream_lifecycle_identities: Vec<StreamLifecycleIdentities>,
    ) -> Result<Self, NexusTransformationError> {
        stream_lifecycle_identities.sort_by(|left, right| left.stream().cmp(right.stream()));
        for adjacent in stream_lifecycle_identities.windows(2) {
            if adjacent[0].stream() == adjacent[1].stream() {
                return Err(NexusTransformationError::DuplicateStreamLifecycleStream {
                    stream: adjacent[0].stream().clone(),
                });
            }
        }
        self.stream_lifecycle_identities = stream_lifecycle_identities;
        Ok(self)
    }

    /// Canonically ordered strict stream lifecycle assignments.
    pub fn stream_lifecycle_identities(&self) -> &[StreamLifecycleIdentities] {
        &self.stream_lifecycle_identities
    }

    fn lower_item(
        &self,
        item: &WholeEthosItem,
        attributes: WholeLogosTypeAttributes,
    ) -> Result<WholeLogosItem, NexusTransformationError> {
        match item {
            WholeEthosItem::Newtype(newtype) => Ok(WholeLogosItem::Newtype(
                self.lower_newtype(newtype)?.with_attributes(attributes),
            )),
            WholeEthosItem::Struct(structure) => Ok(WholeLogosItem::Struct(
                self.lower_struct(structure)?.with_attributes(attributes),
            )),
            WholeEthosItem::Enumeration(enumeration) => Ok(WholeLogosItem::Enumeration(
                self.lower_enumeration(enumeration)?
                    .with_attributes(attributes),
            )),
            WholeEthosItem::StreamInitiation(initiation) => Ok(WholeLogosItem::StreamLifecycle(
                self.lower_stream_initiation(initiation)?,
            )),
        }
    }

    fn lower_stream_initiation(
        &self,
        initiation: &WholeEthosStreamInitiation,
    ) -> Result<WholeLogosStreamLifecycle, NexusTransformationError> {
        let identities = self
            .stream_lifecycle_identities
            .binary_search_by(|candidate| candidate.stream().cmp(&initiation.stream))
            .map(|index| &self.stream_lifecycle_identities[index])
            .map_err(
                |_| NexusTransformationError::MissingStreamLifecycleIdentities {
                    stream: initiation.stream.clone(),
                },
            )?;
        let handle_identity = identities.handle().clone();
        Ok(WholeLogosStreamLifecycle::new(
            initiation.stream.clone(),
            WholeLogosStreamInitiation::new(
                identities.initiation_input().clone(),
                self.lower_reference(&initiation.query)?,
                WholeLogosStreamHandle::new(
                    handle_identity.clone(),
                    self.lower_reference(&initiation.event)?,
                ),
                identities.initiation_refusal().clone(),
            ),
            WholeLogosStreamTermination::new(
                identities.termination_input().clone(),
                handle_identity,
                identities.termination_refusal().clone(),
            ),
        ))
    }

    fn lower_newtype(
        &self,
        newtype: &WholeEthosNewtype,
    ) -> Result<WholeLogosNewtype, NexusTransformationError> {
        let WholeEthosAttributes = *newtype.attributes();
        Ok(WholeLogosNewtype::new(
            Self::lower_visibility(*newtype.visibility()),
            newtype.name().clone(),
            Self::lower_visibility(*newtype.wrapped_field().visibility()),
            self.lower_reference(newtype.wrapped_field().reference())?,
        )
        .with_type_parameters(
            newtype
                .type_parameters()
                .iter()
                .map(Self::lower_type_parameter)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn lower_struct(
        &self,
        structure: &WholeEthosStruct,
    ) -> Result<WholeLogosStruct, NexusTransformationError> {
        Ok(WholeLogosStruct::new(
            WholeLogosVisibility::Public,
            structure.name().clone(),
            structure
                .fields()
                .iter()
                .map(|field| self.lower_reference(field))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn lower_enumeration(
        &self,
        enumeration: &WholeEthosEnumeration,
    ) -> Result<WholeLogosEnumeration, NexusTransformationError> {
        let WholeEthosAttributes = *enumeration.attributes();
        Ok(WholeLogosEnumeration::new(
            Self::lower_visibility(*enumeration.visibility()),
            enumeration.name().clone(),
            enumeration
                .variants()
                .iter()
                .map(|variant| self.lower_variant(variant))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn lower_variant(
        &self,
        variant: &WholeEthosVariant,
    ) -> Result<WholeLogosVariant, NexusTransformationError> {
        let WholeEthosAttributes = *variant.attributes();
        let payload = match variant.payload() {
            WholeEthosVariantPayload::Unit => WholeLogosVariantPayload::Unit,
            WholeEthosVariantPayload::Tuple(fields) => {
                let fields = WholeLogosTupleFields::new(
                    fields
                        .fields()
                        .iter()
                        .map(|field| self.lower_reference(field))
                        .collect::<Result<Vec<_>, _>>()?,
                )
                .map_err(|_| {
                    NexusTransformationError::EmptyVariantTupleFields {
                        variant: variant.name().clone(),
                    }
                })?;
                WholeLogosVariantPayload::Tuple(fields)
            }
        };
        Ok(WholeLogosVariant::new(variant.name().clone(), payload))
    }

    fn lower_trait(
        &self,
        trait_definition: &WholeEthosTrait,
    ) -> Result<WholeLogosTraitDef, NexusTransformationError> {
        Ok(WholeLogosTraitDef::new(
            WholeLogosVisibility::Public,
            trait_definition.name().clone(),
            vec![],
        ))
    }

    fn lower_reference(
        &self,
        reference: &WholeEthosTypeReference,
    ) -> Result<WholeLogosTypeReference, NexusTransformationError> {
        Ok(match reference {
            WholeEthosTypeReference::Identity(identity) => {
                WholeLogosTypeReference::Identity(self.map_reference(identity))
            }
            WholeEthosTypeReference::Parameter(parameter) => {
                WholeLogosTypeReference::Parameter(parameter.name().clone())
            }
            WholeEthosTypeReference::Application(application) => {
                WholeLogosTypeReference::Application(self.lower_application(application)?)
            }
        })
    }

    fn lower_type_parameter(
        parameter: &WholeEthosTypeParameter,
    ) -> Result<WholeLogosTypeParameter, NexusTransformationError> {
        let WholeEthosQuality::Trait(quality) = parameter.quality() else {
            return Err(NexusTransformationError::TypeParameterQualityMustBeTrait {
                quality: parameter.quality().identity().clone(),
            });
        };
        Ok(WholeLogosTypeParameter::new(
            parameter.name().clone(),
            quality.clone(),
        ))
    }

    fn lower_application(
        &self,
        application: &WholeEthosTypeApplication,
    ) -> Result<WholeLogosTypeApplication, NexusTransformationError> {
        let WholeEthosQuality::Shape(head) = application.head() else {
            return Err(NexusTransformationError::TypeApplicationHeadMustBeShape {
                quality: application.head().identity().clone(),
            });
        };
        WholeLogosTypeApplication::new(
            self.map_reference(head),
            application
                .arguments()
                .iter()
                .map(|argument| self.lower_reference(argument))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .map_err(|_| NexusTransformationError::EmptyTypeApplicationArguments { head: head.clone() })
    }

    fn map_reference(&self, source: &VocabularyEncodedId) -> VocabularyEncodedId {
        self.reference_mappings
            .binary_search_by(|mapping| mapping.source().cmp(source))
            .map(|index| self.reference_mappings[index].target().clone())
            .unwrap_or_else(|_| source.clone())
    }

    const fn lower_visibility(visibility: WholeEthosVisibility) -> WholeLogosVisibility {
        match visibility {
            WholeEthosVisibility::Public => WholeLogosVisibility::Public,
            WholeEthosVisibility::Private => WholeLogosVisibility::Private,
        }
    }
}

impl NexusStructuralTransformation for NexusTransformation {
    fn lower(&self, ethos: &WholeEthos) -> Result<WholeLogos, NexusTransformationError> {
        let WholeEthosBody::Nexus(body) = ethos.body() else {
            return Err(NexusTransformationError::UnsupportedFileKind {
                expected: WholeEthosFileKind::Nexus,
                found: ethos.header().kind(),
            });
        };
        let mut items = Vec::with_capacity(body.traits().len() + body.types().len());
        items.extend(
            body.traits()
                .iter()
                .map(|trait_definition| {
                    self.lower_trait(trait_definition)
                        .map(WholeLogosItem::TraitDef)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        items.extend(
            self.lower_type_declarations(body.types(), WholeLogosTypeAttributes::Plain)?
                .into_items(),
        );
        Ok(WholeLogos::new(items))
    }
}

impl TypeDeclarationStructuralTransformation for NexusTransformation {
    fn lower_type_declarations(
        &self,
        items: &[WholeEthosItem],
        attributes: WholeLogosTypeAttributes,
    ) -> Result<WholeLogos, NexusTransformationError> {
        Ok(WholeLogos::new(
            items
                .iter()
                .map(|item| self.lower_item(item, attributes))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl InterfaceTypeStructuralTransformation for NexusTransformation {
    fn lower_interface_types(
        &self,
        ethos: &WholeEthos,
    ) -> Result<WholeLogos, NexusTransformationError> {
        let WholeEthosBody::Interface(body) = ethos.body() else {
            return Err(NexusTransformationError::UnsupportedFileKind {
                expected: WholeEthosFileKind::Interface,
                found: ethos.header().kind(),
            });
        };
        self.lower_type_declarations(body.types(), WholeLogosTypeAttributes::Wire)
    }
}

impl InterfaceStructuralTransformation for NexusTransformation {
    fn lower_interface(
        &self,
        ethos: &WholeEthos,
        roles: &InterfaceRoleIdentities,
    ) -> Result<InterfaceTransformationOutcome, NexusTransformationError> {
        let WholeEthosBody::Interface(body) = ethos.body() else {
            return Err(NexusTransformationError::UnsupportedFileKind {
                expected: WholeEthosFileKind::Interface,
                found: ethos.header().kind(),
            });
        };

        let mut items = Vec::with_capacity(
            (body.inputs().len() + body.outputs().len() + body.refusals().len()) * 2
                + body.types().len(),
        );
        for input in body.inputs() {
            items.push(WholeLogosItem::Newtype(
                self.lower_newtype(input)?
                    .with_attributes(WholeLogosTypeAttributes::Wire),
            ));
            items.push(Self::role_membership(roles.input(), input.name()));
        }
        for output in body.outputs() {
            items.push(WholeLogosItem::Newtype(
                self.lower_newtype(output)?
                    .with_attributes(WholeLogosTypeAttributes::Wire),
            ));
            items.push(Self::role_membership(roles.output(), output.name()));
        }
        for refusal in body.refusals() {
            items.push(WholeLogosItem::Struct(
                self.lower_struct(refusal)?
                    .with_attributes(WholeLogosTypeAttributes::Wire),
            ));
            items.push(Self::role_membership(roles.refusal(), refusal.name()));
        }

        for item in body.types() {
            items.push(self.lower_item(item, WholeLogosTypeAttributes::Wire)?);
        }

        Ok(InterfaceTransformationOutcome {
            logos: WholeLogos::new(items),
        })
    }
}

impl SemaStructuralTransformation for NexusTransformation {
    fn lower_sema(
        &self,
        ethos: &WholeEthos,
        provenance: &dyn NomosStorageProvenance,
    ) -> Result<SemaTransformationOutcome, NexusTransformationError> {
        let WholeEthosBody::Sema(body) = ethos.body() else {
            return Err(NexusTransformationError::UnsupportedFileKind {
                expected: WholeEthosFileKind::Sema,
                found: ethos.header().kind(),
            });
        };

        let mut declared_records = Vec::with_capacity(body.record_types().len());
        for item in body.record_types() {
            let name = match item {
                WholeEthosItem::Newtype(newtype) => newtype.name(),
                WholeEthosItem::Struct(structure) => structure.name(),
                WholeEthosItem::Enumeration(enumeration) => enumeration.name(),
                WholeEthosItem::StreamInitiation(initiation) => {
                    return Err(NexusTransformationError::InvalidSemaRecordDeclaration {
                        identity: initiation.stream.clone(),
                    });
                }
            };
            declared_records.push(name.clone());
        }
        declared_records.sort();
        for adjacent in declared_records.windows(2) {
            if adjacent[0] == adjacent[1] {
                return Err(NexusTransformationError::DuplicateSemaRecordIdentity {
                    identity: adjacent[0].clone(),
                });
            }
        }

        let mut table_names = body
            .tables()
            .iter()
            .map(|table| table.name().clone())
            .collect::<Vec<_>>();
        table_names.sort();
        for adjacent in table_names.windows(2) {
            if adjacent[0] == adjacent[1] {
                return Err(NexusTransformationError::DuplicateSemaTableIdentity {
                    identity: adjacent[0].clone(),
                });
            }
        }

        let mut items = self
            .lower_type_declarations(body.record_types(), WholeLogosTypeAttributes::Stored)?
            .into_items();
        for table in body.tables() {
            let WholeEthosTypeReference::Identity(record) = table.record() else {
                return Err(NexusTransformationError::InvalidSemaTableRecordShape {
                    table: table.name().clone(),
                });
            };
            let WholeEthosTypeReference::Identity(key) = table.key().archive_type() else {
                return Err(NexusTransformationError::InvalidSemaTableKeyShape {
                    table: table.name().clone(),
                });
            };
            if !provenance.declares(record) {
                return Err(NexusTransformationError::SemaTableRecordNotBundleOwned {
                    table: table.name().clone(),
                    record: record.clone(),
                });
            }
            let key_archive_identity =
                provenance.sema_table_key_archive_identity(table.name(), table.key())?;
            if &key_archive_identity != key {
                return Err(
                    NexusTransformationError::SemaTableKeyArchiveIdentityMismatch {
                        table: table.name().clone(),
                        declared: key.clone(),
                        provenanced: key_archive_identity,
                    },
                );
            }
            let record_storage = provenance.storage_fingerprint(table.record())?;
            let key_storage = provenance.storage_fingerprint(table.key().archive_type())?;
            let table = WholeLogosTable::new(
                table.name().clone(),
                WholeLogosTypeReference::Identity(self.map_reference(record)),
                WholeLogosSemaTableKey::new(self.map_reference(key)),
                record_storage,
                key_storage,
            );
            let table = match provenance.preserved_sema_family(
                table.name(),
                record,
                key,
                record_storage,
                key_storage,
            )? {
                Some(physical) => table.with_preserved_sema_family(physical),
                None => table,
            };
            items.push(WholeLogosItem::Table(table));
        }

        Ok(SemaTransformationOutcome {
            logos: WholeLogos::new(items),
        })
    }
}

impl NexusTransformation {
    fn role_membership(
        role: &VocabularyEncodedId,
        declaration: &VocabularyEncodedId,
    ) -> WholeLogosItem {
        WholeLogosItem::TraitImpl(WholeLogosTraitImpl::new(
            WholeLogosTypeReference::Identity(role.clone()),
            WholeLogosTypeReference::Identity(declaration.clone()),
            Vec::new(),
        ))
    }
}

fn storage_shape_hasher(kind: &[u8]) -> IdentityHasher {
    let mut hasher = IdentityHasher::unprimed();
    hasher.update_length_prefixed(b"protos-sema-stored-shape-v1");
    hasher.update_length_prefixed(kind);
    hasher
}

fn update_count(hasher: &mut IdentityHasher, count: usize) {
    let count = u64::try_from(count).expect("Rust collection length fits the u64 shape format");
    hasher.update_length_prefixed(&count.to_be_bytes());
}

fn update_identity(hasher: &mut IdentityHasher, identity: &VocabularyEncodedId) {
    let root = match identity.root_variant() {
        VocabularyRoot::Universal => 0_u8,
        VocabularyRoot::Rust => 1_u8,
    };
    hasher.update_length_prefixed(&[root]);
    update_count(hasher, identity.chain().len());
    for local in identity.chain() {
        hasher.update_length_prefixed(&local.value().to_be_bytes());
    }
}

fn storage_declaration_identity(item: &WholeEthosItem) -> Option<&VocabularyEncodedId> {
    match item {
        WholeEthosItem::Newtype(newtype) => Some(newtype.name()),
        WholeEthosItem::Struct(structure) => Some(structure.name()),
        WholeEthosItem::Enumeration(enumeration) => Some(enumeration.name()),
        WholeEthosItem::StreamInitiation(_) => None,
    }
}

/// Caller-authored generated identities for one complete stream lifecycle.
///
/// The authored stream declaration names initiation only. This record keeps
/// the separately generated initiation and termination operations explicit so
/// the lowerer can produce a complete contract without synthesizing names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamLifecycleIdentities {
    stream: VocabularyEncodedId,
    initiation_input: VocabularyEncodedId,
    handle: VocabularyEncodedId,
    initiation_refusal: VocabularyEncodedId,
    termination_input: VocabularyEncodedId,
    termination_refusal: VocabularyEncodedId,
}

impl StreamLifecycleIdentities {
    /// Validate one complete set of distinct Universal lifecycle identities.
    pub fn new(
        stream: VocabularyEncodedId,
        initiation_input: VocabularyEncodedId,
        handle: VocabularyEncodedId,
        initiation_refusal: VocabularyEncodedId,
        termination_input: VocabularyEncodedId,
        termination_refusal: VocabularyEncodedId,
    ) -> Result<Self, NexusTransformationError> {
        let roles = [
            ("stream", &stream),
            ("initiation input", &initiation_input),
            ("handle", &handle),
            ("initiation refusal", &initiation_refusal),
            ("termination input", &termination_input),
            ("termination refusal", &termination_refusal),
        ];
        for (role, identity) in &roles {
            if identity.root_variant() != &VocabularyRoot::Universal {
                return Err(NexusTransformationError::StreamLifecycleIdentityRoot {
                    role,
                    found: *identity.root_variant(),
                });
            }
        }
        for (index, (role, identity)) in roles.iter().enumerate() {
            if let Some((prior_role, _)) = roles[..index]
                .iter()
                .find(|(_, prior_identity)| *prior_identity == *identity)
            {
                return Err(NexusTransformationError::DuplicateStreamLifecycleIdentity {
                    first_role: prior_role,
                    second_role: role,
                    identity: (*identity).clone(),
                });
            }
        }
        Ok(Self {
            stream,
            initiation_input,
            handle,
            initiation_refusal,
            termination_input,
            termination_refusal,
        })
    }

    /// Authored stream declaration identity.
    pub const fn stream(&self) -> &VocabularyEncodedId {
        &self.stream
    }

    /// Generated initiation-input identity.
    pub const fn initiation_input(&self) -> &VocabularyEncodedId {
        &self.initiation_input
    }

    /// Generated typed-stream handle identity.
    pub const fn handle(&self) -> &VocabularyEncodedId {
        &self.handle
    }

    /// Generated initiation-refusal identity.
    pub const fn initiation_refusal(&self) -> &VocabularyEncodedId {
        &self.initiation_refusal
    }

    /// Generated termination-input identity.
    pub const fn termination_input(&self) -> &VocabularyEncodedId {
        &self.termination_input
    }

    /// Generated termination-refusal identity.
    pub const fn termination_refusal(&self) -> &VocabularyEncodedId {
        &self.termination_refusal
    }
}

/// One exact Universal-to-Rust reference relationship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NexusVocabularyReferenceMapping {
    source: VocabularyEncodedId,
    target: VocabularyEncodedId,
}

// Trait exception — too trivial: validated construction and read-only access to
// one structural mapping record.
impl NexusVocabularyReferenceMapping {
    /// Construct one exact mapping across the typed vocabulary boundary.
    pub fn new(
        source: VocabularyEncodedId,
        target: VocabularyEncodedId,
    ) -> Result<Self, NexusTransformationError> {
        if source.root_variant() != &VocabularyRoot::Universal {
            return Err(NexusTransformationError::MappingSourceRoot {
                found: *source.root_variant(),
            });
        }
        if target.root_variant() != &VocabularyRoot::Rust {
            return Err(NexusTransformationError::MappingTargetRoot {
                found: *target.root_variant(),
            });
        }
        Ok(Self { source, target })
    }

    /// Exact Universal reference identity.
    pub const fn source(&self) -> &VocabularyEncodedId {
        &self.source
    }

    /// Exact Rust vocabulary identity.
    pub const fn target(&self) -> &VocabularyEncodedId {
        &self.target
    }
}

/// Typed refusal from the Nexus structural boundary.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NexusTransformationError {
    /// The typed document selected another file kind.
    #[error("{expected:?} transformation received {found:?} Ethos")]
    UnsupportedFileKind {
        /// Required file kind for the selected transformation.
        expected: WholeEthosFileKind,
        /// Actual header/body kind.
        found: WholeEthosFileKind,
    },
    /// A generated lifecycle identity was outside Universal vocabulary.
    #[error("stream lifecycle {role} identity must be Universal, found {found:?}")]
    StreamLifecycleIdentityRoot {
        /// Lifecycle role selected by the caller.
        role: &'static str,
        /// Actual vocabulary root.
        found: VocabularyRoot,
    },
    /// Two generated lifecycle roles reused one identity.
    #[error("stream lifecycle roles {first_role} and {second_role} share identity {identity:?}")]
    DuplicateStreamLifecycleIdentity {
        /// First lifecycle role.
        first_role: &'static str,
        /// Second lifecycle role.
        second_role: &'static str,
        /// Reused identity.
        identity: VocabularyEncodedId,
    },
    /// More than one lifecycle assignment targeted one authored stream.
    #[error("stream lifecycle assignment is duplicated for {stream:?}")]
    DuplicateStreamLifecycleStream {
        /// Repeated authored stream identity.
        stream: VocabularyEncodedId,
    },
    /// The caller supplied no generated lifecycle identities for an authored
    /// stream, so Nomos cannot lower it without allocating names.
    #[error("stream lifecycle identities are missing for {stream:?}")]
    MissingStreamLifecycleIdentities {
        /// Authored stream identity.
        stream: VocabularyEncodedId,
    },
    /// A malformed Whole-Ethos value bypassed its non-empty application law.
    #[error("Nexus cannot lower an empty type application headed by {head:?}")]
    EmptyTypeApplicationArguments {
        /// Authored application head.
        head: VocabularyEncodedId,
    },
    /// A malformed Whole-Ethos parameter carried a shape where a trait bound is required.
    #[error("Nexus cannot lower shape quality {quality:?} as a type-parameter trait")]
    TypeParameterQualityMustBeTrait {
        /// Quality supplied in the wrong role.
        quality: VocabularyEncodedId,
    },
    /// A malformed Whole-Ethos application carried a trait where a shape is required.
    #[error("Nexus cannot lower trait quality {quality:?} as a type-application shape")]
    TypeApplicationHeadMustBeShape {
        /// Quality supplied in the wrong role.
        quality: VocabularyEncodedId,
    },
    /// Sema storage layouts cannot be derived from an item-local generic pickup.
    #[error("Sema storage shape cannot resolve type parameter {name:?}")]
    UnresolvedTypeParameter {
        /// Parameter name absent from concrete Sema storage.
        name: VocabularyEncodedId,
    },
    /// A tuple variant carried no payload fields.
    #[error("tuple variant {variant:?} requires at least one payload field")]
    EmptyVariantTupleFields {
        /// Exact variant identity.
        variant: VocabularyEncodedId,
    },
    /// A mapping source was not Universal vocabulary.
    #[error("Nexus mapping source must be Universal, found {found:?}")]
    MappingSourceRoot {
        /// Actual source root.
        found: VocabularyRoot,
    },
    /// A mapping target was not Rust vocabulary.
    #[error("Nexus mapping target must be Rust, found {found:?}")]
    MappingTargetRoot {
        /// Actual target root.
        found: VocabularyRoot,
    },
    /// One source reference was mapped more than once.
    #[error("Nexus mapping source {identity:?} is duplicated")]
    DuplicateMappingSource {
        /// Repeated source identity.
        identity: VocabularyEncodedId,
    },
    /// An external storage provenance owner omitted its immutable source or
    /// revision evidence.
    #[error("storage provenance owner {field} must be non-empty")]
    StorageProvenanceOwnerEmpty { field: &'static str },
    /// A successor record tried to claim an unchanged producer instead of
    /// keeping the ordinary single-owner provenance form.
    #[error("external storage successor evidence must name a distinct physical and compiled owner")]
    ExternalStorageSuccessorUnchanged,
    /// A successor record omitted its immutable proof revision.
    #[error("external storage successor evidence revision must be non-empty")]
    ExternalStorageEvidenceRevisionEmpty,
    /// A successor record omitted its immutable proof digest.
    #[error("external storage successor evidence digest must be non-zero")]
    ExternalStorageEvidenceDigestEmpty,
    /// A successor record named no covered external type identities.
    #[error("external storage successor evidence must name at least one type identity")]
    ExternalStorageSuccessorTypesEmpty,
    /// A successor record repeated one covered external type identity.
    #[error("external storage successor evidence repeats type identity {identity:?}")]
    ExternalStorageSuccessorTypeDuplicate { identity: VocabularyEncodedId },
    /// One required archive-ABI predicate was not proven.
    #[error("external storage successor evidence did not prove {check}")]
    ArchiveAbiCheckNotProven { check: &'static str },
    /// The successor's compiled owner differed from the active external owner.
    #[error(
        "external storage successor compiled owner {evidence:?} differs from configured owner {configured:?}"
    )]
    ExternalStorageSuccessorOwnerMismatch {
        configured: StorageProvenanceOwner,
        evidence: StorageProvenanceOwner,
    },
    /// The direct external identity was omitted from its successor's sealed
    /// covered set.
    #[error("external storage successor evidence does not cover {identity:?}")]
    ExternalStorageSuccessorIdentityMissing { identity: VocabularyEncodedId },
    /// External storage evidence must preserve a Universal source identity.
    #[error("external storage provenance identity must be Universal, found {found:?}")]
    StorageProvenanceIdentityRoot { found: VocabularyRoot },
    /// More than one external producer claimed the same source identity.
    #[error("external storage provenance for {identity:?} is duplicated")]
    DuplicateExternalStorageProvenance { identity: VocabularyEncodedId },
    /// A type identity was both authored by the bundle and declared external.
    #[error("storage provenance ownership conflicts for {identity:?}")]
    StorageProvenanceOwnershipConflict { identity: VocabularyEncodedId },
    /// More than one document in the bundle authored the same type identity.
    #[error("bundle storage declaration {identity:?} is duplicated")]
    DuplicateBundleStorageDeclaration { identity: VocabularyEncodedId },
    /// A reachable external storage type has no owner/revision/fingerprint
    /// evidence from its published producer.
    #[error("external storage provenance is missing for {identity:?}")]
    MissingExternalStorageProvenance { identity: VocabularyEncodedId },
    /// The locally generated stored declaration graph contains a cycle; the
    /// bounded structural fingerprint deliberately has no fixpoint machinery.
    #[error("Sema storage shape contains a cycle through {identity:?}")]
    CyclicSemaStorageShape { identity: VocabularyEncodedId },
    /// A configured positional role identity was outside Universal vocabulary.
    #[error("Interface {role} role must be Universal, found {found:?}")]
    InterfaceRoleRoot {
        /// Positional role name.
        role: &'static str,
        /// Actual identity root.
        found: VocabularyRoot,
    },
    /// Two positional roles were assigned the same trait identity.
    #[error("Interface roles {first_role} and {second_role} share identity {identity:?}")]
    DuplicateInterfaceRoleIdentity {
        /// First positional role.
        first_role: &'static str,
        /// Second positional role.
        second_role: &'static str,
        /// Reused universal trait identity.
        identity: VocabularyEncodedId,
    },
    /// A Sema record-types position contained an operator application rather
    /// than a stored value declaration.
    #[error("Sema record declaration {identity:?} is not a stored value shape")]
    InvalidSemaRecordDeclaration { identity: VocabularyEncodedId },
    /// Two Sema record declarations reused one identity.
    #[error("Sema record identity {identity:?} is declared more than once")]
    DuplicateSemaRecordIdentity { identity: VocabularyEncodedId },
    /// Two Sema tables reused one stable identity.
    #[error("Sema table identity {identity:?} is declared more than once")]
    DuplicateSemaTableIdentity { identity: VocabularyEncodedId },
    /// A table's record type was not pre-registered as a bundle declaration.
    #[error("Sema table {table:?} record {record:?} is not owned by this bundle")]
    SemaTableRecordNotBundleOwned {
        /// Stable table identity.
        table: VocabularyEncodedId,
        /// Unknown or foreign record identity.
        record: VocabularyEncodedId,
    },
    /// A Sema table attempted to store an applied type instead of one record.
    #[error("Sema table {table:?} has an unsupported record type application")]
    InvalidSemaTableRecordShape { table: VocabularyEncodedId },
    /// A Sema table attempted to use an applied key type outside the current
    /// one-identity key contract.
    #[error("Sema table {table:?} has an unsupported key type application")]
    InvalidSemaTableKeyShape { table: VocabularyEncodedId },
    /// A table key named a type outside the complete authored bundle.  An
    /// external storage fingerprint is evidence for an archive layout, never
    /// authority to define lookup semantics.
    #[error("Sema table {table:?} key {key:?} is not owned by this bundle")]
    SemaTableKeyNotBundleOwned {
        /// Stable table identity.
        table: VocabularyEncodedId,
        /// Unknown or foreign key archive identity.
        key: VocabularyEncodedId,
    },
    /// A source table key must be a newtype so generated Rust can project its
    /// one declared payload without scanning archive bytes.
    #[error("Sema table {table:?} key {key:?} is not a source newtype")]
    SemaTableKeyNotNewtype {
        /// Stable table identity.
        table: VocabularyEncodedId,
        /// Declared non-newtype key identity.
        key: VocabularyEncodedId,
    },
    /// Provenance attempted to substitute a different archive identity for a
    /// source-declared table key.
    #[error(
        "Sema table {table:?} declared key {declared:?} differs from provenanced archive identity {provenanced:?}"
    )]
    SemaTableKeyArchiveIdentityMismatch {
        /// Stable table identity.
        table: VocabularyEncodedId,
        /// Source-declared key archive identity.
        declared: VocabularyEncodedId,
        /// Identity returned by the provenance boundary.
        provenanced: VocabularyEncodedId,
    },
    /// One adopted physical family attempted to bind a non-Universal semantic
    /// identity.
    #[error("preserved Sema family {position} identity must be Universal, found {found:?}")]
    PreservedSemaFamilyIdentityRoot {
        /// Identity role in the adoption proof.
        position: &'static str,
        /// Actual vocabulary root.
        found: VocabularyRoot,
    },
    /// The proof named a physical descriptor outside the sealed current
    /// Spirit-v14 catalogue.
    #[error("preserved Sema family {family_name:?} at table {table_name:?} is not catalogued")]
    UnknownPreservedSemaFamily {
        /// Physical table coordinate.
        table_name: String,
        /// Physical record-family coordinate.
        family_name: String,
    },
    /// A catalogue family carried a non-authoritative physical schema hash.
    #[error("preserved Sema family {family_name:?} schema hash differs from the sealed v14 value")]
    PreservedSemaFamilySchemaHashMismatch {
        /// Physical family coordinate.
        family_name: String,
        /// Caller-supplied schema hash.
        found: [u8; 32],
        /// Sealed current-v14 schema hash.
        expected: [u8; 32],
    },
    /// A proof attempted to use a known physical family for a different
    /// semantic table or archived type identity.
    #[error(
        "preserved Sema family {position} identity {found:?} differs from its sealed v14 identity {expected:?}"
    )]
    PreservedSemaFamilyIdentityMismatch {
        /// Bound identity role.
        position: &'static str,
        /// Caller-supplied identity.
        found: VocabularyEncodedId,
        /// Sealed source identity.
        expected: VocabularyEncodedId,
    },
    /// The physical descriptor was not cited from the current immutable
    /// Spirit-v14 revision.
    #[error(
        "preserved Sema family Spirit revision {found:?} differs from sealed revision {expected}"
    )]
    PreservedSemaFamilyRevisionMismatch {
        /// Caller-supplied revision.
        found: String,
        /// Sealed revision.
        expected: &'static str,
    },
    /// Adoption supports only the current physical store schema.
    #[error("preserved Sema family store schema {found} differs from sealed schema {expected}")]
    PreservedSemaFamilyStoreSchemaMismatch {
        /// Caller-supplied store schema.
        found: u64,
        /// Sealed store schema.
        expected: u64,
    },
    /// More than one physical family attempted to bind one semantic table.
    #[error("preserved Sema family is duplicated for semantic table {table:?}")]
    DuplicatePreservedSemaFamily {
        /// Repeated semantic table identity.
        table: VocabularyEncodedId,
    },
    /// The cited semantic table's record or key type no longer matches the
    /// adoption proof.
    #[error(
        "preserved Sema family table {table:?} record/key identities differ from its adoption proof"
    )]
    PreservedSemaFamilyTableTypeMismatch {
        /// Semantic table identity.
        table: VocabularyEncodedId,
    },
    /// The cited semantic table's complete record/key layouts no longer match
    /// the evidence that authorized physical-descriptor adoption.
    #[error(
        "preserved Sema family table {table:?} record/key layout fingerprints differ from its adoption proof"
    )]
    PreservedSemaFamilyLayoutMismatch {
        /// Semantic table identity.
        table: VocabularyEncodedId,
    },
}

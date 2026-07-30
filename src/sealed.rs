//! Production sealing for authored Nomos.
//!
//! Immutable Capsule identity covers exactly the canonical archive of the
//! phase-stable authored transformer set. Human-readable NameTree projection is
//! a separate, versioned, integrity-authenticated archive bound to that identity.

use std::collections::BTreeSet;

use capsule_content_identity::{
    ArchiveError, CapsuleIdentity, CapsuleIdentityPreimage, DomainSeparation, HashDomain,
    IntegrityDigest, LayoutVersion, PortableArchive,
};
use encoded_name_table::Name;
use protos::{Capsule, Nomos};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::LandingShape;
use thiserror::Error;

use crate::{
    AuthoredBindingIdentity, AuthoredNomosError, AuthoredTransformerIdentity,
    AuthoredTransformerSet, LoadedNomosPopulation, NomosNameTable, TemplateFuture,
    TemplateFutureOutput, TemplateTerm, TemplateValue,
};

/// The monotonically advancing version of one Capsule's spelling projection.
#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct NameTreeProjectionVersion(u64);

impl NameTreeProjectionVersion {
    /// The first projection version for newly sealed content.
    pub const fn initial() -> Self {
        Self(0)
    }

    /// Construct an explicitly persisted projection version.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The stored integer.
    pub const fn value(self) -> u64 {
        self.0
    }

    /// The next projection version, refusing overflow.
    pub const fn checked_successor(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

/// One reachable complete encoded-ID chain and its final segment spelling.
#[derive(
    rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, Ord, PartialEq, PartialOrd,
)]
pub struct NameTreeProjectionEntry(VocabularyEncodedId, Name);

impl NameTreeProjectionEntry {
    /// The complete root-fronted identity chain.
    pub const fn encoded_id(&self) -> &VocabularyEncodedId {
        &self.0
    }

    /// The exact spelling of the chain's final segment.
    pub fn spelling(&self) -> &str {
        self.1.as_str()
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq)]
struct SealedNomosCapsuleArchive(CapsuleIdentity, AuthoredTransformerSet);

/// Immutable, content-identified authored Nomos.
///
/// Spelling, projection version, provenance, package revision, live-slot state,
/// and evaluator caches have no field in this carrier.
#[derive(Clone, Debug, PartialEq)]
pub struct SealedNomosCapsule {
    identity: CapsuleIdentity,
    transformers: AuthoredTransformerSet,
}

impl SealedNomosCapsule {
    fn seal(transformers: AuthoredTransformerSet) -> Result<Self, NomosSealError> {
        validate_encoded_chains(&transformers)?;
        let content = transformers.to_archive_bytes()?;
        let hash = CapsuleIdentityPreimage::new(content.as_ref()).derive_content_addressed_hash();
        Ok(Self {
            identity: CapsuleIdentity::nomos(hash),
            transformers,
        })
    }

    /// The immutable Nomos Capsule identity.
    pub const fn content_identity(&self) -> CapsuleIdentity {
        self.identity
    }

    /// The pure content hash within the Nomos outer variant.
    pub const fn content_addressed_hash(&self) -> capsule_content_identity::ContentAddressedHash {
        self.identity.content_addressed_hash()
    }

    /// The validated, full-chain authored declarations.
    pub const fn transformers(&self) -> &AuthoredTransformerSet {
        &self.transformers
    }

    /// A kind-typed Protos view with no projection material in its pin position.
    pub fn as_protos_capsule(&self) -> Capsule<Nomos, ()> {
        Capsule::new(self.content_addressed_hash(), ())
    }

    /// Canonical immutable Capsule bytes.
    pub fn to_archive_bytes(&self) -> Result<Vec<u8>, NomosSealError> {
        let archive = SealedNomosCapsuleArchive(self.identity, self.transformers.clone());
        Ok(archive.to_archive_bytes()?.as_ref().to_vec())
    }

    /// Restore and fully validate immutable Capsule bytes.
    pub fn from_archive_bytes(bytes: &[u8]) -> Result<Self, NomosSealError> {
        let SealedNomosCapsuleArchive(identity, transformers) =
            SealedNomosCapsuleArchive::from_archive_bytes(bytes)?;
        if !matches!(identity, CapsuleIdentity::Nomos(_)) {
            return Err(NomosSealError::WrongCapsuleKind);
        }

        let canonical = AuthoredTransformerSet::try_new(transformers.declarations().to_vec())?;
        if canonical != transformers {
            return Err(NomosSealError::NonCanonicalContent);
        }
        let sealed = Self::seal(canonical)?;
        if sealed.identity != identity {
            return Err(NomosSealError::ContentIdentityMismatch);
        }
        if sealed.to_archive_bytes()?.as_slice() != bytes {
            return Err(NomosSealError::NonCanonicalCapsuleArchive);
        }
        Ok(sealed)
    }
}

/// Integrity domain for the separately stored NameTree projection.
struct NameTreeProjectionIntegrityDomain;

impl HashDomain for NameTreeProjectionIntegrityDomain {
    fn separation() -> DomainSeparation {
        DomainSeparation::Contextual {
            context: "core-nomos reachable NameTree projection",
            layout: LayoutVersion::new(1),
        }
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
struct NameTreeProjectionPayload(
    NameTreeProjectionVersion,
    CapsuleIdentity,
    Vec<NameTreeProjectionEntry>,
);

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone)]
struct NameTreeProjectionArchive(
    NameTreeProjectionPayload,
    IntegrityDigest<NameTreeProjectionIntegrityDomain>,
);

/// A versioned, authenticated spelling projection for one immutable Capsule.
#[derive(Clone)]
pub struct AuthenticatedNameTreeProjection {
    payload: NameTreeProjectionPayload,
    integrity: IntegrityDigest<NameTreeProjectionIntegrityDomain>,
}

impl std::fmt::Debug for AuthenticatedNameTreeProjection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuthenticatedNameTreeProjection")
            .field("version", &self.version())
            .field("content_identity", &self.content_identity())
            .field("entries", &self.entries())
            .field("integrity", &self.integrity.to_hexadecimal())
            .finish()
    }
}

impl PartialEq for AuthenticatedNameTreeProjection {
    fn eq(&self, other: &Self) -> bool {
        self.payload == other.payload && self.integrity == other.integrity
    }
}

impl Eq for AuthenticatedNameTreeProjection {}

impl AuthenticatedNameTreeProjection {
    fn seal(
        capsule: &SealedNomosCapsule,
        names: &NomosNameTable,
        version: NameTreeProjectionVersion,
    ) -> Result<Self, NomosSealError> {
        let reachable = reachable_identities(capsule.transformers());
        let mut entries = Vec::with_capacity(reachable.len());
        for encoded_id in reachable {
            let spelling = names.spelling(&encoded_id).ok_or_else(|| {
                NomosSealError::MissingReachableName {
                    encoded_id: encoded_id.clone(),
                }
            })?;
            entries.push(NameTreeProjectionEntry(
                encoded_id,
                Name::new(spelling.to_owned()),
            ));
        }
        let payload = NameTreeProjectionPayload(version, capsule.content_identity(), entries);
        let integrity = IntegrityDigest::of_core(&payload)?;
        Ok(Self { payload, integrity })
    }

    /// The stored projection version.
    pub const fn version(&self) -> NameTreeProjectionVersion {
        self.payload.0
    }

    /// The immutable Capsule identity this projection renders.
    pub const fn content_identity(&self) -> CapsuleIdentity {
        self.payload.1
    }

    /// Canonically ordered reachable full-chain spellings.
    pub fn entries(&self) -> &[NameTreeProjectionEntry] {
        &self.payload.2
    }

    /// The projection integrity digest bytes.
    pub const fn integrity_bytes(&self) -> &[u8; 32] {
        self.integrity.bytes()
    }

    /// Render every segment of one complete identity chain.
    pub fn render_chain(&self, encoded_id: &VocabularyEncodedId) -> Option<Vec<&str>> {
        identity_prefixes(encoded_id)
            .into_iter()
            .map(|prefix| {
                self.entries()
                    .binary_search_by(|entry| entry.encoded_id().cmp(&prefix))
                    .ok()
                    .map(|index| self.entries()[index].spelling())
            })
            .collect()
    }

    /// Resolve an exact rendered chain through the projected tables.
    pub fn resolve_chain(
        &self,
        root: VocabularyRoot,
        spellings: &[&str],
    ) -> Option<VocabularyEncodedId> {
        let mut chain = Vec::with_capacity(spellings.len());
        for spelling in spellings {
            let next = self.entries().iter().find(|entry| {
                entry.encoded_id().root_variant() == &root
                    && entry.encoded_id().chain().len() == chain.len() + 1
                    && entry.encoded_id().chain()[..chain.len()] == chain
                    && entry.spelling() == *spelling
            })?;
            chain.push(next.encoded_id().local());
        }
        VocabularyEncodedId::new(root, chain).ok()
    }

    /// Rebuild the exact reachable NameTable view.
    pub fn to_name_table(&self) -> NomosNameTable {
        NomosNameTable::from_map(
            self.entries()
                .iter()
                .map(|entry| (entry.encoded_id().clone(), entry.1.clone()))
                .collect(),
        )
    }

    /// Canonical separately stored projection bytes.
    pub fn to_archive_bytes(&self) -> Result<Vec<u8>, NomosSealError> {
        let archive = NameTreeProjectionArchive(self.payload.clone(), self.integrity);
        Ok(archive.to_archive_bytes()?.as_ref().to_vec())
    }

    /// Restore a projection, refusing archive corruption, noncanonical ordering,
    /// and integrity mismatch.
    pub fn from_archive_bytes(bytes: &[u8]) -> Result<Self, NomosSealError> {
        let NameTreeProjectionArchive(payload, integrity) =
            NameTreeProjectionArchive::from_archive_bytes(bytes)?;
        if payload
            .2
            .iter()
            .any(|entry| entry.encoded_id().chain().is_empty())
        {
            return Err(NomosSealError::InvalidEncodedChain {
                position: "NameTree projection entry",
            });
        }
        if payload
            .2
            .windows(2)
            .any(|pair| pair[0].encoded_id() >= pair[1].encoded_id())
        {
            return Err(NomosSealError::NonCanonicalProjection);
        }
        if IntegrityDigest::of_core(&payload)? != integrity {
            return Err(NomosSealError::ProjectionIntegrityMismatch);
        }
        let projection = Self { payload, integrity };
        if projection.to_archive_bytes()?.as_slice() != bytes {
            return Err(NomosSealError::NonCanonicalProjectionArchive);
        }
        Ok(projection)
    }

    /// Verify the projection is the exact reachable closure for `capsule`.
    pub fn verify_for(&self, capsule: &SealedNomosCapsule) -> Result<(), NomosSealError> {
        if self.content_identity() != capsule.content_identity() {
            return Err(NomosSealError::ProjectionContentMismatch);
        }
        let expected = reachable_identities(capsule.transformers());
        if self.entries().len() != expected.len()
            || !self
                .entries()
                .iter()
                .map(NameTreeProjectionEntry::encoded_id)
                .eq(expected.iter())
        {
            return Err(NomosSealError::ProjectionReachabilityMismatch);
        }
        Ok(())
    }
}

/// One validated immutable Capsule and its separately stored current projection.
#[derive(Clone, Debug, PartialEq)]
pub struct SealedNomosPopulation {
    capsule: SealedNomosCapsule,
    projection: AuthenticatedNameTreeProjection,
}

impl SealedNomosPopulation {
    /// The immutable content carrier.
    pub const fn capsule(&self) -> &SealedNomosCapsule {
        &self.capsule
    }

    /// The separately versioned spelling projection.
    pub const fn projection(&self) -> &AuthenticatedNameTreeProjection {
        &self.projection
    }

    /// Restore independently persisted parts and verify their binding.
    pub fn from_archive_parts(
        capsule_bytes: &[u8],
        projection_bytes: &[u8],
    ) -> Result<Self, NomosSealError> {
        let capsule = SealedNomosCapsule::from_archive_bytes(capsule_bytes)?;
        let projection = AuthenticatedNameTreeProjection::from_archive_bytes(projection_bytes)?;
        projection.verify_for(&capsule)?;
        Ok(Self {
            capsule,
            projection,
        })
    }
}

impl LoadedNomosPopulation {
    /// Derive the immutable whole-Capsule identity without sealing projection
    /// metadata.
    pub fn content_identity(&self) -> Result<CapsuleIdentity, NomosSealError> {
        Ok(SealedNomosCapsule::seal(self.transformers().clone())?.content_identity())
    }

    /// Seal immutable content and its current reachable spelling projection.
    pub fn seal(
        &self,
        version: NameTreeProjectionVersion,
    ) -> Result<SealedNomosPopulation, NomosSealError> {
        let capsule = SealedNomosCapsule::seal(self.transformers().clone())?;
        let projection = AuthenticatedNameTreeProjection::seal(&capsule, self.names(), version)?;
        Ok(SealedNomosPopulation {
            capsule,
            projection,
        })
    }

    /// Advance projection metadata for already sealed, byte-identical content.
    pub fn advance_projection(
        &self,
        current: &SealedNomosPopulation,
    ) -> Result<SealedNomosPopulation, NomosSealError> {
        let capsule = SealedNomosCapsule::seal(self.transformers().clone())?;
        if capsule.content_identity() != current.capsule().content_identity() {
            return Err(NomosSealError::ProjectionContentMismatch);
        }
        let version = current
            .projection()
            .version()
            .checked_successor()
            .ok_or(NomosSealError::ProjectionVersionExhausted)?;
        let projection = AuthenticatedNameTreeProjection::seal(&capsule, self.names(), version)?;
        Ok(SealedNomosPopulation {
            capsule,
            projection,
        })
    }
}

fn reachable_identities(transformers: &AuthoredTransformerSet) -> BTreeSet<VocabularyEncodedId> {
    // [not-understood-by-psyche, NomosTrainAddendum-2026-07-30]
    // This reversible closure covers authored transformers, bindings, literal
    // names, Invoke targets, and every ancestor prefix. Structural constructor
    // and landing type IDs are validated below but remain shared grammar
    // vocabulary rather than per-package spelling projection.
    let mut reachable = BTreeSet::new();
    for declaration in transformers.declarations() {
        insert_identity(&mut reachable, declaration.name().encoded_id());
        for parameter in declaration.input().parameters() {
            insert_identity(&mut reachable, parameter.binding().encoded_id());
        }
        collect_value(&mut reachable, declaration.result());
        for requirement in declaration.invoke_requirements() {
            collect_future(&mut reachable, requirement.future());
        }
    }
    reachable
}

fn insert_identity(
    reachable: &mut BTreeSet<VocabularyEncodedId>,
    encoded_id: &VocabularyEncodedId,
) {
    reachable.extend(identity_prefixes(encoded_id));
}

fn identity_prefixes(encoded_id: &VocabularyEncodedId) -> Vec<VocabularyEncodedId> {
    (1..=encoded_id.chain().len())
        .map(|length| {
            VocabularyEncodedId::new(
                *encoded_id.root_variant(),
                encoded_id.chain()[..length].to_vec(),
            )
            .expect("a non-empty prefix is a valid encoded identity")
        })
        .collect()
}

fn collect_value(
    reachable: &mut BTreeSet<VocabularyEncodedId>,
    value: &TemplateValue<VocabularyRoot>,
) {
    for field in value.fields() {
        collect_term(reachable, field.term());
    }
}

fn collect_term(
    reachable: &mut BTreeSet<VocabularyEncodedId>,
    term: &TemplateTerm<VocabularyRoot>,
) {
    match term {
        TemplateTerm::Declaration(encoded_id)
        | TemplateTerm::Reference(encoded_id)
        | TemplateTerm::Literal(encoded_id) => insert_identity(reachable, encoded_id),
        TemplateTerm::Nested(value) => collect_value(reachable, value),
        TemplateTerm::Sequence(terms) => {
            for term in terms {
                collect_term(reachable, term);
            }
        }
        TemplateTerm::Future(future) => collect_future(reachable, future),
        TemplateTerm::Scalar(_) => {}
    }
}

fn collect_future(reachable: &mut BTreeSet<VocabularyEncodedId>, future: &TemplateFuture) {
    match future {
        TemplateFuture::Realize { binding, .. } | TemplateFuture::Splice { binding } => {
            insert_identity(reachable, binding.encoded_id());
        }
        TemplateFuture::Invoke(transformer) => {
            insert_identity(reachable, transformer.encoded_id());
        }
    }
}

fn validate_encoded_chains(transformers: &AuthoredTransformerSet) -> Result<(), NomosSealError> {
    for declaration in transformers.declarations() {
        validate_identity(declaration.name().encoded_id(), "transformer")?;
        AuthoredTransformerIdentity::try_new(declaration.name().encoded_id().clone())?;
        for parameter in declaration.input().parameters() {
            validate_identity(parameter.binding().encoded_id(), "binding")?;
            AuthoredBindingIdentity::try_new(parameter.binding().encoded_id().clone())?;
            validate_output(parameter.output())?;
        }
        validate_value(declaration.result())?;
        validate_output(declaration.output())?;
        for requirement in declaration.invoke_requirements() {
            validate_future(requirement.future())?;
            validate_output(requirement.output())?;
        }
    }
    Ok(())
}

fn validate_identity(
    encoded_id: &VocabularyEncodedId,
    position: &'static str,
) -> Result<(), NomosSealError> {
    if encoded_id.chain().is_empty() {
        return Err(NomosSealError::InvalidEncodedChain { position });
    }
    Ok(())
}

fn validate_value(value: &TemplateValue<VocabularyRoot>) -> Result<(), NomosSealError> {
    validate_identity(
        value.constructor().type_id().encoded_id(),
        "template constructor type",
    )?;
    for field in value.fields() {
        validate_term(field.term())?;
    }
    Ok(())
}

fn validate_term(term: &TemplateTerm<VocabularyRoot>) -> Result<(), NomosSealError> {
    match term {
        TemplateTerm::Declaration(encoded_id) => {
            validate_identity(encoded_id, "template declaration")
        }
        TemplateTerm::Reference(encoded_id) => validate_identity(encoded_id, "template reference"),
        TemplateTerm::Literal(encoded_id) => validate_identity(encoded_id, "template literal"),
        TemplateTerm::Nested(value) => validate_value(value),
        TemplateTerm::Sequence(terms) => {
            for term in terms {
                validate_term(term)?;
            }
            Ok(())
        }
        TemplateTerm::Future(future) => validate_future(future),
        TemplateTerm::Scalar(_) => Ok(()),
    }
}

fn validate_future(future: &TemplateFuture) -> Result<(), NomosSealError> {
    match future {
        TemplateFuture::Realize { binding, .. } | TemplateFuture::Splice { binding } => {
            validate_identity(binding.encoded_id(), "future binding")
        }
        TemplateFuture::Invoke(transformer) => {
            validate_identity(transformer.encoded_id(), "Invoke target")
        }
    }
}

fn validate_output(output: &TemplateFutureOutput<VocabularyRoot>) -> Result<(), NomosSealError> {
    validate_landing(output.landing())
}

fn validate_landing(landing: &LandingShape<VocabularyRoot>) -> Result<(), NomosSealError> {
    match landing {
        LandingShape::Literal(encoded_id) => validate_identity(encoded_id, "landing literal"),
        LandingShape::Type(encoded_type) => {
            validate_identity(encoded_type.encoded_id(), "landing type")
        }
        LandingShape::Sequence { element, .. } => validate_landing(element),
        LandingShape::Declaration | LandingShape::Reference | LandingShape::Scalar(_) => Ok(()),
    }
}

/// A typed refusal before partially restored or partially sealed state escapes.
#[derive(Debug, Error)]
pub enum NomosSealError {
    #[error(transparent)]
    Archive(#[from] ArchiveError),

    #[error(transparent)]
    Authored(#[from] AuthoredNomosError),

    #[error("stored Capsule has a non-Nomos outer identity variant")]
    WrongCapsuleKind,

    #[error("stored authored declarations are not in canonical order")]
    NonCanonicalContent,

    #[error("stored Capsule archive is not the canonical encoding")]
    NonCanonicalCapsuleArchive,

    #[error("stored {position} has an empty encoded-ID chain")]
    InvalidEncodedChain { position: &'static str },

    #[error("stored Capsule identity does not match its canonical authored content")]
    ContentIdentityMismatch,

    #[error("reachable encoded identity {encoded_id:?} has no spelling projection")]
    MissingReachableName { encoded_id: VocabularyEncodedId },

    #[error("stored NameTree projection entries are not strictly canonical")]
    NonCanonicalProjection,

    #[error("stored NameTree projection archive is not the canonical encoding")]
    NonCanonicalProjectionArchive,

    #[error("stored NameTree projection integrity authentication failed")]
    ProjectionIntegrityMismatch,

    #[error("NameTree projection is bound to different immutable content")]
    ProjectionContentMismatch,

    #[error("NameTree projection is not the exact reachable content closure")]
    ProjectionReachabilityMismatch,

    #[error("NameTree projection version cannot advance past u64::MAX")]
    ProjectionVersionExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_content_ignores_unreachable_projection_names() {
        let transformers = AuthoredTransformerSet::try_new(Vec::new()).expect("empty set");
        let first = LoadedNomosPopulation::from_typed(
            transformers.clone(),
            NomosNameTable::from_map(std::collections::BTreeMap::new()),
        )
        .seal(NameTreeProjectionVersion::initial())
        .expect("seal empty content");

        let unreachable = VocabularyEncodedId::new(
            VocabularyRoot::Universal,
            vec![encoded_name_table::LocalEncodedId::new(42)],
        )
        .expect("fixture identity");
        let second = LoadedNomosPopulation::from_typed(
            transformers,
            NomosNameTable::from_map(std::collections::BTreeMap::from([(
                unreachable,
                Name::new("unreachable"),
            )])),
        )
        .seal(NameTreeProjectionVersion::initial())
        .expect("seal with unreachable name");

        assert_eq!(first.capsule(), second.capsule());
        assert_eq!(first.projection(), second.projection());
    }

    #[test]
    fn projection_tamper_fails_authentication() {
        let sealed = LoadedNomosPopulation::from_typed(
            AuthoredTransformerSet::try_new(Vec::new()).expect("empty set"),
            NomosNameTable::from_map(std::collections::BTreeMap::new()),
        )
        .seal(NameTreeProjectionVersion::initial())
        .expect("seal empty content");
        let mut payload = sealed.projection().payload.clone();
        payload.0 = NameTreeProjectionVersion::new(1);
        let tampered = NameTreeProjectionArchive(payload, sealed.projection().integrity)
            .to_archive_bytes()
            .expect("archive tampered projection");

        assert!(matches!(
            AuthenticatedNameTreeProjection::from_archive_bytes(tampered.as_ref()),
            Err(NomosSealError::ProjectionIntegrityMismatch)
        ));
    }

    #[test]
    fn duplicate_leaf_spellings_under_distinct_modules_resolve_distinctly() {
        let id = |chain: &[u16]| {
            VocabularyEncodedId::new(
                VocabularyRoot::Universal,
                chain
                    .iter()
                    .copied()
                    .map(encoded_name_table::LocalEncodedId::new)
                    .collect(),
            )
            .expect("fixture identity")
        };
        let payload = NameTreeProjectionPayload(
            NameTreeProjectionVersion::initial(),
            CapsuleIdentity::nomos(
                CapsuleIdentityPreimage::new([1]).derive_content_addressed_hash(),
            ),
            vec![
                NameTreeProjectionEntry(id(&[1]), Name::new("first")),
                NameTreeProjectionEntry(id(&[1, 7]), Name::new("same")),
                NameTreeProjectionEntry(id(&[2]), Name::new("second")),
                NameTreeProjectionEntry(id(&[2, 7]), Name::new("same")),
            ],
        );
        let projection = AuthenticatedNameTreeProjection {
            integrity: IntegrityDigest::of_core(&payload).expect("projection integrity"),
            payload,
        };

        assert_eq!(
            projection.resolve_chain(VocabularyRoot::Universal, &["first", "same"]),
            Some(id(&[1, 7]))
        );
        assert_eq!(
            projection.resolve_chain(VocabularyRoot::Universal, &["second", "same"]),
            Some(id(&[2, 7]))
        );
    }

    #[test]
    fn projection_bound_to_another_outer_identity_refuses() {
        let sealed = LoadedNomosPopulation::from_typed(
            AuthoredTransformerSet::try_new(Vec::new()).expect("empty set"),
            NomosNameTable::from_map(std::collections::BTreeMap::new()),
        )
        .seal(NameTreeProjectionVersion::initial())
        .expect("seal empty content");
        let mut payload = sealed.projection().payload.clone();
        payload.1 = CapsuleIdentity::ethos(sealed.capsule().content_addressed_hash());
        let projection = AuthenticatedNameTreeProjection {
            integrity: IntegrityDigest::of_core(&payload).expect("projection integrity"),
            payload,
        };

        assert!(matches!(
            projection.verify_for(sealed.capsule()),
            Err(NomosSealError::ProjectionContentMismatch)
        ));
    }

    #[test]
    fn production_seal_module_has_no_legacy_fixture_identity_path() {
        let source = include_str!("sealed.rs");
        for forbidden in [
            concat!("Macro", "Package"),
            concat!("Macro", "Identity"),
            concat!("Package", "Revision"),
            concat!("crate::", "NameTable"),
        ] {
            assert!(
                !source.contains(forbidden),
                "production seal must not mention legacy {forbidden}"
            );
        }
    }
}

//! The contract-first, pre-gate ScopeOf transformation slice.
//!
//! This module recognizes the typed three-atom Ethos application and prepares
//! one root-level Nomos mirror of its source enumeration. It does not realize a
//! Logos value: generated output identity and recursive descent remain explicit
//! typed gates.

use core_ethos::{
    WholeEthos, WholeEthosBody, WholeEthosEnumeration, WholeEthosItem, WholeEthosTypeReference,
    WholeEthosVariantPayload, WholeEthosVisibility,
};
use core_logos::{WholeLogosEnumeration, WholeLogosVisibility};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

/// The semantic content of one recognized ScopeOf declaration.
// [assumption primary-zjo-A1 — local contract ownership]
pub trait ScopeOfDeclarationContract {
    /// The authored output declaration identity.
    fn target(&self) -> &VocabularyEncodedId;

    /// The referenced source-domain declaration identity.
    fn source(&self) -> &VocabularyEncodedId;
}

/// Recognition of the exact typed ScopeOf application in Whole Ethos.
pub trait ScopeOfDeclarationRecognition {
    /// Return `None` for an unrelated item and a typed declaration for ScopeOf.
    fn recognize(
        &self,
        item: &WholeEthosItem,
    ) -> Result<Option<ScopeOfDeclaration>, ScopeOfRefusal>;
}

/// Resolution of a recognized declaration against one complete Ethos value.
pub trait ScopeOfSourceResolution {
    /// Resolve exactly one source enumeration by its complete encoded identity.
    fn resolve<'ethos>(
        &self,
        ethos: &'ethos WholeEthos,
        declaration: &ScopeOfDeclaration,
    ) -> Result<&'ethos WholeEthosEnumeration, ScopeOfRefusal>;
}

/// Non-recursive planning of the root Nomos mirror.
pub trait ScopeOfNomosPlanning {
    /// Prepare one root enumeration without following any child edge.
    fn plan_root(
        &self,
        declaration: &ScopeOfDeclaration,
        source: &WholeEthosEnumeration,
    ) -> Result<ScopeOfNomosEnumeration, ScopeOfRefusal>;
}

/// The stopped boundary from the handwritten Nomos mirror to concrete Logos.
pub trait ScopeOfLogosRealization {
    /// Refuse until the operation/translator boundary supplies output identity.
    // [assumption primary-zjo-A9 — existing WholeLogos mirror target]
    fn realize(
        &self,
        plan: &ScopeOfNomosEnumeration,
    ) -> Result<WholeLogosEnumeration, ScopeOfRefusal>;
}

/// Inspection of every ruling gate retained by a root-level plan.
pub trait ScopeOfGateObservations {
    /// Identity gates occur for every produced variant; recursion gates occur
    /// only for payload-bearing source variants.
    fn gates(&self) -> Vec<ScopeOfGate>;
}

/// Exact configured identities needed to recognize and inspect ScopeOf.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeOfTransformer {
    scope_of_head: VocabularyEncodedId,
    root_all_variant: VocabularyEncodedId,
}

// Trait exception — too trivial: validated construction and field access only.
impl ScopeOfTransformer {
    /// Construct the pre-gate transformer from translator-issued identities.
    pub fn try_new(
        scope_of_head: VocabularyEncodedId,
        root_all_variant: VocabularyEncodedId,
    ) -> Result<Self, ScopeOfRefusal> {
        for configured in [
            ScopeOfConfiguredIdentity {
                position: ScopeOfConfiguredIdentityPosition::ScopeOfHead,
                identity: &scope_of_head,
            },
            ScopeOfConfiguredIdentity {
                position: ScopeOfConfiguredIdentityPosition::RootAllVariant,
                identity: &root_all_variant,
            },
        ] {
            if configured.identity.root_variant() != &VocabularyRoot::Universal {
                return Err(ScopeOfRefusal::NonUniversalConfiguration {
                    position: configured.position,
                    found: *configured.identity.root_variant(),
                });
            }
        }
        Ok(Self {
            scope_of_head,
            root_all_variant,
        })
    }

    /// The exact application-head identity recognized by this transformer.
    pub const fn scope_of_head(&self) -> &VocabularyEncodedId {
        &self.scope_of_head
    }

    /// The exact source-variant identity required for root `All`.
    pub const fn root_all_variant(&self) -> &VocabularyEncodedId {
        &self.root_all_variant
    }
}

struct ScopeOfConfiguredIdentity<'identity> {
    position: ScopeOfConfiguredIdentityPosition,
    identity: &'identity VocabularyEncodedId,
}

/// One recognized typed application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeOfDeclaration {
    target: VocabularyEncodedId,
    source: VocabularyEncodedId,
}

impl ScopeOfDeclarationContract for ScopeOfDeclaration {
    fn target(&self) -> &VocabularyEncodedId {
        &self.target
    }

    fn source(&self) -> &VocabularyEncodedId {
        &self.source
    }
}

impl ScopeOfDeclarationRecognition for ScopeOfTransformer {
    fn recognize(
        &self,
        item: &WholeEthosItem,
    ) -> Result<Option<ScopeOfDeclaration>, ScopeOfRefusal> {
        let WholeEthosItem::Newtype(newtype) = item else {
            return Ok(None);
        };
        let WholeEthosTypeReference::Application(application) = newtype.wrapped_field().reference()
        else {
            return Ok(None);
        };

        // [assumption primary-zjo-A2 — exact legacy spelling is recognized]
        // [assumption primary-zjo-A3 — the unsupported spelling refuses]
        // ScopeOf is a later complex transformer, not an angle Shape. Keep an
        // old type-application spelling visible as a typed refusal rather than
        // assigning it a new ontology category.
        if application.head().identity() != &self.scope_of_head {
            return Ok(None);
        }
        Err(ScopeOfRefusal::LegacyScopeOfApplicationUnsupported {
            target: newtype.name().clone(),
        })
    }
}

impl ScopeOfSourceResolution for ScopeOfTransformer {
    fn resolve<'ethos>(
        &self,
        ethos: &'ethos WholeEthos,
        declaration: &ScopeOfDeclaration,
    ) -> Result<&'ethos WholeEthosEnumeration, ScopeOfRefusal> {
        let mut resolved = None;
        let mut wrong_kind = false;

        let items = match ethos.body() {
            WholeEthosBody::Interface(body) => body.types(),
            WholeEthosBody::Nexus(body) => body.types(),
            WholeEthosBody::Sema(body) => body.record_types(),
        };
        for item in items {
            match item {
                WholeEthosItem::Enumeration(enumeration)
                    if enumeration.name() == declaration.source() =>
                {
                    if resolved.replace(enumeration).is_some() || wrong_kind {
                        return Err(ScopeOfRefusal::DuplicateSourceDeclaration {
                            source_identity: declaration.source().clone(),
                        });
                    }
                }
                WholeEthosItem::Newtype(newtype) if newtype.name() == declaration.source() => {
                    if resolved.is_some() || wrong_kind {
                        return Err(ScopeOfRefusal::DuplicateSourceDeclaration {
                            source_identity: declaration.source().clone(),
                        });
                    }
                    wrong_kind = true;
                }
                WholeEthosItem::Struct(structure) if structure.name() == declaration.source() => {
                    if resolved.is_some() || wrong_kind {
                        return Err(ScopeOfRefusal::DuplicateSourceDeclaration {
                            source_identity: declaration.source().clone(),
                        });
                    }
                    wrong_kind = true;
                }
                WholeEthosItem::StreamInitiation(initiation)
                    if initiation.stream == *declaration.source() =>
                {
                    if resolved.is_some() || wrong_kind {
                        return Err(ScopeOfRefusal::DuplicateSourceDeclaration {
                            source_identity: declaration.source().clone(),
                        });
                    }
                    wrong_kind = true;
                }
                WholeEthosItem::Newtype(_)
                | WholeEthosItem::Struct(_)
                | WholeEthosItem::Enumeration(_)
                | WholeEthosItem::StreamInitiation(_) => {}
            }
        }

        if wrong_kind {
            return Err(ScopeOfRefusal::SourceIsNotEnumeration {
                source_identity: declaration.source().clone(),
            });
        }
        resolved.ok_or_else(|| ScopeOfRefusal::SourceMissing {
            source_identity: declaration.source().clone(),
        })
    }
}

impl ScopeOfNomosPlanning for ScopeOfTransformer {
    fn plan_root(
        &self,
        declaration: &ScopeOfDeclaration,
        source: &WholeEthosEnumeration,
    ) -> Result<ScopeOfNomosEnumeration, ScopeOfRefusal> {
        // [assumption primary-zjo-A4 — one-root-level planning]
        if source.name() != declaration.source() {
            return Err(ScopeOfRefusal::SourceIdentityMismatch {
                expected: declaration.source().clone(),
                found: source.name().clone(),
            });
        }

        // [assumption primary-zjo-A8 — source carries root All]
        let root_all_count = source
            .variants()
            .iter()
            .filter(|variant| variant.name() == &self.root_all_variant)
            .count();
        match root_all_count {
            0 => {
                return Err(ScopeOfRefusal::RootAllMissing {
                    source_identity: source.name().clone(),
                });
            }
            1 => {}
            found => {
                return Err(ScopeOfRefusal::DuplicateRootAll {
                    source_identity: source.name().clone(),
                    found,
                });
            }
        }

        // [assumption primary-zjo-A7 — visibility and source order are retained]
        let visibility = match source.visibility() {
            WholeEthosVisibility::Public => WholeLogosVisibility::Public,
            WholeEthosVisibility::Private => WholeLogosVisibility::Private,
        };
        let mut variants = Vec::with_capacity(source.variants().len());
        for variant in source.variants() {
            // [assumption primary-zjo-A6 — admitted one-level payload shapes]
            let payload = match variant.payload() {
                WholeEthosVariantPayload::Unit => ScopeOfNomosVariantPayload::Unit,
                WholeEthosVariantPayload::Tuple(fields) if fields.fields().len() == 1 => {
                    let WholeEthosTypeReference::Identity(source_domain) = &fields.fields()[0]
                    else {
                        return Err(ScopeOfRefusal::ChildIsNotIdentity {
                            variant: variant.name().clone(),
                        });
                    };
                    // [assumption primary-zjo-A10 — child edge is not traversal]
                    ScopeOfNomosVariantPayload::Child {
                        source_domain: source_domain.clone(),
                    }
                }
                WholeEthosVariantPayload::Tuple(fields) => {
                    return Err(ScopeOfRefusal::ChildFieldCount {
                        variant: variant.name().clone(),
                        found: fields.fields().len(),
                    });
                }
            };

            variants.push(ScopeOfNomosVariant {
                // [assumption primary-zjo-A5 — unresolved name dependency]
                // [assumption primary-zjo-A11 — only the translator allocates identity]
                name: ScopeOfVariantNamePromise {
                    source_variant: variant.name().clone(),
                },
                payload,
            });
        }

        Ok(ScopeOfNomosEnumeration {
            visibility,
            name: declaration.target().clone(),
            variants,
        })
    }
}

impl ScopeOfLogosRealization for ScopeOfTransformer {
    fn realize(
        &self,
        plan: &ScopeOfNomosEnumeration,
    ) -> Result<WholeLogosEnumeration, ScopeOfRefusal> {
        let Some(first) = plan.variants.first() else {
            return Err(ScopeOfRefusal::PlannedRootHasNoVariants {
                target: plan.name.clone(),
            });
        };
        Err(ScopeOfRefusal::GeneratedOutputIdentityRequired {
            source_variant: first.name.source_variant.clone(),
        })
    }
}

/// The handwritten Nomos counterpart of [`WholeLogosEnumeration`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeOfNomosEnumeration {
    visibility: WholeLogosVisibility,
    name: VocabularyEncodedId,
    variants: Vec<ScopeOfNomosVariant>,
}

// Trait exception — too trivial: read-only field access for the mirror contract.
impl ScopeOfNomosEnumeration {
    /// Output visibility retained by the mirror.
    pub const fn visibility(&self) -> &WholeLogosVisibility {
        &self.visibility
    }

    /// The authored root output declaration identity.
    pub const fn name(&self) -> &VocabularyEncodedId {
        &self.name
    }

    /// Root variants in source order.
    pub fn variants(&self) -> &[ScopeOfNomosVariant] {
        &self.variants
    }
}

impl ScopeOfGateObservations for ScopeOfNomosEnumeration {
    fn gates(&self) -> Vec<ScopeOfGate> {
        let mut gates = Vec::new();
        for variant in &self.variants {
            gates.push(ScopeOfGate::GeneratedOutputIdentity {
                source_variant: variant.name.source_variant.clone(),
            });
            if let ScopeOfNomosVariantPayload::Child { source_domain } = &variant.payload {
                gates.push(ScopeOfGate::RecursiveDescent {
                    source_domain: source_domain.clone(),
                });
            }
        }
        gates
    }
}

/// The handwritten Nomos counterpart of one `WholeLogosVariant`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeOfNomosVariant {
    name: ScopeOfVariantNamePromise,
    payload: ScopeOfNomosVariantPayload,
}

// Trait exception — too trivial: read-only field access for the mirror contract.
impl ScopeOfNomosVariant {
    /// The unresolved name dependency for this output variant.
    pub const fn name(&self) -> &ScopeOfVariantNamePromise {
        &self.name
    }

    /// The one-level payload plan.
    pub const fn payload(&self) -> &ScopeOfNomosVariantPayload {
        &self.payload
    }
}

/// A dependency promise, not a generated output identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeOfVariantNamePromise {
    source_variant: VocabularyEncodedId,
}

// Trait exception — too trivial: read-only field access for the promise.
impl ScopeOfVariantNamePromise {
    /// The source variant on which future output identity depends.
    pub const fn source_variant(&self) -> &VocabularyEncodedId {
        &self.source_variant
    }
}

/// One-level payload planning without recursive descent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeOfNomosVariantPayload {
    /// The source variant has no child domain.
    Unit,
    /// A child dependency is recorded but never resolved or traversed here.
    Child {
        /// The existing source-domain declaration identity.
        source_domain: VocabularyEncodedId,
    },
}

/// An unresolved requirement encountered before concrete Logos can exist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeOfGate {
    /// A produced variant requires output identity from the operation/translator boundary.
    GeneratedOutputIdentity {
        /// The source variant that makes the dependency visible.
        source_variant: VocabularyEncodedId,
    },
    /// A payload-bearing source variant requires a separately ruled recursive walk.
    RecursiveDescent {
        /// The referenced child-domain declaration.
        source_domain: VocabularyEncodedId,
    },
}

/// Configured identity positions validated before recognition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeOfConfiguredIdentityPosition {
    /// The exact ScopeOf application head.
    ScopeOfHead,
    /// The exact root `All` source variant.
    RootAllVariant,
}

/// Typed refusals from the pre-gate ScopeOf slice.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ScopeOfRefusal {
    /// The superseded angle-form ScopeOf spelling belongs to the later
    /// transformer schema, not to the strict type-application ontology.
    #[error("ScopeOf declaration {target:?} uses an unsupported legacy type application")]
    LegacyScopeOfApplicationUnsupported { target: VocabularyEncodedId },
    /// A configured identity did not belong to Universal vocabulary.
    #[error("configured {position:?} identity belongs to {found:?}; expected Universal")]
    NonUniversalConfiguration {
        /// Which configured identity was invalid.
        position: ScopeOfConfiguredIdentityPosition,
        /// The unexpected vocabulary root.
        found: VocabularyRoot,
    },
    /// A matching ScopeOf application carried a non-identity operand.
    #[error("ScopeOf target {target:?} has a non-identity source operand")]
    SourceOperandNotIdentity {
        /// The authored target declaration.
        target: VocabularyEncodedId,
    },
    /// No item declared the referenced source identity.
    #[error("ScopeOf source {source_identity:?} is missing")]
    SourceMissing {
        /// The unresolved source declaration.
        source_identity: VocabularyEncodedId,
    },
    /// The referenced source was a newtype rather than an enumeration.
    #[error("ScopeOf source {source_identity:?} is not an enumeration")]
    SourceIsNotEnumeration {
        /// The wrong-kind source declaration.
        source_identity: VocabularyEncodedId,
    },
    /// More than one item declared the referenced source identity.
    #[error("ScopeOf source {source_identity:?} is declared more than once")]
    DuplicateSourceDeclaration {
        /// The duplicated source declaration.
        source_identity: VocabularyEncodedId,
    },
    /// A caller paired a declaration with the wrong source enumeration.
    #[error("ScopeOf source mismatch: expected {expected:?}, found {found:?}")]
    SourceIdentityMismatch {
        /// Source referenced by the ScopeOf declaration.
        expected: VocabularyEncodedId,
        /// Source supplied to planning.
        found: VocabularyEncodedId,
    },
    /// The source did not carry its required root `All` variant.
    #[error("ScopeOf source {source_identity:?} has no root All variant")]
    RootAllMissing {
        /// The source enumeration.
        source_identity: VocabularyEncodedId,
    },
    /// The source repeated the configured root `All` identity.
    #[error("ScopeOf source {source_identity:?} repeats root All {found} times")]
    DuplicateRootAll {
        /// The source enumeration.
        source_identity: VocabularyEncodedId,
        /// Number of matching variants.
        found: usize,
    },
    /// A payload-bearing source variant did not have exactly one field.
    #[error("ScopeOf variant {variant:?} has {found} child fields; expected one")]
    ChildFieldCount {
        /// The source variant.
        variant: VocabularyEncodedId,
        /// Number of positional fields.
        found: usize,
    },
    /// A payload-bearing source variant referenced an application instead of one child domain.
    #[error("ScopeOf variant {variant:?} has a non-identity child reference")]
    ChildIsNotIdentity {
        /// The source variant.
        variant: VocabularyEncodedId,
    },
    /// Concrete output cannot be created before output identity is ruled and supplied.
    #[error("ScopeOf output variant derived from {source_variant:?} requires generated identity")]
    GeneratedOutputIdentityRequired {
        /// The source variant exposing the first output-identity gate.
        source_variant: VocabularyEncodedId,
    },
    /// An invalid externally restored plan had no variants.
    #[error("planned ScopeOf root {target:?} has no variants")]
    PlannedRootHasNoVariants {
        /// The authored root output declaration.
        target: VocabularyEncodedId,
    },
}

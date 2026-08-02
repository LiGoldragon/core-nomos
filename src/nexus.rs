//! Allocation-free structural lowering for Nexus and Interface type documents.
//!
//! Nexus traits are emitted before their operand types, following the trait-first
//! ontology discipline. Declarations retain their translator-issued identities;
//! only exact caller-supplied reference mappings may cross into Rust vocabulary.
//! Nexus declarations remain plain. The narrow Interface surface lowers only its
//! shared `types` position and selects the canonical `WireAttributes` emission
//! policy; Input/Output/Refusal membership and refusal behavior remain outside
//! this slice.

use nexus_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosEnumeration, WholeEthosFileKind,
    WholeEthosItem, WholeEthosMethod, WholeEthosNewtype, WholeEthosOperatorApplication,
    WholeEthosStruct, WholeEthosTrait, WholeEthosTypeApplication, WholeEthosTypeReference,
    WholeEthosVariant, WholeEthosVariantPayload, WholeEthosVisibility,
};
use nexus_core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype, WholeLogosStruct,
    WholeLogosTraitDef, WholeLogosTraitMethod, WholeLogosTupleFields, WholeLogosTypeApplication,
    WholeLogosTypeAttributes, WholeLogosTypeReference, WholeLogosVariant, WholeLogosVariantPayload,
    WholeLogosVisibility,
};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

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

/// Exact, allocation-free Nexus lowering data.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NexusTransformation {
    reference_mappings: Vec<NexusVocabularyReferenceMapping>,
}

// Trait exception — too trivial: constructor and read-only data ergonomics for
// the NexusStructuralTransformation implementation.
impl NexusTransformation {
    /// Construct an identity-reference Nexus transformation.
    pub const fn new() -> Self {
        Self {
            reference_mappings: Vec::new(),
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
        Ok(Self { reference_mappings })
    }

    /// Canonically ordered exact reference mappings.
    pub fn reference_mappings(&self) -> &[NexusVocabularyReferenceMapping] {
        &self.reference_mappings
    }

    fn lower_item(
        &self,
        item: &WholeEthosItem,
        attributes: WholeLogosTypeAttributes,
    ) -> Result<WholeLogosItem, NexusTransformationError> {
        match item {
            WholeEthosItem::Newtype(newtype) => Ok(WholeLogosItem::Newtype(
                self.lower_newtype(newtype).with_attributes(attributes),
            )),
            WholeEthosItem::Struct(structure) => Ok(WholeLogosItem::Struct(
                self.lower_struct(structure).with_attributes(attributes),
            )),
            WholeEthosItem::Enumeration(enumeration) => Ok(WholeLogosItem::Enumeration(
                self.lower_enumeration(enumeration)?
                    .with_attributes(attributes),
            )),
            WholeEthosItem::OperatorApplication(application) => {
                Err(Self::unsupported_application(application))
            }
        }
    }

    fn lower_newtype(&self, newtype: &WholeEthosNewtype) -> WholeLogosNewtype {
        let WholeEthosAttributes = *newtype.attributes();
        WholeLogosNewtype::new(
            Self::lower_visibility(*newtype.visibility()),
            newtype.name().clone(),
            Self::lower_visibility(*newtype.wrapped_field().visibility()),
            self.lower_reference(newtype.wrapped_field().reference()),
        )
    }

    fn lower_struct(&self, structure: &WholeEthosStruct) -> WholeLogosStruct {
        WholeLogosStruct::new(
            WholeLogosVisibility::Public,
            structure.name().clone(),
            structure
                .fields()
                .iter()
                .map(|field| self.lower_reference(field))
                .collect(),
        )
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
                        .collect(),
                )
                .map_err(|error| {
                    NexusTransformationError::UnsupportedVariantTupleArity {
                        found: error.found(),
                    }
                })?;
                WholeLogosVariantPayload::Tuple(fields)
            }
        };
        Ok(WholeLogosVariant::new(variant.name().clone(), payload))
    }

    fn lower_trait(&self, trait_definition: &WholeEthosTrait) -> WholeLogosTraitDef {
        WholeLogosTraitDef::new(
            WholeLogosVisibility::Public,
            trait_definition.name().clone(),
            trait_definition
                .methods()
                .iter()
                .map(|method| self.lower_method(method))
                .collect(),
        )
    }

    fn lower_method(&self, method: &WholeEthosMethod) -> WholeLogosTraitMethod {
        WholeLogosTraitMethod::new(
            method.name().clone(),
            method
                .parameters()
                .iter()
                .map(|parameter| self.lower_reference(parameter))
                .collect(),
            self.lower_reference(method.return_type()),
        )
    }

    fn lower_reference(&self, reference: &WholeEthosTypeReference) -> WholeLogosTypeReference {
        match reference {
            WholeEthosTypeReference::Identity(identity) => {
                WholeLogosTypeReference::Identity(self.map_reference(identity))
            }
            WholeEthosTypeReference::Application(application) => {
                WholeLogosTypeReference::Application(self.lower_application(application))
            }
        }
    }

    fn lower_application(
        &self,
        application: &WholeEthosTypeApplication,
    ) -> WholeLogosTypeApplication {
        WholeLogosTypeApplication::new(
            self.map_reference(application.head()),
            self.lower_reference(application.payload()),
        )
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

    fn unsupported_application(
        application: &WholeEthosOperatorApplication,
    ) -> NexusTransformationError {
        NexusTransformationError::UnsupportedOperatorApplication {
            operator: application.operator().clone(),
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
            body.traits().iter().map(|trait_definition| {
                WholeLogosItem::TraitDef(self.lower_trait(trait_definition))
            }),
        );
        for item in body.types() {
            items.push(self.lower_item(item, WholeLogosTypeAttributes::Plain)?);
        }
        Ok(WholeLogos::new(items))
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
        let items = body
            .types()
            .iter()
            .map(|item| self.lower_item(item, WholeLogosTypeAttributes::Wire))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WholeLogos::new(items))
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
    /// Operator-family semantics are outside this transformer.
    #[error("Nexus type section contains unsupported operator application {operator:?}")]
    UnsupportedOperatorApplication {
        /// Authored operator identity.
        operator: VocabularyEncodedId,
    },
    /// A tuple variant carried anything other than one payload field.
    #[error("tuple variant payload requires exactly one field, found {found}")]
    UnsupportedVariantTupleArity {
        /// Refused positional-field count.
        found: usize,
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
}

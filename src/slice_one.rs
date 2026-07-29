//! The direct typed transformation used by the first vertical slice.
//!
//! This path consumes and produces positional carriers. It preserves complete
//! encoded-ID chains and has no access to any legacy or identity-allocation
//! facility.

use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use slice_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosEnumeration, WholeEthosItem, WholeEthosNewtype,
    WholeEthosTypeApplication, WholeEthosTypeReference, WholeEthosVariant,
    WholeEthosVariantPayload, WholeEthosVisibility,
};
use slice_core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype, WholeLogosTupleFields,
    WholeLogosTypeApplication, WholeLogosTypeReference, WholeLogosVariant,
    WholeLogosVariantPayload, WholeLogosVisibility,
};

/// The complete Ethos-to-Logos transformation admitted by the first slice.
///
/// Its ordered reference mappings are typed transformation data. Declarations
/// retain their Universal identities while explicitly mapped references move
/// to immutable language vocabulary.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SliceOneTransformation(Vec<SliceOneVocabularyReferenceMapping>);

impl SliceOneTransformation {
    /// Construct the identity first-slice transformation.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Construct a transformation with exact typed reference mappings.
    ///
    /// Mapping order is not semantic. Sources are stored canonically, and one
    /// source cannot be rebound by a later entry.
    pub fn with_reference_mappings(
        mut mappings: Vec<SliceOneVocabularyReferenceMapping>,
    ) -> Result<Self, SliceOneTransformationError> {
        mappings.sort_by(|left, right| left.source().cmp(right.source()));
        for adjacent in mappings.windows(2) {
            if adjacent[0].source() == adjacent[1].source() {
                return Err(SliceOneTransformationError::DuplicateMappingSource {
                    source: adjacent[0].source().clone(),
                });
            }
        }
        Ok(Self(mappings))
    }

    /// Canonically ordered exact reference mappings.
    pub fn reference_mappings(&self) -> &[SliceOneVocabularyReferenceMapping] {
        &self.0
    }

    /// Lower ordered whole-Ethos content into ordered whole-Logos content.
    pub fn lower(&self, ethos: &WholeEthos) -> WholeLogos {
        WholeLogos::new(
            ethos
                .items()
                .iter()
                .map(|item| self.lower_item(item))
                .collect(),
        )
    }

    fn lower_item(&self, item: &WholeEthosItem) -> WholeLogosItem {
        match item {
            WholeEthosItem::Newtype(newtype) => {
                WholeLogosItem::Newtype(self.lower_newtype(newtype))
            }
            WholeEthosItem::Enumeration(enumeration) => {
                WholeLogosItem::Enumeration(self.lower_enumeration(enumeration))
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

    fn lower_enumeration(&self, enumeration: &WholeEthosEnumeration) -> WholeLogosEnumeration {
        let WholeEthosAttributes = *enumeration.attributes();

        WholeLogosEnumeration::new(
            Self::lower_visibility(*enumeration.visibility()),
            enumeration.name().clone(),
            enumeration
                .variants()
                .iter()
                .map(|variant| self.lower_variant(variant))
                .collect(),
        )
    }

    fn lower_variant(&self, variant: &WholeEthosVariant) -> WholeLogosVariant {
        let WholeEthosAttributes = *variant.attributes();
        let payload = match variant.payload() {
            WholeEthosVariantPayload::Unit => WholeLogosVariantPayload::Unit,
            WholeEthosVariantPayload::Tuple(fields) => {
                let result = WholeLogosTupleFields::new(
                    fields
                        .fields()
                        .iter()
                        .map(|reference| self.lower_reference(reference))
                        .collect(),
                );
                let Ok(fields) = result else { unreachable!() };
                WholeLogosVariantPayload::Tuple(fields)
            }
        };
        WholeLogosVariant::new(variant.name().clone(), payload)
    }

    fn lower_reference(&self, reference: &WholeEthosTypeReference) -> WholeLogosTypeReference {
        match reference {
            WholeEthosTypeReference::Identity(encoded_id) => {
                WholeLogosTypeReference::Identity(self.map_reference(encoded_id))
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
        self.0
            .binary_search_by(|mapping| mapping.source().cmp(source))
            .map(|index| self.0[index].target().clone())
            .unwrap_or_else(|_| source.clone())
    }

    const fn lower_visibility(visibility: WholeEthosVisibility) -> WholeLogosVisibility {
        match visibility {
            WholeEthosVisibility::Public => WholeLogosVisibility::Public,
            WholeEthosVisibility::Private => WholeLogosVisibility::Private,
        }
    }
}

/// One exact Universal-to-Rust reference relationship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceOneVocabularyReferenceMapping(VocabularyEncodedId, VocabularyEncodedId);

impl SliceOneVocabularyReferenceMapping {
    /// Construct one mapping, refusing roots outside its typed boundary.
    pub fn new(
        source: VocabularyEncodedId,
        target: VocabularyEncodedId,
    ) -> Result<Self, SliceOneTransformationError> {
        if source.root_variant() != &VocabularyRoot::Universal {
            return Err(SliceOneTransformationError::MappingSourceRoot {
                found: *source.root_variant(),
            });
        }
        if target.root_variant() != &VocabularyRoot::Rust {
            return Err(SliceOneTransformationError::MappingTargetRoot {
                found: *target.root_variant(),
            });
        }
        Ok(Self(source, target))
    }

    /// The exact Universal reference identity.
    pub const fn source(&self) -> &VocabularyEncodedId {
        &self.0
    }

    /// The exact Rust vocabulary identity used by Logos.
    pub const fn target(&self) -> &VocabularyEncodedId {
        &self.1
    }
}

/// Typed refusal while constructing first-slice transformation data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SliceOneTransformationError {
    /// A mapping source was not Universal vocabulary.
    MappingSourceRoot {
        /// Root carried by the refused source.
        found: VocabularyRoot,
    },
    /// A mapping target was not Rust vocabulary.
    MappingTargetRoot {
        /// Root carried by the refused target.
        found: VocabularyRoot,
    },
    /// More than one mapping tried to bind the same Universal source.
    DuplicateMappingSource {
        /// Repeated exact source identity.
        source: VocabularyEncodedId,
    },
}

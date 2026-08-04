//! The direct typed transformation used by the first vertical slice.
//!
//! This path consumes and produces typed carriers. It preserves complete
//! encoded-ID chains and has no access to any legacy or identity-allocation
//! facility.

use core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosEnumeration, WholeEthosItem,
    WholeEthosNewtype, WholeEthosQuality, WholeEthosStruct, WholeEthosTypeApplication,
    WholeEthosTypeParameter, WholeEthosTypeReference, WholeEthosVariant,
    WholeEthosVariantPayload, WholeEthosVisibility,
};
use core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype, WholeLogosTupleFields,
    WholeLogosStruct, WholeLogosTypeApplication, WholeLogosTypeParameter,
    WholeLogosTypeReference, WholeLogosVariant, WholeLogosVariantPayload, WholeLogosVisibility,
};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};

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
    pub fn lower(&self, ethos: &WholeEthos) -> Result<WholeLogos, SliceOneTransformationError> {
        let mut items = Vec::new();
        match ethos.body() {
            WholeEthosBody::Interface(body) => {
                items.extend(body.inputs().iter().map(|newtype| {
                    self.lower_newtype(newtype).map(WholeLogosItem::Newtype)
                }).collect::<Result<Vec<_>, _>>()?);
                items.extend(body.outputs().iter().map(|newtype| {
                    self.lower_newtype(newtype).map(WholeLogosItem::Newtype)
                }).collect::<Result<Vec<_>, _>>()?);
                items.extend(body.refusals().iter().map(|structure| {
                    self.lower_struct(structure).map(WholeLogosItem::Struct)
                }).collect::<Result<Vec<_>, _>>()?);
                items.extend(body.types().iter().map(|item| self.lower_item(item))
                    .collect::<Result<Vec<_>, _>>()?);
            }
            WholeEthosBody::Nexus(body) => {
                items.extend(body.types().iter().map(|item| self.lower_item(item))
                    .collect::<Result<Vec<_>, _>>()?);
                if !body.traits().is_empty() {
                    return Err(SliceOneTransformationError::UnsupportedNexusTraits);
                }
            }
            WholeEthosBody::Sema(body) => {
                items.extend(body.record_types().iter().map(|item| self.lower_item(item))
                    .collect::<Result<Vec<_>, _>>()?);
                if !body.tables().is_empty() {
                    return Err(SliceOneTransformationError::UnsupportedSemaTables);
                }
            }
        }
        Ok(WholeLogos::new(items))
    }

    fn lower_item(&self, item: &WholeEthosItem) -> Result<WholeLogosItem, SliceOneTransformationError> {
        match item {
            WholeEthosItem::Newtype(newtype) => self.lower_newtype(newtype).map(WholeLogosItem::Newtype),
            WholeEthosItem::Struct(structure) => self.lower_struct(structure).map(WholeLogosItem::Struct),
            WholeEthosItem::Enumeration(enumeration) => self.lower_enumeration(enumeration).map(WholeLogosItem::Enumeration),
            WholeEthosItem::StreamInitiation(initiation) => Err(SliceOneTransformationError::StreamLifecycleIdentitiesRequired {
                stream: initiation.stream.clone(),
            }),
        }
    }

    fn lower_newtype(&self, newtype: &WholeEthosNewtype) -> Result<WholeLogosNewtype, SliceOneTransformationError> {
        let WholeEthosAttributes = *newtype.attributes();

        Ok(WholeLogosNewtype::new(
            Self::lower_visibility(*newtype.visibility()),
            newtype.name().clone(),
            Self::lower_visibility(*newtype.wrapped_field().visibility()),
            self.lower_reference(newtype.wrapped_field().reference())?,
        ).with_type_parameters(newtype.type_parameters().iter().map(Self::lower_type_parameter)
            .collect::<Result<Vec<_>, _>>()?))
    }

    fn lower_struct(&self, structure: &WholeEthosStruct) -> Result<WholeLogosStruct, SliceOneTransformationError> {
        if structure.fields().iter().any(reference_contains_parameter) {
            return Err(SliceOneTransformationError::UnsupportedParameterizedDeclaration {
                kind: "struct",
                declaration: structure.name().clone(),
            });
        }
        Ok(WholeLogosStruct::new(
            WholeLogosVisibility::Public,
            structure.name().clone(),
            structure.fields().iter().map(|field| self.lower_reference(field))
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn lower_enumeration(&self, enumeration: &WholeEthosEnumeration) -> Result<WholeLogosEnumeration, SliceOneTransformationError> {
        let WholeEthosAttributes = *enumeration.attributes();
        if enumeration.variants().iter().any(|variant| match variant.payload() {
            WholeEthosVariantPayload::Unit => false,
            WholeEthosVariantPayload::Tuple(fields) => {
                fields.fields().iter().any(reference_contains_parameter)
            }
        }) {
            return Err(SliceOneTransformationError::UnsupportedParameterizedDeclaration {
                kind: "enumeration",
                declaration: enumeration.name().clone(),
            });
        }

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

    fn lower_variant(&self, variant: &WholeEthosVariant) -> Result<WholeLogosVariant, SliceOneTransformationError> {
        let WholeEthosAttributes = *variant.attributes();
        let payload = match variant.payload() {
            WholeEthosVariantPayload::Unit => WholeLogosVariantPayload::Unit,
            WholeEthosVariantPayload::Tuple(fields) => {
                let fields = WholeLogosTupleFields::new(
                    fields
                        .fields()
                        .iter()
                        .map(|reference| self.lower_reference(reference))
                        .collect::<Result<Vec<_>, _>>()?,
                ).map_err(|_| SliceOneTransformationError::EmptyTupleFields {
                    variant: variant.name().clone(),
                })?;
                WholeLogosVariantPayload::Tuple(fields)
            }
        };
        Ok(WholeLogosVariant::new(variant.name().clone(), payload))
    }

    fn lower_reference(&self, reference: &WholeEthosTypeReference) -> Result<WholeLogosTypeReference, SliceOneTransformationError> {
        Ok(match reference {
            WholeEthosTypeReference::Identity(encoded_id) => {
                WholeLogosTypeReference::Identity(self.map_reference(encoded_id))
            }
            WholeEthosTypeReference::Parameter(parameter) => {
                WholeLogosTypeReference::Parameter(parameter.name().clone())
            }
            WholeEthosTypeReference::Application(application) => {
                WholeLogosTypeReference::Application(self.lower_application(application)?)
            }
        })
    }

    fn lower_type_parameter(parameter: &WholeEthosTypeParameter) -> Result<WholeLogosTypeParameter, SliceOneTransformationError> {
        let WholeEthosQuality::Trait(bound) = parameter.quality() else {
            return Err(SliceOneTransformationError::TypeParameterQualityMustBeTrait {
                quality: parameter.quality().identity().clone(),
            });
        };
        Ok(WholeLogosTypeParameter::new(parameter.name().clone(), bound.clone()))
    }

    fn lower_application(
        &self,
        application: &WholeEthosTypeApplication,
    ) -> Result<WholeLogosTypeApplication, SliceOneTransformationError> {
        let WholeEthosQuality::Shape(head) = application.head() else {
            return Err(SliceOneTransformationError::TypeApplicationHeadMustBeShape {
                quality: application.head().identity().clone(),
            });
        };
        WholeLogosTypeApplication::new(
            self.map_reference(head),
            application.arguments().iter().map(|argument| self.lower_reference(argument))
                .collect::<Result<Vec<_>, _>>()?,
        ).map_err(|_| SliceOneTransformationError::EmptyTypeApplicationArguments { head: head.clone() })
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

fn reference_contains_parameter(reference: &WholeEthosTypeReference) -> bool {
    match reference {
        WholeEthosTypeReference::Identity(_) => false,
        WholeEthosTypeReference::Parameter(_) => true,
        WholeEthosTypeReference::Application(application) => {
            application.arguments().iter().any(reference_contains_parameter)
        }
    }
}

/// One exact Universal-to-Rust reference relationship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceOneVocabularyReferenceMapping {
    source: VocabularyEncodedId,
    target: VocabularyEncodedId,
}

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
        Ok(Self { source, target })
    }

    /// The exact Universal reference identity.
    pub const fn source(&self) -> &VocabularyEncodedId {
        &self.source
    }

    /// The exact Rust vocabulary identity used by Logos.
    pub const fn target(&self) -> &VocabularyEncodedId {
        &self.target
    }
}

/// Typed refusal while constructing first-slice transformation data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SliceOneTransformationError {
    /// An item-local parameter must retain a trait-quality bound.
    TypeParameterQualityMustBeTrait { quality: VocabularyEncodedId },
    /// A type application may only be headed by a structural Shape.
    TypeApplicationHeadMustBeShape { quality: VocabularyEncodedId },
    /// The destination carrier rejects an empty application argument sequence.
    EmptyTypeApplicationArguments { head: VocabularyEncodedId },
    /// The destination carrier rejects an empty variant tuple.
    EmptyTupleFields { variant: VocabularyEncodedId },
    /// First-slice has no translator-issued lifecycle identities to lower this stream.
    StreamLifecycleIdentitiesRequired { stream: VocabularyEncodedId },
    /// Logos has no parameter carrier for this declaration shape yet.
    UnsupportedParameterizedDeclaration {
        kind: &'static str,
        declaration: VocabularyEncodedId,
    },
    /// Trait definitions are outside the direct first-slice projection.
    UnsupportedNexusTraits,
    /// Sema table storage fingerprints are outside the direct first-slice projection.
    UnsupportedSemaTables,
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

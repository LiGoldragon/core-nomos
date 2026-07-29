//! The direct typed transformation used by the first vertical slice.
//!
//! This path consumes and produces positional carriers. It preserves complete
//! encoded-ID chains and has no access to any legacy or identity-allocation
//! facility.

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
/// This zero-state value makes the transformation itself data while keeping
/// every operation total over the slice's closed item vocabulary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SliceOneTransformation;

impl SliceOneTransformation {
    /// Construct the first-slice transformation.
    pub const fn new() -> Self {
        Self
    }

    /// Lower ordered whole-Ethos content into ordered whole-Logos content.
    pub fn lower(self, ethos: &WholeEthos) -> WholeLogos {
        WholeLogos::new(ethos.items().iter().map(Self::lower_item).collect())
    }

    fn lower_item(item: &WholeEthosItem) -> WholeLogosItem {
        match item {
            WholeEthosItem::Newtype(newtype) => {
                WholeLogosItem::Newtype(Self::lower_newtype(newtype))
            }
            WholeEthosItem::Enumeration(enumeration) => {
                WholeLogosItem::Enumeration(Self::lower_enumeration(enumeration))
            }
        }
    }

    fn lower_newtype(newtype: &WholeEthosNewtype) -> WholeLogosNewtype {
        let WholeEthosAttributes = *newtype.attributes();

        WholeLogosNewtype::new(
            Self::lower_visibility(*newtype.visibility()),
            newtype.name().clone(),
            Self::lower_visibility(*newtype.wrapped_field().visibility()),
            Self::lower_reference(newtype.wrapped_field().reference()),
        )
    }

    fn lower_enumeration(enumeration: &WholeEthosEnumeration) -> WholeLogosEnumeration {
        let WholeEthosAttributes = *enumeration.attributes();

        WholeLogosEnumeration::new(
            Self::lower_visibility(*enumeration.visibility()),
            enumeration.name().clone(),
            enumeration
                .variants()
                .iter()
                .map(Self::lower_variant)
                .collect(),
        )
    }

    fn lower_variant(variant: &WholeEthosVariant) -> WholeLogosVariant {
        let WholeEthosAttributes = *variant.attributes();
        let payload = match variant.payload() {
            WholeEthosVariantPayload::Unit => WholeLogosVariantPayload::Unit,
            WholeEthosVariantPayload::Tuple(fields) => {
                let result = WholeLogosTupleFields::new(
                    fields.fields().iter().map(Self::lower_reference).collect(),
                );
                let Ok(fields) = result else { unreachable!() };
                WholeLogosVariantPayload::Tuple(fields)
            }
        };
        WholeLogosVariant::new(variant.name().clone(), payload)
    }

    fn lower_reference(reference: &WholeEthosTypeReference) -> WholeLogosTypeReference {
        match reference {
            WholeEthosTypeReference::Identity(encoded_id) => {
                WholeLogosTypeReference::Identity(encoded_id.clone())
            }
            WholeEthosTypeReference::Application(application) => {
                WholeLogosTypeReference::Application(Self::lower_application(application))
            }
        }
    }

    fn lower_application(application: &WholeEthosTypeApplication) -> WholeLogosTypeApplication {
        WholeLogosTypeApplication::new(
            application.head().clone(),
            Self::lower_reference(application.payload()),
        )
    }

    const fn lower_visibility(visibility: WholeEthosVisibility) -> WholeLogosVisibility {
        match visibility {
            WholeEthosVisibility::Public => WholeLogosVisibility::Public,
            WholeEthosVisibility::Private => WholeLogosVisibility::Private,
        }
    }
}

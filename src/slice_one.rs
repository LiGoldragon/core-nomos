//! The direct typed transformation used by the first vertical slice.
//!
//! This path consumes and produces positional carriers. It preserves complete
//! encoded-ID chains and has no access to any legacy or identity-allocation
//! facility.

use slice_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosItem, WholeEthosNewtype, WholeEthosVisibility,
};
use slice_core_logos::{WholeLogos, WholeLogosItem, WholeLogosNewtype, WholeLogosVisibility};

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
        }
    }

    fn lower_newtype(newtype: &WholeEthosNewtype) -> WholeLogosNewtype {
        let WholeEthosAttributes = *newtype.attributes();

        WholeLogosNewtype::new(
            Self::lower_visibility(*newtype.visibility()),
            newtype.name().clone(),
            Self::lower_visibility(*newtype.wrapped_field().visibility()),
            newtype.wrapped_field().reference().clone(),
        )
    }

    const fn lower_visibility(visibility: WholeEthosVisibility) -> WholeLogosVisibility {
        match visibility {
            WholeEthosVisibility::Public => WholeLogosVisibility::Public,
            WholeEthosVisibility::Private => WholeLogosVisibility::Private,
        }
    }
}

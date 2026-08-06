//! Direct lowering from an authority-sealed bootstrap transaction.
//!
//! The boundary deliberately accepts no draft, decoded document, naming table,
//! or textual projection. A transaction can reach it only after its naming
//! authority has sealed the complete bootstrap meaning. The reader has already
//! placed every unordered declaration collection in canonical identity order;
//! lowering retains that order by walking each collection exactly once.

use core_ethos::bootstrap::{
    BootstrapBody, BootstrapNamingAuthority, BootstrapReadError, BootstrapReader, Declaration,
    InterfaceRole, NomosDeclaration, ParameterBinder, PreparedBootstrapTransaction, TypeBody,
    TypeDeclaration, TypeExpression, VariantBody,
};
use core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype, WholeLogosStruct,
    WholeLogosTupleFields, WholeLogosTypeApplication, WholeLogosTypeReference, WholeLogosVariant,
    WholeLogosVariantPayload, WholeLogosVisibility,
};
use signal_sema_translator::VocabularyEncodedId;

/// The direct authority-sealed bootstrap-to-Logos transformation.
///
/// This transformation is stateless because every identity and every ordering
/// decision belongs to the prepared transaction. In particular, it never
/// creates identities or reconstructs a transitional whole-Ethos value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BootstrapSliceOneLowering;

impl BootstrapSliceOneLowering {
    /// Construct the direct prepared-transaction lowering boundary.
    pub const fn new() -> Self {
        Self
    }

    /// Lower the supported Nexus or role-free Interface type algebra from one
    /// sealed transaction.
    ///
    /// The authority parameter remains part of the accepted type. Callers
    /// cannot substitute an unsealed draft or a decoded bootstrap document.
    pub fn lower<Authority: BootstrapNamingAuthority>(
        &self,
        reader: &BootstrapReader<Authority>,
        transaction: &PreparedBootstrapTransaction<Authority>,
    ) -> Result<WholeLogos, BootstrapSliceOneLoweringError> {
        reader.validate_transaction(transaction)?;
        let declarations = match &transaction.decoded().document.body {
            BootstrapBody::Interface(body) => {
                if let Some(membership) = body.memberships.first() {
                    return Err(BootstrapSliceOneLoweringError::InterfaceRole {
                        role: membership.role,
                        target: membership.target.clone(),
                    });
                }
                &body.types
            }
            BootstrapBody::Nexus(body) => {
                if let Some(declaration) = body.traits.first() {
                    return Err(BootstrapSliceOneLoweringError::Trait {
                        declaration: declaration.name.clone(),
                    });
                }
                &body.types
            }
            BootstrapBody::Sema(body) => {
                return Err(body.tables.first().map_or(
                    BootstrapSliceOneLoweringError::Sema,
                    |table| BootstrapSliceOneLoweringError::Table {
                        declaration: table.name.clone(),
                    },
                ));
            }
        };

        let items = declarations
            .iter()
            .map(|declaration| match declaration {
                Declaration::Type(declaration) => self.lower_type(declaration),
                Declaration::Nomos(NomosDeclaration::StreamInitiation(declaration)) => {
                    Err(BootstrapSliceOneLoweringError::Stream {
                        declaration: declaration.name.clone(),
                    })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WholeLogos::new(items))
    }

    fn lower_type(
        &self,
        declaration: &TypeDeclaration,
    ) -> Result<WholeLogosItem, BootstrapSliceOneLoweringError> {
        Ok(match &declaration.body {
            TypeBody::Newtype(wrapped) => WholeLogosItem::Newtype(WholeLogosNewtype::new(
                WholeLogosVisibility::Public,
                declaration.name.clone(),
                WholeLogosVisibility::Private,
                self.lower_expression(&declaration.name, wrapped)?,
            )),
            TypeBody::Struct(fields) => WholeLogosItem::Struct(WholeLogosStruct::new(
                WholeLogosVisibility::Public,
                declaration.name.clone(),
                fields
                    .iter()
                    .map(|field| self.lower_expression(&declaration.name, field))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            TypeBody::Enum(variants) => WholeLogosItem::Enumeration(WholeLogosEnumeration::new(
                WholeLogosVisibility::Public,
                declaration.name.clone(),
                variants
                    .iter()
                    .map(|variant| {
                        let payload = match &variant.body {
                            VariantBody::Unit => WholeLogosVariantPayload::Unit,
                            VariantBody::Unary(field) => WholeLogosVariantPayload::Tuple(
                                WholeLogosTupleFields::new(vec![
                                    self.lower_expression(&declaration.name, field)?,
                                ])
                                .map_err(|_| {
                                    BootstrapSliceOneLoweringError::EmptyVariantProduct {
                                        variant: variant.name.clone(),
                                    }
                                })?,
                            ),
                            VariantBody::Product(fields) => WholeLogosVariantPayload::Tuple(
                                WholeLogosTupleFields::new(
                                    fields
                                        .iter()
                                        .map(|field| {
                                            self.lower_expression(&declaration.name, field)
                                        })
                                        .collect::<Result<Vec<_>, _>>()?,
                                )
                                .map_err(|_| {
                                    BootstrapSliceOneLoweringError::EmptyVariantProduct {
                                        variant: variant.name.clone(),
                                    }
                                })?,
                            ),
                        };
                        Ok::<_, BootstrapSliceOneLoweringError>(WholeLogosVariant::new(
                            variant.name.clone(),
                            payload,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        })
    }

    fn lower_expression(
        &self,
        declaration: &VocabularyEncodedId,
        expression: &TypeExpression,
    ) -> Result<WholeLogosTypeReference, BootstrapSliceOneLoweringError> {
        Ok(match expression {
            TypeExpression::Reference(reference) => {
                WholeLogosTypeReference::Identity(reference.clone())
            }
            TypeExpression::ShapeApplication(application) => WholeLogosTypeReference::Application(
                WholeLogosTypeApplication::new(
                    application.shape.clone(),
                    application
                        .arguments
                        .iter()
                        .map(|argument| self.lower_expression(declaration, argument))
                        .collect::<Result<Vec<_>, _>>()?,
                )
                .map_err(|_| {
                    BootstrapSliceOneLoweringError::EmptyShapeApplication {
                        shape: application.shape.clone(),
                    }
                })?,
            ),
            TypeExpression::TraitRequirement(requirement) => {
                return Err(BootstrapSliceOneLoweringError::TraitRequirement {
                    declaration: declaration.clone(),
                    binder: requirement.binder().clone(),
                    required_traits: requirement.required_traits().to_vec(),
                });
            }
        })
    }
}

/// Exact typed refusal from direct prepared-bootstrap lowering.
#[derive(Debug, thiserror::Error)]
pub enum BootstrapSliceOneLoweringError {
    /// The matching reader rejected authority or a prepared-model invariant.
    #[error("prepared bootstrap transaction failed validation: {0}")]
    Validation(#[from] BootstrapReadError),
    /// An Interface-owned role relation cannot be erased into a plain type.
    #[error("direct Slice One lowering does not support Interface role {role:?} on {target:?}")]
    InterfaceRole {
        /// Exact role carried by the prepared transaction.
        role: InterfaceRole,
        /// Exact target of the refused role relationship.
        target: VocabularyEncodedId,
    },
    /// An authored Stream needs its complete lifecycle lowering.
    #[error("direct Slice One lowering does not support Stream declaration {declaration:?}")]
    Stream {
        /// Exact authored direct Stream output identity.
        declaration: VocabularyEncodedId,
    },
    /// A Nexus Trait cannot be silently omitted from a type-only projection.
    #[error("direct Slice One lowering does not support Trait declaration {declaration:?}")]
    Trait {
        /// Exact authored Trait identity.
        declaration: VocabularyEncodedId,
    },
    /// Sema is not part of the type-only Slice One projection.
    #[error("direct Slice One lowering does not accept a Sema document")]
    Sema,
    /// A persistent table needs a storage-aware Logos lowering.
    #[error("direct Slice One lowering does not support table declaration {declaration:?}")]
    Table {
        /// Exact authored table identity.
        declaration: VocabularyEncodedId,
    },
    /// A local parameter and all of its Trait constraints remain unsupported.
    #[error(
        "direct Slice One lowering does not support a Trait requirement in {declaration:?}: {binder:?} requires {required_traits:?}"
    )]
    TraitRequirement {
        /// Exact containing type declaration.
        declaration: VocabularyEncodedId,
        /// Exact inferred or named local binder.
        binder: ParameterBinder,
        /// Canonically ordered, nonempty required Trait identities.
        required_traits: Vec<VocabularyEncodedId>,
    },
    /// A prepared Shape application violated the Logos nonempty invariant.
    #[error("prepared Shape application {shape:?} has no arguments")]
    EmptyShapeApplication {
        /// Exact Shape identity.
        shape: VocabularyEncodedId,
    },
    /// A prepared product violated the Logos nonempty tuple invariant.
    #[error("prepared variant {variant:?} has an empty product payload")]
    EmptyVariantProduct {
        /// Exact variant identity.
        variant: VocabularyEncodedId,
    },
}

//! Direct lowering from an authority-sealed bootstrap transaction.
//!
//! The boundary deliberately accepts no draft, decoded document, naming table,
//! or textual projection. A transaction can reach it only after its naming
//! authority has sealed the complete bootstrap meaning. The reader has already
//! placed every unordered declaration collection in canonical identity order;
//! lowering retains that order by walking each collection exactly once.

use std::collections::{BTreeMap, BTreeSet};

use core_ethos::bootstrap::{
    BootstrapBody, BootstrapNamingAuthority, BootstrapReadError, BootstrapReader, Declaration,
    EthosKind, InterfaceRole, ParameterBinder, PreparedBootstrapTransaction,
    TypeBody, TypeDeclaration, TypeExpression, VariantBody,
};
use core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype, WholeLogosSemaTableKey,
    WholeLogosStorageFingerprint, WholeLogosStruct, WholeLogosTable, WholeLogosTupleFields,
    WholeLogosTypeApplication, WholeLogosTypeAttributes, WholeLogosTypeReference,
    WholeLogosVariant, WholeLogosVariantPayload, WholeLogosVisibility,
};
use name_table::EncodedName;

use crate::storage_shape::{storage_shape_hasher, update_count, update_identity};

/// Immutable producer evidence attached to an external storage contract.
///
/// The source and revision describe the authority that published the supplied
/// fingerprint. They do not participate in the fingerprint itself: repinning an
/// ABI-identical producer changes provenance, not stored bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageProvenanceOwner {
    source: String,
    revision: String,
}

// Trait exception — construction and read-only field access are this value's
// invariants, not a reusable operation family.
impl StorageProvenanceOwner {
    /// Bind one published producer source and immutable revision.
    pub fn new(source: String, revision: String) -> Result<Self, BootstrapSliceOneLoweringError> {
        if source.is_empty() {
            return Err(
                BootstrapSliceOneLoweringError::StorageProvenanceOwnerEmpty { field: "source" },
            );
        }
        if revision.is_empty() {
            return Err(
                BootstrapSliceOneLoweringError::StorageProvenanceOwnerEmpty { field: "revision" },
            );
        }
        Ok(Self { source, revision })
    }

    /// Published producer that owns this storage contract.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Immutable producer revision that supplied the contract.
    pub fn revision(&self) -> &str {
        &self.revision
    }
}

/// Explicit authority evidence for one nonlocal stored type.
///
/// Lowering never treats an encoded identity as archive compatibility. A Sema
/// reference outside its own sealed document is admitted only when the caller
/// supplies this identity, fingerprint, and producer owner together.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalStorageProvenance {
    identity: EncodedName,
    fingerprint: WholeLogosStorageFingerprint,
    owner: StorageProvenanceOwner,
}

// Trait exception — construction and read-only field access are this value's
// invariants, not a reusable operation family.
impl ExternalStorageProvenance {
    /// Bind one external type to its exact storage evidence.
    pub fn new(
        identity: EncodedName,
        fingerprint: [u8; 32],
        owner: StorageProvenanceOwner,
    ) -> Self {
        Self {
            identity,
            fingerprint: WholeLogosStorageFingerprint::new(fingerprint),
            owner,
        }
    }

    /// Imported identity covered by this evidence.
    pub const fn identity(&self) -> &EncodedName {
        &self.identity
    }

    /// Exact archived storage-shape evidence supplied by its owner.
    pub const fn fingerprint(&self) -> WholeLogosStorageFingerprint {
        self.fingerprint
    }

    /// Published producer and immutable revision for the evidence.
    pub const fn owner(&self) -> &StorageProvenanceOwner {
        &self.owner
    }
}

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
                        target: membership.target,
                    });
                }
                &body.types
            }
            BootstrapBody::Nexus(body) => {
                if let Some(declaration) = body.traits.first() {
                    return Err(BootstrapSliceOneLoweringError::Trait {
                        declaration: declaration.name,
                    });
                }
                &body.types
            }
            BootstrapBody::Sema(body) => {
                return Err(body.tables.first().map_or(
                    BootstrapSliceOneLoweringError::Sema,
                    |table| BootstrapSliceOneLoweringError::Table {
                        declaration: table.name,
                    },
                ));
            }
        };

        let items = declarations
            .iter()
            .map(|declaration| {
                let Declaration::Type(declaration) = declaration;
                self.lower_type(declaration)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WholeLogos::new(items))
    }

    /// Lower one authority-sealed Sema document into stored record declarations
    /// followed by its table specifications.
    ///
    /// Table records and keys must be declared by this exact Sema document, and
    /// keys must be newtypes so their generated key projection is structural.
    /// Referenced types outside the document require explicit published storage
    /// provenance; an encoded identity or Rust spelling is never ABI evidence.
    pub fn lower_sema<Authority: BootstrapNamingAuthority>(
        &self,
        reader: &BootstrapReader<Authority>,
        transaction: &PreparedBootstrapTransaction<Authority>,
        external_storage: &[ExternalStorageProvenance],
    ) -> Result<WholeLogos, BootstrapSliceOneLoweringError> {
        reader.validate_transaction(transaction)?;
        let BootstrapBody::Sema(body) = &transaction.decoded().document.body else {
            return Err(BootstrapSliceOneLoweringError::ExpectedSema {
                found: transaction.decoded().document.header.kind,
            });
        };

        let mut items = body
            .record_types
            .iter()
            .map(|declaration| self.lower_stored_type(declaration))
            .collect::<Result<Vec<_>, _>>()?;
        let inventory = BootstrapStorageInventory::new(&items, external_storage)?;
        let tables = body
            .tables
            .iter()
            .map(|table| {
                inventory.declaration(&table.record_type).ok_or(
                    BootstrapSliceOneLoweringError::SemaTableRecordNotDeclared {
                        table: table.name,
                        record: table.record_type,
                    },
                )?;
                let key = inventory.declaration(&table.key_type).ok_or(
                    BootstrapSliceOneLoweringError::SemaTableKeyNotDeclared {
                        table: table.name,
                        key: table.key_type,
                    },
                )?;
                if !matches!(key, WholeLogosItem::Newtype(_)) {
                    return Err(BootstrapSliceOneLoweringError::SemaTableKeyNotNewtype {
                        table: table.name,
                        key: table.key_type,
                    });
                }

                let record_storage = inventory.storage_fingerprint(&table.record_type)?;
                let key_storage = inventory.storage_fingerprint(&table.key_type)?;
                Ok(WholeLogosItem::Table(WholeLogosTable::new(
                    table.name,
                    WholeLogosTypeReference::Identity(table.record_type),
                    WholeLogosSemaTableKey::new(table.key_type),
                    record_storage,
                    key_storage,
                )))
            })
            .collect::<Result<Vec<_>, BootstrapSliceOneLoweringError>>()?;
        items.extend(tables);
        Ok(WholeLogos::new(items))
    }

    fn lower_stored_type(
        &self,
        declaration: &TypeDeclaration,
    ) -> Result<WholeLogosItem, BootstrapSliceOneLoweringError> {
        Ok(match self.lower_type(declaration)? {
            WholeLogosItem::Newtype(item) => {
                WholeLogosItem::Newtype(item.with_attributes(WholeLogosTypeAttributes::Stored))
            }
            WholeLogosItem::Struct(item) => {
                WholeLogosItem::Struct(item.with_attributes(WholeLogosTypeAttributes::Stored))
            }
            WholeLogosItem::Enumeration(item) => {
                WholeLogosItem::Enumeration(item.with_attributes(WholeLogosTypeAttributes::Stored))
            }
            _ => unreachable!("bootstrap type lowering emits only nominal types"),
        })
    }

    fn lower_type(
        &self,
        declaration: &TypeDeclaration,
    ) -> Result<WholeLogosItem, BootstrapSliceOneLoweringError> {
        Ok(match &declaration.body {
            TypeBody::Newtype(wrapped) => WholeLogosItem::Newtype(WholeLogosNewtype::new(
                WholeLogosVisibility::Public,
                declaration.name,
                WholeLogosVisibility::Private,
                self.lower_expression(&declaration.name, wrapped)?,
            )),
            TypeBody::Struct(fields) => WholeLogosItem::Struct(WholeLogosStruct::new(
                WholeLogosVisibility::Public,
                declaration.name,
                fields
                    .iter()
                    .map(|field| self.lower_expression(&declaration.name, field))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            TypeBody::Enum(variants) => WholeLogosItem::Enumeration(WholeLogosEnumeration::new(
                WholeLogosVisibility::Public,
                declaration.name,
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
                                        variant: variant.name,
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
                                        variant: variant.name,
                                    }
                                })?,
                            ),
                        };
                        Ok::<_, BootstrapSliceOneLoweringError>(WholeLogosVariant::new(
                            variant.name,
                            payload,
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        })
    }

    fn lower_expression(
        &self,
        declaration: &EncodedName,
        expression: &TypeExpression,
    ) -> Result<WholeLogosTypeReference, BootstrapSliceOneLoweringError> {
        Ok(match expression {
            TypeExpression::Reference(reference) => WholeLogosTypeReference::Identity(*reference),
            TypeExpression::ShapeApplication(application) => WholeLogosTypeReference::Application(
                WholeLogosTypeApplication::new(
                    application.shape,
                    application
                        .arguments
                        .iter()
                        .map(|argument| self.lower_expression(declaration, argument))
                        .collect::<Result<Vec<_>, _>>()?,
                )
                .map_err(|_| {
                    BootstrapSliceOneLoweringError::EmptyShapeApplication {
                        shape: application.shape,
                    }
                })?,
            ),
            TypeExpression::TraitRequirement(requirement) => {
                return Err(BootstrapSliceOneLoweringError::TraitRequirement {
                    declaration: *declaration,
                    binder: requirement.binder().clone(),
                    required_traits: requirement.required_traits().to_vec(),
                });
            }
        })
    }
}

struct BootstrapStorageInventory<'a> {
    declarations: BTreeMap<EncodedName, &'a WholeLogosItem>,
    external: BTreeMap<EncodedName, &'a ExternalStorageProvenance>,
}

impl<'a> BootstrapStorageInventory<'a> {
    fn new(
        declarations: &'a [WholeLogosItem],
        external: &'a [ExternalStorageProvenance],
    ) -> Result<Self, BootstrapSliceOneLoweringError> {
        let mut by_identity = BTreeMap::new();
        for declaration in declarations {
            let identity = storage_declaration_identity(declaration)
                .expect("stored bootstrap lowering emits only nominal declarations");
            if by_identity.insert(*identity, declaration).is_some() {
                return Err(BootstrapSliceOneLoweringError::DuplicateSemaRecord {
                    identity: *identity,
                });
            }
        }

        let mut external_by_identity = BTreeMap::new();
        for provenance in external {
            if by_identity.contains_key(provenance.identity()) {
                return Err(
                    BootstrapSliceOneLoweringError::StorageProvenanceOwnershipConflict {
                        identity: *provenance.identity(),
                    },
                );
            }
            if external_by_identity
                .insert(*provenance.identity(), provenance)
                .is_some()
            {
                return Err(
                    BootstrapSliceOneLoweringError::DuplicateExternalStorageProvenance {
                        identity: *provenance.identity(),
                    },
                );
            }
        }
        Ok(Self {
            declarations: by_identity,
            external: external_by_identity,
        })
    }

    fn declaration(&self, identity: &EncodedName) -> Option<&WholeLogosItem> {
        self.declarations.get(identity).copied()
    }

    fn storage_fingerprint(
        &self,
        identity: &EncodedName,
    ) -> Result<WholeLogosStorageFingerprint, BootstrapSliceOneLoweringError> {
        self.storage_fingerprint_identity(identity, &mut BTreeSet::new())
    }

    fn storage_fingerprint_identity(
        &self,
        identity: &EncodedName,
        visiting: &mut BTreeSet<EncodedName>,
    ) -> Result<WholeLogosStorageFingerprint, BootstrapSliceOneLoweringError> {
        let Some(declaration) = self.declarations.get(identity).copied() else {
            return self
                .external
                .get(identity)
                .map(|provenance| provenance.fingerprint())
                .ok_or(
                    BootstrapSliceOneLoweringError::MissingExternalStorageProvenance {
                        identity: *identity,
                    },
                );
        };
        if !visiting.insert(*identity) {
            return Err(BootstrapSliceOneLoweringError::CyclicSemaStorageShape {
                identity: *identity,
            });
        }
        let fingerprint = match declaration {
            WholeLogosItem::Newtype(newtype) => {
                let wrapped = self.storage_fingerprint_reference(newtype.wrapped(), visiting)?;
                let mut hasher = storage_shape_hasher(b"newtype");
                update_identity(&mut hasher, identity);
                hasher.update_length_prefixed(&wrapped.bytes());
                WholeLogosStorageFingerprint::new(hasher.finalize_bytes())
            }
            WholeLogosItem::Struct(structure) => {
                let mut hasher = storage_shape_hasher(b"struct");
                update_identity(&mut hasher, identity);
                update_count(&mut hasher, structure.fields().len());
                for field in structure.fields() {
                    let field = self.storage_fingerprint_reference(field, visiting)?;
                    hasher.update_length_prefixed(&field.bytes());
                }
                WholeLogosStorageFingerprint::new(hasher.finalize_bytes())
            }
            WholeLogosItem::Enumeration(enumeration) => {
                let mut hasher = storage_shape_hasher(b"enumeration");
                update_identity(&mut hasher, identity);
                update_count(&mut hasher, enumeration.variants().len());
                for variant in enumeration.variants() {
                    update_identity(&mut hasher, variant.name());
                    match variant.payload() {
                        WholeLogosVariantPayload::Unit => {
                            hasher.update_length_prefixed(b"unit");
                        }
                        WholeLogosVariantPayload::Tuple(fields) => {
                            hasher.update_length_prefixed(b"tuple");
                            update_count(&mut hasher, fields.fields().len());
                            for field in fields.fields() {
                                let field = self.storage_fingerprint_reference(field, visiting)?;
                                hasher.update_length_prefixed(&field.bytes());
                            }
                        }
                    }
                }
                WholeLogosStorageFingerprint::new(hasher.finalize_bytes())
            }
            _ => {
                return Err(
                    BootstrapSliceOneLoweringError::InvalidSemaRecordDeclaration {
                        identity: *identity,
                    },
                );
            }
        };
        visiting.remove(identity);
        Ok(fingerprint)
    }

    fn storage_fingerprint_reference(
        &self,
        reference: &WholeLogosTypeReference,
        visiting: &mut BTreeSet<EncodedName>,
    ) -> Result<WholeLogosStorageFingerprint, BootstrapSliceOneLoweringError> {
        match reference {
            WholeLogosTypeReference::Identity(identity) => {
                self.storage_fingerprint_identity(identity, visiting)
            }
            WholeLogosTypeReference::Application(application) => {
                let head = self.storage_fingerprint_identity(application.head(), visiting)?;
                let mut hasher = storage_shape_hasher(b"application");
                update_identity(&mut hasher, application.head());
                hasher.update_length_prefixed(&head.bytes());
                update_count(&mut hasher, application.arguments().len());
                for argument in application.arguments() {
                    let argument = self.storage_fingerprint_reference(argument, visiting)?;
                    hasher.update_length_prefixed(&argument.bytes());
                }
                Ok(WholeLogosStorageFingerprint::new(hasher.finalize_bytes()))
            }
            WholeLogosTypeReference::Parameter(parameter) => {
                Err(BootstrapSliceOneLoweringError::StorageTypeParameter {
                    parameter: *parameter,
                })
            }
        }
    }
}

fn storage_declaration_identity(item: &WholeLogosItem) -> Option<&EncodedName> {
    match item {
        WholeLogosItem::Newtype(item) => Some(item.name()),
        WholeLogosItem::Struct(item) => Some(item.name()),
        WholeLogosItem::Enumeration(item) => Some(item.name()),
        _ => None,
    }
}

/// Exact typed refusal from direct prepared-bootstrap lowering.
#[derive(Debug, thiserror::Error)]
pub enum BootstrapSliceOneLoweringError {
    /// The matching reader rejected authority or a prepared-model invariant.
    #[error("prepared bootstrap transaction failed validation: {0}")]
    Validation(#[from] BootstrapReadError),
    /// A producer owner omitted one required provenance component.
    #[error("external storage provenance owner has empty {field}")]
    StorageProvenanceOwnerEmpty { field: &'static str },
    /// An Interface-owned role relation cannot be erased into a plain type.
    #[error("direct Slice One lowering does not support Interface role {role:?} on {target:?}")]
    InterfaceRole {
        /// Exact role carried by the prepared transaction.
        role: InterfaceRole,
        /// Exact target of the refused role relationship.
        target: EncodedName,
    },
    /// A Nexus Trait cannot be silently omitted from a type-only projection.
    #[error("direct Slice One lowering does not support Trait declaration {declaration:?}")]
    Trait {
        /// Exact authored Trait identity.
        declaration: EncodedName,
    },
    /// Sema is not part of the type-only Slice One projection.
    #[error("direct Slice One lowering does not accept a Sema document")]
    Sema,
    /// A persistent table needs a storage-aware Logos lowering.
    #[error("direct Slice One lowering does not support table declaration {declaration:?}")]
    Table {
        /// Exact authored table identity.
        declaration: EncodedName,
    },
    /// The storage-aware entry point received another bootstrap file kind.
    #[error("strict Sema lowering received {found:?} source")]
    ExpectedSema { found: EthosKind },
    /// Two local record declarations carried the same encoded identity.
    #[error("strict Sema lowering received duplicate record identity {identity:?}")]
    DuplicateSemaRecord { identity: EncodedName },
    /// A table's record type was not declared in this Sema document.
    #[error("Sema table {table:?} record {record:?} is not declared by this Sema document")]
    SemaTableRecordNotDeclared {
        table: EncodedName,
        record: EncodedName,
    },
    /// A table's key type was not declared in this Sema document.
    #[error("Sema table {table:?} key {key:?} is not declared by this Sema document")]
    SemaTableKeyNotDeclared {
        table: EncodedName,
        key: EncodedName,
    },
    /// A locally declared table key was not a newtype with structural key projection.
    #[error("Sema table {table:?} key {key:?} is not a newtype")]
    SemaTableKeyNotNewtype {
        table: EncodedName,
        key: EncodedName,
    },
    /// A local declaration could not be represented as stored data.
    #[error("Sema record declaration {identity:?} is not a supported stored nominal type")]
    InvalidSemaRecordDeclaration { identity: EncodedName },
    /// External provenance repeated one encoded identity.
    #[error("external storage provenance repeats identity {identity:?}")]
    DuplicateExternalStorageProvenance { identity: EncodedName },
    /// An identity was claimed by both the local Sema document and external provenance.
    #[error("storage provenance ownership conflicts for local identity {identity:?}")]
    StorageProvenanceOwnershipConflict { identity: EncodedName },
    /// A reachable nonlocal storage type had no explicit owner fingerprint.
    #[error("storage type {identity:?} has no explicit external provenance")]
    MissingExternalStorageProvenance { identity: EncodedName },
    /// Recursive storage shapes have no finite archive fingerprint in this stage.
    #[error("Sema storage shape is cyclic through {identity:?}")]
    CyclicSemaStorageShape { identity: EncodedName },
    /// Stored record fields cannot retain unresolved local type parameters.
    #[error("Sema storage shape contains unresolved parameter {parameter:?}")]
    StorageTypeParameter { parameter: EncodedName },
    /// A local parameter and all of its Trait constraints remain unsupported.
    #[error(
        "direct Slice One lowering does not support a Trait requirement in {declaration:?}: {binder:?} requires {required_traits:?}"
    )]
    TraitRequirement {
        /// Exact containing type declaration.
        declaration: EncodedName,
        /// Exact inferred or named local binder.
        binder: ParameterBinder,
        /// Canonically ordered, nonempty required Trait identities.
        required_traits: Vec<EncodedName>,
    },
    /// A prepared Shape application violated the Logos nonempty invariant.
    #[error("prepared Shape application {shape:?} has no arguments")]
    EmptyShapeApplication {
        /// Exact Shape identity.
        shape: EncodedName,
    },
    /// A prepared product violated the Logos nonempty tuple invariant.
    #[error("prepared variant {variant:?} has an empty product payload")]
    EmptyVariantProduct {
        /// Exact variant identity.
        variant: EncodedName,
    },
}

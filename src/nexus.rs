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
use nexus_core_ethos::{
    WholeEthos, WholeEthosAttributes, WholeEthosBody, WholeEthosEnumeration, WholeEthosFileKind,
    WholeEthosItem, WholeEthosNewtype, WholeEthosStreamInitiation,
    WholeEthosStruct, WholeEthosTable, WholeEthosTrait, WholeEthosTypeApplication,
    WholeEthosTypeParameter, WholeEthosTypeReference, WholeEthosVariant, WholeEthosVariantPayload,
    WholeEthosVisibility,
};
use nexus_core_logos::{
    WholeLogos, WholeLogosEnumeration, WholeLogosItem, WholeLogosNewtype,
    WholeLogosStorageFingerprint, WholeLogosStreamHandle, WholeLogosStreamInitiation,
    WholeLogosStreamLifecycle, WholeLogosStreamTermination, WholeLogosStruct, WholeLogosTable,
    WholeLogosTraitDef, WholeLogosTraitImpl, WholeLogosTupleFields,
    WholeLogosTypeApplication, WholeLogosTypeAttributes, WholeLogosTypeParameter,
    WholeLogosTypeReference, WholeLogosVariant, WholeLogosVariantPayload, WholeLogosVisibility,
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
    /// Lower stored record declarations and every table whose record type is
    /// declared by this document. Valid imported-record tables remain explicit
    /// typed deferrals; malformed record/key shapes refuse the whole projection.
    fn lower_sema(
        &self,
        ethos: &WholeEthos,
    ) -> Result<SemaTransformationOutcome, NexusTransformationError>;
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
    deferred_tables: Vec<WholeEthosTable>,
}

// Trait exception — too trivial: read-only outcome ergonomics.
impl SemaTransformationOutcome {
    /// Stored record declarations followed by structurally supported tables.
    pub const fn logos(&self) -> &WholeLogos {
        &self.logos
    }

    /// Valid imported-record tables retained for a later producer.
    pub fn deferred_tables(&self) -> &[WholeEthosTable] {
        &self.deferred_tables
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
    storage_fingerprints: Vec<SemaStorageTypeFingerprintMapping>,
    stream_lifecycle_identities: Vec<StreamLifecycleIdentities>,
}

// Trait exception — too trivial: constructor and read-only data ergonomics for
// the NexusStructuralTransformation implementation.
impl NexusTransformation {
    /// Construct an identity-reference Nexus transformation.
    pub const fn new() -> Self {
        Self {
            reference_mappings: Vec::new(),
            storage_fingerprints: Vec::new(),
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
            storage_fingerprints: Vec::new(),
            stream_lifecycle_identities: Vec::new(),
        })
    }

    /// Canonically ordered exact reference mappings.
    pub fn reference_mappings(&self) -> &[NexusVocabularyReferenceMapping] {
        &self.reference_mappings
    }

    /// Attach exact storage contracts for non-local types used by Sema table
    /// record graphs or keys.
    pub fn with_storage_fingerprints(
        mut self,
        mut storage_fingerprints: Vec<SemaStorageTypeFingerprintMapping>,
    ) -> Result<Self, NexusTransformationError> {
        storage_fingerprints.sort_by(|left, right| left.source().cmp(right.source()));
        for adjacent in storage_fingerprints.windows(2) {
            if adjacent[0].source() == adjacent[1].source() {
                return Err(
                    NexusTransformationError::DuplicateSemaStorageFingerprintSource {
                        identity: adjacent[0].source().clone(),
                    },
                );
            }
        }
        self.storage_fingerprints = storage_fingerprints;
        Ok(self)
    }

    /// Canonically ordered external storage contracts.
    pub fn storage_fingerprints(&self) -> &[SemaStorageTypeFingerprintMapping] {
        &self.storage_fingerprints
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
                .collect(),
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

    fn lower_type_parameter(parameter: &WholeEthosTypeParameter) -> WholeLogosTypeParameter {
        WholeLogosTypeParameter::new(parameter.name().clone(), parameter.quality().clone())
    }

    fn lower_application(
        &self,
        application: &WholeEthosTypeApplication,
    ) -> Result<WholeLogosTypeApplication, NexusTransformationError> {
        WholeLogosTypeApplication::new(
            self.map_reference(application.head()),
            application
                .arguments()
                .iter()
                .map(|argument| self.lower_reference(argument))
                .collect::<Result<Vec<_>, _>>()?,
        )
        .map_err(
            |_| NexusTransformationError::EmptyTypeApplicationArguments {
                head: application.head().clone(),
            },
        )
    }

    fn map_reference(&self, source: &VocabularyEncodedId) -> VocabularyEncodedId {
        self.reference_mappings
            .binary_search_by(|mapping| mapping.source().cmp(source))
            .map(|index| self.reference_mappings[index].target().clone())
            .unwrap_or_else(|_| source.clone())
    }

    fn storage_fingerprint(
        &self,
        reference: &WholeEthosTypeReference,
        declarations: &BTreeMap<VocabularyEncodedId, &WholeEthosItem>,
        visiting: &mut BTreeSet<VocabularyEncodedId>,
    ) -> Result<WholeLogosStorageFingerprint, NexusTransformationError> {
        match reference {
            WholeEthosTypeReference::Identity(identity) => {
                if let Some(declaration) = declarations.get(identity) {
                    self.local_storage_fingerprint(identity, declaration, declarations, visiting)
                } else {
                    self.external_storage_fingerprint(identity)
                }
            }
            WholeEthosTypeReference::Parameter(parameter) => {
                Err(NexusTransformationError::UnresolvedTypeParameter {
                    name: parameter.name().clone(),
                })
            }
            WholeEthosTypeReference::Application(application) => {
                let head = self.external_storage_fingerprint(application.head())?;
                let mut hasher = storage_shape_hasher(b"application");
                update_identity(&mut hasher, application.head());
                update_identity(&mut hasher, &self.map_reference(application.head()));
                hasher.update_length_prefixed(&head.bytes());
                update_count(&mut hasher, application.arguments().len());
                for argument in application.arguments() {
                    let argument = self.storage_fingerprint(argument, declarations, visiting)?;
                    hasher.update_length_prefixed(&argument.bytes());
                }
                Ok(WholeLogosStorageFingerprint::new(hasher.finalize_bytes()))
            }
        }
    }

    fn local_storage_fingerprint(
        &self,
        identity: &VocabularyEncodedId,
        declaration: &WholeEthosItem,
        declarations: &BTreeMap<VocabularyEncodedId, &WholeEthosItem>,
        visiting: &mut BTreeSet<VocabularyEncodedId>,
    ) -> Result<WholeLogosStorageFingerprint, NexusTransformationError> {
        if !visiting.insert(identity.clone()) {
            return Err(NexusTransformationError::CyclicSemaStorageShape {
                identity: identity.clone(),
            });
        }
        let result = match declaration {
            WholeEthosItem::Newtype(newtype) => {
                let wrapped = self.storage_fingerprint(
                    newtype.wrapped_field().reference(),
                    declarations,
                    visiting,
                )?;
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
                    let field = self.storage_fingerprint(field, declarations, visiting)?;
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
                                let field =
                                    self.storage_fingerprint(field, declarations, visiting)?;
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

    fn external_storage_fingerprint(
        &self,
        source: &VocabularyEncodedId,
    ) -> Result<WholeLogosStorageFingerprint, NexusTransformationError> {
        let index = self
            .storage_fingerprints
            .binary_search_by(|mapping| mapping.source().cmp(source))
            .map_err(
                |_| NexusTransformationError::MissingSemaStorageFingerprint {
                    identity: source.clone(),
                },
            )?;
        let mapping = &self.storage_fingerprints[index];
        let mut hasher = storage_shape_hasher(b"external");
        update_identity(&mut hasher, source);
        update_identity(&mut hasher, &self.map_reference(source));
        hasher.update_length_prefixed(&mapping.fingerprint().bytes());
        Ok(WholeLogosStorageFingerprint::new(hasher.finalize_bytes()))
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
    ) -> Result<SemaTransformationOutcome, NexusTransformationError> {
        let WholeEthosBody::Sema(body) = ethos.body() else {
            return Err(NexusTransformationError::UnsupportedFileKind {
                expected: WholeEthosFileKind::Sema,
                found: ethos.header().kind(),
            });
        };

        let mut declared_records = Vec::with_capacity(body.record_types().len());
        let mut record_declarations = BTreeMap::new();
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
            record_declarations.insert(name.clone(), item);
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
        let mut deferred_tables = Vec::new();
        for table in body.tables() {
            let WholeEthosTypeReference::Identity(record) = table.record() else {
                return Err(NexusTransformationError::InvalidSemaTableRecordShape {
                    table: table.name().clone(),
                });
            };
            let WholeEthosTypeReference::Identity(key) = table.key() else {
                return Err(NexusTransformationError::InvalidSemaTableKeyShape {
                    table: table.name().clone(),
                });
            };
            if declared_records.binary_search(record).is_err() {
                deferred_tables.push(table.clone());
                continue;
            }
            let record_storage = self.storage_fingerprint(
                table.record(),
                &record_declarations,
                &mut BTreeSet::new(),
            )?;
            let key_storage =
                self.storage_fingerprint(table.key(), &record_declarations, &mut BTreeSet::new())?;
            items.push(WholeLogosItem::Table(WholeLogosTable::new(
                table.name().clone(),
                WholeLogosTypeReference::Identity(self.map_reference(record)),
                WholeLogosTypeReference::Identity(self.map_reference(key)),
                record_storage,
                key_storage,
            )));
        }

        Ok(SemaTransformationOutcome {
            logos: WholeLogos::new(items),
            deferred_tables,
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

/// One caller-supplied content/ABI fingerprint for a non-local storage type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemaStorageTypeFingerprintMapping {
    source: VocabularyEncodedId,
    fingerprint: WholeLogosStorageFingerprint,
}

impl SemaStorageTypeFingerprintMapping {
    /// Bind one Universal type identity to its authoritative external storage
    /// contract without allocating or deriving another identity.
    pub fn new(
        source: VocabularyEncodedId,
        fingerprint: [u8; 32],
    ) -> Result<Self, NexusTransformationError> {
        if source.root_variant() != &VocabularyRoot::Universal {
            return Err(NexusTransformationError::SemaStorageFingerprintSourceRoot {
                found: *source.root_variant(),
            });
        }
        Ok(Self {
            source,
            fingerprint: WholeLogosStorageFingerprint::new(fingerprint),
        })
    }

    /// External Universal type identity.
    pub const fn source(&self) -> &VocabularyEncodedId {
        &self.source
    }

    /// Assembly-supplied storage contract.
    pub const fn fingerprint(&self) -> WholeLogosStorageFingerprint {
        self.fingerprint
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
    /// Sema storage layouts cannot be derived from an item-local generic pickup.
    #[error("Sema storage shape cannot resolve type parameter {name:?}")]
    UnresolvedTypeParameter {
        /// Parameter name absent from concrete Sema storage.
        name: VocabularyEncodedId,
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
    /// A storage contract was assigned to a non-Universal source type.
    #[error("Sema storage fingerprint source must be Universal, found {found:?}")]
    SemaStorageFingerprintSourceRoot { found: VocabularyRoot },
    /// One external storage type received multiple compatibility contracts.
    #[error("Sema storage fingerprint source {identity:?} is duplicated")]
    DuplicateSemaStorageFingerprintSource { identity: VocabularyEncodedId },
    /// A reachable non-local storage type has no authoritative compatibility
    /// contract supplied by the owning assembly.
    #[error("Sema storage type {identity:?} has no caller-supplied fingerprint")]
    MissingSemaStorageFingerprint { identity: VocabularyEncodedId },
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
    /// A Sema table attempted to store an applied type instead of one record.
    #[error("Sema table {table:?} has an unsupported record type application")]
    InvalidSemaTableRecordShape { table: VocabularyEncodedId },
    /// A Sema table attempted to use an applied key type outside the current
    /// one-identity key contract.
    #[error("Sema table {table:?} has an unsupported key type application")]
    InvalidSemaTableKeyShape { table: VocabularyEncodedId },
}

//! # core-nomos
//!
//! The stringless encoded form of Nomos, the transformation language. A
//! transformer is typed data — never text, never Rust — that lowers the Ethos
//! encoded form into the Logos encoded form. Generated programs, not
//! rendered-source equality, are the acceptance surface.
//!
//! ## EncodedNomos and the plain TextualNomos base door
//!
//! EncodedNomos includes transformers as typed data and retains the legacy
//! execution fixture for equivalence work. The phase-stable
//! [`AuthoredTransformerDeclaration`] is the stringless target of the approved
//! [`TextualNomos`] base door: it retains complete encoded-ID chains and durable
//! invocation targets before atomic sealing. [`TextualNomos`] uses the
//! plain Standard raw-discovery profile and the shared structural evaluator.
//! Its Logos-shaped result grammar and landing carrier are computed together
//! from the Logos declarations. The isolated ScopeOf bootstrap uses the
//! separately sanctioned handwritten mirror until Nomos can emit both sides of
//! the pair itself.
//!
//! ## Transformer kinds
//!
//! - **Named** ([`MacroKind::Named`]) — dispatched by minted [`MacroIdentity`]; an
//!   unknown named invocation is an error. `WireAttributes` is named.
//! - **Structural** ([`MacroKind::Structural`]) — a per-section default selected by
//!   an Ethos declaration's kind; `WireNewtype` and the particular-struct transformer are
//!   structural.
//! - **Recursive** ([`MacroKind::Recursive`]) — entered from one structural
//!   section and evaluates a finite, singly owned source-enumeration tree.
//!
//! The retained legacy `MacroPackage` fixture authors only Named and Structural
//! declarations.
//!
//! ## Production seal
//!
//! [`SealedNomosCapsule`] hashes exactly canonical [`AuthoredTransformerSet`]
//! bytes. [`AuthenticatedNameTreeProjection`] stores only the versioned,
//! integrity-authenticated reachable spelling closure bound to that immutable
//! identity. Rename advances the projection while preserving Capsule bytes and
//! identity. Live-slot seating and persistence are later daemon work.
//!
//! [`MacroPackage`], [`MacroIdentity`], and [`PackageRevision`] are retained
//! only as the legacy execution fixture for equivalence work.
//!
//! ## The authored future algebra
//!
//! A production result template is Logos encoded-form data whose authored
//! operations are [`TemplateFuture`] **Realize** / **Invoke** / **Splice** /
//! **InsertAt**. Recursive spelling remains `Invoke`; compilation replaces its
//! lexical self-call with an internal judgment that has no authored spelling.
//! The legacy [`Escape`] algebra remains the three-operation `MacroPackage`
//! evidence surface.
//!
//! ## The engine
//!
//! [`MacroPackage::apply`] takes the ethos encoded form and its NameTable and
//! returns [`Lowering`]: logos encoded items and a Logos NameTable composed with the
//! Ethos compatibility slice. Identifiers retain their namespace tag, and conversions are typed
//! end to end, outside text.

pub mod authored;
pub mod bootstrap;
pub mod capsule;
pub mod definition;
pub mod domain;
pub mod engine;
pub mod error;
pub mod fixtures;
pub mod generation;
pub mod identity;
pub mod manifest;
pub mod meta;
mod name_boundary;
pub mod native;
pub mod nexus;
pub mod package;
pub mod prelude;
pub mod scope_of;
pub mod sealed;
pub mod slice_one;
mod storage_shape;
pub mod template;
pub mod template_language;
pub mod textual;

pub use authored::{
    AuthoredBindingIdentity, AuthoredIdentityPosition, AuthoredInputParameter,
    AuthoredInputSignature, AuthoredNomosError, AuthoredTransformerDeclaration,
    AuthoredTransformerIdentity, AuthoredTransformerSet,
};
pub use bootstrap::{BootstrapSliceOneLowering, BootstrapSliceOneLoweringError};
pub use capsule::capsule_from_issued_hash;
pub use definition::MacroDefinition;
pub use domain::EncodedNomosDomain;
pub use engine::Lowering;
pub use error::NomosError;
pub use identity::{MacroIdentity, MacroKind, SectionDefault};
pub use manifest::{
    NomosFileManifest, NomosManifestFile, NomosManifestLoadError, NomosSourcePath,
    PlannedNomosPopulation,
};
pub use meta::{BoundInput, InputParameter, InputSignature, MetaType, MetaValue};
pub use native::{
    NativeAuthoredEvaluator, NativeEvaluatedField, NativeEvaluatedLogos, NativeEvaluatedTerm,
    NativeEvaluatedValue, NativeEvaluationError, NativeIdentityPosition, NativeLogosNameTree,
    NativeLogosPopulation, NativeNameTreeBoundary, NativeNameTreePlan, NativeNameTreeRefusal,
    NativePopulationArchiveError, NativeReferenceMapping, NativeReferenceUniverse,
    NativeTransformation,
};
pub use nexus::{
    ArchiveAbiEquivalenceChecks, BundleStorageProvenance, ExternalStorageProvenance,
    ExternalStorageSuccessorEvidence, InterfaceRoleIdentities, InterfaceStructuralTransformation,
    InterfaceTransformationOutcome, InterfaceTypeStructuralTransformation,
    NexusStructuralTransformation, NexusTransformation, NexusTransformationError,
    NexusVocabularyReferenceMapping, NomosStorageProvenance, PreservedSemaFamilyProvenance,
    SemaStructuralTransformation, SemaTransformationOutcome, StorageProvenanceOwner,
    StreamLifecycleIdentities, TypeDeclarationStructuralTransformation,
};
pub use package::{MacroDefinitions, MacroPackage, PackageRevision};
pub use prelude::{GENERATED_MARKER, ModuleHead};
pub use scope_of::{
    ScopeOfConfiguredIdentityPosition, ScopeOfDeclaration, ScopeOfDeclarationContract,
    ScopeOfDeclarationRecognition, ScopeOfGate, ScopeOfGateObservations, ScopeOfLogosRealization,
    ScopeOfNomosEnumeration, ScopeOfNomosPlanning, ScopeOfNomosVariant, ScopeOfNomosVariantPayload,
    ScopeOfRefusal, ScopeOfSourceResolution, ScopeOfTransformer, ScopeOfVariantNamePromise,
};
pub use sealed::{
    AuthenticatedNameTreeProjection, NameTreeProjectionEntry, NameTreeProjectionVersion,
    NomosSealError, SealedNomosCapsule, SealedNomosPopulation,
};
pub use slice_one::{
    SliceOneTransformation, SliceOneTransformationError, SliceOneVocabularyReferenceMapping,
};
pub use template::{
    BindingRef, EnumerationTemplate, Escape, FieldNameRule, GenerationClass, ItemTemplate,
    NameTransform, NewtypeTemplate, Realize, ResultTemplate, Scalar, Sequence, SequenceItem,
    Splice, SpliceElement, StructTemplate,
};
pub use template_language::{
    InsertAt, LogosTemplateLanguage, TemplateConstructorDeclaration, TemplateFieldDeclaration,
    TemplateFieldValue, TemplateFormDeclaration, TemplateFuture, TemplateFutureKind,
    TemplateFutureOutput, TemplateFutureRequirement, TemplateLandingField, TemplateLandingShape,
    TemplateLanguage, TemplateLanguageError, TemplateRootOutput, TemplateRootOutputSelector,
    TemplateTerm, TemplateTypeDeclaration, TemplateValue, TemplateValueError,
};
pub use textual::{
    DecodedNomosDocument, LoadedNomosDocument, LoadedNomosPopulation, NomosLoadError,
    NomosModulePath, NomosNameTable, NomosNameTableConstructionError, PlannedNomosLoad,
    PlannedOccurrenceKind, TextualNomos, TextualNomosError, TextualNomosMetaType,
    TextualNomosTypeIds, TextualNomosWords,
};

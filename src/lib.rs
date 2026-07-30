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
//! from the Logos declarations; no transformer-specific or Logos-type-specific
//! authored twin exists.
//!
//! ## The two legacy execution kinds
//!
//! - **Named** ([`MacroKind::Named`]) — dispatched by minted [`MacroIdentity`]; an
//!   unknown named invocation is an error. `WireAttributes` is named.
//! - **Structural** ([`MacroKind::Structural`]) — a per-section default selected by
//!   an Ethos declaration's kind; `WireNewtype` and the particular-struct transformer are
//!   structural.
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
//! ## The closed escape algebra
//!
//! A result template is logos-encoded-form data whose non-literal positions are the
//! closed set [`Escape`] = **Realize** / **Invoke** / **Splice**. A [`NameTransform`]
//! is typed intent carried by `Realize`, not a fourth escape; `NameTableBoundary`
//! performs the derived-name work at the NameTable/emission boundary.
//!
//! ## The engine
//!
//! [`MacroPackage::apply`] takes the ethos encoded form and its NameTable and
//! returns [`Lowering`]: logos encoded items and a Logos NameTable composed with the
//! Ethos compatibility slice. Identifiers retain their namespace tag, and conversions are typed
//! end to end, outside text.

pub mod authored;
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
pub mod package;
pub mod prelude;
pub mod sealed;
pub mod slice_one;
pub mod template;
pub mod template_language;
pub mod textual;

pub use authored::{
    AuthoredBindingIdentity, AuthoredIdentityPosition, AuthoredInputParameter,
    AuthoredInputSignature, AuthoredNomosError, AuthoredTransformerDeclaration,
    AuthoredTransformerIdentity, AuthoredTransformerSet,
};
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
pub use package::{MacroDefinitions, MacroPackage, PackageRevision};
pub use prelude::{GENERATED_MARKER, ModuleHead};
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
    LogosTemplateLanguage, TemplateConstructorDeclaration, TemplateFieldDeclaration,
    TemplateFieldValue, TemplateFormDeclaration, TemplateFuture, TemplateFutureKind,
    TemplateFutureOutput, TemplateFutureRequirement, TemplateLandingField, TemplateLandingShape,
    TemplateLanguage, TemplateLanguageError, TemplateRootOutput, TemplateRootOutputSelector,
    TemplateTerm, TemplateTypeDeclaration, TemplateValue, TemplateValueError,
};
pub use textual::{
    DecodedNomosDocument, LoadedNomosDocument, LoadedNomosPopulation, NomosLoadError,
    NomosModulePath, NomosNameTable, PlannedNomosLoad, PlannedOccurrenceKind, TextualNomos,
    TextualNomosError, TextualNomosMetaType, TextualNomosTypeIds, TextualNomosWords,
};

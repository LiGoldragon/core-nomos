//! Phase-stable authored Nomos declarations.
//!
//! This is the stringless value produced after a TextualNomos boundary has
//! resolved declaration and reference spellings. Every identity is a complete
//! root-fronted encoded-ID chain. Package-local [`crate::MacroIdentity`] values
//! do not exist at this phase: a later atomic seal rebinds durable transformer
//! references into the sealed execution table.
//!
//! The result surface is a typed Logos skeleton with typed escape positions.
//! It is never a text or string template.

use std::collections::BTreeSet;

use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use thiserror::Error;

use crate::{FieldNameRule, MacroKind, MetaType, NameTransform};

/// A durable identity position whose production root is fixed by the authored
/// Nomos contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoredIdentityPosition {
    /// The declared or referenced transformer.
    Transformer,
    /// One input binding local to a transformer declaration.
    Binding,
}

/// A typed refusal while constructing the authored-stage value.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum AuthoredNomosError {
    /// Authored transformer and binding declarations belong to Universal.
    #[error("{position:?} identity belongs to {found:?}; expected Universal")]
    WrongRoot {
        position: AuthoredIdentityPosition,
        found: VocabularyRoot,
    },

    /// One input signature declared the same durable binding twice.
    #[error("input signature declares binding {binding:?} more than once")]
    DuplicateBinding { binding: VocabularyEncodedId },

    /// A typed escape references no binding in its declaration's input signature.
    #[error("typed Logos skeleton references undeclared binding {binding:?}")]
    UndeclaredBinding { binding: VocabularyEncodedId },

    /// A Logos path must carry at least one complete encoded-ID segment.
    #[error("authored Logos path is empty")]
    EmptyPath,
}

/// The translator-issued durable identity of one transformer.
#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct AuthoredTransformerIdentity(VocabularyEncodedId);

impl AuthoredTransformerIdentity {
    /// Admit a Universal durable identity without flattening its module chain.
    pub fn try_new(encoded_id: VocabularyEncodedId) -> Result<Self, AuthoredNomosError> {
        require_universal(&encoded_id, AuthoredIdentityPosition::Transformer)?;
        Ok(Self(encoded_id))
    }

    /// The complete root-fronted encoded-ID chain.
    pub fn encoded_id(&self) -> &VocabularyEncodedId {
        &self.0
    }

    /// Recover the complete root-fronted encoded-ID chain.
    pub fn into_encoded_id(self) -> VocabularyEncodedId {
        self.0
    }
}

/// The translator-issued durable identity of one input binding.
#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Clone,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct AuthoredBindingIdentity(VocabularyEncodedId);

impl AuthoredBindingIdentity {
    /// Admit a Universal durable identity without flattening its module chain.
    pub fn try_new(encoded_id: VocabularyEncodedId) -> Result<Self, AuthoredNomosError> {
        require_universal(&encoded_id, AuthoredIdentityPosition::Binding)?;
        Ok(Self(encoded_id))
    }

    /// The complete root-fronted encoded-ID chain.
    pub fn encoded_id(&self) -> &VocabularyEncodedId {
        &self.0
    }

    /// Recover the complete root-fronted encoded-ID chain.
    pub fn into_encoded_id(self) -> VocabularyEncodedId {
        self.0
    }
}

fn require_universal(
    encoded_id: &VocabularyEncodedId,
    position: AuthoredIdentityPosition,
) -> Result<(), AuthoredNomosError> {
    let found = *encoded_id.root_variant();
    if found != VocabularyRoot::Universal {
        return Err(AuthoredNomosError::WrongRoot { position, found });
    }
    Ok(())
}

/// One authored input position: durable binding identity, then meta-type.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredInputParameter(AuthoredBindingIdentity, MetaType);

impl AuthoredInputParameter {
    pub fn new(binding: AuthoredBindingIdentity, meta: MetaType) -> Self {
        Self(binding, meta)
    }

    pub fn binding(&self) -> &AuthoredBindingIdentity {
        &self.0
    }

    pub const fn meta(&self) -> MetaType {
        self.1
    }
}

/// The ordered, positional input signature of an authored transformer.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredInputSignature(Vec<AuthoredInputParameter>);

impl AuthoredInputSignature {
    /// Construct a signature, refusing duplicate durable binding identities.
    pub fn try_new(parameters: Vec<AuthoredInputParameter>) -> Result<Self, AuthoredNomosError> {
        let mut bindings = BTreeSet::new();
        for parameter in &parameters {
            let binding = parameter.binding().encoded_id();
            if !bindings.insert(binding.clone()) {
                return Err(AuthoredNomosError::DuplicateBinding {
                    binding: binding.clone(),
                });
            }
        }
        Ok(Self(parameters))
    }

    /// The unit input signature.
    pub fn unit() -> Self {
        Self(Vec::new())
    }

    pub fn parameters(&self) -> &[AuthoredInputParameter] {
        &self.0
    }

    fn contains(&self, binding: &AuthoredBindingIdentity) -> bool {
        self.0
            .iter()
            .any(|parameter| parameter.binding() == binding)
    }
}

/// A positional scalar in the authored typed Logos skeleton.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub enum AuthoredScalar<Literal> {
    Literal(Literal),
    Escape(AuthoredEscape),
}

/// An ordered vector position in the authored typed Logos skeleton.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct AuthoredSequence<Literal>(Vec<AuthoredSequenceItem<Literal>>);

impl<Literal> AuthoredSequence<Literal> {
    pub fn new(items: Vec<AuthoredSequenceItem<Literal>>) -> Self {
        Self(items)
    }

    pub fn of(item: AuthoredSequenceItem<Literal>) -> Self {
        Self(vec![item])
    }

    pub fn items(&self) -> &[AuthoredSequenceItem<Literal>] {
        &self.0
    }
}

/// One literal or escape position within an authored sequence.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub enum AuthoredSequenceItem<Literal> {
    Literal(Literal),
    Escape(AuthoredEscape),
}

/// The closed authored escape set. Invocation retains durable target identity;
/// it is not rebound to a package-local execution index during text decoding.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredEscape {
    Realize(AuthoredRealize),
    Invoke(AuthoredTransformerIdentity),
    Splice(AuthoredSplice),
}

/// One top-level input binding used by a typed escape.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredBindingRef {
    Input(AuthoredBindingIdentity),
}

impl AuthoredBindingRef {
    fn identity(&self) -> &AuthoredBindingIdentity {
        match self {
            Self::Input(identity) => identity,
        }
    }
}

/// Realize one bound value at a typed Logos position.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredRealize(AuthoredBindingRef, NameTransform);

impl AuthoredRealize {
    pub fn new(binding: AuthoredBindingRef, transform: NameTransform) -> Self {
        Self(binding, transform)
    }

    pub fn binding(&self) -> &AuthoredBindingRef {
        &self.0
    }

    pub const fn transform(&self) -> NameTransform {
        self.1
    }
}

/// Expand one bound sequence at a typed Logos vector position.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredSplice(AuthoredBindingRef, AuthoredSpliceElement);

impl AuthoredSplice {
    pub fn new(binding: AuthoredBindingRef, element: AuthoredSpliceElement) -> Self {
        Self(binding, element)
    }

    pub fn binding(&self) -> &AuthoredBindingRef {
        &self.0
    }

    pub fn element(&self) -> &AuthoredSpliceElement {
        &self.1
    }
}

/// The current per-element production set for an authored splice.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredSpliceElement {
    Field(AuthoredVisibility, FieldNameRule),
    Variant,
}

/// Logos visibility stored as typed data with full chains for module paths.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredVisibility {
    Public,
    Crate,
    Module(AuthoredPath),
    Private,
}

/// A non-empty Logos path whose segments retain complete encoded-ID chains.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredPath(Vec<VocabularyEncodedId>);

impl AuthoredPath {
    pub fn try_new(segments: Vec<VocabularyEncodedId>) -> Result<Self, AuthoredNomosError> {
        if segments.is_empty() {
            return Err(AuthoredNomosError::EmptyPath);
        }
        Ok(Self(segments))
    }

    pub fn segments(&self) -> &[VocabularyEncodedId] {
        &self.0
    }
}

/// A full-chain Logos type-reference position.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub enum AuthoredTypeReference {
    Path(AuthoredPath),
    Application(AuthoredTypeApplication),
    Reference(AuthoredReferenceType),
    ImplTrait(AuthoredImplTraitType),
    Slice(AuthoredSliceType),
    Tuple(AuthoredTupleType),
    Lifetime(VocabularyEncodedId),
}

/// A Logos type application: head path, then positional arguments.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct AuthoredTypeApplication(
    AuthoredPath,
    #[rkyv(omit_bounds)] Vec<AuthoredTypeReference>,
);

impl AuthoredTypeApplication {
    pub fn new(head: AuthoredPath, arguments: Vec<AuthoredTypeReference>) -> Self {
        Self(head, arguments)
    }

    pub fn head(&self) -> &AuthoredPath {
        &self.0
    }

    pub fn arguments(&self) -> &[AuthoredTypeReference] {
        &self.1
    }
}

/// A Logos reference type.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct AuthoredReferenceType(
    Option<VocabularyEncodedId>,
    AuthoredReferenceMutability,
    #[rkyv(omit_bounds)] Box<AuthoredTypeReference>,
);

impl AuthoredReferenceType {
    pub fn new(
        lifetime: Option<VocabularyEncodedId>,
        mutability: AuthoredReferenceMutability,
        referent: AuthoredTypeReference,
    ) -> Self {
        Self(lifetime, mutability, Box::new(referent))
    }

    pub fn lifetime(&self) -> Option<&VocabularyEncodedId> {
        self.0.as_ref()
    }

    pub fn mutability(&self) -> &AuthoredReferenceMutability {
        &self.1
    }

    pub fn referent(&self) -> &AuthoredTypeReference {
        &self.2
    }
}

/// Whether a Logos reference is shared or mutable.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredReferenceMutability {
    Shared,
    Mutable,
}

/// A Logos slice type.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct AuthoredSliceType(#[rkyv(omit_bounds)] Box<AuthoredTypeReference>);

impl AuthoredSliceType {
    pub fn new(element: AuthoredTypeReference) -> Self {
        Self(Box::new(element))
    }

    pub fn element(&self) -> &AuthoredTypeReference {
        &self.0
    }
}

/// A positional Logos tuple type.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct AuthoredTupleType(#[rkyv(omit_bounds)] Vec<AuthoredTypeReference>);

impl AuthoredTupleType {
    pub fn new(elements: Vec<AuthoredTypeReference>) -> Self {
        Self(elements)
    }

    pub fn elements(&self) -> &[AuthoredTypeReference] {
        &self.0
    }
}

/// A positional Logos `impl Trait` type.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct AuthoredImplTraitType(#[rkyv(omit_bounds)] Vec<AuthoredTypeReference>);

impl AuthoredImplTraitType {
    pub fn new(bounds: Vec<AuthoredTypeReference>) -> Self {
        Self(bounds)
    }

    pub fn bounds(&self) -> &[AuthoredTypeReference] {
        &self.0
    }
}

/// Logos generic parameters with every named position carrying a full chain.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredGenerics(Vec<AuthoredGenericParameter>);

impl AuthoredGenerics {
    pub fn new(parameters: Vec<AuthoredGenericParameter>) -> Self {
        Self(parameters)
    }

    pub fn none() -> Self {
        Self(Vec::new())
    }

    pub fn parameters(&self) -> &[AuthoredGenericParameter] {
        &self.0
    }
}

/// One typed Logos generic parameter.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredGenericParameter {
    Type(AuthoredTypeParameter),
    Lifetime(VocabularyEncodedId),
}

/// One type parameter: durable name, then ordered trait-bound paths.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredTypeParameter(VocabularyEncodedId, Vec<AuthoredPath>);

impl AuthoredTypeParameter {
    pub fn new(name: VocabularyEncodedId, bounds: Vec<AuthoredPath>) -> Self {
        Self(name, bounds)
    }

    pub fn name(&self) -> &VocabularyEncodedId {
        &self.0
    }

    pub fn bounds(&self) -> &[AuthoredPath] {
        &self.1
    }
}

/// One positional Logos field literal.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredField(
    AuthoredVisibility,
    VocabularyEncodedId,
    AuthoredTypeReference,
);

impl AuthoredField {
    pub fn new(
        visibility: AuthoredVisibility,
        name: VocabularyEncodedId,
        type_reference: AuthoredTypeReference,
    ) -> Self {
        Self(visibility, name, type_reference)
    }

    pub fn visibility(&self) -> &AuthoredVisibility {
        &self.0
    }

    pub fn name(&self) -> &VocabularyEncodedId {
        &self.1
    }

    pub fn type_reference(&self) -> &AuthoredTypeReference {
        &self.2
    }
}

/// One Logos enumeration variant literal.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredVariant(VocabularyEncodedId, AuthoredVariantPayload);

impl AuthoredVariant {
    pub fn new(name: VocabularyEncodedId, payload: AuthoredVariantPayload) -> Self {
        Self(name, payload)
    }

    pub fn name(&self) -> &VocabularyEncodedId {
        &self.0
    }

    pub fn payload(&self) -> &AuthoredVariantPayload {
        &self.1
    }
}

/// The positional payload of one Logos enumeration variant literal.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredVariantPayload {
    Unit,
    Tuple(Vec<AuthoredTypeReference>),
    Struct(Vec<AuthoredField>),
}

/// A Logos attribute literal with no spelling-bearing position.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub enum AuthoredAttribute {
    Derive(AuthoredDeriveGroup),
    Configuration(AuthoredConfigurationAttribute),
    Cfg(AuthoredConfigurationPredicate),
    ToolPath(AuthoredPath),
    HelperDerive(AuthoredHelperDerive),
}

/// An ordered Logos derive group.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredDeriveGroup(Vec<AuthoredPath>);

impl AuthoredDeriveGroup {
    pub fn new(paths: Vec<AuthoredPath>) -> Self {
        Self(paths)
    }

    pub fn paths(&self) -> &[AuthoredPath] {
        &self.0
    }
}

/// A typed Logos configuration predicate.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredConfigurationPredicate {
    Feature(VocabularyEncodedId),
}

/// One conditional Logos attribute.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
#[rkyv(
    serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source),
    deserialize_bounds(__D::Error: rkyv::rancor::Source),
    bytecheck(bounds(__C: rkyv::validation::ArchiveContext, __C::Error: rkyv::rancor::Source)),
)]
pub struct AuthoredConfigurationAttribute(
    AuthoredConfigurationPredicate,
    #[rkyv(omit_bounds)] Box<AuthoredAttribute>,
);

impl AuthoredConfigurationAttribute {
    pub fn new(predicate: AuthoredConfigurationPredicate, inner: AuthoredAttribute) -> Self {
        Self(predicate, Box::new(inner))
    }

    pub fn predicate(&self) -> &AuthoredConfigurationPredicate {
        &self.0
    }

    pub fn inner(&self) -> &AuthoredAttribute {
        &self.1
    }
}

/// A namespaced Logos helper-derive attribute.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredHelperDerive(AuthoredPath, AuthoredDeriveGroup);

impl AuthoredHelperDerive {
    pub fn new(path: AuthoredPath, derived: AuthoredDeriveGroup) -> Self {
        Self(path, derived)
    }

    pub fn path(&self) -> &AuthoredPath {
        &self.0
    }

    pub fn derived(&self) -> &AuthoredDeriveGroup {
        &self.1
    }
}

/// The authored result: a typed Logos skeleton or attribute sequence.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredResultSkeleton {
    Item(AuthoredItemSkeleton),
    Attributes(AuthoredSequence<AuthoredAttribute>),
}

/// One item-producing typed Logos skeleton.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum AuthoredItemSkeleton {
    Newtype(AuthoredNewtypeSkeleton),
    Struct(AuthoredStructSkeleton),
    Enumeration(AuthoredEnumerationSkeleton),
}

/// A positional tuple-newtype skeleton.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredNewtypeSkeleton(
    AuthoredVisibility,
    AuthoredSequence<AuthoredAttribute>,
    AuthoredScalar<VocabularyEncodedId>,
    AuthoredScalar<AuthoredTypeReference>,
);

impl AuthoredNewtypeSkeleton {
    pub fn new(
        visibility: AuthoredVisibility,
        attributes: AuthoredSequence<AuthoredAttribute>,
        name: AuthoredScalar<VocabularyEncodedId>,
        wrapped: AuthoredScalar<AuthoredTypeReference>,
    ) -> Self {
        Self(visibility, attributes, name, wrapped)
    }

    pub fn visibility(&self) -> &AuthoredVisibility {
        &self.0
    }

    pub fn attributes(&self) -> &AuthoredSequence<AuthoredAttribute> {
        &self.1
    }

    pub fn name(&self) -> &AuthoredScalar<VocabularyEncodedId> {
        &self.2
    }

    pub fn wrapped(&self) -> &AuthoredScalar<AuthoredTypeReference> {
        &self.3
    }
}

/// A positional named-field struct skeleton.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredStructSkeleton(
    AuthoredVisibility,
    AuthoredSequence<AuthoredAttribute>,
    AuthoredScalar<VocabularyEncodedId>,
    AuthoredGenerics,
    AuthoredSequence<AuthoredField>,
);

impl AuthoredStructSkeleton {
    pub fn new(
        visibility: AuthoredVisibility,
        attributes: AuthoredSequence<AuthoredAttribute>,
        name: AuthoredScalar<VocabularyEncodedId>,
        generics: AuthoredGenerics,
        fields: AuthoredSequence<AuthoredField>,
    ) -> Self {
        Self(visibility, attributes, name, generics, fields)
    }

    pub fn visibility(&self) -> &AuthoredVisibility {
        &self.0
    }

    pub fn attributes(&self) -> &AuthoredSequence<AuthoredAttribute> {
        &self.1
    }

    pub fn name(&self) -> &AuthoredScalar<VocabularyEncodedId> {
        &self.2
    }

    pub fn generics(&self) -> &AuthoredGenerics {
        &self.3
    }

    pub fn fields(&self) -> &AuthoredSequence<AuthoredField> {
        &self.4
    }
}

/// A positional enumeration skeleton.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredEnumerationSkeleton(
    AuthoredVisibility,
    AuthoredSequence<AuthoredAttribute>,
    AuthoredScalar<VocabularyEncodedId>,
    AuthoredGenerics,
    AuthoredSequence<AuthoredVariant>,
);

impl AuthoredEnumerationSkeleton {
    pub fn new(
        visibility: AuthoredVisibility,
        attributes: AuthoredSequence<AuthoredAttribute>,
        name: AuthoredScalar<VocabularyEncodedId>,
        generics: AuthoredGenerics,
        variants: AuthoredSequence<AuthoredVariant>,
    ) -> Self {
        Self(visibility, attributes, name, generics, variants)
    }

    pub fn visibility(&self) -> &AuthoredVisibility {
        &self.0
    }

    pub fn attributes(&self) -> &AuthoredSequence<AuthoredAttribute> {
        &self.1
    }

    pub fn name(&self) -> &AuthoredScalar<VocabularyEncodedId> {
        &self.2
    }

    pub fn generics(&self) -> &AuthoredGenerics {
        &self.3
    }

    pub fn variants(&self) -> &AuthoredSequence<AuthoredVariant> {
        &self.4
    }
}

/// One authored transformer declaration in durable pre-seal form.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct AuthoredTransformerDeclaration(
    AuthoredTransformerIdentity,
    MacroKind,
    AuthoredInputSignature,
    AuthoredResultSkeleton,
);

impl AuthoredTransformerDeclaration {
    /// Construct one declaration and refuse every escape whose binding is absent
    /// from the positional input signature.
    pub fn try_new(
        name: AuthoredTransformerIdentity,
        kind: MacroKind,
        input: AuthoredInputSignature,
        result: AuthoredResultSkeleton,
    ) -> Result<Self, AuthoredNomosError> {
        validate_result_bindings(&input, &result)?;
        Ok(Self(name, kind, input, result))
    }

    pub fn name(&self) -> &AuthoredTransformerIdentity {
        &self.0
    }

    pub const fn kind(&self) -> MacroKind {
        self.1
    }

    pub fn input(&self) -> &AuthoredInputSignature {
        &self.2
    }

    pub fn result(&self) -> &AuthoredResultSkeleton {
        &self.3
    }
}

fn validate_result_bindings(
    input: &AuthoredInputSignature,
    result: &AuthoredResultSkeleton,
) -> Result<(), AuthoredNomosError> {
    match result {
        AuthoredResultSkeleton::Item(AuthoredItemSkeleton::Newtype(skeleton)) => {
            validate_sequence(input, skeleton.attributes())?;
            validate_scalar(input, skeleton.name())?;
            validate_scalar(input, skeleton.wrapped())
        }
        AuthoredResultSkeleton::Item(AuthoredItemSkeleton::Struct(skeleton)) => {
            validate_sequence(input, skeleton.attributes())?;
            validate_scalar(input, skeleton.name())?;
            validate_sequence(input, skeleton.fields())
        }
        AuthoredResultSkeleton::Item(AuthoredItemSkeleton::Enumeration(skeleton)) => {
            validate_sequence(input, skeleton.attributes())?;
            validate_scalar(input, skeleton.name())?;
            validate_sequence(input, skeleton.variants())
        }
        AuthoredResultSkeleton::Attributes(attributes) => validate_sequence(input, attributes),
    }
}

fn validate_scalar<Literal>(
    input: &AuthoredInputSignature,
    scalar: &AuthoredScalar<Literal>,
) -> Result<(), AuthoredNomosError> {
    if let AuthoredScalar::Escape(escape_node) = scalar {
        validate_escape(input, escape_node)?;
    }
    Ok(())
}

fn validate_sequence<Literal>(
    input: &AuthoredInputSignature,
    sequence: &AuthoredSequence<Literal>,
) -> Result<(), AuthoredNomosError> {
    for item in sequence.items() {
        if let AuthoredSequenceItem::Escape(escape_node) = item {
            validate_escape(input, escape_node)?;
        }
    }
    Ok(())
}

fn validate_escape(
    input: &AuthoredInputSignature,
    escape_node: &AuthoredEscape,
) -> Result<(), AuthoredNomosError> {
    let binding = match escape_node {
        AuthoredEscape::Realize(realize) => Some(realize.binding().identity()),
        AuthoredEscape::Invoke(_) => None,
        AuthoredEscape::Splice(splice) => Some(splice.binding().identity()),
    };
    if let Some(binding) = binding
        && !input.contains(binding)
    {
        return Err(AuthoredNomosError::UndeclaredBinding {
            binding: binding.encoded_id().clone(),
        });
    }
    Ok(())
}

//! The lowering engine: apply a macro package to a `EncodedSchema` declaration set,
//! producing `EncodedLogos` items and the extended, continuous logos NameTable.
//!
//! Conversions are typed end to end, outside text. Named invocations resolve or
//! error loudly; structural defaults cover plain declarations; recursive
//! invocation is bounded by cycle rejection; template realization produces genuine
//! `core_logos` values; and the NameTable is extended continuously — schema
//! identifiers keep their indices and logos names append, with every template
//! literal re-interned through the package's authoring table into the extension.

use core_logos::{
    Attribute, ConfigurationAttribute, ConfigurationPredicate, DeriveGroup, EncodedItem,
    Enumeration, Field, HelperDerive, ImplTraitType, Newtype, PathNode, ReferenceType, SliceType,
    Struct, TupleType, TypeApplication, TypeReference, Variant, VariantPayload, Visibility,
};
use core_schema::{EncodedDeclaration, EncodedSchema, EncodedType};
use name_table::{Identifier, NameTable};
use structural_codec::{Converted, EncodedConversion};

use crate::error::NomosError;
use crate::identity::{MacroIdentity, SectionDefault};
use crate::meta::{BoundInput, InputSignature, MetaType, MetaValue};
use crate::name_boundary::NameTableBoundary;
use crate::package::MacroPackage;
use crate::template::{
    BindingRef, EnumerationTemplate, Escape, ItemTemplate, NameTransform, NewtypeTemplate, Realize,
    ResultTemplate, Scalar, Sequence, SequenceItem, Splice, SpliceElement, StructTemplate,
};

/// The result of lowering a schema: the produced `EncodedLogos` items, in declaration
/// order, and the extended logos NameTable that resolves every identifier they
/// carry. The NameTable owns a Logos slice and borrows the completed Schema and
/// LogosStandard slices; Logos-only names allocate in that owned slice.
#[derive(Clone, Debug)]
pub struct Lowering {
    /// The lowered items, one per schema declaration.
    pub items: Vec<EncodedItem>,
    /// The extended, continuous logos NameTable.
    pub names: NameTable,
}

impl MacroPackage {
    /// Apply this package to a schema, lowering every declaration through its
    /// section's structural default macro.
    pub fn apply(
        &self,
        schema: &EncodedSchema,
        schema_names: &NameTable,
    ) -> Result<Lowering, NomosError> {
        self.ensure_authoring_names()?;
        let mut evaluator = Evaluator::new(self, schema_names)?;
        let items = evaluator.lower_schema(schema)?;
        Ok(Lowering {
            items,
            names: evaluator.into_names()?,
        })
    }

    /// Apply this package to a schema through the *enriched* selection: the
    /// per-declaration structural lowering first (the data declarations), then the
    /// generation classes ([`crate::GenerationClass`]) in the package's selection
    /// order — class A, then B, then C, then D, the reference fixture's own document order. The
    /// returned items are the whole ordered run, resolved by one continuous logos
    /// NameTable. A package with an empty selection produces exactly what
    /// [`apply`](Self::apply) does.
    pub fn apply_enriched(
        &self,
        schema: &EncodedSchema,
        schema_names: &NameTable,
    ) -> Result<Lowering, NomosError> {
        self.ensure_authoring_names()?;
        let mut evaluator = Evaluator::new(self, schema_names)?;
        let mut items = evaluator.lower_schema(schema)?;
        for class in self.selection() {
            items.extend(evaluator.generate_class(class, schema)?);
        }
        Ok(Lowering {
            items,
            names: evaluator.into_names()?,
        })
    }
}

/// A [`Lowering`] IS a [`Converted`] `Vec<EncodedItem>`-plus-names: the domain-named
/// result of the lowering and the reusable-trait output of an [`EncodedConversion`] are
/// the same data, so the trait face costs no new representation.
impl From<Lowering> for Converted<Vec<EncodedItem>> {
    fn from(lowering: Lowering) -> Self {
        Converted {
            target: lowering.items,
            names: lowering.names,
        }
    }
}

/// The schema→logos lowering IS the reference [`EncodedConversion`] instance — the
/// psyche's real type conversion `EncodedForm<Schema> -> EncodedForm<Logos>` seated as
/// the truth-side pairing in `structural-codec`. The source is the schema
/// [`EncodedForm`](structural_codec::EncodedForm) (`EncodedSchema`); the target is the
/// lowered logos item set (`Vec<EncodedItem>`, the logos EncodedForm); and the continuous
/// NameTable threads the layer, schema indices preserved and logos names appended. No
/// text crosses this path — the signature carries no `&str`/`String`, which is the
/// structural proof that the conversion is a real type conversion, not string
/// manipulation. It delegates to the eponymous [`apply`](MacroPackage::apply).
impl EncodedConversion for MacroPackage {
    type Source = EncodedSchema;
    type Target = Vec<EncodedItem>;
    type Error = NomosError;

    fn convert(
        &self,
        source: &EncodedSchema,
        names: &NameTable,
    ) -> Result<Converted<Vec<EncodedItem>>, NomosError> {
        Ok(self.apply(source, names)?.into())
    }
}

/// A produced fragment — what evaluating a result template yields. A structural
/// default yields an item; a recursively-invoked attribute macro yields a vector.
enum Fragment {
    // Boxed: a `EncodedItem` now carries whole impl-block/method-body trees, dwarfing
    // the attribute-vector variant, so the box keeps the enum small.
    Item(Box<EncodedItem>),
    Attributes(Vec<Attribute>),
}

/// The stateful lowering walk: the package (for macro lookup and literal name
/// remapping), the extended logos NameTable being built, and the active-invocation
/// stack for cycle rejection. Crate-visible so the enriched generation classes
/// ([`crate::generation`]) can append their items into the same continuous NameTable
/// the declaration lowering built.
pub(crate) struct Evaluator<'package> {
    package: &'package MacroPackage,
    /// The sole NameTable/emission boundary. Typed evaluation asks this boundary to
    /// allocate or project names but never reads or constructs text itself.
    pub(crate) names: NameTableBoundary<'package>,
    active: Vec<MacroIdentity>,
}

impl<'package> Evaluator<'package> {
    fn new(package: &'package MacroPackage, schema_names: &NameTable) -> Result<Self, NomosError> {
        Ok(Self {
            package,
            names: NameTableBoundary::new(package.names(), schema_names)?,
            active: Vec::new(),
        })
    }

    fn into_names(self) -> Result<NameTable, NomosError> {
        self.names.into_names()
    }

    fn lower_schema(&mut self, schema: &EncodedSchema) -> Result<Vec<EncodedItem>, NomosError> {
        schema
            .declarations()
            .iter()
            .map(|declaration| self.lower_declaration(declaration))
            .collect()
    }

    fn lower_declaration(
        &mut self,
        declaration: &EncodedDeclaration,
    ) -> Result<EncodedItem, NomosError> {
        let value = declaration.value();
        let section = SectionDefault::of_encoded_type(value);
        let identity = self
            .package
            .structural_default(section)
            .ok_or(NomosError::NoStructuralDefault(section))?;
        let definition = self
            .package
            .definition(identity)
            .ok_or(NomosError::UnknownMacro(identity))?;
        let bound = self.bind_input(&definition.input, value)?;
        self.active.push(identity);
        let fragment = self.evaluate(&definition.template, &bound)?;
        self.active.pop();
        match fragment {
            Fragment::Item(item) => {
                Ok((*item).with_visibility(self.lower_visibility(declaration.visibility())))
            }
            Fragment::Attributes(_) => Err(NomosError::FragmentKind(
                "a structural default produced attributes, not an item",
            )),
        }
    }

    /// The visibility contact point where two enums meet: core-schema's coarse
    /// declaration visibility lowers into core-logos' richer `Visibility`. The
    /// schema declaration's `Public`/`Private` is an authoritative API promise and
    /// stamps the produced item, overriding the visibility the macro template
    /// proposed; core-logos then stores that final visibility explicitly. Settled
    /// psyche ruling (primary-56d1.29): schema visibility is authoritative and
    /// core-nomos must lower it faithfully into the generated Rust.
    fn lower_visibility(&self, visibility: core_schema::Visibility) -> Visibility {
        match visibility {
            core_schema::Visibility::Public => Visibility::Public,
            core_schema::Visibility::Private => Visibility::Private,
        }
    }

    /// Fill the macro's input meta-shape from the declaration it lowers
    /// (object-to-object): a `Name` binds the declaration identifier, a `Type` a
    /// newtype's wrapped reference, `Fields` a struct's fields.
    fn bind_input(
        &self,
        signature: &InputSignature,
        value: &EncodedType,
    ) -> Result<BoundInput, NomosError> {
        let mut bound = BoundInput::new();
        for parameter in &signature.parameters {
            let meta_value = match parameter.meta {
                MetaType::Name => MetaValue::Name(value.identifier()),
                MetaType::Type => match value {
                    EncodedType::Newtype(newtype) => MetaValue::Type(newtype.reference().clone()),
                    _ => {
                        return Err(NomosError::MetaShape {
                            meta: MetaType::Type,
                        });
                    }
                },
                MetaType::Fields => match value {
                    EncodedType::Struct(structure) => {
                        MetaValue::Fields(structure.fields().to_vec())
                    }
                    _ => {
                        return Err(NomosError::MetaShape {
                            meta: MetaType::Fields,
                        });
                    }
                },
                MetaType::Variants => match value {
                    EncodedType::Enumeration(enumeration) => {
                        MetaValue::Variants(enumeration.variants().to_vec())
                    }
                    _ => {
                        return Err(NomosError::MetaShape {
                            meta: MetaType::Variants,
                        });
                    }
                },
            };
            bound.bind(parameter.binding, meta_value);
        }
        Ok(bound)
    }

    fn evaluate(
        &mut self,
        template: &ResultTemplate,
        bound: &BoundInput,
    ) -> Result<Fragment, NomosError> {
        match template {
            ResultTemplate::Item(item) => {
                Ok(Fragment::Item(Box::new(self.evaluate_item(item, bound)?)))
            }
            ResultTemplate::Attributes(sequence) => {
                Ok(Fragment::Attributes(self.evaluate_attributes(sequence)?))
            }
        }
    }

    fn evaluate_item(
        &mut self,
        item: &ItemTemplate,
        bound: &BoundInput,
    ) -> Result<EncodedItem, NomosError> {
        match item {
            ItemTemplate::Newtype(template) => self.evaluate_newtype(template, bound),
            ItemTemplate::Struct(template) => self.evaluate_struct(template, bound),
            ItemTemplate::Enumeration(template) => self.evaluate_enumeration(template, bound),
        }
    }

    fn evaluate_newtype(
        &mut self,
        template: &NewtypeTemplate,
        bound: &BoundInput,
    ) -> Result<EncodedItem, NomosError> {
        let attributes = self.evaluate_attributes(&template.attributes)?;
        let name = self.evaluate_name(&template.name, bound)?;
        let wrapped = self.evaluate_type(&template.wrapped, bound)?;
        Ok(EncodedItem::Newtype(Newtype {
            visibility: template.visibility.clone(),
            attributes,
            name,
            // The structural newtype default emits `pub struct Name(Wrapped);` — a
            // private tuple field. The only `pub`-field tuple struct in the surveyed
            // reference fixtures is the class-D `TraceEvent` declaration, hand-built by the
            // TraceSupport generator, not a structural default. So the field
            // visibility here is the literal `Private` that projects to nothing.
            wrapped_visibility: Visibility::Private,
            wrapped,
        }))
    }

    fn evaluate_struct(
        &mut self,
        template: &StructTemplate,
        bound: &BoundInput,
    ) -> Result<EncodedItem, NomosError> {
        let attributes = self.evaluate_attributes(&template.attributes)?;
        let name = self.evaluate_name(&template.name, bound)?;
        let fields = self.evaluate_fields(&template.fields, bound)?;
        Ok(EncodedItem::Struct(Struct {
            visibility: template.visibility.clone(),
            attributes,
            name,
            generics: template.generics.clone(),
            fields,
        }))
    }

    fn evaluate_enumeration(
        &mut self,
        template: &EnumerationTemplate,
        bound: &BoundInput,
    ) -> Result<EncodedItem, NomosError> {
        let mut attributes = self.evaluate_attributes(&template.attributes)?;
        let name = self.evaluate_name(&template.name, bound)?;
        let variants = self.evaluate_variants(&template.variants, bound)?;
        if variants
            .iter()
            .any(|variant| !matches!(variant.payload, VariantPayload::Unit))
        {
            self.names.remove_copy_derive(&mut attributes)?;
        }
        Ok(EncodedItem::Enumeration(Enumeration {
            visibility: template.visibility.clone(),
            attributes,
            name,
            generics: template.generics.clone(),
            variants,
        }))
    }

    fn evaluate_name(
        &mut self,
        slot: &Scalar<Identifier>,
        bound: &BoundInput,
    ) -> Result<Identifier, NomosError> {
        match slot {
            Scalar::Literal(identifier) => self.names.place_literal_name(*identifier),
            Scalar::Escape(Escape::Realize(realize)) => self.realize_name(realize, bound),
            Scalar::Escape(Escape::Invoke(_)) => Err(NomosError::EscapeShape(
                "an invoke cannot fill a name position",
            )),
            Scalar::Escape(Escape::Splice(_)) => Err(NomosError::EscapeShape(
                "a splice cannot fill a name position",
            )),
        }
    }

    fn realize_name(
        &mut self,
        realize: &Realize,
        bound: &BoundInput,
    ) -> Result<Identifier, NomosError> {
        let BindingRef::Input(binding) = realize.binding;
        match bound
            .value(binding)
            .ok_or(NomosError::UnboundInput(binding))?
        {
            MetaValue::Name(identifier) => {
                self.names.transform_name(*identifier, realize.transform)
            }
            _ => Err(NomosError::NameTransformShape),
        }
    }

    fn evaluate_type(
        &mut self,
        slot: &Scalar<TypeReference>,
        bound: &BoundInput,
    ) -> Result<TypeReference, NomosError> {
        match slot {
            Scalar::Literal(reference) => self.remap_type_reference(reference),
            Scalar::Escape(Escape::Realize(realize)) => {
                if realize.transform != NameTransform::Identity {
                    return Err(NomosError::EscapeShape(
                        "a realized type takes no name transform",
                    ));
                }
                let BindingRef::Input(binding) = realize.binding;
                match bound
                    .value(binding)
                    .ok_or(NomosError::UnboundInput(binding))?
                {
                    MetaValue::Type(reference) => {
                        let reference = reference.clone();
                        self.lower_reference(&reference)
                    }
                    _ => Err(NomosError::EscapeShape(
                        "a realize of a non-type binding cannot fill a type position",
                    )),
                }
            }
            Scalar::Escape(Escape::Invoke(_)) => Err(NomosError::EscapeShape(
                "an invoke cannot fill a type position",
            )),
            Scalar::Escape(Escape::Splice(_)) => Err(NomosError::EscapeShape(
                "a splice cannot fill a type position",
            )),
        }
    }

    fn evaluate_attributes(
        &mut self,
        sequence: &Sequence<Attribute>,
    ) -> Result<Vec<Attribute>, NomosError> {
        let mut out = Vec::new();
        for item in &sequence.items {
            match item {
                SequenceItem::Literal(attribute) => out.push(self.remap_attribute(attribute)?),
                SequenceItem::Escape(Escape::Invoke(identity)) => {
                    out.extend(self.invoke_attributes(*identity)?);
                }
                SequenceItem::Escape(Escape::Realize(_)) => {
                    return Err(NomosError::EscapeShape(
                        "there is no attribute binding to realize (schema carries no attributes)",
                    ));
                }
                SequenceItem::Escape(Escape::Splice(_)) => {
                    return Err(NomosError::EscapeShape(
                        "there is no attribute binding to splice (schema carries no attributes)",
                    ));
                }
            }
        }
        Ok(out)
    }

    /// Recursively invoke a macro that produces an attribute vector, rejecting a
    /// cycle. This is the concrete recursive invocation the ruling requires
    /// (WireNewtype invokes WireAttributes).
    fn invoke_attributes(&mut self, identity: MacroIdentity) -> Result<Vec<Attribute>, NomosError> {
        if self.active.contains(&identity) {
            return Err(NomosError::RecursionCycle(identity));
        }
        let template = self
            .package
            .definition(identity)
            .ok_or(NomosError::UnknownMacro(identity))?
            .template
            .clone();
        let bound = BoundInput::new();
        self.active.push(identity);
        let fragment = self.evaluate(&template, &bound)?;
        self.active.pop();
        match fragment {
            Fragment::Attributes(attributes) => Ok(attributes),
            Fragment::Item(_) => Err(NomosError::FragmentKind(
                "an attribute invocation produced an item, not attributes",
            )),
        }
    }

    fn evaluate_fields(
        &mut self,
        sequence: &Sequence<Field>,
        bound: &BoundInput,
    ) -> Result<Vec<Field>, NomosError> {
        let mut out = Vec::new();
        for item in &sequence.items {
            match item {
                SequenceItem::Literal(field) => out.push(self.remap_field(field)?),
                SequenceItem::Escape(Escape::Splice(splice)) => {
                    out.extend(self.splice_fields(splice, bound)?);
                }
                SequenceItem::Escape(Escape::Realize(_)) => {
                    return Err(NomosError::EscapeShape(
                        "a realize cannot fill the fields position; fields splice",
                    ));
                }
                SequenceItem::Escape(Escape::Invoke(_)) => {
                    return Err(NomosError::EscapeShape(
                        "an invoke into the fields position is a growth point, unsupported",
                    ));
                }
            }
        }
        Ok(out)
    }

    fn splice_fields(
        &mut self,
        splice: &Splice,
        bound: &BoundInput,
    ) -> Result<Vec<Field>, NomosError> {
        let BindingRef::Input(binding) = splice.binding;
        let schema_fields = match bound
            .value(binding)
            .ok_or(NomosError::UnboundInput(binding))?
        {
            MetaValue::Fields(fields) => fields.clone(),
            _ => {
                return Err(NomosError::EscapeShape(
                    "a splice of a non-fields binding cannot fill the fields position",
                ));
            }
        };
        let SpliceElement::Field {
            visibility,
            name_rule,
        } = &splice.element
        else {
            return Err(NomosError::EscapeShape(
                "a variant splice cannot fill fields",
            ));
        };
        let names = self.names.field_names(&schema_fields, *name_rule)?;
        let mut out = Vec::with_capacity(schema_fields.len());
        for (field, name) in schema_fields.iter().zip(names) {
            let type_reference = self.lower_reference(field.reference())?;
            out.push(Field {
                visibility: visibility.clone(),
                name,
                type_reference,
            });
        }
        Ok(out)
    }

    fn evaluate_variants(
        &mut self,
        sequence: &Sequence<Variant>,
        bound: &BoundInput,
    ) -> Result<Vec<Variant>, NomosError> {
        let mut output = Vec::new();
        for item in &sequence.items {
            match item {
                SequenceItem::Literal(variant) => output.push(variant.clone()),
                SequenceItem::Escape(Escape::Splice(splice)) => {
                    let BindingRef::Input(binding) = splice.binding;
                    let variants = match bound
                        .value(binding)
                        .ok_or(NomosError::UnboundInput(binding))?
                    {
                        MetaValue::Variants(variants) => variants.clone(),
                        _ => {
                            return Err(NomosError::EscapeShape(
                                "a non-variants binding cannot fill variants",
                            ));
                        }
                    };
                    if !matches!(splice.element, SpliceElement::Variant) {
                        return Err(NomosError::EscapeShape(
                            "a field splice cannot fill variants",
                        ));
                    }
                    for variant in variants {
                        let payload = match variant.payload() {
                            None => VariantPayload::Unit,
                            Some(reference) => {
                                VariantPayload::Tuple(vec![self.lower_reference(reference)?])
                            }
                        };
                        output.push(Variant {
                            name: variant.identifier(),
                            payload,
                        });
                    }
                }
                SequenceItem::Escape(_) => {
                    return Err(NomosError::EscapeShape("enum variants require a splice"));
                }
            }
        }
        Ok(output)
    }

    /// Lower a typed schema reference through the NameTable boundary. The evaluator
    /// never reads or introduces a spelling while performing this conversion.
    pub(crate) fn lower_reference(
        &mut self,
        reference: &core_schema::EncodedReference,
    ) -> Result<TypeReference, NomosError> {
        self.names.lower_reference(reference)
    }

    fn remap_path(&mut self, path: &PathNode) -> Result<PathNode, NomosError> {
        let mut segments = Vec::with_capacity(path.segments.len());
        for segment in &path.segments {
            segments.push(self.names.place_literal_name(*segment)?);
        }
        Ok(PathNode { segments })
    }

    /// Re-intern every identifier a template-literal type carries into the extended
    /// logos table. Exhaustive over the whole `TypeReference` algebra: the
    /// data-declaration templates author only `Path`/`Application`, but the class-A/B/C/D
    /// ergonomics templates author impl-block signature and const types — `&String`,
    /// `impl Into<String>`, `&'static [&'static str]`, the `'static` lifetime — so the
    /// remap threads the continuous-identifier-space renaming through every position.
    fn remap_type_reference(
        &mut self,
        reference: &TypeReference,
    ) -> Result<TypeReference, NomosError> {
        match reference {
            TypeReference::Path(path) => Ok(TypeReference::Path(self.remap_path(path)?)),
            TypeReference::Application(application) => Ok(TypeReference::Application(
                self.remap_application(application)?,
            )),
            TypeReference::Reference(reference) => {
                let lifetime = match reference.lifetime {
                    Some(lifetime) => Some(self.names.place_literal_name(lifetime)?),
                    None => None,
                };
                Ok(TypeReference::Reference(ReferenceType {
                    lifetime,
                    mutability: reference.mutability.clone(),
                    referent: Box::new(self.remap_type_reference(&reference.referent)?),
                }))
            }
            TypeReference::ImplTrait(impl_trait) => {
                let bounds = impl_trait
                    .bounds
                    .iter()
                    .map(|bound| self.remap_type_reference(bound))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeReference::ImplTrait(ImplTraitType { bounds }))
            }
            TypeReference::Slice(slice) => Ok(TypeReference::Slice(SliceType {
                element: Box::new(self.remap_type_reference(&slice.element)?),
            })),
            TypeReference::Tuple(tuple) => {
                let elements = tuple
                    .elements
                    .iter()
                    .map(|element| self.remap_type_reference(element))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeReference::Tuple(TupleType { elements }))
            }
            TypeReference::Lifetime(lifetime) => Ok(TypeReference::Lifetime(
                self.names.place_literal_name(*lifetime)?,
            )),
        }
    }

    fn remap_application(
        &mut self,
        application: &TypeApplication,
    ) -> Result<TypeApplication, NomosError> {
        let head = self.remap_path(&application.head)?;
        let arguments = application
            .arguments
            .iter()
            .map(|argument| self.remap_type_reference(argument))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TypeApplication { head, arguments })
    }

    fn remap_attribute(&mut self, attribute: &Attribute) -> Result<Attribute, NomosError> {
        match attribute {
            Attribute::Derive(group) => Ok(Attribute::Derive(self.remap_derive(group)?)),
            Attribute::Configuration(configuration) => {
                let predicate = self.remap_predicate(&configuration.predicate)?;
                let inner = Box::new(self.remap_attribute(&configuration.inner)?);
                Ok(Attribute::Configuration(ConfigurationAttribute {
                    predicate,
                    inner,
                }))
            }
            Attribute::ToolPath(path) => Ok(Attribute::ToolPath(self.remap_path(path)?)),
            Attribute::HelperDerive(helper) => Ok(Attribute::HelperDerive(HelperDerive {
                path: self.remap_path(&helper.path)?,
                derived: self.remap_derive(&helper.derived)?,
            })),
            // A plain cfg gate remaps its predicate name like `cfg_attr` does — the
            // one continuous-identifier-space remapping, forward-compatible if a
            // future template ever authors a gated item.
            Attribute::Cfg(predicate) => Ok(Attribute::Cfg(self.remap_predicate(predicate)?)),
        }
    }

    fn remap_derive(&mut self, group: &DeriveGroup) -> Result<DeriveGroup, NomosError> {
        let paths = group
            .paths
            .iter()
            .map(|path| self.remap_path(path))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DeriveGroup { paths })
    }

    fn remap_predicate(
        &mut self,
        predicate: &ConfigurationPredicate,
    ) -> Result<ConfigurationPredicate, NomosError> {
        match predicate {
            ConfigurationPredicate::Feature(identifier) => Ok(ConfigurationPredicate::Feature(
                self.names.place_literal_name(*identifier)?,
            )),
        }
    }

    fn remap_field(&mut self, field: &Field) -> Result<Field, NomosError> {
        Ok(Field {
            visibility: field.visibility.clone(),
            name: self.names.place_literal_name(field.name)?,
            type_reference: self.remap_type_reference(&field.type_reference)?,
        })
    }
}

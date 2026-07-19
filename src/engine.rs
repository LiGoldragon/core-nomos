//! The lowering engine: apply a macro package to a `CoreSchema` declaration set,
//! producing `CoreLogos` items and the extended, continuous logos NameTable.
//!
//! Conversions are typed end to end, outside text. Named invocations resolve or
//! error loudly; structural defaults cover plain declarations; recursive
//! invocation is bounded by cycle rejection; template realization produces genuine
//! `core_logos` values; and the NameTable is extended continuously — schema
//! identifiers keep their indices and logos names append, with every template
//! literal re-interned through the package's authoring table into the extension.

use std::collections::BTreeMap;

use core_logos::{
    Attribute, ConfigurationAttribute, ConfigurationPredicate, CoreItem, DeriveGroup, Enumeration,
    Field, HelperDerive, ImplTraitType, Newtype, PathNode, ReferenceType, SliceType, Struct,
    TupleType, TypeApplication, TypeReference, Variant, VariantPayload, Visibility,
};
use core_schema::{CoreDeclaration, CoreField, CoreReference, CoreSchema, CoreType};
use name_table::{Identifier, Name, NameTable};
use structural_codec::{Converted, EncodedConversion};

use crate::error::NomosError;
use crate::identity::{MacroIdentity, SectionDefault};
use crate::meta::{BoundInput, InputSignature, MetaType, MetaValue};
use crate::package::MacroPackage;
use crate::template::{
    BindingRef, EnumerationTemplate, Escape, FieldNameRule, ItemTemplate, NameTransform,
    NewtypeTemplate, Realize, ResultTemplate, Scalar, Sequence, SequenceItem, Splice,
    SpliceElement, StructTemplate,
};

/// The result of lowering a schema: the produced `CoreLogos` items, in declaration
/// order, and the extended logos NameTable that resolves every identifier they
/// carry. The NameTable begins as an `extend_from` of the schema table (schema
/// indices preserved) and appends logos-only names (derive paths, leaf type names,
/// derived field names).
#[derive(Clone, Debug)]
pub struct Lowering {
    /// The lowered items, one per schema declaration.
    pub items: Vec<CoreItem>,
    /// The extended, continuous logos NameTable.
    pub names: NameTable,
}

impl MacroPackage {
    /// Apply this package to a schema, lowering every declaration through its
    /// section's structural default macro.
    pub fn apply(
        &self,
        schema: &CoreSchema,
        schema_names: &NameTable,
    ) -> Result<Lowering, NomosError> {
        let mut evaluator = Evaluator::new(self, schema_names);
        let items = evaluator.lower_schema(schema)?;
        Ok(Lowering {
            items,
            names: evaluator.into_names(),
        })
    }

    /// Apply this package to a schema through the *enriched* selection: the
    /// per-declaration structural lowering first (the data declarations), then the
    /// generation classes ([`crate::GenerationClass`]) in the package's selection
    /// order — class A, then B, then C, then D, the golden's own document order. The
    /// returned items are the whole ordered run, resolved by one continuous logos
    /// NameTable. A package with an empty selection produces exactly what
    /// [`apply`](Self::apply) does.
    pub fn apply_enriched(
        &self,
        schema: &CoreSchema,
        schema_names: &NameTable,
    ) -> Result<Lowering, NomosError> {
        let mut evaluator = Evaluator::new(self, schema_names);
        let mut items = evaluator.lower_schema(schema)?;
        for class in self.selection() {
            items.extend(evaluator.generate_class(class, schema)?);
        }
        Ok(Lowering {
            items,
            names: evaluator.into_names(),
        })
    }
}

/// A [`Lowering`] IS a [`Converted`] `Vec<CoreItem>`-plus-names: the domain-named
/// result of the lowering and the reusable-trait output of an [`EncodedConversion`] are
/// the same data, so the trait face costs no new representation.
impl From<Lowering> for Converted<Vec<CoreItem>> {
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
/// [`EncodedForm`](structural_codec::EncodedForm) (`CoreSchema`); the target is the
/// lowered logos item set (`Vec<CoreItem>`, the logos EncodedForm); and the continuous
/// NameTable threads the layer, schema indices preserved and logos names appended. No
/// text crosses this path — the signature carries no `&str`/`String`, which is the
/// structural proof that the conversion is a real type conversion, not string
/// manipulation. It delegates to the eponymous [`apply`](MacroPackage::apply).
impl EncodedConversion for MacroPackage {
    type Source = CoreSchema;
    type Target = Vec<CoreItem>;
    type Error = NomosError;

    fn convert(
        &self,
        source: &CoreSchema,
        names: &NameTable,
    ) -> Result<Converted<Vec<CoreItem>>, NomosError> {
        Ok(self.apply(source, names)?.into())
    }
}

/// A produced fragment — what evaluating a result template yields. A structural
/// default yields an item; a recursively-invoked attribute macro yields a vector.
enum Fragment {
    // Boxed: a `CoreItem` now carries whole impl-block/method-body trees, dwarfing
    // the attribute-vector variant, so the box keeps the enum small.
    Item(Box<CoreItem>),
    Attributes(Vec<Attribute>),
}

/// The stateful lowering walk: the package (for macro lookup and literal name
/// remapping), the extended logos NameTable being built, and the active-invocation
/// stack for cycle rejection. Crate-visible so the enriched generation classes
/// ([`crate::generation`]) can append their items into the same continuous NameTable
/// the declaration lowering built.
pub(crate) struct Evaluator<'package> {
    package: &'package MacroPackage,
    pub(crate) names: NameTable,
    active: Vec<MacroIdentity>,
}

impl<'package> Evaluator<'package> {
    fn new(package: &'package MacroPackage, schema_names: &NameTable) -> Self {
        Self {
            package,
            names: NameTable::extend_from(schema_names),
            active: Vec::new(),
        }
    }

    fn into_names(self) -> NameTable {
        self.names
    }

    fn lower_schema(&mut self, schema: &CoreSchema) -> Result<Vec<CoreItem>, NomosError> {
        schema
            .declarations()
            .iter()
            .map(|declaration| self.lower_declaration(declaration))
            .collect()
    }

    fn lower_declaration(&mut self, declaration: &CoreDeclaration) -> Result<CoreItem, NomosError> {
        let value = declaration.value();
        let section = SectionDefault::of_core_type(value);
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
        value: &CoreType,
    ) -> Result<BoundInput, NomosError> {
        let mut bound = BoundInput::new();
        for parameter in &signature.parameters {
            let meta_value = match parameter.meta {
                MetaType::Name => MetaValue::Name(value.identifier()),
                MetaType::Type => match value {
                    CoreType::Newtype(newtype) => MetaValue::Type(newtype.reference().clone()),
                    _ => {
                        return Err(NomosError::MetaShape {
                            meta: MetaType::Type,
                        });
                    }
                },
                MetaType::Fields => match value {
                    CoreType::Struct(structure) => MetaValue::Fields(structure.fields().to_vec()),
                    _ => {
                        return Err(NomosError::MetaShape {
                            meta: MetaType::Fields,
                        });
                    }
                },
                MetaType::Variants => match value {
                    CoreType::Enumeration(enumeration) => {
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
    ) -> Result<CoreItem, NomosError> {
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
    ) -> Result<CoreItem, NomosError> {
        let attributes = self.evaluate_attributes(&template.attributes)?;
        let name = self.evaluate_name(&template.name, bound)?;
        let wrapped = self.evaluate_type(&template.wrapped, bound)?;
        Ok(CoreItem::Newtype(Newtype {
            visibility: template.visibility.clone(),
            attributes,
            name,
            // The structural newtype default emits `pub struct Name(Wrapped);` — a
            // private tuple field. The only `pub`-field tuple struct in the surveyed
            // goldens is the class-D `TraceEvent` declaration, hand-built by the
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
    ) -> Result<CoreItem, NomosError> {
        let attributes = self.evaluate_attributes(&template.attributes)?;
        let name = self.evaluate_name(&template.name, bound)?;
        let fields = self.evaluate_fields(&template.fields, bound)?;
        Ok(CoreItem::Struct(Struct {
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
    ) -> Result<CoreItem, NomosError> {
        let mut attributes = self.evaluate_attributes(&template.attributes)?;
        let name = self.evaluate_name(&template.name, bound)?;
        let variants = self.evaluate_variants(&template.variants, bound)?;
        if variants
            .iter()
            .any(|variant| !matches!(variant.payload, VariantPayload::Unit))
        {
            for attribute in &mut attributes {
                if let Attribute::Derive(group) = attribute {
                    group.paths.retain(|path| match path.resolve(&self.names) {
                        Ok(segments) => segments.as_slice() != [Name::new("Copy")],
                        Err(_) => true,
                    });
                }
            }
        }
        Ok(CoreItem::Enumeration(Enumeration {
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
            Scalar::Literal(identifier) => self.place_literal_name(*identifier),
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
            MetaValue::Name(identifier) => self.transform_name(*identifier, realize.transform),
            _ => Err(NomosError::NameTransformShape),
        }
    }

    /// Apply a name transform, reusing name-table's single home of the derived-name
    /// walk. `Identity` returns the (schema/logos) identifier verbatim; the derived
    /// transforms resolve, walk, and re-intern into the extended table — where
    /// interning dedups, so a derivation that reproduces an existing name returns
    /// its existing identifier (the continuous space).
    fn transform_name(
        &mut self,
        identifier: Identifier,
        transform: NameTransform,
    ) -> Result<Identifier, NomosError> {
        match transform {
            NameTransform::Identity => Ok(identifier),
            NameTransform::FieldName => {
                let derived = self.names.resolve(identifier)?.field_name();
                Ok(self.names.intern(Name::new(derived)))
            }
            NameTransform::Screaming => {
                let derived = self.names.resolve(identifier)?.screaming();
                Ok(self.names.intern(Name::new(derived)))
            }
            NameTransform::PascalCase => {
                let derived = self.names.resolve(identifier)?.pascal_case();
                Ok(self.names.intern(Name::new(derived)))
            }
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
        let group_names = self.derive_group_names(&schema_fields)?;
        let mut out = Vec::with_capacity(schema_fields.len());
        for (field, group_name) in schema_fields.iter().zip(group_names) {
            let type_reference = self.lower_reference(field.reference())?;
            let name = self.field_name(field, group_name, *name_rule)?;
            out.push(Field {
                visibility: visibility.clone(),
                name,
                type_reference,
            });
        }
        Ok(out)
    }

    /// The deterministic Rust field names for an ordered field group — the psyche's
    /// same-typed-field rule (directed work, 2026-07-19: "create a deterministic rule
    /// for structs that contain more than one field with the same type"). Each field's
    /// base name is the `snake_case` of its type, core-schema's single-field
    /// [`CoreReference::derived_field_name`]. When a type names more than one field in
    /// the group, every one of those fields is distinguished by prefixing the ordinal
    /// English word of its position among the same-typed fields — `first_state_digest`,
    /// `second_state_digest`; a type naming exactly one field keeps the bare base name,
    /// which is the degenerate empty-ordinal case, not a separate branch. The result is
    /// a pure function of field position and type: no stored or authored name is read,
    /// adding a later field of another type never moves an earlier field's name, and the
    /// same struct always lowers to the same field names.
    fn derive_group_names(&self, fields: &[CoreField]) -> Result<Vec<Name>, NomosError> {
        let base_names = fields
            .iter()
            .map(|field| field.reference().derived_field_name(&self.names))
            .collect::<Result<Vec<String>, _>>()?;
        let mut totals: BTreeMap<&str, usize> = BTreeMap::new();
        for base in &base_names {
            *totals.entry(base.as_str()).or_default() += 1;
        }
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        let mut names = Vec::with_capacity(base_names.len());
        for base in &base_names {
            let occurrence = {
                let count = seen.entry(base.as_str()).or_default();
                *count += 1;
                *count
            };
            let name = if totals[base.as_str()] > 1 {
                Name::new(format!(
                    "{}_{base}",
                    SameTypeOrdinal(occurrence).ordinal_word()
                ))
            } else {
                Name::new(base.clone())
            };
            names.push(name);
        }
        Ok(names)
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

    /// Select a produced field's name per the field-name rule, given the field's
    /// already-computed group name (its base name, ordinal-disambiguated against
    /// same-typed siblings by [`derive_group_names`](Self::derive_group_names)).
    /// `FieldRuleDispatch` distinguishes an *elided* field (schema name equals the
    /// `field_name` of its type — take the derived group name) from an *explicit* one
    /// (keep the schema name), matching schema's own decode-time Field-rule split. Post
    /// field-name-ban a decoded field is always elided, so the group name — and its
    /// same-typed disambiguation — governs; `PreserveSchema` remains for a
    /// programmatically constructed Core that carries verbatim distinct names.
    fn field_name(
        &mut self,
        field: &CoreField,
        group_name: Name,
        rule: FieldNameRule,
    ) -> Result<Identifier, NomosError> {
        match rule {
            FieldNameRule::PreserveSchema => Ok(field.identifier()),
            FieldNameRule::AlwaysDeriveFromType => Ok(self.names.intern(group_name)),
            // Elided when the schema name equals the reference's derived name,
            // explicit otherwise. The derive-vs-preserve decision is the shared
            // `CoreField::name_is_derivable` predicate in core-schema, so this
            // Nomos-lowering site and schema's own textual codec cannot drift.
            FieldNameRule::FieldRuleDispatch => {
                if field.name_is_derivable(&self.names)? {
                    Ok(self.names.intern(group_name))
                } else {
                    Ok(field.identifier())
                }
            }
        }
    }

    /// Lower a schema type reference into a `CoreLogos` type — dispatched by kind
    /// and projection, never by a head string. Exhaustive over `CoreReference`.
    /// Crate-visible: the enriched generation classes lower newtype-wrapped and
    /// variant-payload references through the same single home.
    pub(crate) fn lower_reference(
        &mut self,
        reference: &CoreReference,
    ) -> Result<TypeReference, NomosError> {
        match reference {
            CoreReference::Integer => Ok(TypeReference::Path(self.leaf_path("Integer"))),
            CoreReference::String => Ok(TypeReference::Path(self.leaf_path("String"))),
            CoreReference::Boolean => Ok(TypeReference::Path(self.leaf_path("Boolean"))),
            CoreReference::Bytes => Ok(TypeReference::Path(self.leaf_path("Bytes"))),
            CoreReference::Plain(identifier) => Ok(TypeReference::Path(PathNode {
                segments: vec![*identifier],
            })),
            CoreReference::SingleTypeApplication {
                projection,
                argument,
            } => {
                let head = self.single_projection_head(projection);
                let argument = self.lower_reference(argument)?;
                Ok(TypeReference::Application(TypeApplication {
                    head: self.leaf_path(head),
                    arguments: vec![argument],
                }))
            }
            CoreReference::MultiTypeApplication {
                projection,
                arguments,
            } => {
                let head = self.multi_projection_head(projection);
                let arguments = arguments
                    .iter()
                    .map(|argument| self.lower_reference(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeReference::Application(TypeApplication {
                    head: self.leaf_path(head),
                    arguments,
                }))
            }
            CoreReference::ValueApplication { .. } => Err(NomosError::UnsupportedReference(
                "a byte-length value application has no CoreLogos type-argument home",
            )),
        }
    }

    /// The Rust head spelling of a single-argument type projection. Vector and
    /// Optional are verified against the goldens (`Vec<_>`, `Option<_>`); ScopeOf
    /// is a flagged best guess (absent from the surveyed corpus).
    fn single_projection_head(
        &self,
        projection: &core_schema::SingleTypeReferenceProjection,
    ) -> &'static str {
        use core_schema::SingleTypeReferenceProjection::{Optional, ScopeOf, Vector};
        match projection {
            Vector => "Vec",
            Optional => "Option",
            ScopeOf => "ScopeOf",
        }
    }

    fn multi_projection_head(
        &self,
        projection: &core_schema::MultiTypeReferenceProjection,
    ) -> &'static str {
        use core_schema::MultiTypeReferenceProjection::Map;
        match projection {
            Map => "Map",
        }
    }

    /// Intern a logos-only leaf or head name into the extended table, returning a
    /// single-segment path. Interning dedups, so a leaf name that a schema
    /// identifier already carries reuses that identifier.
    fn leaf_path(&mut self, text: &str) -> PathNode {
        PathNode {
            segments: vec![self.names.intern(Name::new(text))],
        }
    }

    /// Re-intern a template-literal name (authored against the package's NameTable)
    /// into the extended logos table. This is the runtime realization of the one
    /// continuous identifier space: the package's names append to the schema
    /// allocation, deduped.
    fn place_literal_name(&mut self, identifier: Identifier) -> Result<Identifier, NomosError> {
        let name = self.package.names().resolve(identifier)?.clone();
        Ok(self.names.intern(name))
    }

    fn remap_path(&mut self, path: &PathNode) -> Result<PathNode, NomosError> {
        let mut segments = Vec::with_capacity(path.segments.len());
        for segment in &path.segments {
            segments.push(self.place_literal_name(*segment)?);
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
                    Some(lifetime) => Some(self.place_literal_name(lifetime)?),
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
            TypeReference::Lifetime(lifetime) => {
                Ok(TypeReference::Lifetime(self.place_literal_name(*lifetime)?))
            }
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
                self.place_literal_name(*identifier)?,
            )),
        }
    }

    fn remap_field(&mut self, field: &Field) -> Result<Field, NomosError> {
        Ok(Field {
            visibility: field.visibility.clone(),
            name: self.place_literal_name(field.name)?,
            type_reference: self.remap_type_reference(&field.type_reference)?,
        })
    }
}

/// A one-based position within a group of same-typed struct fields. Its ordinal
/// English word is how the deterministic same-typed-field rule tells such fields
/// apart when lowering to a target that needs distinct field identifiers (Rust).
/// Position is the only input, so the word is a total, stable function of it.
struct SameTypeOrdinal(usize);

impl SameTypeOrdinal {
    /// The ordinal English word for this position, in `snake_case`: `first`,
    /// `second`, `third`, `twenty_first`, `one_hundredth`. Spelled by the cardinal
    /// words of the position with the final word ordinalized, so every position —
    /// however large — has a full-English name and never falls back to a numeral.
    fn ordinal_word(&self) -> String {
        let cardinal = Self::cardinal_word(self.0);
        let (prefix, last) = match cardinal.rsplit_once('_') {
            Some((prefix, last)) => (Some(prefix), last),
            None => (None, cardinal.as_str()),
        };
        let ordinal_last = match last {
            "one" => "first".to_owned(),
            "two" => "second".to_owned(),
            "three" => "third".to_owned(),
            "five" => "fifth".to_owned(),
            "eight" => "eighth".to_owned(),
            "nine" => "ninth".to_owned(),
            "twelve" => "twelfth".to_owned(),
            word if word.ends_with('y') => format!("{}ieth", &word[..word.len() - 1]),
            word => format!("{word}th"),
        };
        match prefix {
            Some(prefix) => format!("{prefix}_{ordinal_last}"),
            None => ordinal_last,
        }
    }

    /// The cardinal English words for a count, in `snake_case` (`twenty_three`,
    /// `one_hundred_five`). Total over `usize` by recursing through the scale words.
    fn cardinal_word(count: usize) -> String {
        const ONES: [&str; 20] = [
            "zero",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "seventeen",
            "eighteen",
            "nineteen",
        ];
        const TENS: [&str; 10] = [
            "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];
        const SCALES: [(usize, &str); 5] = [
            (1_000_000_000_000, "trillion"),
            (1_000_000_000, "billion"),
            (1_000_000, "million"),
            (1_000, "thousand"),
            (100, "hundred"),
        ];
        if count < 20 {
            return ONES[count].to_owned();
        }
        if count < 100 {
            let tens = TENS[count / 10];
            return if count % 10 == 0 {
                tens.to_owned()
            } else {
                format!("{tens}_{}", ONES[count % 10])
            };
        }
        for (value, word) in SCALES {
            if count >= value {
                let high = Self::cardinal_word(count / value);
                let remainder = count % value;
                return if remainder == 0 {
                    format!("{high}_{word}")
                } else {
                    format!("{high}_{word}_{}", Self::cardinal_word(remainder))
                };
            }
        }
        unreachable!("counts below 100 are handled above")
    }
}

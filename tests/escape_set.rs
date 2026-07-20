//! Closed `$x` / `$@xs` Nomos escape-set coverage.
//!
//! These tests exercise definition checking before evaluation: malformed template
//! data cannot reach a schema expansion path.

use core_logos::{EncodedItem, Field, Generics, TypeReference, Visibility};
use core_nomos::{
    BindingRef, EnumerationTemplate, Escape, EscapeKind, InputParameter, InputSignature,
    ItemTemplate, MacroDefinition, MacroIdentity, MacroKind, MacroPackage, MetaType,
    NewtypeTemplate, NomosError, PackageRevision, Realize, ResultTemplate, Scalar, Sequence,
    SequenceItem, Splice, SpliceElement, StructTemplate, TemplatePosition,
};
use core_schema::{
    EncodedDeclaration, EncodedField, EncodedReference, EncodedSchema, EncodedStruct, EncodedType,
};
use name_table::{Identifier, IdentifierNamespace, Name, NameTable};

/// A test-only package authoring one binding and one template definition.
fn package_with_definition(
    meta: MetaType,
    template: impl FnOnce(Identifier) -> ResultTemplate,
) -> MacroPackage {
    let mut package = MacroPackage::new(PackageRevision(1));
    let binding = package.author_name("value");
    let macro_name = package.author_name("Check");
    package.register(MacroDefinition {
        name: macro_name,
        kind: MacroKind::Named,
        input: InputSignature {
            parameters: vec![InputParameter { binding, meta }],
        },
        template: template(binding),
    });
    package
}

/// A fixed-arity newtype template that exposes the wrapped type boundary.
fn newtype_with_wrapped(wrapped: Scalar<TypeReference>) -> ResultTemplate {
    ResultTemplate::Item(ItemTemplate::Newtype(NewtypeTemplate {
        visibility: Visibility::Public,
        attributes: Sequence { items: Vec::new() },
        name: Scalar::Literal(Identifier::Nomos(0)),
        wrapped,
    }))
}

/// A record template whose fields position is a vector-element boundary.
fn struct_with_fields(fields: Sequence<Field>) -> ResultTemplate {
    ResultTemplate::Item(ItemTemplate::Struct(StructTemplate {
        visibility: Visibility::Public,
        attributes: Sequence { items: Vec::new() },
        name: Scalar::Literal(Identifier::Nomos(0)),
        generics: Generics::none(),
        fields,
    }))
}

/// An enum template whose variants position is a vector-element boundary.
fn enumeration_with_variants(variants: Sequence<core_logos::Variant>) -> ResultTemplate {
    ResultTemplate::Item(ItemTemplate::Enumeration(EnumerationTemplate {
        visibility: Visibility::Public,
        attributes: Sequence { items: Vec::new() },
        name: Scalar::Literal(Identifier::Nomos(0)),
        generics: Generics::none(),
        variants,
    }))
}

/// One field-vector splice using the only field projection that the typed grammar
/// admits at the record-vector boundary.
fn field_splice(binding: Identifier) -> Escape {
    Escape::Splice(Splice {
        binding: BindingRef::Input(binding),
        element: SpliceElement::Field {
            visibility: Visibility::Public,
            name_rule: core_nomos::FieldNameRule::FieldRuleDispatch,
        },
    })
}

#[test]
fn escape_spellings_are_exact_and_the_escape_enum_is_exhaustive() {
    assert_eq!(EscapeKind::Realize.spelling(), "$x");
    assert_eq!(EscapeKind::Splice.spelling(), "$@xs");

    let kinds = [
        Escape::Realize(Realize {
            binding: BindingRef::Input(Identifier::Nomos(0)),
        })
        .kind(),
        Escape::Splice(Splice {
            binding: BindingRef::Input(Identifier::Nomos(0)),
            element: SpliceElement::Variant,
        })
        .kind(),
    ];
    assert_eq!(kinds, [EscapeKind::Realize, EscapeKind::Splice]);
}

#[test]
fn splice_flattens_empty_and_multiple_field_vectors() {
    for field_count in [0, 2] {
        let mut names = NameTable::new(IdentifierNamespace::Schema);
        let structure = names.intern(Name::new("Entry")).expect("structure name");
        let fields = (0..field_count)
            .map(|index| {
                let name = names
                    .intern(Name::new(format!("field_{index}")))
                    .expect("field name");
                EncodedField::new(name, EncodedReference::Integer)
            })
            .collect();
        let schema = EncodedSchema::new(vec![EncodedDeclaration::public(EncodedType::Struct(
            EncodedStruct::new(structure, fields),
        ))]);
        let lowering = MacroPackage::plain_fixture()
            .apply(&schema, &names)
            .expect("typed field-vector splice lowers");
        let EncodedItem::Struct(item) = &lowering.items[0] else {
            panic!("struct input must lower to struct output");
        };
        assert_eq!(item.fields.len(), field_count);
    }
}

#[test]
fn wrong_vector_element_type_is_rejected_before_expansion() {
    let package = package_with_definition(MetaType::Variants, |binding| {
        struct_with_fields(Sequence::of(SequenceItem::Escape(field_splice(binding))))
    });
    assert!(matches!(
        package.check(),
        Err(NomosError::EscapeBinding {
            escape: EscapeKind::Splice,
            expected: MetaType::Fields,
            actual: MetaType::Variants,
            ..
        })
    ));
}

#[test]
fn non_vector_splice_input_is_rejected_before_expansion() {
    let package = package_with_definition(MetaType::Name, |binding| {
        struct_with_fields(Sequence::of(SequenceItem::Escape(field_splice(binding))))
    });
    assert!(matches!(
        package.check(),
        Err(NomosError::EscapeBinding {
            escape: EscapeKind::Splice,
            expected: MetaType::Fields,
            actual: MetaType::Name,
            ..
        })
    ));
}

#[test]
fn fixed_record_and_enum_variant_positions_reject_wrong_escape_forms() {
    let record_package = package_with_definition(MetaType::Fields, |binding| {
        newtype_with_wrapped(Scalar::Escape(field_splice(binding)))
    });
    assert!(matches!(
        record_package.check(),
        Err(NomosError::EscapePlacement {
            escape: EscapeKind::Splice,
            position: TemplatePosition::Type,
        })
    ));

    let enum_package = package_with_definition(MetaType::Variants, |binding| {
        enumeration_with_variants(Sequence::of(SequenceItem::Escape(Escape::Realize(
            Realize {
                binding: BindingRef::Input(binding),
            },
        ))))
    });
    assert!(matches!(
        enum_package.check(),
        Err(NomosError::EscapePlacement {
            escape: EscapeKind::Realize,
            position: TemplatePosition::VariantElement,
        })
    ));
}

#[test]
fn recursive_invocation_is_not_an_escape_and_is_rejected_outside_attributes() {
    let package = package_with_definition(MetaType::Fields, |_| {
        struct_with_fields(Sequence::of(SequenceItem::RecursiveInvoke(
            MacroIdentity::new(0),
        )))
    });
    assert!(matches!(
        package.check(),
        Err(NomosError::RecursiveInvocation(
            TemplatePosition::FieldElement
        ))
    ));
}

#[test]
fn invalid_definition_fails_before_any_schema_expansion() {
    let package = package_with_definition(MetaType::Fields, |binding| {
        newtype_with_wrapped(Scalar::Escape(field_splice(binding)))
    });
    let schema = EncodedSchema::new(Vec::new());
    let names = NameTable::new(IdentifierNamespace::Schema);
    assert!(matches!(
        package.apply(&schema, &names),
        Err(NomosError::EscapePlacement {
            escape: EscapeKind::Splice,
            position: TemplatePosition::Type,
        })
    ));
}

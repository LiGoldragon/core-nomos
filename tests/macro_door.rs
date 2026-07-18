//! The Nomos raw-NOTA macro door (epic primary-56d1.41): a macro definition decodes
//! from raw positional NOTA text into a `MacroDefinition` value EQUAL to the one built
//! in Rust — proven by equality assertion — and re-encodes to the same canonical text.
//!
//! The strict invariant holds: decode and encode both run through the ONE trusted
//! structural-codec evaluator walking a sealed, disjointness-proved structuretree plus
//! the nametree. There is no bespoke per-type parse or print path; every spelling is a
//! `StructuralForm` in the sealed table. (This door uses `StructuralEvaluator`
//! directly, the exact mechanism the shared `TextualForm` trait wraps; it cannot yet
//! adopt the trait because that lives in a newer structural-codec rev whose adoption
//! here is blocked by the `textual-rust` pin — epic .44, gated out of this wave.)
//!
//! The RAW door spells every construct as the positional data record it is (the ruled
//! Protos laws): a struct is a `{ … }` record, an enum variant carrying a payload is
//! `Head.payload`, a vector is `[ … ]`, a name is a camelCase atom, a type/kind is a
//! PascalCase atom. Escapes are ordinary data records — a `Realize` is
//! `Realize.{ Input.name Identity }`, an `Invoke` is `Invoke.99` — with NO sigil; the
//! `$` / `<<>>` pretty surface is the deferred TextualNomos form (.42), not this door.
//!
//! Table-seal site: `TextualNomos::build` -> `AddressedStructuralTable::seal` (proved
//! disjoint via `validate_disjoint`). Evaluator entry: `TextualNomos::decode_macro` /
//! `encode_macro` -> `StructuralEvaluator::decode` / `encode`.

use core_logos::Visibility;
use core_nomos::{
    BindingRef, Escape, InputParameter, InputSignature, ItemTemplate, MacroDefinition,
    MacroIdentity, MacroKind, MetaType, NameTransform, NewtypeTemplate, Realize, ResultTemplate,
    Scalar, SectionDefault, Sequence, SequenceItem,
};
use name_table::{Identifier, Name, NameTable};
use raw_discovery::{Delimiter, Recognizer};
use structural_codec::ids::{
    CoreConstructorId, PositionalSignature, ScopedCoreTypeId, StructuralRevision,
};
use structural_codec::table::{
    AddressedStructuralTable, CoreLayoutIdentity, RawProfileIdentity, TableIdentityPayload,
};
use structural_codec::value::{ScalarValue, StructuralValue};
use structural_codec::{
    AtomForm, CanonicalText, CaseExpectation, ConstructorCodec, LeafForm, ScalarLeaf, SequenceForm,
    StructuralEntry, StructuralEvaluator, StructuralForm,
};

// The macro-door structuretree's type ids (locals in the fixture universe namespace).
const MACRO_DEFINITION: ScopedCoreTypeId = ScopedCoreTypeId::fixture(1);
const MACRO_KIND: ScopedCoreTypeId = ScopedCoreTypeId::fixture(2);
const SECTION_DEFAULT: ScopedCoreTypeId = ScopedCoreTypeId::fixture(3);
const INPUT_PARAMETER: ScopedCoreTypeId = ScopedCoreTypeId::fixture(4);
const META_TYPE: ScopedCoreTypeId = ScopedCoreTypeId::fixture(5);
const RESULT_TEMPLATE: ScopedCoreTypeId = ScopedCoreTypeId::fixture(6);
const ITEM_TEMPLATE: ScopedCoreTypeId = ScopedCoreTypeId::fixture(7);
const NEWTYPE_TEMPLATE: ScopedCoreTypeId = ScopedCoreTypeId::fixture(8);
const SEQUENCE_ITEM: ScopedCoreTypeId = ScopedCoreTypeId::fixture(9);
const ESCAPE: ScopedCoreTypeId = ScopedCoreTypeId::fixture(10);
const REALIZE: ScopedCoreTypeId = ScopedCoreTypeId::fixture(11);
const BINDING_REF: ScopedCoreTypeId = ScopedCoreTypeId::fixture(12);
const NAME_TRANSFORM: ScopedCoreTypeId = ScopedCoreTypeId::fixture(13);
const VISIBILITY: ScopedCoreTypeId = ScopedCoreTypeId::fixture(14);
const SCALAR: ScopedCoreTypeId = ScopedCoreTypeId::fixture(15);

// Constructor indices for the enum entries.
const KIND_NAMED: u32 = 0;
const KIND_STRUCTURAL: u32 = 1;
const SECTION_NEWTYPE: u32 = 0;
const SECTION_STRUCT: u32 = 1;
const SECTION_ENUMERATION: u32 = 2;
const META_NAME: u32 = 0;
const META_TYPE_KIND: u32 = 1;
const META_FIELDS: u32 = 2;
const META_VARIANTS: u32 = 3;
const ESCAPE_REALIZE: u32 = 0;
const ESCAPE_INVOKE: u32 = 1;
const TRANSFORM_IDENTITY: u32 = 0;
const TRANSFORM_FIELD_NAME: u32 = 1;
const TRANSFORM_SCREAMING: u32 = 2;
const TRANSFORM_PASCAL: u32 = 3;

/// The keyword lexicon: every structural keyword the `Literal` forms match on decode
/// and resolve on encode.
struct Lexicon {
    names: NameTable,
    named: Identifier,
    structural: Identifier,
    section_newtype: Identifier,
    section_struct: Identifier,
    section_enumeration: Identifier,
    meta_name: Identifier,
    meta_type: Identifier,
    meta_fields: Identifier,
    meta_variants: Identifier,
    item: Identifier,
    template_newtype: Identifier,
    sequence_escape: Identifier,
    escape_realize: Identifier,
    escape_invoke: Identifier,
    binding_input: Identifier,
    transform_identity: Identifier,
    transform_field_name: Identifier,
    transform_screaming: Identifier,
    transform_pascal: Identifier,
    visibility_public: Identifier,
}

impl Lexicon {
    fn build() -> Self {
        let mut names = NameTable::new();
        let mut keyword = |text: &str| names.intern(Name::new(text));
        let named = keyword("Named");
        let structural = keyword("Structural");
        let section_newtype = keyword("Newtype");
        let section_struct = keyword("Struct");
        let section_enumeration = keyword("Enumeration");
        let meta_name = keyword("Name");
        let meta_type = keyword("Type");
        let meta_fields = keyword("Fields");
        let meta_variants = keyword("Variants");
        let item = keyword("Item");
        // The ItemTemplate::Newtype tag reuses the "Newtype" keyword atom.
        let template_newtype = section_newtype;
        let sequence_escape = keyword("Escape");
        let escape_realize = keyword("Realize");
        let escape_invoke = keyword("Invoke");
        let binding_input = keyword("Input");
        let transform_identity = keyword("Identity");
        let transform_field_name = keyword("FieldName");
        let transform_screaming = keyword("Screaming");
        let transform_pascal = keyword("PascalCase");
        let visibility_public = keyword("Public");
        Self {
            names,
            named,
            structural,
            section_newtype,
            section_struct,
            section_enumeration,
            meta_name,
            meta_type,
            meta_fields,
            meta_variants,
            item,
            template_newtype,
            sequence_escape,
            escape_realize,
            escape_invoke,
            binding_input,
            transform_identity,
            transform_field_name,
            transform_screaming,
            transform_pascal,
            visibility_public,
        }
    }
}

/// One raw textual door onto Nomos macro definitions: the sealed structuretree plus
/// the keyword lexicon.
struct TextualNomos {
    table: AddressedStructuralTable,
    lexicon: Lexicon,
}

impl TextualNomos {
    fn build() -> Self {
        let lexicon = Lexicon::build();
        let entries = vec![
            Self::macro_definition_entry(),
            Self::macro_kind_entry(&lexicon),
            Self::section_default_entry(&lexicon),
            Self::input_parameter_entry(),
            Self::meta_type_entry(&lexicon),
            Self::result_template_entry(&lexicon),
            Self::item_template_entry(&lexicon),
            Self::newtype_template_entry(),
            Self::sequence_item_entry(&lexicon),
            Self::escape_entry(&lexicon),
            Self::realize_entry(),
            Self::binding_ref_entry(&lexicon),
            Self::name_transform_entry(&lexicon),
            Self::visibility_entry(&lexicon),
            Self::scalar_entry(&lexicon),
        ];
        let payload = TableIdentityPayload {
            core_universe: structural_codec::ids::FIXTURE_UNIVERSE,
            core_layout_identity: CoreLayoutIdentity([9u8; 32]),
            raw_profile_identity: RawProfileIdentity([1u8; 32]),
            committed_lexicon: b"core-nomos-macro-door".to_vec(),
            leaf_codec_contracts: Vec::new(),
            entries: entries
                .into_iter()
                .map(|entry| (entry.core_type, entry))
                .collect(),
        };
        let table = AddressedStructuralTable::seal(StructuralRevision::new(1), payload)
            .expect("seal the macro-door structuretree");
        table
            .validate_disjoint()
            .expect("every decode alternative is provably disjoint");
        Self { table, lexicon }
    }

    fn evaluator(&self) -> StructuralEvaluator<'_> {
        StructuralEvaluator::with_lexicon(&self.table, &self.lexicon.names)
    }

    fn decode_macro(&self, text: &str, names: &mut NameTable) -> MacroDefinition {
        let document = Recognizer::standard().recognize(text).expect("recognize");
        let block = document.root_object_at(0).expect("one root object");
        let mirror = self
            .evaluator()
            .decode(MACRO_DEFINITION, block, names)
            .expect("decode macro definition");
        self.reify_definition(&mirror)
    }

    fn encode_macro(&self, definition: &MacroDefinition, names: &mut NameTable) -> String {
        let mirror = self.reflect_definition(definition, names);
        let block = self
            .evaluator()
            .encode(MACRO_DEFINITION, &mirror, names)
            .expect("encode macro definition");
        block.canonical_text()
    }

    // ===== structuretree authoring =====

    fn solo(core_type: ScopedCoreTypeId, form: StructuralForm) -> StructuralEntry {
        StructuralEntry::new(core_type, vec![Self::codec(core_type, 0, form)])
    }

    fn codec(core_type: ScopedCoreTypeId, index: u32, form: StructuralForm) -> ConstructorCodec {
        ConstructorCodec::new(
            CoreConstructorId::new(core_type, index),
            vec![form.clone()],
            form,
            PositionalSignature::default(),
        )
    }

    fn keyword_application(keyword: Identifier, payload: StructuralForm) -> StructuralForm {
        StructuralForm::application(StructuralForm::Literal(keyword), payload)
    }

    fn brace(fields: Vec<StructuralForm>) -> StructuralForm {
        StructuralForm::Delimited {
            delimiter: Delimiter::Brace,
            sequence: SequenceForm::Product(fields),
        }
    }

    fn vector(element: StructuralForm) -> StructuralForm {
        StructuralForm::Delimited {
            delimiter: Delimiter::SquareBracket,
            sequence: SequenceForm::zero_or_more(element),
        }
    }

    fn camel_atom() -> StructuralForm {
        StructuralForm::Atom(AtomForm::with_case(CaseExpectation::CamelCase))
    }

    /// `{ <name> <kind> [<params>] <template> }`
    fn macro_definition_entry() -> StructuralEntry {
        Self::solo(
            MACRO_DEFINITION,
            Self::brace(vec![
                StructuralForm::pascal_atom(),
                StructuralForm::Delegate(MACRO_KIND),
                Self::vector(StructuralForm::Delegate(INPUT_PARAMETER)),
                StructuralForm::Delegate(RESULT_TEMPLATE),
            ]),
        )
    }

    /// `Named` | `Structural.<section>`
    fn macro_kind_entry(lexicon: &Lexicon) -> StructuralEntry {
        StructuralEntry::new(
            MACRO_KIND,
            vec![
                Self::codec(
                    MACRO_KIND,
                    KIND_NAMED,
                    StructuralForm::Literal(lexicon.named),
                ),
                Self::codec(
                    MACRO_KIND,
                    KIND_STRUCTURAL,
                    Self::keyword_application(
                        lexicon.structural,
                        StructuralForm::Delegate(SECTION_DEFAULT),
                    ),
                ),
            ],
        )
    }

    fn section_default_entry(lexicon: &Lexicon) -> StructuralEntry {
        StructuralEntry::new(
            SECTION_DEFAULT,
            vec![
                Self::codec(
                    SECTION_DEFAULT,
                    SECTION_NEWTYPE,
                    StructuralForm::Literal(lexicon.section_newtype),
                ),
                Self::codec(
                    SECTION_DEFAULT,
                    SECTION_STRUCT,
                    StructuralForm::Literal(lexicon.section_struct),
                ),
                Self::codec(
                    SECTION_DEFAULT,
                    SECTION_ENUMERATION,
                    StructuralForm::Literal(lexicon.section_enumeration),
                ),
            ],
        )
    }

    /// `{ <binding> <meta> }`
    fn input_parameter_entry() -> StructuralEntry {
        Self::solo(
            INPUT_PARAMETER,
            Self::brace(vec![
                Self::camel_atom(),
                StructuralForm::Delegate(META_TYPE),
            ]),
        )
    }

    fn meta_type_entry(lexicon: &Lexicon) -> StructuralEntry {
        StructuralEntry::new(
            META_TYPE,
            vec![
                Self::codec(
                    META_TYPE,
                    META_NAME,
                    StructuralForm::Literal(lexicon.meta_name),
                ),
                Self::codec(
                    META_TYPE,
                    META_TYPE_KIND,
                    StructuralForm::Literal(lexicon.meta_type),
                ),
                Self::codec(
                    META_TYPE,
                    META_FIELDS,
                    StructuralForm::Literal(lexicon.meta_fields),
                ),
                Self::codec(
                    META_TYPE,
                    META_VARIANTS,
                    StructuralForm::Literal(lexicon.meta_variants),
                ),
            ],
        )
    }

    /// `Item.<item-template>` (the Attributes result is deferred).
    fn result_template_entry(lexicon: &Lexicon) -> StructuralEntry {
        Self::solo(
            RESULT_TEMPLATE,
            Self::keyword_application(lexicon.item, StructuralForm::Delegate(ITEM_TEMPLATE)),
        )
    }

    /// `Newtype.<newtype-template>` (Struct / Enumeration templates deferred).
    fn item_template_entry(lexicon: &Lexicon) -> StructuralEntry {
        Self::solo(
            ITEM_TEMPLATE,
            Self::keyword_application(
                lexicon.template_newtype,
                StructuralForm::Delegate(NEWTYPE_TEMPLATE),
            ),
        )
    }

    /// `{ <visibility> [<attrs>] <name-scalar> <wrapped-scalar> }`
    fn newtype_template_entry() -> StructuralEntry {
        Self::solo(
            NEWTYPE_TEMPLATE,
            Self::brace(vec![
                StructuralForm::Delegate(VISIBILITY),
                Self::vector(StructuralForm::Delegate(SEQUENCE_ITEM)),
                StructuralForm::Delegate(SCALAR),
                StructuralForm::Delegate(SCALAR),
            ]),
        )
    }

    /// `Escape.<escape>` — a `Scalar` position's escape (the `Literal` scalar surface
    /// is deferred with the rest of the pretty form).
    fn scalar_entry(lexicon: &Lexicon) -> StructuralEntry {
        Self::solo(
            SCALAR,
            Self::keyword_application(lexicon.sequence_escape, StructuralForm::Delegate(ESCAPE)),
        )
    }

    /// `Escape.<escape>` (the `Literal` element is deferred).
    fn sequence_item_entry(lexicon: &Lexicon) -> StructuralEntry {
        Self::solo(
            SEQUENCE_ITEM,
            Self::keyword_application(lexicon.sequence_escape, StructuralForm::Delegate(ESCAPE)),
        )
    }

    /// `Realize.<realize>` | `Invoke.<identity>` (Splice deferred).
    fn escape_entry(lexicon: &Lexicon) -> StructuralEntry {
        StructuralEntry::new(
            ESCAPE,
            vec![
                Self::codec(
                    ESCAPE,
                    ESCAPE_REALIZE,
                    Self::keyword_application(
                        lexicon.escape_realize,
                        StructuralForm::Delegate(REALIZE),
                    ),
                ),
                Self::codec(
                    ESCAPE,
                    ESCAPE_INVOKE,
                    Self::keyword_application(
                        lexicon.escape_invoke,
                        StructuralForm::Leaf(LeafForm::scalar(ScalarLeaf::Integer)),
                    ),
                ),
            ],
        )
    }

    /// `{ <binding-ref> <transform> }`
    fn realize_entry() -> StructuralEntry {
        Self::solo(
            REALIZE,
            Self::brace(vec![
                StructuralForm::Delegate(BINDING_REF),
                StructuralForm::Delegate(NAME_TRANSFORM),
            ]),
        )
    }

    /// `Input.<binding-name>`
    fn binding_ref_entry(lexicon: &Lexicon) -> StructuralEntry {
        Self::solo(
            BINDING_REF,
            Self::keyword_application(lexicon.binding_input, Self::camel_atom()),
        )
    }

    fn name_transform_entry(lexicon: &Lexicon) -> StructuralEntry {
        StructuralEntry::new(
            NAME_TRANSFORM,
            vec![
                Self::codec(
                    NAME_TRANSFORM,
                    TRANSFORM_IDENTITY,
                    StructuralForm::Literal(lexicon.transform_identity),
                ),
                Self::codec(
                    NAME_TRANSFORM,
                    TRANSFORM_FIELD_NAME,
                    StructuralForm::Literal(lexicon.transform_field_name),
                ),
                Self::codec(
                    NAME_TRANSFORM,
                    TRANSFORM_SCREAMING,
                    StructuralForm::Literal(lexicon.transform_screaming),
                ),
                Self::codec(
                    NAME_TRANSFORM,
                    TRANSFORM_PASCAL,
                    StructuralForm::Literal(lexicon.transform_pascal),
                ),
            ],
        )
    }

    /// `Public` (the golden macro's only visibility; other variants deferred).
    fn visibility_entry(lexicon: &Lexicon) -> StructuralEntry {
        Self::solo(
            VISIBILITY,
            StructuralForm::Literal(lexicon.visibility_public),
        )
    }

    // ===== reify: mirror -> MacroDefinition =====

    fn reify_definition(&self, mirror: &StructuralValue) -> MacroDefinition {
        let (_constructor, body) = Self::chosen(mirror);
        let fields = Self::delimited(body);
        let [name, kind, parameters, template] = fields.as_slice() else {
            panic!("macro definition arity");
        };
        MacroDefinition {
            name: Self::atom(name),
            kind: Self::reify_kind(kind),
            input: InputSignature {
                parameters: Self::delimited(parameters)
                    .iter()
                    .map(|parameter| Self::reify_parameter(Self::delegated(parameter)))
                    .collect(),
            },
            template: self.reify_template(Self::delegated(template)),
        }
    }

    fn reify_kind(mirror: &StructuralValue) -> MacroKind {
        let (constructor, payload) = Self::chosen(Self::delegated(mirror));
        match constructor {
            KIND_NAMED => MacroKind::Named,
            KIND_STRUCTURAL => {
                MacroKind::Structural(Self::reify_section(Self::application_body(payload)))
            }
            other => panic!("macro kind constructor {other}"),
        }
    }

    fn reify_section(mirror: &StructuralValue) -> SectionDefault {
        let (constructor, _) = Self::chosen(Self::delegated(mirror));
        match constructor {
            SECTION_NEWTYPE => SectionDefault::Newtype,
            SECTION_STRUCT => SectionDefault::Struct,
            SECTION_ENUMERATION => SectionDefault::Enumeration,
            other => panic!("section default constructor {other}"),
        }
    }

    fn reify_parameter(mirror: &StructuralValue) -> InputParameter {
        let (_constructor, body) = Self::chosen(mirror);
        let fields = Self::delimited(body);
        let [binding, meta] = fields.as_slice() else {
            panic!("input parameter arity");
        };
        InputParameter {
            binding: Self::atom(binding),
            meta: Self::reify_meta(Self::delegated(meta)),
        }
    }

    fn reify_meta(mirror: &StructuralValue) -> MetaType {
        let (constructor, _) = Self::chosen(mirror);
        match constructor {
            META_NAME => MetaType::Name,
            META_TYPE_KIND => MetaType::Type,
            META_FIELDS => MetaType::Fields,
            META_VARIANTS => MetaType::Variants,
            other => panic!("meta type constructor {other}"),
        }
    }

    fn reify_template(&self, mirror: &StructuralValue) -> ResultTemplate {
        // Item.<item-template>
        let (_item, item_payload) = Self::chosen(mirror);
        let item_template = Self::delegated(Self::application_body(item_payload));
        // Newtype.<newtype-template>
        let (_newtype, newtype_payload) = Self::chosen(item_template);
        let newtype_template = Self::delegated(Self::application_body(newtype_payload));
        let (_ctor, body) = Self::chosen(newtype_template);
        let fields = Self::delimited(body);
        let [visibility, attributes, name, wrapped] = fields.as_slice() else {
            panic!("newtype template arity");
        };
        ResultTemplate::Item(ItemTemplate::Newtype(NewtypeTemplate {
            visibility: Self::reify_visibility(Self::delegated(visibility)),
            attributes: Sequence {
                items: Self::delimited(attributes)
                    .iter()
                    .map(|item| SequenceItem::Escape(Self::reify_escape(Self::delegated(item))))
                    .collect(),
            },
            name: Scalar::Escape(Self::reify_escape(Self::delegated(name))),
            wrapped: Scalar::Escape(Self::reify_escape(Self::delegated(wrapped))),
        }))
    }

    fn reify_visibility(mirror: &StructuralValue) -> Visibility {
        let (_constructor, _) = Self::chosen(mirror);
        Visibility::Public
    }

    fn reify_escape(mirror: &StructuralValue) -> Escape {
        // Sequence/Scalar Escape wrapper: `Escape.<escape>`.
        let (_escape_tag, escape_payload) = Self::chosen(mirror);
        let inner = Self::delegated(Self::application_body(escape_payload));
        let (constructor, payload) = Self::chosen(inner);
        match constructor {
            ESCAPE_REALIZE => Escape::Realize(Self::reify_realize(Self::delegated(
                Self::application_body(payload),
            ))),
            ESCAPE_INVOKE => Escape::Invoke(MacroIdentity::new(Self::scalar_integer(
                Self::application_body(payload),
            ) as u32)),
            other => panic!("escape constructor {other}"),
        }
    }

    fn reify_realize(mirror: &StructuralValue) -> Realize {
        let (_constructor, body) = Self::chosen(mirror);
        let fields = Self::delimited(body);
        let [binding, transform] = fields.as_slice() else {
            panic!("realize arity");
        };
        Realize {
            binding: Self::reify_binding(Self::delegated(binding)),
            transform: Self::reify_transform(Self::delegated(transform)),
        }
    }

    fn reify_binding(mirror: &StructuralValue) -> BindingRef {
        let (_input, payload) = Self::chosen(mirror);
        BindingRef::Input(Self::atom(Self::application_body(payload)))
    }

    fn reify_transform(mirror: &StructuralValue) -> NameTransform {
        let (constructor, _) = Self::chosen(mirror);
        match constructor {
            TRANSFORM_IDENTITY => NameTransform::Identity,
            TRANSFORM_FIELD_NAME => NameTransform::FieldName,
            TRANSFORM_SCREAMING => NameTransform::Screaming,
            TRANSFORM_PASCAL => NameTransform::PascalCase,
            other => panic!("name transform constructor {other}"),
        }
    }

    // ===== reflect: MacroDefinition -> mirror =====

    fn reflect_definition(
        &self,
        definition: &MacroDefinition,
        names: &mut NameTable,
    ) -> StructuralValue {
        let parameters = definition
            .input
            .parameters
            .iter()
            .map(|parameter| {
                StructuralValue::Delegated(Box::new(self.reflect_parameter(parameter, names)))
            })
            .collect();
        StructuralValue::chosen(
            0,
            StructuralValue::Delimited(vec![
                StructuralValue::Atom(definition.name),
                StructuralValue::Delegated(Box::new(self.reflect_kind(&definition.kind, names))),
                StructuralValue::Delimited(parameters),
                StructuralValue::Delegated(Box::new(
                    self.reflect_template(&definition.template, names),
                )),
            ]),
        )
    }

    fn reflect_kind(&self, kind: &MacroKind, names: &mut NameTable) -> StructuralValue {
        match kind {
            MacroKind::Named => StructuralValue::chosen(
                KIND_NAMED,
                StructuralValue::Atom(self.keyword(names, "Named")),
            ),
            MacroKind::Structural(section) => Self::keyword_chosen(
                KIND_STRUCTURAL,
                self.keyword(names, "Structural"),
                StructuralValue::Delegated(Box::new(self.reflect_section(section, names))),
            ),
        }
    }

    fn reflect_section(&self, section: &SectionDefault, names: &mut NameTable) -> StructuralValue {
        let (constructor, keyword) = match section {
            SectionDefault::Newtype => (SECTION_NEWTYPE, "Newtype"),
            SectionDefault::Struct => (SECTION_STRUCT, "Struct"),
            SectionDefault::Enumeration => (SECTION_ENUMERATION, "Enumeration"),
        };
        StructuralValue::chosen(
            constructor,
            StructuralValue::Atom(self.keyword(names, keyword)),
        )
    }

    fn reflect_parameter(
        &self,
        parameter: &InputParameter,
        names: &mut NameTable,
    ) -> StructuralValue {
        StructuralValue::chosen(
            0,
            StructuralValue::Delimited(vec![
                StructuralValue::Atom(parameter.binding),
                StructuralValue::Delegated(Box::new(self.reflect_meta(&parameter.meta, names))),
            ]),
        )
    }

    fn reflect_meta(&self, meta: &MetaType, names: &mut NameTable) -> StructuralValue {
        let (constructor, keyword) = match meta {
            MetaType::Name => (META_NAME, "Name"),
            MetaType::Type => (META_TYPE_KIND, "Type"),
            MetaType::Fields => (META_FIELDS, "Fields"),
            MetaType::Variants => (META_VARIANTS, "Variants"),
        };
        StructuralValue::chosen(
            constructor,
            StructuralValue::Atom(self.keyword(names, keyword)),
        )
    }

    fn reflect_template(
        &self,
        template: &ResultTemplate,
        names: &mut NameTable,
    ) -> StructuralValue {
        let ResultTemplate::Item(ItemTemplate::Newtype(newtype)) = template else {
            panic!("only ResultTemplate::Item(Newtype) is authored");
        };
        let body = StructuralValue::Delimited(vec![
            StructuralValue::Delegated(Box::new(
                self.reflect_visibility(&newtype.visibility, names),
            )),
            StructuralValue::Delimited(
                newtype
                    .attributes
                    .items
                    .iter()
                    .map(|item| {
                        StructuralValue::Delegated(Box::new(
                            self.reflect_sequence_item(item, names),
                        ))
                    })
                    .collect(),
            ),
            StructuralValue::Delegated(Box::new(
                self.reflect_scalar_escape(Self::scalar_escape(&newtype.name), names),
            )),
            StructuralValue::Delegated(Box::new(
                self.reflect_scalar_escape(Self::scalar_escape(&newtype.wrapped), names),
            )),
        ]);
        // Item.Newtype.<body>
        let newtype_template = Self::keyword_chosen(
            0,
            self.keyword(names, "Newtype"),
            StructuralValue::Delegated(Box::new(StructuralValue::chosen(0, body))),
        );
        Self::keyword_chosen(
            0,
            self.keyword(names, "Item"),
            StructuralValue::Delegated(Box::new(newtype_template)),
        )
    }

    fn reflect_visibility(
        &self,
        visibility: &Visibility,
        names: &mut NameTable,
    ) -> StructuralValue {
        let Visibility::Public = visibility else {
            panic!("only Visibility::Public is authored");
        };
        StructuralValue::chosen(0, StructuralValue::Atom(self.keyword(names, "Public")))
    }

    fn reflect_sequence_item(
        &self,
        item: &SequenceItem<core_logos::Attribute>,
        names: &mut NameTable,
    ) -> StructuralValue {
        let SequenceItem::Escape(escape) = item else {
            panic!("only SequenceItem::Escape is authored");
        };
        Self::keyword_chosen(
            0,
            self.keyword(names, "Escape"),
            StructuralValue::Delegated(Box::new(self.reflect_escape(escape, names))),
        )
    }

    fn reflect_scalar_escape(&self, escape: &Escape, names: &mut NameTable) -> StructuralValue {
        Self::keyword_chosen(
            0,
            self.keyword(names, "Escape"),
            StructuralValue::Delegated(Box::new(self.reflect_escape(escape, names))),
        )
    }

    /// The escape a `Scalar` position carries (both golden scalars are `Escape`; the
    /// `Literal` scalar surface is deferred with the rest of the pretty form).
    fn scalar_escape<L>(scalar: &Scalar<L>) -> &Escape {
        match scalar {
            Scalar::Escape(escape) => escape,
            Scalar::Literal(_) => panic!("only Scalar::Escape is authored"),
        }
    }

    fn reflect_escape(&self, escape: &Escape, names: &mut NameTable) -> StructuralValue {
        match escape {
            Escape::Realize(realize) => Self::keyword_chosen(
                ESCAPE_REALIZE,
                self.keyword(names, "Realize"),
                StructuralValue::Delegated(Box::new(self.reflect_realize(realize, names))),
            ),
            Escape::Invoke(identity) => Self::keyword_chosen(
                ESCAPE_INVOKE,
                self.keyword(names, "Invoke"),
                StructuralValue::Scalar(ScalarValue::Integer(i64::from(identity.value()))),
            ),
            Escape::Splice(_) => panic!("Splice escape is deferred"),
        }
    }

    fn reflect_realize(&self, realize: &Realize, names: &mut NameTable) -> StructuralValue {
        StructuralValue::chosen(
            0,
            StructuralValue::Delimited(vec![
                StructuralValue::Delegated(Box::new(self.reflect_binding(&realize.binding, names))),
                StructuralValue::Delegated(Box::new(
                    self.reflect_transform(&realize.transform, names),
                )),
            ]),
        )
    }

    fn reflect_binding(&self, binding: &BindingRef, names: &mut NameTable) -> StructuralValue {
        let BindingRef::Input(identifier) = binding;
        Self::keyword_chosen(
            0,
            self.keyword(names, "Input"),
            StructuralValue::Atom(*identifier),
        )
    }

    fn reflect_transform(
        &self,
        transform: &NameTransform,
        names: &mut NameTable,
    ) -> StructuralValue {
        let (constructor, keyword) = match transform {
            NameTransform::Identity => (TRANSFORM_IDENTITY, "Identity"),
            NameTransform::FieldName => (TRANSFORM_FIELD_NAME, "FieldName"),
            NameTransform::Screaming => (TRANSFORM_SCREAMING, "Screaming"),
            NameTransform::PascalCase => (TRANSFORM_PASCAL, "PascalCase"),
        };
        StructuralValue::chosen(
            constructor,
            StructuralValue::Atom(self.keyword(names, keyword)),
        )
    }

    // ===== mirror-shape helpers =====

    fn keyword_chosen(
        constructor: u32,
        keyword: Identifier,
        payload: StructuralValue,
    ) -> StructuralValue {
        StructuralValue::chosen(
            constructor,
            StructuralValue::Application(
                Box::new(StructuralValue::Atom(keyword)),
                Box::new(payload),
            ),
        )
    }

    fn keyword(&self, names: &mut NameTable, text: &str) -> Identifier {
        let _ = &self.lexicon;
        names.intern(Name::new(text))
    }

    fn chosen(value: &StructuralValue) -> (u32, &StructuralValue) {
        match value {
            StructuralValue::Chosen {
                constructor,
                payload,
            } => (*constructor, payload.as_ref()),
            other => panic!("expected a constructor-tagged value, got {other:?}"),
        }
    }

    fn delegated(value: &StructuralValue) -> &StructuralValue {
        match value {
            StructuralValue::Delegated(inner) => inner.as_ref(),
            other => panic!("expected a delegated value, got {other:?}"),
        }
    }

    fn application_body(value: &StructuralValue) -> &StructuralValue {
        match value {
            StructuralValue::Application(_head, payload) => payload.as_ref(),
            other => panic!("expected an application, got {other:?}"),
        }
    }

    fn delimited(value: &StructuralValue) -> &Vec<StructuralValue> {
        match value {
            StructuralValue::Delimited(children) => children,
            other => panic!("expected a delimited block, got {other:?}"),
        }
    }

    fn atom(value: &StructuralValue) -> Identifier {
        match value {
            StructuralValue::Atom(identifier) => *identifier,
            other => panic!("expected an atom, got {other:?}"),
        }
    }

    fn scalar_integer(value: &StructuralValue) -> i64 {
        match value {
            StructuralValue::Scalar(ScalarValue::Integer(integer)) => *integer,
            other => panic!("expected an integer scalar, got {other:?}"),
        }
    }
}

/// The witness `MacroDefinition` — the real Newtype macro registration authored in
/// Rust (core-nomos/tests/pipeline.rs:522-548), built here against `names` so the
/// decoded value shares its NameTable identities.
fn witness_macro(names: &mut NameTable) -> MacroDefinition {
    let name_binding = names.intern(Name::new("name"));
    let type_binding = names.intern(Name::new("type"));
    let newtype_name = names.intern(Name::new("Newtype"));
    MacroDefinition {
        name: newtype_name,
        kind: MacroKind::Structural(SectionDefault::Newtype),
        input: InputSignature {
            parameters: vec![
                InputParameter {
                    binding: name_binding,
                    meta: MetaType::Name,
                },
                InputParameter {
                    binding: type_binding,
                    meta: MetaType::Type,
                },
            ],
        },
        template: ResultTemplate::Item(ItemTemplate::Newtype(NewtypeTemplate {
            visibility: Visibility::Public,
            attributes: Sequence::of(SequenceItem::Escape(Escape::Invoke(MacroIdentity::new(99)))),
            name: Scalar::Escape(Escape::Realize(Realize {
                binding: BindingRef::Input(name_binding),
                transform: NameTransform::Identity,
            })),
            wrapped: Scalar::Escape(Escape::Realize(Realize {
                binding: BindingRef::Input(type_binding),
                transform: NameTransform::Identity,
            })),
        })),
    }
}

/// The witness: the Rust-built macro definition reflects to canonical raw-NOTA text
/// through the sealed structuretree, and that text decodes back to a value EQUAL to
/// the Rust-built one — the raw door reads and writes a macro definition as positional
/// data, no sigil, no bespoke parser.
#[test]
fn newtype_macro_registration_decodes_from_nota_text() {
    let door = TextualNomos::build();
    let mut names = NameTable::new();
    let witness = witness_macro(&mut names);

    // reflect + encode: the macro value -> canonical raw-NOTA text through the organs.
    let text = door.encode_macro(&witness, &mut names);
    println!("macro-door raw NOTA text:\n{text}");

    // decode + reify: the text -> a MacroDefinition, EQUAL to the Rust-built witness.
    let decoded = door.decode_macro(&text, &mut names);
    assert_eq!(
        witness, decoded,
        "the decoded macro equals the Rust-built one"
    );

    // The recovered value re-encodes to byte-identical text.
    let re_encoded = door.encode_macro(&decoded, &mut names);
    assert_eq!(text, re_encoded, "the canonical raw-NOTA text is stable");

    // Every structural datum is visible in the raw text.
    for datum in [
        "Newtype",
        "Structural",
        "Name",
        "Type",
        "Item",
        "Public",
        "Escape",
        "Invoke",
        "99",
        "Realize",
        "Input",
        "Identity",
    ] {
        assert!(
            text.contains(datum),
            "raw text must carry `{datum}`: {text}"
        );
    }
}

//! The fixture macros as data: the wire package (WireNewtype delegating to the
//! recursive WireAttributes, and the particular-struct structural default) and the
//! plain package (the same shape with the bare two-attribute preamble). These are
//! `MacroPackage` constructors — the macros exist only as authored data, never as
//! code — and are the packages the capstone applies to real schema.

use core_logos::{
    Attribute, ConfigurationAttribute, ConfigurationPredicate, DeriveGroup, Generics, PathNode,
    Visibility,
};

use crate::definition::MacroDefinition;
use crate::identity::{MacroKind, SectionDefault};
use crate::meta::{InputParameter, InputSignature, MetaType};
use crate::package::{MacroPackage, PackageRevision};
use crate::template::{
    BindingRef, EnumerationTemplate, Escape, FieldNameRule, GenerationClass, ItemTemplate,
    NameTransform, NewtypeTemplate, Realize, ResultTemplate, Scalar, Sequence, SequenceItem,
    Splice, SpliceElement, StructTemplate,
};

/// Which attribute preamble a fixture package's macros carry. The wire preamble is
/// the standard three-node vector (`#[rustfmt::skip]`, the feature-gated NOTA
/// derive, the rkyv derive); the plain preamble drops the NOTA configuration node,
/// matching the runner reference fixtures.
#[derive(Clone, Copy)]
enum AttributePreamble {
    Wire,
    Plain,
}

impl AttributePreamble {
    fn attributes_macro_name(self) -> &'static str {
        match self {
            Self::Wire => "WireAttributes",
            Self::Plain => "PlainAttributes",
        }
    }

    fn newtype_macro_name(self) -> &'static str {
        match self {
            Self::Wire => "WireNewtype",
            Self::Plain => "PlainNewtype",
        }
    }
}

impl MacroPackage {
    /// The wire package: WireNewtype and the particular-struct default, both
    /// invoking the recursive WireAttributes macro that emits the standard
    /// three-node preamble.
    pub fn wire_fixture() -> Self {
        Self::fixture(AttributePreamble::Wire)
    }

    /// The plain package: the same macro shapes carrying the bare two-node
    /// preamble (`#[rustfmt::skip]` + the rkyv derive), which lowers a schema
    /// newtype to the runner reference fixtures with the prior rendering.
    pub fn plain_fixture() -> Self {
        Self::fixture(AttributePreamble::Plain)
    }

    /// The enriched wire package: the wire fixture's structural defaults (the data
    /// declarations) plus the generation selection nomos-engine applies. Through
    /// [`MacroPackage::apply_enriched`], it lowers a schema's declarations and then
    /// emits, in the reference fixture's document order, the newtype ergonomics, the interface
    /// ergonomics, the wire-contract vocabulary, the wire exchange codec (the working
    /// `encode_signal_frame` / `decode_signal_frame` bodies for the ordinary exchange
    /// leg), the wire exchange envelope (the `RequestPayload` / `SignalOperationHeads` /
    /// `LogVariant` impls, the `ExchangeFrame` aliases, and the `into_frame` /
    /// `into_reply_frame` constructors), and the trace support. The wire and plain
    /// fixtures are unchanged — their selection stays empty.
    ///
    /// The wire-contract class derives its short-header values from the interface
    /// roots' operation positions at generation time (see
    /// `Evaluator::short_header_module`), so the selection carries no transcribed data —
    /// every class is a plain marker.
    pub fn enriched_fixture() -> Self {
        Self::wire_fixture().with_selection(vec![
            GenerationClass::NewtypeErgonomics,
            GenerationClass::InterfaceErgonomics,
            GenerationClass::WireContract,
            GenerationClass::WireExchangeCodec,
            GenerationClass::WireExchangeEnvelope,
            GenerationClass::TraceSupport,
        ])
    }

    fn fixture(kind: AttributePreamble) -> Self {
        let mut package = MacroPackage::new(PackageRevision(1));

        // The input binding names (the derived `{ Name Type }` / `{ Name Fields }`
        // accessors), authored once and shared by every macro's signature.
        let name_binding = package.author_name("name");
        let type_binding = package.author_name("type");
        let fields_binding = package.author_name("fields");
        let variants_binding = package.author_name("variants");

        // The recursive attributes macro (named): a unit input, a literal preamble.
        let attributes_macro_name = package.author_name(kind.attributes_macro_name());
        let preamble = package.preamble_attributes(kind);
        let mut enumeration_preamble = preamble.clone();
        let copy_path = package.author_path(&["Copy"]);
        if let Some(Attribute::Derive(group)) = enumeration_preamble.last_mut() {
            group.paths.insert(4, copy_path);
        }
        let attributes_macro = package.register(MacroDefinition {
            name: attributes_macro_name,
            kind: MacroKind::Named,
            input: InputSignature::unit(),
            template: ResultTemplate::Attributes(Sequence {
                items: preamble.into_iter().map(SequenceItem::Literal).collect(),
            }),
        });
        let enumeration_attributes_name = package.author_name("EnumerationAttributes");
        let enumeration_attributes_macro = package.register(MacroDefinition {
            name: enumeration_attributes_name,
            kind: MacroKind::Named,
            input: InputSignature::unit(),
            template: ResultTemplate::Attributes(Sequence {
                items: enumeration_preamble
                    .into_iter()
                    .map(SequenceItem::Literal)
                    .collect(),
            }),
        });

        // The newtype structural default: name and wrapped type realized, the
        // preamble delegated to the attributes macro.
        let newtype_macro_name = package.author_name(kind.newtype_macro_name());
        package.register(MacroDefinition {
            name: newtype_macro_name,
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
                attributes: Sequence::of(SequenceItem::Escape(Escape::Invoke(attributes_macro))),
                name: Scalar::Escape(Escape::Realize(Realize {
                    binding: BindingRef::Input(name_binding),
                    transform: NameTransform::Identity,
                })),
                wrapped: Scalar::Escape(Escape::Realize(Realize {
                    binding: BindingRef::Input(type_binding),
                    transform: NameTransform::Identity,
                })),
            })),
        });

        // The particular-struct structural default: name realized, fields spliced
        // through the Field-rule dispatch, the preamble delegated.
        let struct_macro_name = package.author_name("ParticularStruct");
        package.register(MacroDefinition {
            name: struct_macro_name,
            kind: MacroKind::Structural(SectionDefault::Struct),
            input: InputSignature {
                parameters: vec![
                    InputParameter {
                        binding: name_binding,
                        meta: MetaType::Name,
                    },
                    InputParameter {
                        binding: fields_binding,
                        meta: MetaType::Fields,
                    },
                ],
            },
            template: ResultTemplate::Item(ItemTemplate::Struct(StructTemplate {
                visibility: Visibility::Public,
                attributes: Sequence::of(SequenceItem::Escape(Escape::Invoke(attributes_macro))),
                name: Scalar::Escape(Escape::Realize(Realize {
                    binding: BindingRef::Input(name_binding),
                    transform: NameTransform::Identity,
                })),
                generics: Generics::none(),
                fields: Sequence::of(SequenceItem::Escape(Escape::Splice(Splice {
                    binding: BindingRef::Input(fields_binding),
                    element: SpliceElement::Field {
                        visibility: Visibility::Public,
                        name_rule: FieldNameRule::FieldRuleDispatch,
                    },
                }))),
            })),
        });

        // The enumeration structural default preserves variant names and lowers
        // optional payload references into tuple payloads.
        let enumeration_macro_name = package.author_name("Enumeration");
        package.register(MacroDefinition {
            name: enumeration_macro_name,
            kind: MacroKind::Structural(SectionDefault::Enumeration),
            input: InputSignature {
                parameters: vec![
                    InputParameter {
                        binding: name_binding,
                        meta: MetaType::Name,
                    },
                    InputParameter {
                        binding: variants_binding,
                        meta: MetaType::Variants,
                    },
                ],
            },
            template: ResultTemplate::Item(ItemTemplate::Enumeration(EnumerationTemplate {
                visibility: Visibility::Public,
                attributes: Sequence::of(SequenceItem::Escape(Escape::Invoke(
                    enumeration_attributes_macro,
                ))),
                name: Scalar::Escape(Escape::Realize(Realize {
                    binding: BindingRef::Input(name_binding),
                    transform: NameTransform::Identity,
                })),
                generics: Generics::none(),
                variants: Sequence::of(SequenceItem::Escape(Escape::Splice(Splice {
                    binding: BindingRef::Input(variants_binding),
                    element: SpliceElement::Variant,
                }))),
            })),
        });

        package
    }

    /// Author the ordered attribute preamble for a fixture package, interning its
    /// path names into the package's NameTable. The `Wire` preamble carries the
    /// feature-gated NOTA derive node between the tool attribute and the rkyv
    /// derive; the `Plain` preamble omits it.
    fn preamble_attributes(&mut self, kind: AttributePreamble) -> Vec<Attribute> {
        let mut attributes = vec![Attribute::ToolPath(self.author_path(&["rustfmt", "skip"]))];
        if let AttributePreamble::Wire = kind {
            let feature = self.author_name("nota-text");
            attributes.push(Attribute::Configuration(ConfigurationAttribute {
                predicate: ConfigurationPredicate::Feature(feature),
                inner: Box::new(Attribute::Derive(DeriveGroup {
                    paths: vec![
                        self.author_path(&["nota", "NotaDecode"]),
                        self.author_path(&["nota", "NotaDecodeTraced"]),
                        self.author_path(&["nota", "NotaEncode"]),
                    ],
                })),
            }));
        }
        attributes.push(Attribute::Derive(DeriveGroup {
            paths: vec![
                self.author_path(&["rkyv", "Archive"]),
                self.author_path(&["rkyv", "Serialize"]),
                self.author_path(&["rkyv", "Deserialize"]),
                self.author_path(&["Clone"]),
                self.author_path(&["Debug"]),
                self.author_path(&["PartialEq"]),
                self.author_path(&["Eq"]),
            ],
        }));
        attributes
    }

    fn author_path(&mut self, segments: &[&str]) -> PathNode {
        PathNode {
            segments: segments
                .iter()
                .map(|segment| self.author_name(segment))
                .collect(),
        }
    }
}

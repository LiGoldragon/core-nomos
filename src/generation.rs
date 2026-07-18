//! The enriched generation classes: the schema-derived *support surface* the wire
//! goldens emit alongside the data declarations — impl blocks (with methods,
//! associated types, and associated consts), functions, consts, const modules, and
//! use imports, as stringless CoreLogos data.
//!
//! Where the per-declaration structural defaults lower one CoreLogos item per schema
//! declaration, a [`GenerationClass`] is a whole-schema generator. It reads the
//! schema's newtype catalogue and its interface roots
//! ([`core_schema::DeclarationRole`]) and appends an ordered run of CoreLogos items
//! into the same continuous logos NameTable the declaration lowering built. Each
//! class builds its fixed method and match skeletons directly — exactly as the fixed
//! module prelude ([`crate::ModuleHead`]) authors its stringless data — with every
//! identifier interned into that one table, no head strings and no text.
//!
//! The verbs live on the crate-internal `Evaluator` that owns the growing NameTable,
//! the same data-bearing walk the declaration lowering uses, so the generated items
//! resolve in the identifier space the declarations already populated.

use core_logos::{
    Alias, ArrayExpression, AssociatedType, Attribute, Block, Call, Callee, ClosureExpression,
    ConfigurationAttribute, ConfigurationPredicate, Const, CoreItem, DeriveGroup, Enumeration,
    Expression, FieldInitializer, Function, Generics, ImplBlock, ImplItem, ImplTraitType,
    IndexExpression, IntegerLiteral, IntegerRepresentation, LetBinding, LetStatement, Match,
    MatchArm, MethodCall, Module, Newtype, Parameter, PathNode, Pattern, PatternElement,
    QualifiedPath, RangeExpression, Receiver, ReferenceExpression, ReferenceMutability,
    ReferenceType, SliceType, Statement, StructLiteral, TryExpression, TupleExpression,
    TupleFieldAccess, TupleType, TupleVariantPattern, TypeApplication, TypeReference, Variant,
    VariantPayload, Visibility,
};
use core_schema::{CoreDeclaration, CoreReference, CoreSchema, CoreType, CoreVariant};
use name_table::{Identifier, Name};
use std::collections::BTreeMap;

/// The little-endian short-header width the codec bodies emit as
/// `SIGNAL_SHORT_HEADER_BYTE_COUNT`, mirroring the hand-written contracts'
/// `SIGNAL_SHORT_HEADER_BYTE_COUNT = 8`.
const SHORT_HEADER_BYTE_COUNT: u64 = 8;

use crate::engine::Evaluator;
use crate::error::NomosError;
use crate::template::GenerationClass;

/// How a newtype's `new` constructor takes its payload — the contact point between
/// the wrapped type's kind and the constructor ergonomics, named rather than a
/// boolean. The `String` scalar leaf takes an `impl Into<String>` and constructs
/// through `.into()`; every other wrapped type takes its value directly.
///
/// LEAN `newtype-into-string-intake`: the `impl Into<String>` intake keys on the
/// `String` scalar leaf only. The standard-newtype corpus also applies it to a
/// `Path` alias of `String` (a `Plain` reference resolving to the string leaf); that
/// alias resolution is out of the spirit-min slice. Trigger to revisit: extending the
/// enriched packages to the standard-newtype corpus.
enum Intake {
    IntoString,
    ByValue,
}

impl Intake {
    fn of(wrapped: &CoreReference) -> Self {
        match wrapped {
            CoreReference::String => Self::IntoString,
            _ => Self::ByValue,
        }
    }
}

/// How an interface-root constructor takes and wraps a variant's payload — the
/// contact point between a variant payload and the newtype catalogue. A payload that
/// is a catalogued newtype is unwrapped: the constructor takes the newtype's inner
/// type and wraps it through `Newtype::new`. Any other payload is taken directly.
enum ConstructorSource {
    Direct(CoreReference),
    Unwrap {
        newtype: Identifier,
        inner: CoreReference,
    },
}

impl ConstructorSource {
    fn of(payload: &CoreReference, catalogue: &BTreeMap<Identifier, CoreReference>) -> Self {
        if let CoreReference::Plain(identifier) = payload {
            if let Some(inner) = catalogue.get(identifier) {
                return Self::Unwrap {
                    newtype: *identifier,
                    inner: inner.clone(),
                };
            }
        }
        Self::Direct(payload.clone())
    }
}

/// One interface root as generation reads it: its declaration name and its variants,
/// owned so the generator can intern and lower while it walks them.
struct InterfaceRoot {
    name: Identifier,
    variants: Vec<CoreVariant>,
}

impl InterfaceRoot {
    /// Read a role-tagged declaration as an interface root, rejecting a non-enum root
    /// loudly (an interface root is always an enumeration of its operations).
    fn of(declaration: &CoreDeclaration) -> Result<Self, NomosError> {
        match declaration.value() {
            CoreType::Enumeration(enumeration) => Ok(Self {
                name: enumeration.identifier(),
                variants: enumeration.variants().to_vec(),
            }),
            _ => Err(NomosError::Generation(
                "an interface root is not an enumeration",
            )),
        }
    }
}

impl Evaluator<'_> {
    /// Build one generation class's ordered CoreLogos items from the schema.
    pub(crate) fn generate_class(
        &mut self,
        class: &GenerationClass,
        schema: &CoreSchema,
    ) -> Result<Vec<CoreItem>, NomosError> {
        match class {
            GenerationClass::NewtypeErgonomics => self.generate_newtype_ergonomics(schema),
            GenerationClass::InterfaceErgonomics => self.generate_interface_ergonomics(schema),
            GenerationClass::WireContract => self.generate_wire_contract(schema),
            GenerationClass::WireExchangeCodec => self.generate_wire_exchange_codec(schema),
            GenerationClass::WireExchangeEnvelope => self.generate_wire_exchange_envelope(schema),
            GenerationClass::TraceSupport => self.generate_trace_support(schema),
        }
    }

    // ---- shared stringless builders ---------------------------------------------

    /// Intern a fixed generation name into the extended logos table (dedup, so a name
    /// a declaration already carries reuses its identifier).
    fn ident(&mut self, text: &str) -> Identifier {
        self.names.intern(Name::new(text))
    }

    /// A path over fixed name segments.
    fn path(&mut self, segments: &[&str]) -> PathNode {
        PathNode {
            segments: segments.iter().map(|segment| self.ident(segment)).collect(),
        }
    }

    /// A path over already-interned identifier segments.
    fn path_of(&self, segments: &[Identifier]) -> PathNode {
        PathNode {
            segments: segments.to_vec(),
        }
    }

    /// A single-segment path type over a fixed name (`Self`, `String`).
    fn type_path(&mut self, segments: &[&str]) -> TypeReference {
        TypeReference::Path(self.path(segments))
    }

    /// The `Self` type, the return type of every constructor.
    fn self_type(&mut self) -> TypeReference {
        self.type_path(&["Self"])
    }

    /// The `&'static str` type shared by every `name()` return and the `HEADS`
    /// element type.
    fn static_str(&mut self) -> TypeReference {
        let lifetime = self.ident("static");
        let referent = self.type_path(&["str"]);
        TypeReference::Reference(ReferenceType {
            lifetime: Some(lifetime),
            mutability: ReferenceMutability::Shared,
            referent: Box::new(referent),
        })
    }

    /// The `#[rustfmt::skip]` attribute every generated item carries.
    fn rustfmt_skip(&mut self) -> Attribute {
        Attribute::ToolPath(self.path(&["rustfmt", "skip"]))
    }

    /// The `#[cfg(feature = "nota-text")]` gate on the `FromStr` / `Display` impls.
    fn cfg_nota(&mut self) -> Attribute {
        let feature = self.ident("nota-text");
        Attribute::Cfg(ConfigurationPredicate::Feature(feature))
    }

    /// The wire enum preamble the route enums and the trace enums carry — the same
    /// three-node preamble as the data enums, with `Copy` (a route/trace enum is
    /// unit- or newtype-payloaded and stays `Copy`).
    fn wire_enum_preamble(&mut self) -> Vec<Attribute> {
        let skip = self.rustfmt_skip();
        let feature = self.ident("nota-text");
        let nota = Attribute::Configuration(ConfigurationAttribute {
            predicate: ConfigurationPredicate::Feature(feature),
            inner: Box::new(Attribute::Derive(DeriveGroup {
                paths: vec![
                    self.path(&["nota", "NotaDecode"]),
                    self.path(&["nota", "NotaDecodeTraced"]),
                    self.path(&["nota", "NotaEncode"]),
                ],
            })),
        });
        let derive = Attribute::Derive(DeriveGroup {
            paths: vec![
                self.path(&["rkyv", "Archive"]),
                self.path(&["rkyv", "Serialize"]),
                self.path(&["rkyv", "Deserialize"]),
                self.path(&["Clone"]),
                self.path(&["Copy"]),
                self.path(&["Debug"]),
                self.path(&["PartialEq"]),
                self.path(&["Eq"]),
            ],
        });
        vec![skip, nota, derive]
    }

    /// A method (associated function) node: the shared shape of every generated
    /// method, dispatched only by its parts.
    fn method(
        &mut self,
        name: Identifier,
        visibility: Visibility,
        receiver: Option<Receiver>,
        parameters: Vec<Parameter>,
        return_type: Option<TypeReference>,
        body: Expression,
    ) -> ImplItem {
        ImplItem::Method(Function {
            attributes: Vec::new(),
            visibility,
            name,
            generics: Generics::none(),
            receiver,
            parameters,
            return_type,
            body: Block {
                statements: Vec::new(),
                tail_expression: body,
            },
        })
    }

    /// A method whose body is a full [`Block`] — a statement run plus a tail — for the
    /// multi-statement codec bodies (`encode_signal_frame` / `decode_signal_frame`).
    /// The single-tail-expression `method` above is the empty-statement special case
    /// of this.
    #[allow(clippy::too_many_arguments)]
    fn method_block(
        &mut self,
        name: Identifier,
        visibility: Visibility,
        receiver: Option<Receiver>,
        parameters: Vec<Parameter>,
        return_type: Option<TypeReference>,
        body: Block,
    ) -> ImplItem {
        ImplItem::Method(Function {
            attributes: Vec::new(),
            visibility,
            name,
            generics: Generics::none(),
            receiver,
            parameters,
            return_type,
            body,
        })
    }

    /// A `payload` parameter of a given type.
    fn payload_parameter(&mut self, type_reference: TypeReference) -> Parameter {
        Parameter {
            name: self.ident("payload"),
            type_reference,
        }
    }

    /// The `payload` value expression.
    fn payload_value(&mut self) -> Expression {
        Expression::Path(self.path(&["payload"]))
    }

    /// A call of a fixed-name path callee.
    fn call_path(&mut self, segments: &[&str], arguments: Vec<Expression>) -> Expression {
        let callee = Callee::Path(self.path(segments));
        Expression::Call(Call {
            callee,
            type_arguments: Vec::new(),
            arguments,
        })
    }

    /// A call of a fixed-name path callee with a turbofish
    /// (`rkyv::to_bytes::<rkyv::rancor::Error>(self)`).
    fn call_path_turbofish(
        &mut self,
        segments: &[&str],
        type_arguments: Vec<TypeReference>,
        arguments: Vec<Expression>,
    ) -> Expression {
        let callee = Callee::Path(self.path(segments));
        Expression::Call(Call {
            callee,
            type_arguments,
            arguments,
        })
    }

    /// A call of a callee path built from interned identifiers (a variant path such
    /// as `Self::Record`).
    fn call_path_of(&self, segments: &[Identifier], arguments: Vec<Expression>) -> Expression {
        Expression::Call(Call {
            callee: Callee::Path(self.path_of(segments)),
            type_arguments: Vec::new(),
            arguments,
        })
    }

    /// The tuple field access `self.0`.
    fn self_field_zero(&self) -> Expression {
        Expression::Field(TupleFieldAccess {
            base: Box::new(Expression::Receiver),
            index: 0,
        })
    }

    /// An inherent impl block (`impl <self_type> { <items> }`).
    fn inherent_impl(&mut self, self_type: TypeReference, items: Vec<ImplItem>) -> CoreItem {
        let skip = self.rustfmt_skip();
        CoreItem::ImplBlock(ImplBlock {
            attributes: vec![skip],
            generics: Generics::none(),
            implemented_trait: None,
            self_type,
            items,
        })
    }

    /// A trait impl block with the given attribute preamble.
    fn trait_impl(
        &mut self,
        attributes: Vec<Attribute>,
        implemented_trait: TypeReference,
        self_type: TypeReference,
        items: Vec<ImplItem>,
    ) -> CoreItem {
        CoreItem::ImplBlock(ImplBlock {
            attributes,
            generics: Generics::none(),
            implemented_trait: Some(implemented_trait),
            self_type,
            items,
        })
    }

    // ---- schema reading ---------------------------------------------------------

    /// The newtype catalogue: every data-type newtype declaration's name mapped to
    /// its wrapped reference. The interface constructors read it to decide whether a
    /// variant payload unwraps.
    fn newtype_catalogue(schema: &CoreSchema) -> BTreeMap<Identifier, CoreReference> {
        let mut catalogue = BTreeMap::new();
        for declaration in schema.data_declarations() {
            if let CoreType::Newtype(newtype) = declaration.value() {
                catalogue.insert(newtype.identifier(), newtype.reference().clone());
            }
        }
        catalogue
    }

    /// The interface roots present in the schema, in document order (input then
    /// output). Classes B/C/D gate on these.
    fn interface_roots(schema: &CoreSchema) -> Result<Vec<InterfaceRoot>, NomosError> {
        [schema.input(), schema.output()]
            .into_iter()
            .flatten()
            .map(InterfaceRoot::of)
            .collect()
    }

    // ---- class A: newtype ergonomics --------------------------------------------

    fn generate_newtype_ergonomics(
        &mut self,
        schema: &CoreSchema,
    ) -> Result<Vec<CoreItem>, NomosError> {
        let mut items = Vec::new();
        for declaration in schema.data_declarations() {
            if let CoreType::Newtype(newtype) = declaration.value() {
                let name = newtype.identifier();
                let wrapped = newtype.reference().clone();
                items.push(self.newtype_inherent_impl(name, &wrapped)?);
                items.push(self.newtype_from_impl(name, &wrapped)?);
            }
        }
        Ok(items)
    }

    fn newtype_inherent_impl(
        &mut self,
        name: Identifier,
        wrapped: &CoreReference,
    ) -> Result<CoreItem, NomosError> {
        let wrapped_type = self.lower_reference(wrapped)?;
        let self_type = TypeReference::Path(self.path_of(&[name]));

        // new(...)
        let new_name = self.ident("new");
        let self_return = self.self_type();
        let new_method = match Intake::of(wrapped) {
            Intake::IntoString => {
                let string = self.type_path(&["String"]);
                let into_head = self.path(&["Into"]);
                let param_type = TypeReference::ImplTrait(ImplTraitType {
                    bounds: vec![TypeReference::Application(TypeApplication {
                        head: into_head,
                        arguments: vec![string],
                    })],
                });
                let parameter = self.payload_parameter(param_type);
                let payload = self.payload_value();
                let into = self.ident("into");
                let body = self.call_path(
                    &["Self"],
                    vec![Expression::MethodCall(MethodCall {
                        receiver: Box::new(payload),
                        method: into,
                        type_arguments: Vec::new(),
                        arguments: Vec::new(),
                    })],
                );
                self.method(
                    new_name,
                    Visibility::Public,
                    None,
                    vec![parameter],
                    Some(self_return),
                    body,
                )
            }
            Intake::ByValue => {
                let parameter = self.payload_parameter(wrapped_type.clone());
                let payload = self.payload_value();
                let body = self.call_path(&["Self"], vec![payload]);
                self.method(
                    new_name,
                    Visibility::Public,
                    None,
                    vec![parameter],
                    Some(self_return),
                    body,
                )
            }
        };

        // payload(&self) -> &<wrapped>
        let payload_name = self.ident("payload");
        let payload_return = TypeReference::Reference(ReferenceType {
            lifetime: None,
            mutability: ReferenceMutability::Shared,
            referent: Box::new(wrapped_type.clone()),
        });
        let payload_body = Expression::Reference(ReferenceExpression {
            referent: Box::new(self.self_field_zero()),
        });
        let payload_method = self.method(
            payload_name,
            Visibility::Public,
            Some(Receiver::Reference),
            Vec::new(),
            Some(payload_return),
            payload_body,
        );

        // into_payload(self) -> <wrapped>
        let into_payload_name = self.ident("into_payload");
        let into_payload_body = self.self_field_zero();
        let into_payload_method = self.method(
            into_payload_name,
            Visibility::Public,
            Some(Receiver::Value),
            Vec::new(),
            Some(wrapped_type),
            into_payload_body,
        );

        Ok(self.inherent_impl(
            self_type,
            vec![new_method, payload_method, into_payload_method],
        ))
    }

    fn newtype_from_impl(
        &mut self,
        name: Identifier,
        wrapped: &CoreReference,
    ) -> Result<CoreItem, NomosError> {
        let wrapped_type = self.lower_reference(wrapped)?;
        let self_type = TypeReference::Path(self.path_of(&[name]));
        let from_head = self.path(&["From"]);
        let implemented_trait = TypeReference::Application(TypeApplication {
            head: from_head,
            arguments: vec![wrapped_type.clone()],
        });
        let from_name = self.ident("from");
        let parameter = self.payload_parameter(wrapped_type);
        let self_return = self.self_type();
        let payload = self.payload_value();
        let body = self.call_path(&["Self", "new"], vec![payload]);
        let from_method = self.method(
            from_name,
            Visibility::Private,
            None,
            vec![parameter],
            Some(self_return),
            body,
        );
        let skip = self.rustfmt_skip();
        Ok(self.trait_impl(vec![skip], implemented_trait, self_type, vec![from_method]))
    }

    // ---- class B: interface ergonomics ------------------------------------------

    fn generate_interface_ergonomics(
        &mut self,
        schema: &CoreSchema,
    ) -> Result<Vec<CoreItem>, NomosError> {
        let roots = Self::interface_roots(schema)?;
        if roots.is_empty() {
            return Err(NomosError::Generation(
                "interface ergonomics needs interface roots, the schema has none",
            ));
        }
        let catalogue = Self::newtype_catalogue(schema);
        let mut items = Vec::new();

        // Constructor impls, one per root.
        for root in &roots {
            items.push(self.interface_constructor_impl(root, &catalogue)?);
        }
        // From impls, one per root per variant.
        for root in &roots {
            for variant in &root.variants {
                items.push(self.interface_from_impl(root.name, variant)?);
            }
        }
        // The cfg-gated FromStr and Display impls, one pair per root.
        for root in &roots {
            items.push(self.interface_from_str_impl(root.name)?);
            items.push(self.interface_display_impl(root.name)?);
        }
        Ok(items)
    }

    fn interface_constructor_impl(
        &mut self,
        root: &InterfaceRoot,
        catalogue: &BTreeMap<Identifier, CoreReference>,
    ) -> Result<CoreItem, NomosError> {
        let self_type = TypeReference::Path(self.path_of(&[root.name]));
        let self_ident = self.self_ident();
        let mut methods = Vec::with_capacity(root.variants.len());
        for variant in &root.variants {
            let payload = variant
                .payload()
                .ok_or(NomosError::Generation(
                    "an interface variant carries no payload to construct from",
                ))?
                .clone();
            let method_name = self.derived_snake_name(variant.identifier())?;
            let self_return = self.self_type();
            let (parameter, body) = match ConstructorSource::of(&payload, catalogue) {
                ConstructorSource::Direct(reference) => {
                    let parameter_type = self.lower_reference(&reference)?;
                    let parameter = self.payload_parameter(parameter_type);
                    let payload_value = self.payload_value();
                    let body =
                        self.call_path_of(&[self_ident, variant.identifier()], vec![payload_value]);
                    (parameter, body)
                }
                ConstructorSource::Unwrap { newtype, inner } => {
                    let parameter_type = self.lower_reference(&inner)?;
                    let parameter = self.payload_parameter(parameter_type);
                    let payload_value = self.payload_value();
                    let new = self.ident("new");
                    let wrap = self.call_path_of(&[newtype, new], vec![payload_value]);
                    let body = self.call_path_of(&[self_ident, variant.identifier()], vec![wrap]);
                    (parameter, body)
                }
            };
            methods.push(self.method(
                method_name,
                Visibility::Public,
                None,
                vec![parameter],
                Some(self_return),
                body,
            ));
        }
        Ok(self.inherent_impl(self_type, methods))
    }

    fn interface_from_impl(
        &mut self,
        root: Identifier,
        variant: &CoreVariant,
    ) -> Result<CoreItem, NomosError> {
        let payload = variant
            .payload()
            .ok_or(NomosError::Generation(
                "an interface variant carries no payload to convert from",
            ))?
            .clone();
        let payload_type = self.lower_reference(&payload)?;
        let self_type = TypeReference::Path(self.path_of(&[root]));
        let from_head = self.path(&["From"]);
        let implemented_trait = TypeReference::Application(TypeApplication {
            head: from_head,
            arguments: vec![payload_type.clone()],
        });
        let from_name = self.ident("from");
        let parameter = self.payload_parameter(payload_type);
        let self_return = self.self_type();
        let payload_value = self.payload_value();
        let self_ident = self.self_ident();
        let body = self.call_path_of(&[self_ident, variant.identifier()], vec![payload_value]);
        let from_method = self.method(
            from_name,
            Visibility::Private,
            None,
            vec![parameter],
            Some(self_return),
            body,
        );
        let skip = self.rustfmt_skip();
        Ok(self.trait_impl(vec![skip], implemented_trait, self_type, vec![from_method]))
    }

    fn interface_from_str_impl(&mut self, root: Identifier) -> Result<CoreItem, NomosError> {
        let self_type = TypeReference::Path(self.path_of(&[root]));
        let implemented_trait = TypeReference::Path(self.path(&["std", "str", "FromStr"]));

        let err_name = self.ident("Err");
        let err_value = self.type_path(&["NotaDecodeError"]);
        let associated_type = ImplItem::AssociatedType(AssociatedType {
            name: err_name,
            value: err_value,
        });

        // fn from_str(source: &str) -> Result<Self, Self::Err>
        let from_str_name = self.ident("from_str");
        let source_name = self.ident("source");
        let str_ref = TypeReference::Reference(ReferenceType {
            lifetime: None,
            mutability: ReferenceMutability::Shared,
            referent: Box::new(self.type_path(&["str"])),
        });
        let parameter = Parameter {
            name: source_name,
            type_reference: str_ref,
        };
        let result_head = self.path(&["Result"]);
        let self_arg = self.self_type();
        let self_err = TypeReference::Path(self.path(&["Self", "Err"]));
        let return_type = TypeReference::Application(TypeApplication {
            head: result_head,
            arguments: vec![self_arg, self_err],
        });
        let source_value = Expression::Path(self.path_of(&[source_name]));
        let nota_source_new = self.call_path(&["NotaSource", "new"], vec![source_value]);
        let parse = self.ident("parse");
        let self_turbofish = self.self_type();
        let body = Expression::MethodCall(MethodCall {
            receiver: Box::new(nota_source_new),
            method: parse,
            type_arguments: vec![self_turbofish],
            arguments: Vec::new(),
        });
        let from_str = self.method(
            from_str_name,
            Visibility::Private,
            None,
            vec![parameter],
            Some(return_type),
            body,
        );

        let skip = self.rustfmt_skip();
        let cfg = self.cfg_nota();
        Ok(self.trait_impl(
            vec![skip, cfg],
            implemented_trait,
            self_type,
            vec![associated_type, from_str],
        ))
    }

    fn interface_display_impl(&mut self, root: Identifier) -> Result<CoreItem, NomosError> {
        let self_type = TypeReference::Path(self.path_of(&[root]));
        let implemented_trait = TypeReference::Path(self.path(&["std", "fmt", "Display"]));

        // fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
        let fmt_name = self.ident("fmt");
        let formatter_name = self.ident("formatter");
        let underscore = self.ident("_");
        let formatter_head = self.path(&["std", "fmt", "Formatter"]);
        let formatter_type = TypeReference::Reference(ReferenceType {
            lifetime: None,
            mutability: ReferenceMutability::Mutable,
            referent: Box::new(TypeReference::Application(TypeApplication {
                head: formatter_head,
                arguments: vec![TypeReference::Lifetime(underscore)],
            })),
        });
        let parameter = Parameter {
            name: formatter_name,
            type_reference: formatter_type,
        };
        let return_type = TypeReference::Path(self.path(&["std", "fmt", "Result"]));

        // formatter.write_str(&<Self as NotaEncode>::to_nota(self))
        let self_qualified = self.self_type();
        let nota_encode = self.path(&["NotaEncode"]);
        let to_nota = self.ident("to_nota");
        let qualified_call = Expression::Call(Call {
            callee: Callee::Qualified(QualifiedPath {
                self_type: self_qualified,
                trait_path: nota_encode,
                member: vec![to_nota],
            }),
            type_arguments: Vec::new(),
            arguments: vec![Expression::Receiver],
        });
        let write_str = self.ident("write_str");
        let formatter_value = Expression::Path(self.path_of(&[formatter_name]));
        let body = Expression::MethodCall(MethodCall {
            receiver: Box::new(formatter_value),
            method: write_str,
            type_arguments: Vec::new(),
            arguments: vec![Expression::Reference(ReferenceExpression {
                referent: Box::new(qualified_call),
            })],
        });
        let fmt = self.method(
            fmt_name,
            Visibility::Private,
            Some(Receiver::Reference),
            vec![parameter],
            Some(return_type),
            body,
        );

        let skip = self.rustfmt_skip();
        let cfg = self.cfg_nota();
        Ok(self.trait_impl(vec![skip, cfg], implemented_trait, self_type, vec![fmt]))
    }

    // ---- the wire contract: the ordinary-exchange wire vocabulary ---------------

    /// The wire vocabulary, in golden document order: the `short_header` const module,
    /// the `SIGNAL_SHORT_HEADER_BYTE_COUNT` byte-count const, the `SignalFrameError`
    /// enum, and the two route enums. These are the types the codec speaks; the
    /// encode/decode bodies over them are the sibling [`Self::generate_wire_exchange_codec`].
    fn generate_wire_contract(&mut self, schema: &CoreSchema) -> Result<Vec<CoreItem>, NomosError> {
        let roots = Self::interface_roots(schema)?;
        if roots.is_empty() {
            return Err(NomosError::Generation(
                "the wire contract needs interface roots, the schema has none",
            ));
        }
        let mut items = Vec::new();
        items.push(self.short_header_module(&roots)?);
        items.push(self.short_header_byte_count_const());
        items.push(self.signal_frame_error_enum());
        for root in &roots {
            items.push(self.route_enum(root)?);
        }
        Ok(items)
    }

    // ---- the wire exchange codec: the encode/decode bodies ----------------------

    /// The ordinary-exchange codec: per interface root the `impl` carrying `route`,
    /// `short_header`, `route_from_short_header`, `encode_signal_frame`, and
    /// `decode_signal_frame`. The bodies are behavioral, not byte-copies of the golden:
    /// they mirror the wire the hand-written signal contracts speak (an 8-byte
    /// little-endian short header ahead of an rkyv archive) in a shape the modeled
    /// statement vocabulary expresses directly (an `.ok_or(…)?` in place of an
    /// `if … { return … }`). The request root's `SignalOperationHeads` impl and the
    /// rest of the envelope surface are the sibling
    /// [`Self::generate_wire_exchange_envelope`].
    fn generate_wire_exchange_codec(
        &mut self,
        schema: &CoreSchema,
    ) -> Result<Vec<CoreItem>, NomosError> {
        let roots = Self::interface_roots(schema)?;
        if roots.is_empty() {
            return Err(NomosError::Generation(
                "the wire exchange codec needs interface roots, the schema has none",
            ));
        }
        let mut items = Vec::new();
        for root in &roots {
            items.push(self.codec_impl(root)?);
        }
        Ok(items)
    }

    // ---- the wire exchange envelope: the ordinary-leg envelope surface ----------

    /// The ordinary-exchange envelope surface the ported daemon and clients speak over
    /// the codec, in the golden's document order: the request root's
    /// `signal_frame::RequestPayload`, `SignalOperationHeads`, and `LogVariant` trait
    /// impls; the `Frame` / `FrameBody` / `Request` / `ReplyEnvelope` / `RequestBuilder`
    /// type aliases over `signal_frame::ExchangeFrame` (the ordinary two-way leg); and
    /// the request root's `into_frame` and the reply root's `into_reply_frame`
    /// constructors. Scope is the ordinary leg only — the aliases name `ExchangeFrame`,
    /// never `StreamingFrame`, whose subscription envelope waits on pending psyche
    /// rulings.
    fn generate_wire_exchange_envelope(
        &mut self,
        schema: &CoreSchema,
    ) -> Result<Vec<CoreItem>, NomosError> {
        let roots = Self::interface_roots(schema)?;
        let request = roots.first().ok_or(NomosError::Generation(
            "the wire exchange envelope needs a request (input) root",
        ))?;
        let reply = roots.get(1).ok_or(NomosError::Generation(
            "the wire exchange envelope needs a reply (output) root",
        ))?;
        let request_name = request.name;
        let reply_name = reply.name;
        Ok(vec![
            self.request_payload_impl(request_name),
            self.signal_operation_heads_impl(request)?,
            self.log_variant_impl(request_name),
            self.frame_alias("Frame", "ExchangeFrame", &[request_name, reply_name]),
            self.frame_alias("FrameBody", "ExchangeFrameBody", &[request_name, reply_name]),
            self.frame_alias("Request", "Request", &[request_name]),
            self.frame_alias("ReplyEnvelope", "Reply", &[reply_name]),
            self.frame_alias("RequestBuilder", "RequestBuilder", &[request_name]),
            self.into_frame_impl(request_name),
            self.into_reply_frame_impl(reply_name),
        ])
    }

    /// The `short_header` const module: one `pub const <ROOT>_<VARIANT>: u64` per
    /// interface-root operation. Each value is derived from the operation's position
    /// — `(root_index << 56) | (variant_index << 48)` — reproducing schema-rust's
    /// legacy `ShortHeader::value` byte layout (the root index in byte 7, the variant
    /// index in byte 6). The roots run in document order (input then output) and each
    /// root's variants in declaration order, so the derived constants match the legacy
    /// emitter's output byte-for-byte. A root or operation index that would not fit
    /// its one-byte field is the layout's genuine invariant and fails loudly.
    ///
    /// LEAN `short-header-derivation-mirrors-legacy`: the byte layout is reproduced
    /// from schema-rust's existing emitter rule, not authored here. Trigger to
    /// revisit: the short-header byte-layout review-later item settles a different
    /// layout, after which this derivation changes with it.
    fn short_header_module(&mut self, roots: &[InterfaceRoot]) -> Result<CoreItem, NomosError> {
        let u64_type = self.type_path(&["u64"]);
        let mut consts = Vec::new();
        for (root_index, root) in roots.iter().enumerate() {
            let root_byte = u8::try_from(root_index).map_err(|_| {
                NomosError::Generation(
                    "an interface root index exceeds the short-header layout's one-byte root field",
                )
            })?;
            for (variant_index, variant) in root.variants.iter().enumerate() {
                let variant_byte = u8::try_from(variant_index).map_err(|_| {
                    NomosError::Generation(
                        "an operation index exceeds the short-header layout's one-byte variant field",
                    )
                })?;
                let value = (u64::from(root_byte) << 56) | (u64::from(variant_byte) << 48);
                let const_name = self.short_header_const_name(root.name, variant.identifier())?;
                consts.push(CoreItem::Const(Const {
                    visibility: Visibility::Public,
                    attributes: Vec::new(),
                    name: const_name,
                    type_reference: u64_type.clone(),
                    value: Expression::IntegerLiteral(IntegerLiteral {
                        value: u128::from(value),
                        representation: IntegerRepresentation::Hexadecimal { minimum_digits: 16 },
                    }),
                }));
            }
        }
        let module_name = self.ident("short_header");
        let skip = self.rustfmt_skip();
        Ok(CoreItem::Module(Module {
            visibility: Visibility::Public,
            attributes: vec![skip],
            name: module_name,
            items: consts,
        }))
    }

    fn route_enum(&mut self, root: &InterfaceRoot) -> Result<CoreItem, NomosError> {
        let name = self.route_enum_name(root.name)?;
        let attributes = self.wire_enum_preamble();
        let variants = root
            .variants
            .iter()
            .map(|variant| Variant {
                name: variant.identifier(),
                payload: VariantPayload::Unit,
            })
            .collect();
        Ok(CoreItem::Enumeration(Enumeration {
            visibility: Visibility::Public,
            attributes,
            name,
            generics: Generics::none(),
            variants,
        }))
    }

    fn signal_operation_heads_impl(
        &mut self,
        request: &InterfaceRoot,
    ) -> Result<CoreItem, NomosError> {
        let self_type = TypeReference::Path(self.path_of(&[request.name]));
        let implemented_trait =
            TypeReference::Path(self.path(&["signal_frame", "SignalOperationHeads"]));
        let heads_name = self.ident("HEADS");
        let static_str = self.static_str();
        let heads_type = TypeReference::Reference(ReferenceType {
            lifetime: Some(self.ident("static")),
            mutability: ReferenceMutability::Shared,
            referent: Box::new(TypeReference::Slice(SliceType {
                element: Box::new(static_str),
            })),
        });
        let mut elements = Vec::with_capacity(request.variants.len());
        for variant in &request.variants {
            elements.push(Expression::StringLiteral(
                self.resolved_text(variant.identifier())?,
            ));
        }
        let heads_value = Expression::Reference(ReferenceExpression {
            referent: Box::new(Expression::Array(ArrayExpression { elements })),
        });
        let heads = ImplItem::AssociatedConst(Const {
            visibility: Visibility::Private,
            attributes: Vec::new(),
            name: heads_name,
            type_reference: heads_type,
            value: heads_value,
        });
        let skip = self.rustfmt_skip();
        Ok(self.trait_impl(vec![skip], implemented_trait, self_type, vec![heads]))
    }

    // ---- wire-contract vocabulary builders --------------------------------------

    /// `#[rustfmt::skip] const SIGNAL_SHORT_HEADER_BYTE_COUNT: usize = 8;` — the
    /// little-endian short-header width shared by every codec body.
    fn short_header_byte_count_const(&mut self) -> CoreItem {
        let skip = self.rustfmt_skip();
        let name = self.ident("SIGNAL_SHORT_HEADER_BYTE_COUNT");
        let usize_type = self.type_path(&["usize"]);
        CoreItem::Const(Const {
            visibility: Visibility::Private,
            attributes: vec![skip],
            name,
            type_reference: usize_type,
            value: Expression::IntegerLiteral(IntegerLiteral {
                value: u128::from(SHORT_HEADER_BYTE_COUNT),
                representation: IntegerRepresentation::Decimal,
            }),
        })
    }

    /// The `SignalFrameError` enum — the codec's fallible result. A tuple `UnknownHeader`
    /// carries the offending header; the rest are unit variants. It derives the value
    /// traits only (no rkyv/nota — it is a local error, never on the wire), so a caller
    /// can `unwrap`/compare it.
    fn signal_frame_error_enum(&mut self) -> CoreItem {
        let skip = self.rustfmt_skip();
        let derive = Attribute::Derive(DeriveGroup {
            paths: vec![
                self.path(&["Clone"]),
                self.path(&["Debug"]),
                self.path(&["PartialEq"]),
                self.path(&["Eq"]),
            ],
        });
        let name = self.ident("SignalFrameError");
        let u64_type = self.type_path(&["u64"]);
        let variants = vec![
            Variant {
                name: self.ident("ArchiveEncode"),
                payload: VariantPayload::Unit,
            },
            Variant {
                name: self.ident("ArchiveDecode"),
                payload: VariantPayload::Unit,
            },
            Variant {
                name: self.ident("FrameTooShort"),
                payload: VariantPayload::Unit,
            },
            Variant {
                name: self.ident("UnknownHeader"),
                payload: VariantPayload::Tuple(vec![u64_type]),
            },
            Variant {
                name: self.ident("HeaderMismatch"),
                payload: VariantPayload::Unit,
            },
        ];
        CoreItem::Enumeration(Enumeration {
            visibility: Visibility::Public,
            attributes: vec![skip, derive],
            name,
            generics: Generics::none(),
            variants,
        })
    }

    // ---- small expression/statement builders (verbs on the growing table) -------

    /// A value path expression over fixed name segments (`SignalFrameError::ArchiveEncode`).
    fn path_expr(&mut self, segments: &[&str]) -> Expression {
        Expression::Path(self.path(segments))
    }

    /// A method call `<receiver>.<method>(<arguments>)` with no turbofish.
    fn method_call(
        &mut self,
        receiver: Expression,
        method: &str,
        arguments: Vec<Expression>,
    ) -> Expression {
        let method = self.ident(method);
        Expression::MethodCall(MethodCall {
            receiver: Box::new(receiver),
            method,
            type_arguments: Vec::new(),
            arguments,
        })
    }

    /// The `?` try operator over a fallible expression.
    fn try_expr(&self, inner: Expression) -> Expression {
        Expression::Try(TryExpression {
            inner: Box::new(inner),
        })
    }

    /// The discarding closure `|_| <body>` the `.map_err` arms use.
    fn closure_discard(&self, body: Expression) -> Expression {
        Expression::Closure(ClosureExpression {
            parameters: vec![PatternElement::Wildcard],
            body: Box::new(body),
        })
    }

    /// A shared reference `&<referent>`.
    fn reference_expr(&self, referent: Expression) -> Expression {
        Expression::Reference(ReferenceExpression {
            referent: Box::new(referent),
        })
    }

    /// A `let <binding> <name> = <value>;` statement.
    fn let_stmt(&mut self, binding: LetBinding, name: &str, value: Expression) -> Statement {
        let name = self.ident(name);
        Statement::Let(LetStatement {
            binding,
            name,
            value,
        })
    }

    /// The `Result<<ok>, <err>>` return type.
    fn result_type(&mut self, ok: TypeReference, err: TypeReference) -> TypeReference {
        let head = self.path(&["Result"]);
        TypeReference::Application(TypeApplication {
            head,
            arguments: vec![ok, err],
        })
    }

    /// The `SignalFrameError` type — the error half of every codec return.
    fn signal_frame_error_type(&mut self) -> TypeReference {
        self.type_path(&["SignalFrameError"])
    }

    /// The `&[u8]` byte-slice parameter type of `decode_signal_frame`.
    fn byte_slice_type(&mut self) -> TypeReference {
        let u8_type = self.type_path(&["u8"]);
        TypeReference::Reference(ReferenceType {
            lifetime: None,
            mutability: ReferenceMutability::Shared,
            referent: Box::new(TypeReference::Slice(SliceType {
                element: Box::new(u8_type),
            })),
        })
    }

    // ---- wire exchange codec builders -------------------------------------------

    /// The pattern that matches one interface variant on `self`: `Self::Record(_)` for
    /// a payload-carrying operation, or the unit path `Self::Version` for a unit one —
    /// the payload/no-payload special case dissolved by reading the variant's payload.
    fn self_variant_pattern(&mut self, variant: &CoreVariant) -> Pattern {
        let self_ident = self.self_ident();
        let path = self.path_of(&[self_ident, variant.identifier()]);
        match variant.payload() {
            Some(_) => Pattern::TupleVariant(TupleVariantPattern {
                path,
                elements: vec![PatternElement::Wildcard],
            }),
            None => Pattern::Path(path),
        }
    }

    /// The codec `impl <Root> { route / short_header / route_from_short_header /
    /// encode_signal_frame / decode_signal_frame }`.
    fn codec_impl(&mut self, root: &InterfaceRoot) -> Result<CoreItem, NomosError> {
        let self_type = TypeReference::Path(self.path_of(&[root.name]));
        let items = vec![
            self.route_method(root)?,
            self.short_header_method(root)?,
            self.route_from_short_header_method(root)?,
            self.encode_signal_frame_method(),
            self.decode_signal_frame_method(root)?,
        ];
        Ok(self.inherent_impl(self_type, items))
    }

    /// `pub fn route(&self) -> <Root>Route { match self { Self::V(_) => <Root>Route::V, … } }`.
    fn route_method(&mut self, root: &InterfaceRoot) -> Result<ImplItem, NomosError> {
        let route_enum = self.route_enum_name(root.name)?;
        let mut arms = Vec::with_capacity(root.variants.len());
        for variant in &root.variants {
            let pattern = self.self_variant_pattern(variant);
            let body = Expression::Path(self.path_of(&[route_enum, variant.identifier()]));
            arms.push(MatchArm { pattern, body });
        }
        let body = Expression::Match(Match {
            scrutinee: Box::new(Expression::Receiver),
            arms,
        });
        let name = self.ident("route");
        let return_type = TypeReference::Path(self.path_of(&[route_enum]));
        Ok(self.method(
            name,
            Visibility::Public,
            Some(Receiver::Reference),
            Vec::new(),
            Some(return_type),
            body,
        ))
    }

    /// `pub fn short_header(&self) -> u64 { match self { Self::V(_) => short_header::ROOT_V, … } }`.
    fn short_header_method(&mut self, root: &InterfaceRoot) -> Result<ImplItem, NomosError> {
        let short_header = self.ident("short_header");
        let mut arms = Vec::with_capacity(root.variants.len());
        for variant in &root.variants {
            let pattern = self.self_variant_pattern(variant);
            let const_name = self.short_header_const_name(root.name, variant.identifier())?;
            let body = Expression::Path(self.path_of(&[short_header, const_name]));
            arms.push(MatchArm { pattern, body });
        }
        let body = Expression::Match(Match {
            scrutinee: Box::new(Expression::Receiver),
            arms,
        });
        let name = self.ident("short_header");
        let return_type = self.type_path(&["u64"]);
        Ok(self.method(
            name,
            Visibility::Public,
            Some(Receiver::Reference),
            Vec::new(),
            Some(return_type),
            body,
        ))
    }

    /// `pub fn route_from_short_header(header: u64) -> Result<<Root>Route, SignalFrameError>`
    /// — the enumerated arms map each known header to `Ok(<Root>Route::V)`, and the
    /// wildcard arm rejects an unknown header with `Err(SignalFrameError::UnknownHeader(header))`.
    fn route_from_short_header_method(
        &mut self,
        root: &InterfaceRoot,
    ) -> Result<ImplItem, NomosError> {
        let short_header = self.ident("short_header");
        let route_enum = self.route_enum_name(root.name)?;
        let header = self.ident("header");
        let mut arms = Vec::with_capacity(root.variants.len() + 1);
        for variant in &root.variants {
            let const_name = self.short_header_const_name(root.name, variant.identifier())?;
            let pattern = Pattern::Path(self.path_of(&[short_header, const_name]));
            let route_value = Expression::Path(self.path_of(&[route_enum, variant.identifier()]));
            let body = self.call_path(&["Ok"], vec![route_value]);
            arms.push(MatchArm { pattern, body });
        }
        // _ => Err(SignalFrameError::UnknownHeader(header))
        let header_value = Expression::Path(self.path_of(&[header]));
        let unknown = self.call_path(&["SignalFrameError", "UnknownHeader"], vec![header_value]);
        let wildcard_body = self.call_path(&["Err"], vec![unknown]);
        arms.push(MatchArm {
            pattern: Pattern::Wildcard,
            body: wildcard_body,
        });
        let scrutinee = Expression::Path(self.path_of(&[header]));
        let body = Expression::Match(Match {
            scrutinee: Box::new(scrutinee),
            arms,
        });
        let parameter = Parameter {
            name: header,
            type_reference: self.type_path(&["u64"]),
        };
        let route_type = TypeReference::Path(self.path_of(&[route_enum]));
        let error_type = self.signal_frame_error_type();
        let return_type = self.result_type(route_type, error_type);
        let name = self.ident("route_from_short_header");
        Ok(self.method(
            name,
            Visibility::Public,
            None,
            vec![parameter],
            Some(return_type),
            body,
        ))
    }

    /// `pub fn encode_signal_frame(&self) -> Result<Vec<u8>, SignalFrameError>` — rkyv
    /// the payload, then prepend the little-endian short header. Mirrors the wire the
    /// hand-written contracts speak (header bytes then archive).
    fn encode_signal_frame_method(&mut self) -> ImplItem {
        // let archive = rkyv::to_bytes::<rkyv::rancor::Error>(self)
        //     .map_err(|_| SignalFrameError::ArchiveEncode)?;
        let rancor_error = self.type_path(&["rkyv", "rancor", "Error"]);
        let to_bytes = self.call_path_turbofish(
            &["rkyv", "to_bytes"],
            vec![rancor_error],
            vec![Expression::Receiver],
        );
        let archive_error = self.path_expr(&["SignalFrameError", "ArchiveEncode"]);
        let closure = self.closure_discard(archive_error);
        let map_err = self.method_call(to_bytes, "map_err", vec![closure]);
        let archive_value = self.try_expr(map_err);
        let statement_archive = self.let_stmt(LetBinding::Immutable, "archive", archive_value);

        // let mut frame = self.short_header().to_le_bytes().to_vec();
        let short_header_call = self.method_call(Expression::Receiver, "short_header", Vec::new());
        let to_le_bytes = self.method_call(short_header_call, "to_le_bytes", Vec::new());
        let to_vec = self.method_call(to_le_bytes, "to_vec", Vec::new());
        let statement_frame = self.let_stmt(LetBinding::Mutable, "frame", to_vec);

        // frame.extend_from_slice(&archive);
        let archive_path = self.path_expr(&["archive"]);
        let archive_reference = self.reference_expr(archive_path);
        let frame_path = self.path_expr(&["frame"]);
        let extend = self.method_call(frame_path, "extend_from_slice", vec![archive_reference]);
        let statement_extend = Statement::Expression(extend);

        // Ok(frame)
        let frame_tail = self.path_expr(&["frame"]);
        let tail = self.call_path(&["Ok"], vec![frame_tail]);

        let block = Block {
            statements: vec![statement_archive, statement_frame, statement_extend],
            tail_expression: tail,
        };
        let u8_type = self.type_path(&["u8"]);
        let vec_head = self.path(&["Vec"]);
        let vec_u8 = TypeReference::Application(TypeApplication {
            head: vec_head,
            arguments: vec![u8_type],
        });
        let error_type = self.signal_frame_error_type();
        let return_type = self.result_type(vec_u8, error_type);
        let name = self.ident("encode_signal_frame");
        self.method_block(
            name,
            Visibility::Public,
            Some(Receiver::Reference),
            Vec::new(),
            Some(return_type),
            block,
        )
    }

    /// `pub fn decode_signal_frame(frame: &[u8]) -> Result<(<Root>Route, Self), SignalFrameError>`
    /// — split the little-endian short header, rkyv the remainder, and reject a header
    /// that the decoded value does not re-derive. Written with `.ok_or(…)?` in place of
    /// an early-return `if`, so the modeled statement vocabulary expresses it directly.
    fn decode_signal_frame_method(&mut self, root: &InterfaceRoot) -> Result<ImplItem, NomosError> {
        // let header = u64::from_le_bytes(
        //     frame.get(..SIGNAL_SHORT_HEADER_BYTE_COUNT)
        //         .ok_or(SignalFrameError::FrameTooShort)?
        //         .try_into()
        //         .map_err(|_| SignalFrameError::FrameTooShort)?,
        // );
        let byte_count = self.path_expr(&["SIGNAL_SHORT_HEADER_BYTE_COUNT"]);
        let range_to = Expression::Range(RangeExpression {
            start: None,
            end: Some(Box::new(byte_count)),
        });
        let frame_get = self.path_expr(&["frame"]);
        let get = self.method_call(frame_get, "get", vec![range_to]);
        let frame_too_short = self.path_expr(&["SignalFrameError", "FrameTooShort"]);
        let ok_or = self.method_call(get, "ok_or", vec![frame_too_short]);
        let ok_or_try = self.try_expr(ok_or);
        let try_into = self.method_call(ok_or_try, "try_into", Vec::new());
        let frame_too_short_two = self.path_expr(&["SignalFrameError", "FrameTooShort"]);
        let try_into_closure = self.closure_discard(frame_too_short_two);
        let try_into_map_err = self.method_call(try_into, "map_err", vec![try_into_closure]);
        let header_bytes = self.try_expr(try_into_map_err);
        let from_le_bytes = self.call_path(&["u64", "from_le_bytes"], vec![header_bytes]);
        let statement_header = self.let_stmt(LetBinding::Immutable, "header", from_le_bytes);

        // let route = Self::route_from_short_header(header)?;
        let header_argument = self.path_expr(&["header"]);
        let route_call =
            self.call_path(&["Self", "route_from_short_header"], vec![header_argument]);
        let route_value = self.try_expr(route_call);
        let statement_route = self.let_stmt(LetBinding::Immutable, "route", route_value);

        // let value = rkyv::from_bytes::<Self, rkyv::rancor::Error>(
        //     &frame[SIGNAL_SHORT_HEADER_BYTE_COUNT..],
        // )
        // .map_err(|_| SignalFrameError::ArchiveDecode)?;
        let self_argument = self.self_type();
        let rancor_error = self.type_path(&["rkyv", "rancor", "Error"]);
        let byte_count_from = self.path_expr(&["SIGNAL_SHORT_HEADER_BYTE_COUNT"]);
        let range_from = Expression::Range(RangeExpression {
            start: Some(Box::new(byte_count_from)),
            end: None,
        });
        let frame_base = self.path_expr(&["frame"]);
        let index = Expression::Index(IndexExpression {
            base: Box::new(frame_base),
            index: Box::new(range_from),
        });
        let index_reference = self.reference_expr(index);
        let from_bytes = self.call_path_turbofish(
            &["rkyv", "from_bytes"],
            vec![self_argument, rancor_error],
            vec![index_reference],
        );
        let archive_decode = self.path_expr(&["SignalFrameError", "ArchiveDecode"]);
        let decode_closure = self.closure_discard(archive_decode);
        let decode_map_err = self.method_call(from_bytes, "map_err", vec![decode_closure]);
        let value_value = self.try_expr(decode_map_err);
        let statement_value = self.let_stmt(LetBinding::Immutable, "value", value_value);

        // let expected = value.short_header();
        let value_receiver = self.path_expr(&["value"]);
        let expected_call = self.method_call(value_receiver, "short_header", Vec::new());
        let statement_expected = self.let_stmt(LetBinding::Immutable, "expected", expected_call);

        // let value = expected.eq(&header).then_some(value)
        //     .ok_or(SignalFrameError::HeaderMismatch)?;
        let expected_receiver = self.path_expr(&["expected"]);
        let header_reference_inner = self.path_expr(&["header"]);
        let header_reference = self.reference_expr(header_reference_inner);
        let equals = self.method_call(expected_receiver, "eq", vec![header_reference]);
        let value_argument = self.path_expr(&["value"]);
        let then_some = self.method_call(equals, "then_some", vec![value_argument]);
        let mismatch = self.path_expr(&["SignalFrameError", "HeaderMismatch"]);
        let checked_ok_or = self.method_call(then_some, "ok_or", vec![mismatch]);
        let checked_value = self.try_expr(checked_ok_or);
        let statement_checked = self.let_stmt(LetBinding::Immutable, "value", checked_value);

        // Ok((route, value))
        let route_element = self.path_expr(&["route"]);
        let value_element = self.path_expr(&["value"]);
        let pair = Expression::Tuple(TupleExpression {
            elements: vec![route_element, value_element],
        });
        let tail = self.call_path(&["Ok"], vec![pair]);

        let block = Block {
            statements: vec![
                statement_header,
                statement_route,
                statement_value,
                statement_expected,
                statement_checked,
            ],
            tail_expression: tail,
        };

        let route_enum = self.route_enum_name(root.name)?;
        let route_type = TypeReference::Path(self.path_of(&[route_enum]));
        let self_type = self.self_type();
        let pair_type = TypeReference::Tuple(TupleType {
            elements: vec![route_type, self_type],
        });
        let error_type = self.signal_frame_error_type();
        let return_type = self.result_type(pair_type, error_type);
        let frame_parameter = Parameter {
            name: self.ident("frame"),
            type_reference: self.byte_slice_type(),
        };
        let name = self.ident("decode_signal_frame");
        Ok(self.method_block(
            name,
            Visibility::Public,
            None,
            vec![frame_parameter],
            Some(return_type),
            block,
        ))
    }

    // ---- wire exchange envelope builders ----------------------------------------

    /// `#[rustfmt::skip] impl signal_frame::RequestPayload for <Root> {}` — the empty
    /// marker impl that admits the request root onto the exchange envelope.
    fn request_payload_impl(&mut self, request: Identifier) -> CoreItem {
        let self_type = TypeReference::Path(self.path_of(&[request]));
        let implemented_trait =
            TypeReference::Path(self.path(&["signal_frame", "RequestPayload"]));
        let skip = self.rustfmt_skip();
        self.trait_impl(vec![skip], implemented_trait, self_type, Vec::new())
    }

    /// `#[rustfmt::skip] impl signal_frame::LogVariant for <Root> { fn log_variant(&self)
    /// -> u64 { self.short_header() } }` — the log discriminant the frame log reads,
    /// delegating to the codec's `short_header`.
    fn log_variant_impl(&mut self, request: Identifier) -> CoreItem {
        let self_type = TypeReference::Path(self.path_of(&[request]));
        let implemented_trait = TypeReference::Path(self.path(&["signal_frame", "LogVariant"]));
        let body = self.method_call(Expression::Receiver, "short_header", Vec::new());
        let name = self.ident("log_variant");
        let return_type = self.type_path(&["u64"]);
        let method = self.method(
            name,
            Visibility::Private,
            Some(Receiver::Reference),
            Vec::new(),
            Some(return_type),
            body,
        );
        let skip = self.rustfmt_skip();
        self.trait_impl(vec![skip], implemented_trait, self_type, vec![method])
    }

    /// A `#[rustfmt::skip] pub type <name> = signal_frame::<target><arguments>;` envelope
    /// alias — `Frame` / `FrameBody` over `ExchangeFrame` / `ExchangeFrameBody` (the
    /// ordinary two-way leg), and the `Request` / `ReplyEnvelope` / `RequestBuilder`
    /// aliases over the request or reply root.
    fn frame_alias(&mut self, name: &str, target: &str, arguments: &[Identifier]) -> CoreItem {
        let skip = self.rustfmt_skip();
        let name = self.ident(name);
        let head = self.path(&["signal_frame", target]);
        let mut argument_types = Vec::with_capacity(arguments.len());
        for argument in arguments {
            argument_types.push(TypeReference::Path(self.path_of(&[*argument])));
        }
        let target = TypeReference::Application(TypeApplication {
            head,
            arguments: argument_types,
        });
        CoreItem::Alias(Alias {
            visibility: Visibility::Public,
            attributes: vec![skip],
            name,
            generics: Generics::none(),
            target,
        })
    }

    /// `signal_frame::ShortHeader::new(self.short_header())` — the short-header value
    /// both envelope constructors prepend, derived from the codec's `short_header`.
    fn short_header_new_value(&mut self) -> Expression {
        let short_header_call = self.method_call(Expression::Receiver, "short_header", Vec::new());
        self.call_path(
            &["signal_frame", "ShortHeader", "new"],
            vec![short_header_call],
        )
    }

    /// The `exchange: signal_frame::ExchangeIdentifier` parameter both envelope
    /// constructors take.
    fn exchange_parameter(&mut self) -> Parameter {
        let name = self.ident("exchange");
        let type_reference = self.type_path(&["signal_frame", "ExchangeIdentifier"]);
        Parameter {
            name,
            type_reference,
        }
    }

    /// A struct-variant literal in shorthand-field form over interned segments:
    /// `FrameBody::Request { exchange, request }`. Every field is shorthand (the field
    /// name and the in-scope binding coincide), so each initializer carries a `None`
    /// value.
    fn struct_literal_shorthand(
        &mut self,
        path_segments: &[&str],
        field_names: &[&str],
    ) -> Expression {
        let path = self.path(path_segments);
        let mut fields = Vec::with_capacity(field_names.len());
        for field in field_names {
            let name = self.ident(field);
            fields.push(FieldInitializer { name, value: None });
        }
        Expression::StructLiteral(StructLiteral { path, fields })
    }

    /// `#[rustfmt::skip] impl <Root> { pub fn into_frame(self, exchange:
    /// signal_frame::ExchangeIdentifier) -> Frame { … } }` — the request constructor
    /// that wraps the payload into a `FrameBody::Request` exchange frame.
    fn into_frame_impl(&mut self, request: Identifier) -> CoreItem {
        let short_header_value = self.short_header_new_value();
        let statement_short_header =
            self.let_stmt(LetBinding::Immutable, "short_header", short_header_value);

        // let request = signal_frame::Request::from_payload(self);
        let request_value = self.call_path(
            &["signal_frame", "Request", "from_payload"],
            vec![Expression::Receiver],
        );
        let statement_request = self.let_stmt(LetBinding::Immutable, "request", request_value);

        // Frame::with_short_header(short_header, FrameBody::Request { exchange, request })
        let short_header_argument = self.path_expr(&["short_header"]);
        let body_literal =
            self.struct_literal_shorthand(&["FrameBody", "Request"], &["exchange", "request"]);
        let tail = self.call_path(
            &["Frame", "with_short_header"],
            vec![short_header_argument, body_literal],
        );

        let block = Block {
            statements: vec![statement_short_header, statement_request],
            tail_expression: tail,
        };
        let exchange_parameter = self.exchange_parameter();
        let return_type = self.type_path(&["Frame"]);
        let name = self.ident("into_frame");
        let method = self.method_block(
            name,
            Visibility::Public,
            Some(Receiver::Value),
            vec![exchange_parameter],
            Some(return_type),
            block,
        );
        let self_type = TypeReference::Path(self.path_of(&[request]));
        self.inherent_impl(self_type, vec![method])
    }

    /// `#[rustfmt::skip] impl <Root> { pub fn into_reply_frame(self, exchange:
    /// signal_frame::ExchangeIdentifier) -> Frame { … } }` — the reply constructor that
    /// wraps the payload into a committed single-`Ok` `FrameBody::Reply` exchange frame.
    fn into_reply_frame_impl(&mut self, reply: Identifier) -> CoreItem {
        let short_header_value = self.short_header_new_value();
        let statement_short_header =
            self.let_stmt(LetBinding::Immutable, "short_header", short_header_value);

        // let reply = signal_frame::Reply::committed(
        //     signal_frame::NonEmpty::single(signal_frame::SubReply::Ok(self)),
        // );
        let ok = self.call_path(
            &["signal_frame", "SubReply", "Ok"],
            vec![Expression::Receiver],
        );
        let single = self.call_path(&["signal_frame", "NonEmpty", "single"], vec![ok]);
        let committed = self.call_path(&["signal_frame", "Reply", "committed"], vec![single]);
        let statement_reply = self.let_stmt(LetBinding::Immutable, "reply", committed);

        // Frame::with_short_header(short_header, FrameBody::Reply { exchange, reply })
        let short_header_argument = self.path_expr(&["short_header"]);
        let body_literal =
            self.struct_literal_shorthand(&["FrameBody", "Reply"], &["exchange", "reply"]);
        let tail = self.call_path(
            &["Frame", "with_short_header"],
            vec![short_header_argument, body_literal],
        );

        let block = Block {
            statements: vec![statement_short_header, statement_reply],
            tail_expression: tail,
        };
        let exchange_parameter = self.exchange_parameter();
        let return_type = self.type_path(&["Frame"]);
        let name = self.ident("into_reply_frame");
        let method = self.method_block(
            name,
            Visibility::Public,
            Some(Receiver::Value),
            vec![exchange_parameter],
            Some(return_type),
            block,
        );
        let self_type = TypeReference::Path(self.path_of(&[reply]));
        self.inherent_impl(self_type, vec![method])
    }

    // ---- class D: trace support -------------------------------------------------

    fn generate_trace_support(&mut self, schema: &CoreSchema) -> Result<Vec<CoreItem>, NomosError> {
        let roots = Self::interface_roots(schema)?;
        if roots.is_empty() {
            return Err(NomosError::Generation(
                "trace support needs interface roots, the schema has none",
            ));
        }
        Ok(vec![
            self.signal_object_name_enum(&roots)?,
            self.signal_object_name_impl(&roots)?,
            self.object_name_enum()?,
            self.trace_event_declaration()?,
            self.object_name_impl()?,
            self.trace_event_impl()?,
        ])
    }

    /// The `pub struct TraceEvent(pub ObjectName);` tuple-struct declaration — a
    /// public newtype whose single tuple field is itself `pub` (layout-4 tuple-field
    /// visibility). It carries the same wire-enum preamble the trace enums carry, and
    /// sits between the `ObjectName` enum and the `impl ObjectName` in the golden's
    /// document order.
    fn trace_event_declaration(&mut self) -> Result<CoreItem, NomosError> {
        let name = self.ident("TraceEvent");
        let object_name = self.ident("ObjectName");
        let attributes = self.wire_enum_preamble();
        Ok(CoreItem::Newtype(Newtype {
            visibility: Visibility::Public,
            attributes,
            name,
            wrapped_visibility: Visibility::Public,
            wrapped: TypeReference::Path(self.path_of(&[object_name])),
        }))
    }

    fn signal_object_name_enum(&mut self, roots: &[InterfaceRoot]) -> Result<CoreItem, NomosError> {
        let name = self.ident("SignalObjectName");
        let attributes = self.wire_enum_preamble();
        let mut variants = Vec::with_capacity(roots.len());
        for root in roots {
            let route_enum = self.route_enum_name(root.name)?;
            variants.push(Variant {
                name: root.name,
                payload: VariantPayload::Tuple(vec![TypeReference::Path(
                    self.path_of(&[route_enum]),
                )]),
            });
        }
        Ok(CoreItem::Enumeration(Enumeration {
            visibility: Visibility::Public,
            attributes,
            name,
            generics: Generics::none(),
            variants,
        }))
    }

    fn signal_object_name_impl(&mut self, roots: &[InterfaceRoot]) -> Result<CoreItem, NomosError> {
        let signal_object_name = self.ident("SignalObjectName");
        let self_type = TypeReference::Path(self.path_of(&[signal_object_name]));
        let route_binding = self.ident("route");
        let self_ident = self.self_ident();
        let mut outer_arms = Vec::with_capacity(roots.len());
        for root in roots {
            let route_enum = self.route_enum_name(root.name)?;
            let mut inner_arms = Vec::with_capacity(root.variants.len());
            for variant in &root.variants {
                let literal = self.signal_object_name_literal(root.name, variant.identifier())?;
                inner_arms.push(MatchArm {
                    pattern: Pattern::Path(self.path_of(&[route_enum, variant.identifier()])),
                    body: Expression::StringLiteral(literal),
                });
            }
            let inner_match = Expression::Match(Match {
                scrutinee: Box::new(Expression::Path(self.path_of(&[route_binding]))),
                arms: inner_arms,
            });
            outer_arms.push(MatchArm {
                pattern: Pattern::TupleVariant(TupleVariantPattern {
                    path: self.path_of(&[self_ident, root.name]),
                    elements: vec![PatternElement::Binding(route_binding)],
                }),
                body: inner_match,
            });
        }
        let body = Expression::Match(Match {
            scrutinee: Box::new(Expression::Receiver),
            arms: outer_arms,
        });
        let name = self.ident("name");
        let return_type = self.static_str();
        let name_method = self.method(
            name,
            Visibility::Public,
            Some(Receiver::Value),
            Vec::new(),
            Some(return_type),
            body,
        );
        Ok(self.inherent_impl(self_type, vec![name_method]))
    }

    fn object_name_enum(&mut self) -> Result<CoreItem, NomosError> {
        let name = self.ident("ObjectName");
        let attributes = self.wire_enum_preamble();
        let signal = self.ident("Signal");
        let signal_object_name = self.ident("SignalObjectName");
        Ok(CoreItem::Enumeration(Enumeration {
            visibility: Visibility::Public,
            attributes,
            name,
            generics: Generics::none(),
            variants: vec![Variant {
                name: signal,
                payload: VariantPayload::Tuple(vec![TypeReference::Path(
                    self.path_of(&[signal_object_name]),
                )]),
            }],
        }))
    }

    fn object_name_impl(&mut self) -> Result<CoreItem, NomosError> {
        let object_name = self.ident("ObjectName");
        let self_type = TypeReference::Path(self.path_of(&[object_name]));
        let object_name_binding = self.ident("object_name");
        let name_method_name = self.ident("name");
        let signal = self.ident("Signal");
        let self_ident = self.self_ident();
        let delegate = Expression::MethodCall(MethodCall {
            receiver: Box::new(Expression::Path(self.path_of(&[object_name_binding]))),
            method: name_method_name,
            type_arguments: Vec::new(),
            arguments: Vec::new(),
        });
        let arm = MatchArm {
            pattern: Pattern::TupleVariant(TupleVariantPattern {
                path: self.path_of(&[self_ident, signal]),
                elements: vec![PatternElement::Binding(object_name_binding)],
            }),
            body: delegate,
        };
        let body = Expression::Match(Match {
            scrutinee: Box::new(Expression::Receiver),
            arms: vec![arm],
        });
        let return_type = self.static_str();
        let name_method = self.method(
            name_method_name,
            Visibility::Public,
            Some(Receiver::Value),
            Vec::new(),
            Some(return_type),
            body,
        );
        Ok(self.inherent_impl(self_type, vec![name_method]))
    }

    fn trace_event_impl(&mut self) -> Result<CoreItem, NomosError> {
        let trace_event = self.ident("TraceEvent");
        let object_name = self.ident("ObjectName");
        let self_type = TypeReference::Path(self.path_of(&[trace_event]));
        let object_name_type = TypeReference::Path(self.path_of(&[object_name]));

        // new(object_name: ObjectName) -> Self { Self(object_name) }
        let new_name = self.ident("new");
        let object_name_param_name = self.ident("object_name");
        let new_parameter = Parameter {
            name: object_name_param_name,
            type_reference: object_name_type.clone(),
        };
        let self_return = self.self_type();
        let new_body = self.call_path(
            &["Self"],
            vec![Expression::Path(self.path_of(&[object_name_param_name]))],
        );
        let new_method = self.method(
            new_name,
            Visibility::Public,
            None,
            vec![new_parameter],
            Some(self_return),
            new_body,
        );

        // object_name(&self) -> ObjectName { self.0 }
        let object_name_accessor = self.ident("object_name");
        let object_name_body = self.self_field_zero();
        let object_name_method = self.method(
            object_name_accessor,
            Visibility::Public,
            Some(Receiver::Reference),
            Vec::new(),
            Some(object_name_type),
            object_name_body,
        );

        // name(&self) -> &'static str { self.0.name() }
        let name_name = self.ident("name");
        let name_return = self.static_str();
        let name_body = Expression::MethodCall(MethodCall {
            receiver: Box::new(self.self_field_zero()),
            method: name_name,
            type_arguments: Vec::new(),
            arguments: Vec::new(),
        });
        let name_method = self.method(
            name_name,
            Visibility::Public,
            Some(Receiver::Reference),
            Vec::new(),
            Some(name_return),
            name_body,
        );

        Ok(self.inherent_impl(self_type, vec![new_method, object_name_method, name_method]))
    }

    // ---- name synthesis (single home for every derived generation name) ---------

    /// The interned `Self` identifier — the keyword path head of every variant
    /// construction and pattern.
    fn self_ident(&mut self) -> Identifier {
        self.ident("Self")
    }

    /// The snake_case constructor/method name derived from a variant name
    /// (`RecordAccepted` → `record_accepted`), interned into the extended table.
    fn derived_snake_name(&mut self, variant: Identifier) -> Result<Identifier, NomosError> {
        let derived = self.names.resolve(variant)?.field_name();
        Ok(self.names.intern(Name::new(derived)))
    }

    /// The `SCREAMING_SNAKE` short-header const name `<ROOT>_<VARIANT>`
    /// (`Input` + `Record` → `INPUT_RECORD`).
    fn short_header_const_name(
        &mut self,
        root: Identifier,
        variant: Identifier,
    ) -> Result<Identifier, NomosError> {
        let root_screaming = self.names.resolve(root)?.screaming();
        let variant_screaming = self.names.resolve(variant)?.screaming();
        let name = format!("{root_screaming}_{variant_screaming}");
        Ok(self.names.intern(Name::new(name)))
    }

    /// The route-enum name `<Root>Route` (`Input` → `InputRoute`).
    fn route_enum_name(&mut self, root: Identifier) -> Result<Identifier, NomosError> {
        let root_name = self.names.resolve(root)?.as_str().to_owned();
        Ok(self.names.intern(Name::new(format!("{root_name}Route"))))
    }

    /// The trace object-name string literal `Signal<Root><Variant>`
    /// (`Input`, `Record` → `SignalInputRecord`). Literal content, hashed data, never
    /// an interned name.
    fn signal_object_name_literal(
        &mut self,
        root: Identifier,
        variant: Identifier,
    ) -> Result<String, NomosError> {
        let root_name = self.names.resolve(root)?.as_str().to_owned();
        let variant_name = self.names.resolve(variant)?.as_str().to_owned();
        Ok(format!("Signal{root_name}{variant_name}"))
    }

    /// The resolved text of an identifier as a string literal's content (a variant
    /// head name in the `HEADS` array).
    fn resolved_text(&self, identifier: Identifier) -> Result<String, NomosError> {
        Ok(self.names.resolve(identifier)?.as_str().to_owned())
    }
}

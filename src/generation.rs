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
    ArrayExpression, AssociatedType, Attribute, Block, Call, Callee, ConfigurationAttribute,
    ConfigurationPredicate, Const, CoreItem, DeriveGroup, Enumeration, Expression, Function,
    Generics, ImplBlock, ImplItem, ImplTraitType, IntegerLiteral, IntegerRepresentation, Match,
    MatchArm, MethodCall, Module, Parameter, PathNode, Pattern, PatternElement, QualifiedPath,
    Receiver, ReferenceExpression, ReferenceMutability, ReferenceType, SliceType, TupleFieldAccess,
    TupleVariantPattern, TypeApplication, TypeReference, Variant, VariantPayload, Visibility,
};
use core_schema::{CoreDeclaration, CoreReference, CoreSchema, CoreType, CoreVariant};
use name_table::{Identifier, Name};
use std::collections::BTreeMap;

use crate::engine::Evaluator;
use crate::error::NomosError;
use crate::template::{GenerationClass, WireContractStub};

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
            GenerationClass::WireContractStub(stub) => {
                self.generate_wire_contract_stub(schema, stub)
            }
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
                tail_expression: body,
            },
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
        Expression::Call(Call { callee, arguments })
    }

    /// A call of a callee path built from interned identifiers (a variant path such
    /// as `Self::Record`).
    fn call_path_of(&self, segments: &[Identifier], arguments: Vec<Expression>) -> Expression {
        Expression::Call(Call {
            callee: Callee::Path(self.path_of(segments)),
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

    // ---- class C: wire contract stub --------------------------------------------

    fn generate_wire_contract_stub(
        &mut self,
        schema: &CoreSchema,
        stub: &WireContractStub,
    ) -> Result<Vec<CoreItem>, NomosError> {
        let roots = Self::interface_roots(schema)?;
        if roots.is_empty() {
            return Err(NomosError::Generation(
                "the wire contract stub needs interface roots, the schema has none",
            ));
        }
        let mut items = Vec::new();
        // The short_header const module.
        items.push(self.short_header_module(&roots, stub)?);
        // The route enums, one per root.
        for root in &roots {
            items.push(self.route_enum(root)?);
        }
        // The SignalOperationHeads associated-const impl for the request root (the
        // input): its HEADS is the request operations' head names.
        let request = roots.first().ok_or(NomosError::Generation(
            "the wire contract stub needs a request (input) root",
        ))?;
        items.push(self.signal_operation_heads_impl(request)?);
        Ok(items)
    }

    fn short_header_module(
        &mut self,
        roots: &[InterfaceRoot],
        stub: &WireContractStub,
    ) -> Result<CoreItem, NomosError> {
        let operation_count: usize = roots.iter().map(|root| root.variants.len()).sum();
        if stub.short_header_values.len() != operation_count {
            return Err(NomosError::Generation(
                "the wire stub's transcribed short-header count does not match the roots' operation count",
            ));
        }
        let u64_type = self.type_path(&["u64"]);
        let mut consts = Vec::with_capacity(operation_count);
        let mut values = stub.short_header_values.iter();
        for root in roots {
            for variant in &root.variants {
                let value = *values.next().ok_or(NomosError::Generation(
                    "ran out of transcribed short-header values",
                ))?;
                let const_name = self.short_header_const_name(root.name, variant.identifier())?;
                consts.push(CoreItem::Const(Const {
                    visibility: Visibility::Public,
                    attributes: Vec::new(),
                    name: const_name,
                    type_reference: u64_type.clone(),
                    value: Expression::IntegerLiteral(IntegerLiteral {
                        value: value as u128,
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
            self.object_name_impl()?,
            self.trace_event_impl()?,
        ])
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

//! The macro input model: the `{ Name Type }` meta-shape as data, and the values
//! bound into it when a macro is applied to a schema declaration.

use core_schema::{CoreField, CoreReference};
use name_table::Identifier;
use std::collections::BTreeMap;

/// A standard input meta-type — the small vocabulary a macro input describes, over
/// what a schema declaration actually carries (nomos-macro-model-v1 §2). An input
/// signature is an inline struct shape over these, not a binding to a named type.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetaType {
    /// A declared identifier — every declaration has one.
    Name,
    /// A type reference — a newtype's wrapped type, a field type, a generic
    /// argument.
    Type,
    /// An ordered vector of a struct's fields.
    Fields,
    /// An ordered vector of an enum's variants (a growth point; kept as a real
    /// sibling so the enum section has a home).
    Variants,
}

/// One input parameter: its in-scope binding name (the derived accessor, e.g.
/// `Name` yields `name`) and the meta-type it stands for. Body accessors resolve
/// against these binding names — there is no separate binder and no `declaration.`
/// prefix (the headless, sound-typing ruling).
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputParameter {
    /// The parameter's in-scope binding name, an identifier in the package's
    /// authoring NameTable (the snake_case accessor `name`/`type`/`fields`, or an
    /// explicit disambiguator where a meta-type repeats).
    pub binding: Identifier,
    /// The meta-type this parameter stands for.
    pub meta: MetaType,
}

/// A macro's input signature — the inline struct shape `{ … }` as data. An empty
/// signature is the unit input (`WireAttributes` takes `{ }`).
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct InputSignature {
    /// The ordered parameters.
    pub parameters: Vec<InputParameter>,
}

impl InputSignature {
    /// The unit input `{ }`.
    pub fn unit() -> Self {
        Self {
            parameters: Vec::new(),
        }
    }
}

/// A value bound into an input parameter when a macro is applied to a declaration.
/// It carries the schema-side substance verbatim (identifiers into the continuous
/// NameTable, and `core_schema` references cloned) — text never enters.
#[derive(Clone, Debug)]
pub enum MetaValue {
    /// A bound name (the declaration's identifier).
    Name(Identifier),
    /// A bound type reference (a newtype's wrapped reference).
    Type(CoreReference),
    /// A bound field vector (a struct's fields).
    Fields(Vec<CoreField>),
}

/// The bound input: each parameter's binding name mapped to the value the
/// declaration supplied. Keyed by the package-authoring identifier, exactly as the
/// template references bindings, so a template lookup is a direct map access.
#[derive(Clone, Debug, Default)]
pub struct BoundInput {
    bindings: BTreeMap<Identifier, MetaValue>,
}

impl BoundInput {
    /// An empty binding (for the unit input).
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a parameter's value.
    pub fn bind(&mut self, binding: Identifier, value: MetaValue) {
        self.bindings.insert(binding, value);
    }

    /// The value bound to a binding name, if any.
    pub fn value(&self, binding: Identifier) -> Option<&MetaValue> {
        self.bindings.get(&binding)
    }
}

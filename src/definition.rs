//! A single macro definition as data.

use name_table::Identifier;

use crate::identity::MacroKind;
use crate::meta::InputSignature;
use crate::template::ResultTemplate;

/// One macro, entirely as data: its stringless name, its kind, its typed input
/// signature (the `{ … }` meta-shape), and its result template (the quoted logos
/// skeleton with escapes). No behavior, no text — a macro is a value.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MacroDefinition {
    /// The macro's name, an identifier into the package's authoring NameTable
    /// (`WireNewtype`). Stringless: renaming is a NameTable edit.
    pub name: Identifier,
    /// Named or structural (and, if structural, which section it defaults).
    pub kind: MacroKind,
    /// The input meta-shape.
    pub input: InputSignature,
    /// The result template.
    pub template: ResultTemplate,
}

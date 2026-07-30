//! The legacy flat macro package retained as an execution fixture.

use std::collections::BTreeMap;

use content_identity::ContentHash;
use name_table::{Identifier, IdentifierNamespace, Name, NameTable};

use crate::definition::MacroDefinition;
use crate::domain::EncodedNomosDomain;
use crate::error::NomosError;
use crate::identity::{MacroIdentity, MacroKind, SectionDefault};
use crate::template::GenerationClass;

/// A legacy fixture revision over the flat loaded-definitions registry.
///
/// This value is not part of the production authored Nomos seal.
#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct PackageRevision(pub u32);

/// The legacy content-identified macro data: the revision, the macro table
/// keyed by minted identity, and the structural section defaults. This is the
/// pre-image of the package's content identity — it holds no names (only
/// identifiers), so the identity is rename-stable by construction.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct MacroDefinitions {
    /// The package revision.
    pub revision: PackageRevision,
    /// The macro table, keyed on minted identity.
    pub macros: BTreeMap<MacroIdentity, MacroDefinition>,
    /// The per-section structural defaults: a ethos declaration of each kind
    /// lowers via its section's default macro.
    pub structural_defaults: BTreeMap<SectionDefault, MacroIdentity>,
}

/// A legacy loaded macro package: the content-identified macro data plus its
/// authoring NameTable sibling. The NameTable is excluded from the fixture's
/// content identity.
///
/// Applying the package to a ethos (`MacroPackage::apply`) re-interns every
/// template-literal name through this sibling into the Logos-owned NameTable
/// composed with the Ethos compatibility slice, which preserves component-owned identifier namespaces at
/// runtime.
#[derive(Clone, Debug)]
pub struct MacroPackage {
    definitions: MacroDefinitions,
    names: NameTable,
    selection: Vec<GenerationClass>,
}

impl MacroPackage {
    /// An empty package at revision `revision`, with no enriched generation
    /// selection.
    pub fn new(revision: PackageRevision) -> Self {
        Self {
            definitions: MacroDefinitions {
                revision,
                macros: BTreeMap::new(),
                structural_defaults: BTreeMap::new(),
            },
            names: NameTable::new(IdentifierNamespace::Nomos),
            selection: Vec::new(),
        }
    }

    /// Intern an authoring name (a macro name, a binding name, or a literal in a
    /// template) into this package's NameTable, returning its identifier.
    pub fn author_name(&mut self, text: &str) -> Result<Identifier, NomosError> {
        Ok(self.names.intern(Name::new(text))?)
    }

    /// Register a macro, minting its identity. A structural macro is also recorded
    /// as its section's default. Ids are minted sequentially from zero, so the
    /// mint is deterministic from the table itself.
    pub fn register(&mut self, definition: MacroDefinition) -> MacroIdentity {
        let identity = MacroIdentity::new(self.definitions.macros.len() as u32);
        if let MacroKind::Structural(section) = definition.kind {
            self.definitions
                .structural_defaults
                .insert(section, identity);
        }
        self.definitions.macros.insert(identity, definition);
        identity
    }

    /// The authoring NameTable sibling.
    pub fn names(&self) -> &NameTable {
        &self.names
    }

    /// The content-identified macro data.
    pub fn definitions(&self) -> &MacroDefinitions {
        &self.definitions
    }

    /// The package revision.
    pub fn revision(&self) -> PackageRevision {
        self.definitions.revision
    }

    /// A macro by minted identity.
    pub fn definition(&self, identity: MacroIdentity) -> Option<&MacroDefinition> {
        self.definitions.macros.get(&identity)
    }

    /// The structural default macro for a declaration section, if the package
    /// defines one.
    pub fn structural_default(&self, section: SectionDefault) -> Option<MacroIdentity> {
        self.definitions.structural_defaults.get(&section).copied()
    }

    /// The enriched generation selection — the ordered generation classes the
    /// enriched apply runs after the per-declaration lowering. Empty for the plain
    /// and wire fixtures, so their behaviour is unchanged; the enriched fixture wires
    /// the class-A/B/C/D selection. Kept outside the content-identity pre-image (that
    /// is the stringless [`MacroDefinitions`] alone), so adding a selection never
    /// moves an existing package's identity.
    pub fn selection(&self) -> &[GenerationClass] {
        &self.selection
    }

    /// Replace the enriched generation selection, returning the package — the
    /// builder verb the enriched fixture uses after registering its structural
    /// defaults.
    pub fn with_selection(mut self, selection: Vec<GenerationClass>) -> Self {
        self.selection = selection;
        self
    }

    /// The package's content identity, over the stringless macro data with the
    /// NameTable excluded. Rename-stable: renaming a macro moves nothing here;
    /// changing a macro's kind, input, or template moves the identity.
    pub fn content_identity(&self) -> Result<ContentHash<EncodedNomosDomain>, NomosError> {
        Ok(ContentHash::of_core(&self.definitions)?)
    }
}

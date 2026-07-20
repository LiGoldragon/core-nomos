//! A single macro definition as data and its pre-expansion type check.

use name_table::Identifier;

use crate::error::NomosError;
use crate::identity::MacroIdentity;
use crate::identity::MacroKind;
use crate::meta::{InputSignature, MetaType};
use crate::template::{
    Escape, EscapeKind, ItemTemplate, ResultTemplate, Scalar, Sequence, SequenceItem, Splice,
    SpliceElement, TemplatePosition,
};

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

/// A data-bearing definition checker. It owns no text: it compares only the closed
/// template-position, escape-kind, and meta-type algebras before evaluation begins.
struct DefinitionChecker<'definition> {
    definition: &'definition MacroDefinition,
    recursive_invocations: Vec<MacroIdentity>,
}

impl<'definition> DefinitionChecker<'definition> {
    fn new(definition: &'definition MacroDefinition) -> Self {
        Self {
            definition,
            recursive_invocations: Vec::new(),
        }
    }

    fn check(mut self) -> Result<Vec<MacroIdentity>, NomosError> {
        match &self.definition.template {
            ResultTemplate::Item(item) => self.check_item(item)?,
            ResultTemplate::Attributes(attributes) => self.check_attributes(attributes)?,
        }
        Ok(self.recursive_invocations)
    }

    fn check_item(&mut self, item: &ItemTemplate) -> Result<(), NomosError> {
        match item {
            ItemTemplate::Newtype(template) => {
                self.check_attributes(&template.attributes)?;
                self.check_scalar(&template.name, TemplatePosition::Name, MetaType::Name)?;
                self.check_scalar(&template.wrapped, TemplatePosition::Type, MetaType::Type)
            }
            ItemTemplate::Struct(template) => {
                self.check_attributes(&template.attributes)?;
                self.check_scalar(&template.name, TemplatePosition::Name, MetaType::Name)?;
                self.check_fields(&template.fields)
            }
            ItemTemplate::Enumeration(template) => {
                self.check_attributes(&template.attributes)?;
                self.check_scalar(&template.name, TemplatePosition::Name, MetaType::Name)?;
                self.check_variants(&template.variants)
            }
        }
    }

    fn check_scalar<Literal>(
        &self,
        scalar: &Scalar<Literal>,
        position: TemplatePosition,
        expected: MetaType,
    ) -> Result<(), NomosError> {
        match scalar {
            Scalar::Literal(_) => Ok(()),
            Scalar::Escape(escape) => self.check_scalar_escape(escape, position, expected),
        }
    }

    fn check_scalar_escape(
        &self,
        escape: &Escape,
        position: TemplatePosition,
        expected: MetaType,
    ) -> Result<(), NomosError> {
        match escape {
            Escape::Realize(realize) => {
                self.check_binding(EscapeKind::Realize, realize.binding, expected)
            }
            Escape::Splice(_) => Err(NomosError::EscapePlacement {
                escape: EscapeKind::Splice,
                position,
            }),
        }
    }

    fn check_attributes<Literal>(
        &mut self,
        sequence: &Sequence<Literal>,
    ) -> Result<(), NomosError> {
        for item in &sequence.items {
            match item {
                SequenceItem::Literal(_) => {}
                SequenceItem::Escape(escape) => {
                    return Err(NomosError::EscapePlacement {
                        escape: escape.kind(),
                        position: TemplatePosition::AttributeElement,
                    });
                }
                SequenceItem::RecursiveInvoke(identity) => {
                    self.recursive_invocations.push(*identity)
                }
            }
        }
        Ok(())
    }

    fn check_fields<Literal>(&self, sequence: &Sequence<Literal>) -> Result<(), NomosError> {
        for item in &sequence.items {
            match item {
                SequenceItem::Literal(_) => {}
                SequenceItem::Escape(Escape::Splice(splice)) => {
                    self.check_splice(
                        splice,
                        TemplatePosition::FieldElement,
                        MetaType::Fields,
                        true,
                    )?;
                }
                SequenceItem::Escape(escape) => {
                    return Err(NomosError::EscapePlacement {
                        escape: escape.kind(),
                        position: TemplatePosition::FieldElement,
                    });
                }
                SequenceItem::RecursiveInvoke(_) => {
                    return Err(NomosError::RecursiveInvocation(
                        TemplatePosition::FieldElement,
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_variants<Literal>(&self, sequence: &Sequence<Literal>) -> Result<(), NomosError> {
        for item in &sequence.items {
            match item {
                SequenceItem::Literal(_) => {}
                SequenceItem::Escape(Escape::Splice(splice)) => {
                    self.check_splice(
                        splice,
                        TemplatePosition::VariantElement,
                        MetaType::Variants,
                        false,
                    )?;
                }
                SequenceItem::Escape(escape) => {
                    return Err(NomosError::EscapePlacement {
                        escape: escape.kind(),
                        position: TemplatePosition::VariantElement,
                    });
                }
                SequenceItem::RecursiveInvoke(_) => {
                    return Err(NomosError::RecursiveInvocation(
                        TemplatePosition::VariantElement,
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_splice(
        &self,
        splice: &Splice,
        position: TemplatePosition,
        expected: MetaType,
        field_projection: bool,
    ) -> Result<(), NomosError> {
        if !position.accepts_splice() {
            return Err(NomosError::EscapePlacement {
                escape: EscapeKind::Splice,
                position,
            });
        }
        self.check_binding(EscapeKind::Splice, splice.binding, expected)?;
        let projection_matches = matches!(
            (&splice.element, field_projection),
            (SpliceElement::Field { .. }, true) | (SpliceElement::Variant, false)
        );
        if projection_matches {
            Ok(())
        } else {
            Err(NomosError::EscapePlacement {
                escape: EscapeKind::Splice,
                position,
            })
        }
    }

    fn check_binding(
        &self,
        escape: EscapeKind,
        binding: crate::template::BindingRef,
        expected: MetaType,
    ) -> Result<(), NomosError> {
        let crate::template::BindingRef::Input(identifier) = binding;
        let actual = self
            .definition
            .input
            .meta_for(identifier)
            .ok_or(NomosError::DefinitionBinding(identifier))?;
        if actual == expected {
            Ok(())
        } else {
            Err(NomosError::EscapeBinding {
                escape,
                binding: identifier,
                expected,
                actual,
            })
        }
    }
}

impl MacroDefinition {
    /// Check every escape boundary in this definition before macro expansion. The
    /// returned identities are the separate recursive-invocation surface forms that
    /// the containing package must resolve.
    pub(crate) fn check(&self) -> Result<Vec<MacroIdentity>, NomosError> {
        DefinitionChecker::new(self).check()
    }
}

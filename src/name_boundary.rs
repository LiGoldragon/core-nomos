//! The NameTable/emission boundary for schema→logos lowering.
//!
//! Nomos evaluation carries only typed encoded values and encoded identifiers. This
//! boundary is the sole owner of NameTable reads, derived-name production, literal
//! remapping, and identifier allocation needed to project that typed result to the
//! logos identifier space. It deliberately contains the stringful work that does
//! not belong in the transform.

use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

use core_logos::{Attribute, PathNode, TypeApplication, TypeReference, standard_name_table};
use core_schema::{EncodedField, EncodedReference};
use name_table::{Identifier, IdentifierNamespace, Name, NameTable, NameTableError};

use crate::error::NomosError;
use crate::template::FieldNameRule;

/// The owned logos NameTable under construction, paired with the package's
/// authoring table. All text-bearing name operations are confined here.
pub(crate) struct NameTableBoundary<'package> {
    package_names: &'package NameTable,
    names: NameTable,
    deferred_allocation_error: Option<NameTableError>,
}

impl<'package> NameTableBoundary<'package> {
    /// Begin the Logos-owned slice and borrow the completed Schema and standard
    /// slices. Borrowing preserves their namespaced identifiers without copying or
    /// renumbering either source table.
    pub(crate) fn new(
        package_names: &'package NameTable,
        schema_names: &NameTable,
    ) -> Result<Self, NomosError> {
        let standard_names = standard_name_table()?;
        let names = NameTable::new(IdentifierNamespace::Logos)
            .compose(schema_names)?
            .compose(&standard_names)?;
        Ok(Self {
            package_names,
            names,
            deferred_allocation_error: None,
        })
    }

    /// Finish the boundary and return the completed logos table.
    pub(crate) fn into_names(self) -> Result<NameTable, NomosError> {
        if let Some(error) = self.deferred_allocation_error {
            return Err(error.into());
        }
        Ok(self.names)
    }

    /// Allocate a fixed projection identifier for an item-construction helper.
    /// Helpers that cannot themselves return a result record the typed failure;
    /// `into_names` returns it before a lowering is observable. This keeps the
    /// complete generator boundary fallible without a panic or fabricated text.
    pub(crate) fn intern(&mut self, name: Name) -> Identifier {
        match self.names.intern(name) {
            Ok(identifier) => identifier,
            Err(error) => {
                if self.deferred_allocation_error.is_none() {
                    self.deferred_allocation_error = Some(error);
                }
                Identifier::Logos(0)
            }
        }
    }

    /// Re-intern a template literal from the package's authoring table into the
    /// logos table.
    pub(crate) fn place_literal_name(
        &mut self,
        identifier: Identifier,
    ) -> Result<Identifier, NomosError> {
        let name = self.package_names.resolve(identifier)?.clone();
        Ok(self.names.intern(name)?)
    }

    /// Derive and allocate every field identifier for an ordered struct field
    /// group. The rule is a function only of field position and type; source names
    /// are read solely when the typed field-rule dispatch explicitly preserves one.
    pub(crate) fn field_names(
        &mut self,
        fields: &[EncodedField],
        rule: FieldNameRule,
    ) -> Result<Vec<Identifier>, NomosError> {
        let group_names = self.derive_group_names(fields)?;
        fields
            .iter()
            .zip(group_names)
            .map(|(field, group_name)| self.field_name(field, group_name, rule))
            .collect()
    }

    /// The deterministic Rust names for an ordered same-typed field group. This
    /// is name work, so its string derivation and allocation belong at the boundary,
    /// not in the typed schema→logos evaluator.
    fn derive_group_names(&self, fields: &[EncodedField]) -> Result<Vec<Name>, NomosError> {
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

    /// Apply a field-name selection rule after the group names have been derived.
    fn field_name(
        &mut self,
        field: &EncodedField,
        group_name: Name,
        rule: FieldNameRule,
    ) -> Result<Identifier, NomosError> {
        match rule {
            FieldNameRule::PreserveSchema => Ok(field.identifier()),
            FieldNameRule::AlwaysDeriveFromType => Ok(self.names.intern(group_name)?),
            FieldNameRule::FieldRuleDispatch => {
                if field.name_is_derivable(&self.names)? {
                    Ok(self.names.intern(group_name)?)
                } else {
                    Ok(field.identifier())
                }
            }
        }
    }

    /// Translate a typed schema reference to its logos type while allocating the
    /// required fixed projection names at the NameTable boundary.
    pub(crate) fn lower_reference(
        &mut self,
        reference: &EncodedReference,
    ) -> Result<TypeReference, NomosError> {
        match reference {
            // Scalar leaves project directly to their canonical Rust types. The
            // Logos standard slice supplies the fixed path identifiers; Nomos does
            // not create scalar aliases in the generated module.
            EncodedReference::Integer => Ok(TypeReference::Path(PathNode {
                segments: vec![core_logos::UNSIGNED_64],
            })),
            EncodedReference::String => Ok(TypeReference::Path(PathNode {
                segments: vec![
                    core_logos::STANDARD_LIBRARY,
                    core_logos::STRING_MODULE,
                    core_logos::STRING,
                ],
            })),
            EncodedReference::Boolean => Ok(TypeReference::Path(PathNode {
                segments: vec![core_logos::RUST_BOOLEAN],
            })),
            EncodedReference::Bytes => Ok(TypeReference::Application(TypeApplication {
                head: PathNode {
                    segments: vec![core_logos::VECTOR],
                },
                arguments: vec![TypeReference::Path(self.leaf_path("u8")?)],
            })),
            EncodedReference::Plain(identifier) => Ok(TypeReference::Path(PathNode {
                segments: vec![*identifier],
            })),
            EncodedReference::SingleTypeApplication {
                projection,
                argument,
            } => {
                let head = self.single_projection_head(projection);
                let argument = self.lower_reference(argument)?;
                Ok(TypeReference::Application(TypeApplication {
                    head: self.leaf_path(head)?,
                    arguments: vec![argument],
                }))
            }
            EncodedReference::MultiTypeApplication {
                projection,
                arguments,
            } => {
                let head = self.multi_projection_head(projection);
                let arguments = arguments
                    .iter()
                    .map(|argument| self.lower_reference(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeReference::Application(TypeApplication {
                    head: self.leaf_path(head)?,
                    arguments,
                }))
            }
            EncodedReference::ValueApplication { .. } => Err(NomosError::UnsupportedReference(
                "a byte-length value application has no EncodedLogos type-argument home",
            )),
        }
    }

    /// Remove `Copy` from a derive group only when a payload makes that trait
    /// invalid. This requires resolving the identifier and therefore belongs at the
    /// NameTable boundary rather than in typed template evaluation.
    pub(crate) fn remove_copy_derive(
        &self,
        attributes: &mut [Attribute],
    ) -> Result<(), NomosError> {
        for attribute in attributes {
            if let Attribute::Derive(group) = attribute {
                group.paths.retain(|path| match path.resolve(&self.names) {
                    Ok(segments) => segments.as_slice() != [Name::new("Copy")],
                    Err(_) => true,
                });
            }
        }
        Ok(())
    }

    /// Derive and allocate a snake_case method name from a variant identifier.
    pub(crate) fn derived_snake_name(
        &mut self,
        variant: Identifier,
    ) -> Result<Identifier, NomosError> {
        let derived = self.names.resolve(variant)?.field_name();
        Ok(self.names.intern(Name::new(derived))?)
    }

    /// Derive and allocate the short-header constant name from its typed root and
    /// variant identifiers.
    pub(crate) fn short_header_const_name(
        &mut self,
        root: Identifier,
        variant: Identifier,
    ) -> Result<Identifier, NomosError> {
        let root_screaming = self.names.resolve(root)?.screaming();
        let variant_screaming = self.names.resolve(variant)?.screaming();
        Ok(self
            .names
            .intern(Name::new(format!("{root_screaming}_{variant_screaming}")))?)
    }

    /// Derive and allocate an interface route-enum name.
    pub(crate) fn route_enum_name(&mut self, root: Identifier) -> Result<Identifier, NomosError> {
        let root_name = self.names.resolve(root)?.as_str().to_owned();
        Ok(self.names.intern(Name::new(format!("{root_name}Route")))?)
    }

    /// Produce an output string literal from typed root and variant identifiers at
    /// the emission boundary.
    pub(crate) fn signal_object_name_literal(
        &self,
        root: Identifier,
        variant: Identifier,
    ) -> Result<String, NomosError> {
        let root_name = self.names.resolve(root)?.as_str().to_owned();
        let variant_name = self.names.resolve(variant)?.as_str().to_owned();
        Ok(format!("Signal{root_name}{variant_name}"))
    }

    /// Project an encoded identifier to an output string literal at the emission
    /// boundary.
    pub(crate) fn resolved_text(&self, identifier: Identifier) -> Result<String, NomosError> {
        Ok(self.names.resolve(identifier)?.as_str().to_owned())
    }

    /// The fixed head of a single-argument schema projection.
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

    /// The fixed head of a multi-argument schema projection.
    fn multi_projection_head(
        &self,
        projection: &core_schema::MultiTypeReferenceProjection,
    ) -> &'static str {
        use core_schema::MultiTypeReferenceProjection::Map;
        match projection {
            Map => "Map",
        }
    }

    /// Intern a fixed logos-only path head.
    fn leaf_path(&mut self, text: &str) -> Result<PathNode, NomosError> {
        Ok(PathNode {
            segments: vec![self.names.intern(Name::new(text))?],
        })
    }
}

impl Deref for NameTableBoundary<'_> {
    type Target = NameTable;

    fn deref(&self) -> &Self::Target {
        &self.names
    }
}

impl DerefMut for NameTableBoundary<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.names
    }
}

/// A one-based position within a group of same-typed struct fields. Its ordinal
/// English word is how the deterministic same-typed-field rule tells such fields
/// apart at the NameTable boundary.
struct SameTypeOrdinal(usize);

impl SameTypeOrdinal {
    /// The ordinal English word for this position, in `snake_case`. It is total over
    /// `usize` and never falls back to a numeral.
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

    /// The cardinal English words for a count, in `snake_case`.
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

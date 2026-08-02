//! Static boundary witness for the isolated ScopeOf pre-gate slice.

const SCOPE_OF_SOURCE: &str = include_str!("../src/scope_of.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");

#[test]
fn scope_of_stays_outside_legacy_recursive_and_identity_machinery() {
    for forbidden in [
        "TemplateValue",
        "TemplateFuture",
        "RecursiveInvoke",
        "Invoke",
        "Native",
        "NameTree",
        "slice_one",
        "name_boundary",
        "LocalEncodedId",
        "VocabularyEncodedId::new",
        "EncodedId::new",
        "Name::",
        ".resolve(",
        ".intern(",
        "format!(",
        "String",
        "Box<",
        "Vec<(",
        "-> (",
    ] {
        assert!(
            !SCOPE_OF_SOURCE.contains(forbidden),
            "ScopeOf pre-gate source must not contain {forbidden}"
        );
    }
}

#[test]
fn every_unruled_choice_is_visible_and_concrete_logos_construction_is_absent() {
    for assumption in [
        "primary-zjo-A1",
        "primary-zjo-A2",
        "primary-zjo-A3",
        "primary-zjo-A4",
        "primary-zjo-A5",
        "primary-zjo-A6",
        "primary-zjo-A7",
        "primary-zjo-A8",
        "primary-zjo-A9",
        "primary-zjo-A10",
        "primary-zjo-A11",
    ] {
        assert!(
            SCOPE_OF_SOURCE.contains(assumption),
            "ScopeOf source must expose {assumption}"
        );
    }
    assert!(!SCOPE_OF_SOURCE.contains("WholeLogosEnumeration::new"));
    assert!(SCOPE_OF_SOURCE.contains("GeneratedOutputIdentityRequired"));
    assert!(SCOPE_OF_SOURCE.contains("RecursiveDescent"));
}

#[test]
fn public_surface_names_the_isolated_mirror() {
    assert!(LIB_SOURCE.contains("pub mod scope_of;"));
    assert!(LIB_SOURCE.contains("ScopeOfLogosRealization"));
    assert!(!LIB_SOURCE.contains("no transformer-specific or Logos-type-specific"));
}

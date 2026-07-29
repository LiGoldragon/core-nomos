//! Static boundary witness for the first direct transformation.

const SLICE_SOURCE: &str = include_str!("../src/slice_one.rs");
const MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn slice_source_has_no_text_or_legacy_operations() {
    for forbidden in [
        "\"",
        "String",
        "&str",
        "NameTable",
        "NameTableBoundary",
        "MacroPackage",
        "prelude",
        "renderer",
        "projection",
        "ordinal",
        "SameTypeOrdinal",
        "textual_rust",
        "LocalEncodedId",
        "allocate",
        "mint",
        "Capsule",
        "NameTree",
    ] {
        assert!(
            !SLICE_SOURCE.contains(forbidden),
            "slice source contains forbidden surface {forbidden:?}"
        );
    }
}

#[test]
fn slice_source_depends_only_on_the_typed_ethos_and_logos_carriers() {
    let direct_imports: Vec<_> = SLICE_SOURCE
        .lines()
        .filter(|line| line.starts_with("use "))
        .collect();

    assert_eq!(direct_imports.len(), 2);
    assert!(direct_imports[0].starts_with("use slice_core_ethos::{"));
    assert!(direct_imports[1].starts_with("use slice_core_logos::{"));
    assert!(!SLICE_SOURCE.contains("crate::"));
}

#[test]
fn slice_dependencies_are_exact_published_producer_revisions() {
    assert!(MANIFEST.contains(
        "slice-core-ethos         = { package = \"core-ethos\", git = \
         \"https://github.com/LiGoldragon/core-ethos.git\", rev = \
         \"a79aeb9a0b2bb304d69d7392147639e13a3d58bc\" }"
    ));
    assert!(MANIFEST.contains(
        "slice-core-logos         = { package = \"core-logos\", git = \
         \"https://github.com/LiGoldragon/core-logos.git\", rev = \
         \"3e4ae814f684b44c0aa45d5887c09a7d61d75db6\" }"
    ));
}

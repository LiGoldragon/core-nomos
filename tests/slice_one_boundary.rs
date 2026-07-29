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
fn slice_source_depends_only_on_typed_identity_and_carriers() {
    let direct_imports: Vec<_> = SLICE_SOURCE
        .lines()
        .filter(|line| line.starts_with("use "))
        .collect();

    assert_eq!(direct_imports.len(), 3);
    assert!(direct_imports[0].starts_with("use core_ethos::{"));
    assert!(direct_imports[1].starts_with("use core_logos::{"));
    assert_eq!(
        direct_imports[2],
        "use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};"
    );
    assert!(!SLICE_SOURCE.contains("crate::"));
}

#[test]
fn slice_dependencies_are_exact_published_producer_revisions() {
    assert!(MANIFEST.contains(
        "core-ethos               = { git = \
         \"https://github.com/LiGoldragon/core-ethos.git\", rev = \
         \"f7e4d127a8393e9ad4754e3cce79d1fca1a9e8c9\" }"
    ));
    assert!(MANIFEST.contains(
        "core-logos               = { git = \
         \"https://github.com/LiGoldragon/core-logos.git\", rev = \
         \"e4cefc413cfafbe589e64d961fba34457d214316\" }"
    ));
    assert!(MANIFEST.contains(
        "textual-rust             = { package = \"rust-logos\", git = \
         \"https://github.com/LiGoldragon/rust-logos.git\", rev = \
         \"c8e4ebc16dbea75880b3034a7c46cb6812ab4ef7\" }"
    ));
    assert!(MANIFEST.contains(
        "structural-codec         = { git = \
         \"https://github.com/LiGoldragon/structural-codec.git\", rev = \
         \"31d2e4c14943802ce75a910edd54ea1796273f78\" }"
    ));
}

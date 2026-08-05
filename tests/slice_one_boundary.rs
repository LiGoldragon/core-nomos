//! Static boundary witness for the first direct transformation.

const SLICE_SOURCE: &str = include_str!("../src/slice_one.rs");
const MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn slice_source_has_no_text_or_legacy_operations() {
    for forbidden in [
        "String",
        "&str",
        "NameTable",
        "NameTableBoundary",
        "MacroPackage",
        "prelude",
        "renderer",
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
         \"508cc55588a3ce5178d8eb79508f7f10a179dc2e\" }"
    ));
    assert!(MANIFEST.contains(
        "core-logos               = { git = \
         \"https://github.com/LiGoldragon/core-logos.git\", rev = \
         \"d1cdc53108446eb9779653936611558afd2cb41d\" }"
    ));
    assert!(MANIFEST.contains(
        "textual-rust             = { package = \"rust-logos\", git = \
         \"https://github.com/LiGoldragon/rust-logos.git\", rev = \
         \"36a294d6a73d05b2daa8d1ae882fd5fb43755719\" }"
    ));
    assert!(MANIFEST.contains(
        "structural-codec         = { git = \
         \"https://github.com/LiGoldragon/structural-codec.git\", rev = \
         \"fc6807f4365cde1551bbfe120520aec68245abdb\" }"
    ));
}

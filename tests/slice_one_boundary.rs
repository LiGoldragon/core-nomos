//! Static boundary witness for the first direct transformation.

const SLICE_SOURCE: &str = include_str!("../src/slice_one.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/bootstrap.rs");
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
fn prepared_bootstrap_boundary_revalidates_the_branded_transaction_directly() {
    assert!(BOOTSTRAP_SOURCE.contains("&BootstrapReader<Authority>"));
    assert!(BOOTSTRAP_SOURCE.contains("&PreparedBootstrapTransaction<Authority>"));
    assert!(BOOTSTRAP_SOURCE.contains("reader.validate_transaction(transaction)?;"));
    for forbidden in [
        "WholeEthos",
        "PreparedBootstrapDraft",
        "to_draft",
        "String",
        "&str",
        "NameTable",
        "LocalEncodedId",
        "allocate",
        "mint",
    ] {
        assert!(
            !BOOTSTRAP_SOURCE.contains(forbidden),
            "bootstrap boundary contains forbidden surface {forbidden:?}"
        );
    }
}

#[test]
fn slice_dependencies_are_exact_published_producer_revisions() {
    assert!(MANIFEST.contains(
        "core-ethos               = { git = \
         \"https://github.com/LiGoldragon/core-ethos.git\", rev = \
         \"7a1384874f3747de97c6ccbb4ae6fa2149b27330\" }"
    ));
    assert!(MANIFEST.contains(
        "core-logos               = { git = \
         \"https://github.com/LiGoldragon/core-logos.git\", rev = \
         \"abee4036fbeb58c767ef7dc3489804e2afd5c6e1\" }"
    ));
    assert!(MANIFEST.contains(
        "textual-rust             = { package = \"rust-logos\", git = \
         \"https://github.com/LiGoldragon/rust-logos.git\", rev = \
         \"250e728fa9e5a02e3c9a6d4f0cfee0683863df83\" }"
    ));
    assert!(MANIFEST.contains(
        "structural-codec         = { git = \
         \"https://github.com/LiGoldragon/structural-codec.git\", rev = \
         \"413e3744569ca237e837a1fd57d9ba6ad6adc3de\" }"
    ));
}

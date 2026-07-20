//! The fixed module head carries only the generated marker and cfg-gated NOTA
//! import. Scalar references project to canonical Rust types directly; no type
//! aliases are emitted.

use core_nomos::{GENERATED_MARKER, ModuleHead};

#[test]
fn the_fixed_module_head_projects_required_surface_without_scalar_aliases() {
    let rendered = ModuleHead::fixed()
        .render()
        .expect("render the module head");
    for fragment in [
        GENERATED_MARKER,
        "pub use nota::{NotaDecodeError, NotaEncode, NotaSource};",
    ] {
        assert!(
            rendered.contains(fragment),
            "head lacks {fragment}: {rendered}"
        );
    }
    for forbidden in [
        "pub type String",
        "pub type Integer",
        "pub type Boolean",
        "pub type Path",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "head contains {forbidden}: {rendered}"
        );
    }
}

#[test]
fn render_sections_omits_the_marker_and_scalar_aliases() {
    let head = ModuleHead::fixed();
    let sections = head.render_sections().expect("render sections");
    assert!(!sections.contains(GENERATED_MARKER), "{sections}");
    assert!(sections.contains("pub use nota"), "{sections}");
    assert!(!sections.contains("pub type"), "{sections}");
}

#[test]
fn the_head_carries_one_import_block() {
    let head = ModuleHead::fixed();
    assert_eq!(head.blocks().len(), 1, "one NOTA import block");
    assert_eq!(head.blocks()[0].len(), 1, "one NOTA import");
}

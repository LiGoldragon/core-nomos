//! The fixed module head carries the generated marker, scalar aliases, and the
//! cfg-gated NOTA import required by every generated wire module.

use core_nomos::{GENERATED_MARKER, ModuleHead};

#[test]
fn the_fixed_module_head_projects_required_surface() {
    let rendered = ModuleHead::fixed()
        .render()
        .expect("render the module head");
    for fragment in [
        GENERATED_MARKER,
        "pub type String = std::string::String;",
        "pub type Integer = u64;",
        "pub type Boolean = bool;",
        "pub type Path = std::string::String;",
        "pub use nota::{NotaDecodeError, NotaEncode, NotaSource};",
    ] {
        assert!(
            rendered.contains(fragment),
            "head lacks {fragment}: {rendered}"
        );
    }
}

#[test]
fn render_sections_omits_the_marker() {
    let head = ModuleHead::fixed();
    let sections = head.render_sections().expect("render sections");
    assert!(!sections.contains(GENERATED_MARKER), "{sections}");
    assert!(sections.contains("pub type String"), "{sections}");
}

#[test]
fn the_head_carries_two_blocks_the_scalar_aliases_and_the_import() {
    let head = ModuleHead::fixed();
    assert_eq!(
        head.blocks().len(),
        2,
        "scalar-alias block, then the import"
    );
    assert_eq!(head.blocks()[0].len(), 4, "four scalar aliases");
    assert_eq!(head.blocks()[1].len(), 1, "one NOTA import");
}

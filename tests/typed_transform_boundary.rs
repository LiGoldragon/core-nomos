//! Architectural guard for the Nomos transform boundary.
//!
//! The evaluator can carry typed values and encoded identifiers, but literal text
//! lookup, allocation, and derivation belong to `NameTableBoundary`.

const TYPED_TRANSFORM: &str = include_str!("../src/engine.rs");

#[test]
fn typed_transform_delegates_name_table_operations_to_the_boundary() {
    for direct_operation in ["Name::new(", ".resolve(", ".intern(", "format!("] {
        assert!(
            !TYPED_TRANSFORM.contains(direct_operation),
            "typed transform must delegate {direct_operation} to NameTableBoundary"
        );
    }
    assert!(
        TYPED_TRANSFORM.contains("NameTableBoundary"),
        "typed transform must use the dedicated NameTable boundary"
    );
}

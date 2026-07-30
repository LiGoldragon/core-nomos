#[test]
fn native_production_door_has_no_legacy_or_string_execution_reachability() {
    let source = include_str!("../src/native.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source precedes unit witnesses");

    for forbidden in [
        "MacroPackage::",
        "apply_enriched(",
        "enriched_fixture(",
        "wire_fixture(",
        "NameTableBoundary",
        "name_boundary::",
        "String::",
        "format!(",
    ] {
        assert!(
            !production.contains(forbidden),
            "native production door contains forbidden path {forbidden}"
        );
    }
    assert!(production.contains("EncodedPopulation<WholeEthos"));
    assert!(production.contains("NativeReferenceUniverse"));
    assert!(production.contains("GenerationClass"));
    assert!(production.contains("[to-be-reviewed-by-psyche]"));
}

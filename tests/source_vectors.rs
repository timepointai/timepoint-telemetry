use tt_core::SourceAttribution;
#[test]
fn source_vectors() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../schema/source-attribution-v1.vectors.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let label = serde_json::from_value::<SourceAttribution>(case["input"].clone())
            .ok()
            .map_or("SOURCE UNKNOWN", |s| s.label());
        assert_eq!(label, case["label"].as_str().unwrap());
    }
}

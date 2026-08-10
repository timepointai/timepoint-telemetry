//! Conformance-vector walker (gate G1): every committed vector in `vectors/*.json` is
//! recomputed and asserted byte-for-byte. Bit-stability means these committed bytes never
//! drift — any change to canonicalization or hashing fails here first.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(serde::Deserialize)]
struct Vector {
    name: String,
    input: Input,
    expected_canonical: String,
    expected_hash: String,
}

#[derive(serde::Deserialize)]
struct Input {
    payload: Option<Value>,
    provenance: Option<Value>,
}

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vectors")
}

fn load_vectors() -> Vec<Vector> {
    let mut paths: Vec<PathBuf> = fs::read_dir(vectors_dir())
        .expect("vectors/ directory exists")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|p| {
            let raw = fs::read_to_string(p).expect("vector file readable");
            serde_json::from_str::<Vector>(&raw)
                .unwrap_or_else(|e| panic!("vector {} must parse: {e}", p.display()))
        })
        .collect()
}

#[test]
fn conformance_vectors_are_bit_stable() {
    let vectors = load_vectors();
    assert!(
        vectors.len() >= 8,
        "at least 8 conformance vectors required, found {}",
        vectors.len()
    );

    for v in &vectors {
        let (canonical, hash) = match (&v.input.payload, &v.input.provenance) {
            (Some(payload), None) => {
                let canonical = tt_core::content_canonical(payload)
                    .unwrap_or_else(|e| panic!("vector {}: content_canonical failed: {e}", v.name));
                let hash = tt_core::content_hash(payload)
                    .unwrap_or_else(|e| panic!("vector {}: content_hash failed: {e}", v.name));
                (canonical, hash)
            }
            (None, Some(provenance)) => (
                tt_core::canonicalize(provenance),
                tt_core::provenance_hash(provenance),
            ),
            _ => panic!(
                "vector {}: input must hold exactly one of payload|provenance",
                v.name
            ),
        };
        assert_eq!(
            canonical, v.expected_canonical,
            "vector {}: canonical bytes drifted",
            v.name
        );
        assert_eq!(hash, v.expected_hash, "vector {}: hash drifted", v.name);

        // Internal consistency of the committed file itself: expected_hash must be the
        // sha256 of expected_canonical, independent of the library under test.
        let independent = format!(
            "sha256:{}",
            hex::encode(Sha256::digest(v.expected_canonical.as_bytes()))
        );
        assert_eq!(
            independent, v.expected_hash,
            "vector {}: committed expected_hash does not match committed expected_canonical",
            v.name
        );
    }
}

#[test]
fn required_vector_cases_are_present() {
    let names: Vec<String> = load_vectors().into_iter().map(|v| v.name).collect();
    for required in [
        "worked-moment-payload",
        "worked-moment-provenance",
        "unicode-payload",
        "key-ordering-utf16-provenance",
        "nested-arrays-provenance",
        "numbers-es6-provenance",
        "numbers-precision-edge-provenance",
        "empty-participants-payload",
        "escaping-payload",
        "hash-coverage-twin-payload",
    ] {
        assert!(
            names.iter().any(|n| n == required),
            "required conformance vector missing: {required}"
        );
    }
}

#[test]
fn hash_coverage_twin_shares_the_worked_moment_hash() {
    // Same {label, occurs_at, participants}, different classification/grounding/basis_note
    // and extra fields — the frozen files themselves prove the coverage rule.
    let vectors = load_vectors();
    let hash_of = |name: &str| -> String {
        vectors
            .iter()
            .find(|v| v.name == name)
            .unwrap_or_else(|| panic!("vector {name} present"))
            .expected_hash
            .clone()
    };
    assert_eq!(
        hash_of("worked-moment-payload"),
        hash_of("hash-coverage-twin-payload"),
        "re-classified/re-grounded claim must keep its content_hash identity"
    );
}

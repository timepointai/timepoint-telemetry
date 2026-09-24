//! The relations vocabulary, `tt-relations/1.0`: the shipped artifact loads, its
//! bytes and surface are pinned, and `vectors/verdicts/relation-verdicts.json` is walked
//! on both tiers (docs/proposals/ENTITY-RELATIONS.md §2.4, §2.5, §8). This
//! walker claims only its own corpus; tests/vectors.rs walks the envelope
//! vectors and python/test_classification_vectors.py the classification ones.

use std::fs;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tt_core::{TtError, Vocabulary};

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// sha256 of `bundle/relations-v1.0.json` as released. Identity is frozen: a
/// change to these bytes is a new release with a new file, never an edit.
const RELATIONS_SHA256: &str = "23b0dc5627301d19f128d96152a84fed87000954e1d9fff571f8658ee81b73bc";

/// sha256 of `bundle/taxonomy-v2.1.json`, identical at 16227a9, 7afbc3b (v2.1.2)
/// and v2.2.0. Shipping a separate vocabulary must leave these bytes alone.
const TAXONOMY_SHA256: &str = "31ed385e26522a5b548f7404f7757ee370ed9783dbd550b05cd69e89e9462113";

fn path(rel: &str) -> String {
    format!("{ROOT}/{rel}")
}

fn sha256_file(rel: &str) -> String {
    hex::encode(Sha256::digest(fs::read(path(rel)).expect("file readable")))
}

fn shipped_value() -> Value {
    serde_json::from_str(&fs::read_to_string(path("bundle/relations-v1.0.json")).unwrap()).unwrap()
}

fn shipped() -> Vocabulary {
    Vocabulary::load_from_file(&path("bundle/relations-v1.0.json")).expect("shipped artifact loads")
}

fn corpus() -> Value {
    serde_json::from_str(
        &fs::read_to_string(path("vectors/verdicts/relation-verdicts.json")).unwrap(),
    )
    .expect("relation-verdicts.json parses")
}

// ---------------------------------------------------------------------------
// RFC 6902, the subset the corpus uses (add, remove, replace). The same
// function is python/gen_relation_vectors.py::apply_patch.

fn unescape(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

fn apply_patch(doc: &Value, patch: &Value) -> Value {
    let mut doc = doc.clone();
    for op in patch.as_array().expect("patch is an array") {
        let kind = op["op"].as_str().expect("op");
        let pointer = op["path"].as_str().expect("path");
        let (parent_ptr, last) = pointer.rsplit_once('/').expect("path has a parent");
        let last = unescape(last);
        let parent = doc
            .pointer_mut(parent_ptr)
            .unwrap_or_else(|| panic!("patch parent {parent_ptr} exists"));
        match parent {
            Value::Array(items) => match kind {
                "add" if last == "-" => items.push(op["value"].clone()),
                "add" => items.insert(last.parse().unwrap(), op["value"].clone()),
                "remove" => {
                    items.remove(last.parse().unwrap());
                }
                "replace" => items[last.parse::<usize>().unwrap()] = op["value"].clone(),
                other => panic!("unsupported op {other}"),
            },
            Value::Object(map) => match kind {
                "add" => {
                    map.insert(last, op["value"].clone());
                }
                "remove" => {
                    map.remove(&last)
                        .unwrap_or_else(|| panic!("remove of absent {pointer}"));
                }
                "replace" => {
                    assert!(map.contains_key(&last), "replace of absent {pointer}");
                    map.insert(last, op["value"].clone());
                }
                other => panic!("unsupported op {other}"),
            },
            _ => panic!("patch parent {parent_ptr} is not a container"),
        }
    }
    doc
}

// ---------------------------------------------------------------------------
// The shipped artifact.

#[test]
fn the_shipped_artifact_loads_with_its_counts() {
    let v = shipped();
    assert_eq!(v.version_string(), "tt-relations/1.0 v1.0.0");
    assert_eq!(v.supersedes, None);
    assert_eq!(v.entity_kinds.len(), 4);
    assert_eq!(v.relation_kinds.len(), 10);
    assert_eq!(v.attribute_types, ["date", "text"]);
    assert!(v.entity_kinds.iter().all(|k| !k.is_retired()));
    assert!(v.relation_kinds.iter().all(|r| !r.is_retired()));
    assert_eq!(v.raw, shipped_value(), "raw is the artifact verbatim");
}

#[test]
fn the_shipped_bytes_are_pinned() {
    assert_eq!(sha256_file("bundle/relations-v1.0.json"), RELATIONS_SHA256);
    assert_eq!(corpus()["vocabulary_sha256"], RELATIONS_SHA256);
}

#[test]
fn the_taxonomy_bytes_are_untouched() {
    assert_eq!(sha256_file("bundle/taxonomy-v2.1.json"), TAXONOMY_SHA256);
}

/// Every id, direction, nature, endpoint pair and attribute, spelled out. The
/// sha256 pin says the bytes did not move; this says what they mean, so that a
/// consumer seeding its own tables from the artifact (beta's entity.edges
/// relation_kinds) can be checked against a readable list.
#[test]
fn the_shipped_surface_is_exactly_this() {
    let v = shipped();
    let kinds: Vec<&str> = v.entity_kinds.iter().map(|k| k.id.as_str()).collect();
    assert_eq!(kinds, ["person", "org", "market", "role"]);

    let o = |ty: &str| json!({"type": ty, "required": false});
    let tenure = json!({"valid_from": o("date"), "valid_to": o("date")});
    let subject_tenure = |required: bool| {
        json!({"subject": {"type": "text", "required": required},
               "valid_from": o("date"), "valid_to": o("date")})
    };
    // One row per relation kind, laid out as a table on purpose.
    #[rustfmt::skip]
    let expected = json!([
        ["holds-office", "directed", "structural", [["person", "role"]], tenure],
        ["office-of", "directed", "structural", [["role", "org"]], {}],
        ["member-of", "directed", "structural", [["person", "org"], ["org", "org"]], tenure],
        ["reports-to", "directed", "structural", [["role", "role"]], tenure],
        ["governs", "directed", "structural", [["org", "org"]], tenure],
        ["vendor-to", "directed", "structural", [["org", "org"]], subject_tenure(false)],
        ["knows", "symmetric", "social", [["person", "person"]], {}],
        ["allied-with", "symmetric", "social",
            [["person", "person"], ["org", "org"], ["person", "org"]], subject_tenure(true)],
        ["opposes", "directed", "social",
            [["person", "person"], ["person", "org"], ["org", "person"], ["org", "org"]],
            subject_tenure(true)],
        ["competes-with", "symmetric", "social",
            [["person", "person"], ["org", "org"]], subject_tenure(false)],
    ]);
    let actual: Vec<Value> = v
        .relation_kinds
        .iter()
        .map(|r| {
            json!([
                r.id,
                r.direction,
                r.nature,
                r.endpoints,
                serde_json::to_value(&r.attributes).unwrap()
            ])
        })
        .collect();
    assert_eq!(Value::Array(actual), expected);

    for r in &v.relation_kinds {
        assert_eq!(r.inverse_label.is_some(), !r.is_symmetric(), "{}", r.id);
    }
}

#[test]
fn load_errors_are_typed() {
    match Vocabulary::load_from_str("{ not json") {
        Err(TtError::Parse(_)) => {}
        other => panic!("expected a parse error, got {other:?}"),
    }
    let broken = apply_patch(
        &shipped_value(),
        &json!([{"op": "replace", "path": "/relation_kinds/6/nature", "value": "personal"}]),
    );
    match Vocabulary::load_from_value(broken) {
        Err(TtError::Invalid(msg)) => assert!(msg.starts_with("bad-nature: "), "{msg}"),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

/// A lone surrogate never reaches `validate_edge`: serde_json refuses it while
/// parsing. python/tt_relations.py, whose parser accepts it, rejects it as
/// `attribute-type` instead (python/test_relation_vectors.py).
#[test]
fn a_lone_surrogate_does_not_parse() {
    assert!(serde_json::from_str::<Value>(r#""\ud800""#).is_err());
    assert_eq!(
        serde_json::from_str::<Value>(r#""\ud83d\ude42""#).unwrap(),
        json!("\u{1F642}")
    );
}

// ---------------------------------------------------------------------------
// The corpus.

fn walk_edges(vocab: &Vocabulary, vectors: &Value) -> usize {
    let vectors = vectors.as_array().expect("an array of vectors");
    for v in vectors {
        let name = v["name"].as_str().unwrap();
        let expect = &v["expect"];
        let accepted = expect["accepted"].as_bool().unwrap();
        match vocab.validate_edge(&v["input"]) {
            Ok(normalized) => {
                assert!(accepted, "{name}: accepted, the vector rejects");
                assert_eq!(normalized, expect["normalized"], "{name}: normalized form");
                // What the validator emits, the validator accepts, unchanged.
                assert_eq!(
                    vocab.validate_edge(&normalized).as_ref(),
                    Ok(&normalized),
                    "{name}: round trip"
                );
            }
            Err(rejections) => {
                assert!(
                    !accepted,
                    "{name}: rejected ({rejections:?}), the vector accepts"
                );
                let want = expect["rejections"].as_array().unwrap();
                // Normative: the multiset of codes.
                let mut got: Vec<&str> = rejections.iter().map(|r| r.code).collect();
                let mut exp: Vec<&str> = want.iter().map(|r| r["code"].as_str().unwrap()).collect();
                got.sort_unstable();
                exp.sort_unstable();
                assert_eq!(got, exp, "{name}: rejection codes");
                // Advisory, which this reference implementation passes: every
                // detail byte for byte, in order.
                assert_eq!(
                    serde_json::to_value(&rejections).unwrap(),
                    Value::Array(want.clone()),
                    "{name}: rejection details"
                );
            }
        }
    }
    vectors.len()
}

#[test]
fn every_edge_vector_on_both_tiers() {
    let doc = corpus();
    let vocab = shipped();
    assert_eq!(doc["vocabulary"], vocab.version_string());
    let n = walk_edges(&vocab, &doc["edges"]);
    assert!(n >= 28, "the change request's 28 cases at least, found {n}");
    for i in 1..=28 {
        let prefix = format!("cr-{i:02} ");
        assert!(
            doc["edges"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["name"].as_str().unwrap().starts_with(&prefix)),
            "change request case {i} is present"
        );
    }
}

#[test]
fn every_fixture_edge_vector_on_both_tiers() {
    let doc = corpus();
    let fixture = apply_patch(&shipped_value(), &doc["fixture"]["patch"]);
    let vocab = Vocabulary::load_from_value(fixture).expect("the fixture loads");
    assert!(vocab.entity_kinds.iter().any(|k| k.is_retired()));
    assert!(vocab.relation_kinds.iter().any(|r| r.is_retired()));
    assert!(walk_edges(&vocab, &doc["fixture_edges"]) > 0);
}

#[test]
fn every_load_vector() {
    let doc = corpus();
    let base = shipped_value();
    let loads = doc["loads"].as_array().unwrap();
    assert!(
        loads.len() >= 8,
        "the change request's load negatives at least"
    );
    for v in loads {
        let name = v["name"].as_str().unwrap();
        let patched = apply_patch(&base, &v["patch"]);
        let want_loads = v["expect"]["loads"].as_bool().unwrap();
        let want = v["expect"]["failures"].as_array().unwrap();
        match Vocabulary::try_from_value(patched) {
            Ok(_) => {
                assert!(want_loads, "{name}: loads, the vector refuses it");
                assert!(want.is_empty(), "{name}");
            }
            Err(failures) => {
                assert!(
                    !want_loads,
                    "{name}: refused ({failures:?}), the vector loads it"
                );
                let mut got: Vec<&str> = failures.iter().map(|f| f.rule).collect();
                let mut exp: Vec<&str> = want.iter().map(|f| f["rule"].as_str().unwrap()).collect();
                got.sort_unstable();
                exp.sort_unstable();
                assert_eq!(got, exp, "{name}: failure rules");
                // Advisory: details byte for byte, in order, except a
                // `malformed` detail, which is the parser's own wording.
                if !got.contains(&"malformed") {
                    assert_eq!(
                        serde_json::to_value(&failures).unwrap(),
                        Value::Array(want.clone()),
                        "{name}: failure details"
                    );
                }
            }
        }
    }
}

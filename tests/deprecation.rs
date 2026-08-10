//! RETIREMENT (GOVERNANCE.md §3).
//!
//! An id never changes meaning and is never deleted, so without a way to retire
//! one the taxonomy could only ever grow. These tests pin the three properties
//! that make retirement safe to rely on: a retired id still reads, resolution
//! follows supersession to the node in use today, and a bundle that would strand
//! a record refuses to load at all.

use snag_core::Bundle;
use serde_json::json;

/// A minimal but structurally valid bundle, so each test states only the thing
/// it is about.
fn bundle_with(nodes: serde_json::Value) -> Result<Bundle, snag_core::SnagError> {
    Bundle::load_from_str(
        &json!({
            "schema": "snag-ontology/1.0",
            "version": "9.9.9",
            "metric": {
                "hierarchy_edge_weight": 1.0,
                "lateral_edge_weight": 1.6,
                "distance": "weighted shortest path"
            },
            "lenses": {
                "A": {"label": "Recorded Public Events", "question": "what did the record keep?",
                      "branches": a_branches()},
                "B": {"label": "Human Action & Behavior", "question": "what were people doing?",
                      "branches": b_branches()}
            },
            "nodes": nodes,
            "lateral_edges": [],
            "bridges": [],
            "kernel": []
        })
        .to_string(),
    )
}

fn node(id: &str, lens: &str, level: &str, parent: Option<&str>) -> serde_json::Value {
    json!({"id": id, "lens": lens, "level": level, "parent": parent,
           "label": id, "definition": format!("definition of {id}")})
}

/// The validator pins the branch counts — exactly 6 under Lens A and 9 under
/// Lens B — so a fixture has to carry the real shape even when the thing under
/// test is one retired species. (That pinning is itself why adding a BRANCH is
/// a code change as well as a bundle change, and belongs in the slowest
/// bracket: GOVERNANCE.md §2.)
fn a_branches() -> Vec<String> {
    (1..=6).map(|i| format!("a-branch-{i}")).collect()
}
fn b_branches() -> Vec<String> {
    (1..=9).map(|i| format!("b-branch-{i}")).collect()
}

fn base() -> Vec<serde_json::Value> {
    a_branches()
        .iter()
        .map(|id| node(id, "A", "branch", None))
        .chain(b_branches().iter().map(|id| node(id, "B", "branch", None)))
        .collect()
}

#[test]
fn a_bundle_with_no_deprecations_still_loads_and_resolves_to_itself() {
    let mut nodes = base();
    nodes.push(node("plain", "B", "species", Some("b-branch-1")));
    let b = bundle_with(json!(nodes)).expect("loads");
    let r = b.resolve("plain").expect("no cycle").expect("known id");
    assert_eq!(r.id, "plain");
    assert!(!r.is_deprecated(), "nothing here is retired");
    assert!(b.deprecated().is_empty());
}

#[test]
fn a_retired_id_still_reads_and_resolution_lands_on_its_successor() {
    let mut nodes = base();
    nodes.push(node("new-home", "B", "species", Some("b-branch-1")));
    let mut old = node("old-name", "B", "species", Some("b-branch-1"));
    old["deprecated_in"] = json!("2.0.0");
    old["superseded_by"] = json!("new-home");
    nodes.push(old);
    let b = bundle_with(json!(nodes)).expect("loads");

    // The retired id is STILL THERE — that is the promise.
    let raw = b.node("old-name").expect("a retired id never stops being readable");
    assert!(raw.is_deprecated());
    assert_eq!(raw.deprecated_in.as_deref(), Some("2.0.0"));

    // ...and resolution moves you to what to use today.
    let now = b.resolve("old-name").expect("no cycle").expect("resolves");
    assert_eq!(now.id, "new-home");
    assert!(!now.is_deprecated());

    assert_eq!(b.deprecated().len(), 1);
}

#[test]
fn a_chain_of_supersessions_resolves_all_the_way_through() {
    let mut nodes = base();
    nodes.push(node("third", "B", "species", Some("b-branch-1")));
    for (id, next) in [("first", "second"), ("second", "third")] {
        let mut n = node(id, "B", "species", Some("b-branch-1"));
        n["deprecated_in"] = json!("2.0.0");
        n["superseded_by"] = json!(next);
        nodes.push(n);
    }
    let b = bundle_with(json!(nodes)).expect("loads");
    assert_eq!(b.resolve("first").expect("no cycle").expect("resolves").id, "third");
}

/// Retired with nothing to replace it is LEGAL — a node that should never have
/// existed has no successor — but it has to say why, or a reader is left with an
/// id that stops working and no account of it.
#[test]
fn retired_with_no_successor_is_legal_only_when_it_says_why() {
    let mut with_note = base();
    let mut n = node("was-a-mistake", "B", "species", Some("b-branch-1"));
    n["deprecated_in"] = json!("2.0.0");
    n["deprecation_note"] = json!("duplicated an existing node; nothing replaces it");
    with_note.push(n);
    let b = bundle_with(json!(with_note)).expect("a stated reason is enough");
    // It resolves to ITSELF: retired-and-unreplaced is a real answer, and must
    // not be confused with "never existed".
    assert_eq!(b.resolve("was-a-mistake").expect("no cycle").expect("known").id, "was-a-mistake");
    assert!(b.resolve("never-existed").expect("no cycle").is_none());

    let mut silent = base();
    let mut m = node("silently-retired", "B", "species", Some("b-branch-1"));
    m["deprecated_in"] = json!("2.0.0");
    silent.push(m);
    let err = bundle_with(json!(silent)).expect_err("silence is refused");
    assert!(
        format!("{err}").contains("neither superseded_by nor deprecation_note"),
        "the refusal names what is missing: {err}"
    );
}

#[test]
fn a_successor_that_does_not_exist_refuses_to_load() {
    let mut nodes = base();
    let mut n = node("points-nowhere", "B", "species", Some("b-branch-1"));
    n["deprecated_in"] = json!("2.0.0");
    n["superseded_by"] = json!("no-such-node");
    nodes.push(n);
    let err = bundle_with(json!(nodes)).expect_err("a dangling successor strands every record");
    assert!(format!("{err}").contains("no-such-node"), "{err}");
}

/// A loop would hang whatever asked. It is caught at load, and named.
#[test]
fn a_supersession_cycle_is_caught_at_load_not_survived() {
    let mut nodes = base();
    for (id, next) in [("ping", "pong"), ("pong", "ping")] {
        let mut n = node(id, "B", "species", Some("b-branch-1"));
        n["deprecated_in"] = json!("2.0.0");
        n["superseded_by"] = json!(next);
        nodes.push(n);
    }
    let err = bundle_with(json!(nodes)).expect_err("a cycle is a broken bundle");
    assert!(format!("{err}").contains("supersession cycle"), "{err}");
}

/// The shipped bundle predates retirement entirely. It must still load, and
/// nothing in it may be retired by accident.
#[test]
fn the_shipped_bundle_parses_unchanged_and_retires_nothing() {
    let b = Bundle::load_from_file(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/bundle/taxonomy-v1.1.json"
    ))
    .expect("the shipped bundle still loads with the new fields absent");
    assert!(
        b.deprecated().is_empty(),
        "v1.1.0 retires nothing; retirement arrives with the version that uses it"
    );
    for n in &b.nodes {
        assert_eq!(
            b.resolve(&n.id).expect("no cycle").expect("known").id,
            n.id,
            "with nothing retired, every id resolves to itself"
        );
    }
}

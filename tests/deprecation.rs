//! RETIREMENT (GOVERNANCE.md §3).
//!
//! An id never changes meaning and is never deleted, so without a way to retire
//! one the taxonomy could only ever grow. These tests pin the three properties
//! that make retirement safe to rely on: a retired id still reads, resolution
//! follows supersession to the node in use today, and a bundle that would strand
//! a record refuses to load at all.

use tt_core::Bundle;
use serde_json::json;

/// A minimal but structurally valid bundle, so each test states only the thing
/// it is about.
fn bundle_with(nodes: serde_json::Value) -> Result<Bundle, tt_core::TtError> {
    bundle_with_branches(nodes, a_branches(), b_branches())
}

fn bundle_with_branches(
    nodes: serde_json::Value,
    a: Vec<String>,
    b: Vec<String>,
) -> Result<Bundle, tt_core::TtError> {
    Bundle::load_from_str(
        &json!({
            "schema": "tt-ontology/1.0",
            "version": "9.9.9",
            "metric": {
                "hierarchy_edge_weight": 1.0,
                "lateral_edge_weight": 1.6,
                "distance": "weighted shortest path"
            },
            "lenses": {
                "A": {"label": "Recorded Public Events", "question": "what did the record keep?",
                      "branches": a},
                "B": {"label": "Human Action & Behavior", "question": "what were people doing?",
                      "branches": b}
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

/// v2.1.0 is the first Structure release: it retires exactly ONE node, with a
/// successor, and everything else still resolves to itself.
#[test]
fn the_shipped_bundle_retires_exactly_one_node_with_a_successor() {
    let b = Bundle::load_from_file(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/bundle/taxonomy-v2.1.json"
    ))
    .expect("the shipped bundle loads");
    let retired: Vec<_> = b.deprecated();
    assert_eq!(retired.len(), 1, "one retirement, deliberately: {retired:?}");
    assert_eq!(retired[0].id, "everyday-movement-and-commute");
    assert_eq!(
        b.resolve("everyday-movement-and-commute").expect("no cycle").expect("known").id,
        "journey-and-travel",
        "the retired id resolves to its successor"
    );
    for n in &b.nodes {
        if n.id == "everyday-movement-and-commute" {
            continue;
        }
        assert_eq!(
            b.resolve(&n.id).expect("no cycle").expect("known").id,
            n.id,
            "every other id resolves to itself"
        );
    }
}

/// THE ONE THAT WOULD HAVE BRICKED A DEPLOY.
///
/// Retirement shipped with the branch counts still counting ROWS. Deprecating a
/// branch and adding its successor leaves seven Lens-A branch nodes though six
/// are in service, so the bundle refused to load — and loading is how a consumer
/// starts. Not a degraded surface: a process that will not boot, on the first
/// Structure release that touched a branch. The counts are over LIVE branches.
#[test]
fn retiring_a_branch_and_naming_its_successor_still_loads() {
    let mut nodes = base();
    // Seven Lens-A branch NODES; six of them in service.
    let mut retired = node("a-branch-6", "A", "branch", None);
    retired["deprecated_in"] = json!("2.0.0");
    retired["superseded_by"] = json!("a-branch-6b");
    nodes.retain(|n| n["id"] != "a-branch-6");
    nodes.push(retired);
    nodes.push(node("a-branch-6b", "A", "branch", None));

    let b = bundle_with_branches(
        json!(nodes),
        // The lens list declares every branch node, retired ones included —
        // they are still nodes and still readable.
        vec![
            "a-branch-1".into(), "a-branch-2".into(), "a-branch-3".into(),
            "a-branch-4".into(), "a-branch-5".into(), "a-branch-6".into(),
            "a-branch-6b".into(),
        ],
        b_branches(),
    )
    .expect("a retired branch plus its successor is six LIVE branches, and must load");

    assert_eq!(b.resolve("a-branch-6").expect("no cycle").expect("known").id, "a-branch-6b");
    assert_eq!(b.deprecated().len(), 1);
}

/// And the frozen number still means something: retiring a branch WITHOUT a
/// successor drops service to five and is refused.
#[test]
fn retiring_a_branch_with_no_successor_is_refused() {
    let mut nodes = base();
    let mut retired = node("a-branch-6", "A", "branch", None);
    retired["deprecated_in"] = json!("2.0.0");
    retired["deprecation_note"] = json!("no successor");
    nodes.retain(|n| n["id"] != "a-branch-6");
    nodes.push(retired);
    let err = bundle_with_branches(json!(nodes), a_branches(), b_branches())
        .expect_err("five live Lens-A branches is not a taxonomy this code knows");
    assert!(format!("{err}").contains("Lens A live branch count"), "{err}");
}

// ---------------------------------------------------------------------------
// PLAN 3.1 — the property the forecast sweep depends on.
//
// `register.rs` scores a forecast HIT by asking whether the forecast's anchor
// appears on the observed moment's parent chain. It compared RAW STRINGS, so a
// retirement broke the comparison silently: after a Structure release the
// anchor and the observation can be the same concept under two ids, the match
// fails, a real HIT scores 0.0 as a MISS — and because that write is guarded
// `WHERE resolved_at IS NULL`, THE VERDICT NEVER REOPENS. 459 anchored
// predicates sit behind it.
//
// The fix resolves both sides first. These pin what `resolve()` has to
// guarantee for that to work.
// ---------------------------------------------------------------------------

#[test]
fn a_retired_id_and_its_successor_resolve_to_one_node() {
    // The exact shape of the bug: two ids, one concept, after a retirement.
    let mut nodes = base();
    nodes.push(node("old-species", "A", "species", Some("a-branch-1")));
    nodes.push(node("new-species", "A", "species", Some("a-branch-1")));
    let mut retired = node("old-species", "A", "species", Some("a-branch-1"));
    retired["deprecated_in"] = json!("2.0.0");
    retired["superseded_by"] = json!("new-species");
    nodes.retain(|n| n["id"] != json!("old-species"));
    nodes.push(retired);

    let b = bundle_with(json!(nodes)).expect("bundle loads");

    let old = b.resolve("old-species").expect("resolves").expect("known id");
    let new = b.resolve("new-species").expect("resolves").expect("known id");
    assert_eq!(old.id, "new-species", "a retired id resolves to its successor");
    assert_eq!(new.id, "new-species", "and the successor resolves to itself");
    assert_eq!(old.id, new.id, "so a stored anchor and a fresh observation MATCH");
}

#[test]
fn an_unknown_id_does_not_resolve_into_something_plausible() {
    // The sweep falls back to the raw string when `resolve` says None, so a
    // typo must stay a miss rather than being quietly mapped onto a real node.
    let b = bundle_with(json!(base())).expect("bundle loads");
    assert!(
        b.resolve("not-a-node-at-all").expect("no error").is_none(),
        "an unknown id is unknown — never resolved into a neighbour"
    );
}

#[test]
fn a_retirement_with_no_successor_resolves_to_itself() {
    // "This was retired and nothing replaces it" is a real answer and must not
    // be confused with "never existed" — a forecast anchored on it still
    // matches an observation carrying the same id.
    let mut nodes = base();
    let mut orphaned = node("gone-species", "A", "species", Some("a-branch-1"));
    orphaned["deprecated_in"] = json!("2.0.0");
    orphaned["deprecation_note"] = json!("recorded in error; nothing replaces it");
    nodes.push(orphaned);

    let b = bundle_with(json!(nodes)).expect("bundle loads");
    let r = b.resolve("gone-species").expect("resolves").expect("still known");
    assert_eq!(r.id, "gone-species", "resolves to itself, so a stored anchor still matches");
    assert!(r.is_deprecated(), "and it is still visibly retired");
}

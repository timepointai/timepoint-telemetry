//! Bundle loader tests: the vendored taxonomy loads and every validate.py count matches;
//! each check family rejects a broken bundle at load (fail boot); parent_chain/bridge_for
//! behave per the bundle.

use serde_json::{Value, json};
use tt_core::{Bundle, TtError};

fn bundle_path() -> String {
    format!("{}/bundle/taxonomy-v2.1.json", env!("CARGO_MANIFEST_DIR"))
}

fn load_real() -> Bundle {
    Bundle::load_from_file(&bundle_path()).expect("vendored bundle loads and validates")
}

fn raw_value() -> Value {
    let raw = std::fs::read_to_string(bundle_path()).expect("bundle file readable");
    serde_json::from_str(&raw).expect("bundle file parses")
}

/// Mutate the raw bundle, reload, and require a validation failure naming the check.
fn assert_invalid(mutated: &Value, expect_substring: &str) {
    match Bundle::load_from_str(&mutated.to_string()) {
        Err(TtError::Invalid(msg)) => assert!(
            msg.contains(expect_substring),
            "expected failure mentioning {expect_substring:?}, got: {msg}"
        ),
        Ok(_) => panic!("mutated bundle must not load (expected {expect_substring:?})"),
        Err(other) => panic!("expected Invalid({expect_substring:?}), got {other:?}"),
    }
}

fn node_index(v: &Value, id: &str) -> usize {
    v["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .position(|n| n["id"] == id)
        .unwrap_or_else(|| panic!("node {id} present"))
}

// ---- the vendored bundle: every validate.py count ----

#[test]
fn vendored_bundle_loads_with_exact_validate_py_counts() {
    let b = load_real();

    assert_eq!(b.version_string(), "tt-ontology/1.0 v2.1.0");
    assert_eq!(b.schema, "tt-ontology/1.0");
    assert_eq!(b.version, "2.1.0");
    // v2.1.0 is the first Structure release: ONE retirement, nothing removed.
    // Every count below is asserted unchanged — a retired node stays in the
    // bundle forever; what ends is its eligibility for new work.
    assert_eq!(b.supersedes.as_deref(), Some("tt-ontology/1.0 v2.0.0"));

    assert_eq!(b.nodes.len(), 149, "nodes total");
    assert_eq!(b.lateral_edges.len(), 151, "lateral edges total");
    assert_eq!(b.bridges.len(), 26, "bridges");
    assert_eq!(b.kernel.len(), 3, "kernel members");

    let count = |lens: &str, level: &str| {
        b.nodes
            .iter()
            .filter(|n| n.lens == lens && n.level == level)
            .count()
    };
    assert_eq!(count("A", "branch"), 6, "Lens A branch");
    assert_eq!(count("A", "species"), 26, "Lens A species");
    assert_eq!(count("A", "subspecies"), 47, "Lens A subspecies");
    assert_eq!(
        b.nodes.iter().filter(|n| n.lens == "A").count(),
        79,
        "Lens A total"
    );
    assert_eq!(count("B", "branch"), 9, "Lens B branch");
    assert_eq!(count("B", "species"), 61, "Lens B species");
    assert_eq!(count("B", "subspecies"), 0, "Lens B subspecies");
    assert_eq!(
        b.nodes.iter().filter(|n| n.lens == "B").count(),
        70,
        "Lens B total"
    );

    let edge_lens = |edge_a: &str| b.node(edge_a).map(|n| n.lens.clone()).unwrap();
    assert_eq!(
        b.lateral_edges
            .iter()
            .filter(|e| edge_lens(&e.a) == "A")
            .count(),
        90,
        "lateral edges (A)"
    );
    assert_eq!(
        b.lateral_edges
            .iter()
            .filter(|e| edge_lens(&e.a) == "B")
            .count(),
        61,
        "lateral edges (B)"
    );

    let rel = |r: &str| b.bridges.iter().filter(|br| br.relation == r).count();
    assert_eq!(rel("constitutes"), 3, "constitutes");
    assert_eq!(rel("scales-up-to"), 6, "scales-up-to");
    assert_eq!(rel("recorded-as"), 14, "recorded-as");
    assert_eq!(rel("unrecorded"), 3, "unrecorded");

    assert_eq!(b.lenses.a.branches.len(), 6);
    assert_eq!(b.lenses.b.branches.len(), 9);
    assert_eq!(b.metric.hierarchy_edge_weight, 1.0);
    assert_eq!(b.metric.lateral_edge_weight, 1.6);

    // The verbatim document survives loading (servers hand it out unchanged).
    assert_eq!(b.raw["schema"], "tt-ontology/1.0");
    assert_eq!(b.raw["supersedes"], "tt-ontology/1.0 v2.0.0");
    assert!(b.raw.get("design_principles").is_some());

    // The chain is now typed, so it can be asked rather than eyeballed: this
    // release answers to its own id AND to the one it replaced.
    assert!(b.is_this_release("tt-ontology/1.0 v2.1.0"));
    assert!(
        b.is_this_release("tt-ontology/1.0 v2.0.0"),
        "the step behind counts"
    );
    assert!(
        !b.is_this_release("clockchain-taxonomy/1.0 v1.1.0-alpha.1"),
        "two steps back does NOT — walking further needs the intervening bundle, \
         and answering yes here would claim a reach the format does not have"
    );

    b.validate().expect("re-validating a loaded bundle passes");
}

// ---- behavior per the bundle ----

#[test]
fn kernel_is_the_three_unrecorded_actions() {
    let b = load_real();
    let mut kernel: Vec<&str> = b.kernel().iter().map(String::as_str).collect();
    kernel.sort_unstable();
    assert_eq!(
        kernel,
        vec![
            "courtship-and-falling-in-love",
            "journey-and-travel",
            "migration-and-resettlement"
        ]
    );
    for id in b.kernel() {
        let br = b.bridge_for(id).expect("kernel action has a bridge row");
        assert_eq!(br.relation, "unrecorded");
        assert_eq!(br.event, None, "kernel bridges carry a null event");
    }
}

#[test]
fn parent_chain_walks_to_the_branch_root() {
    let b = load_real();

    // A-lens subspecies → species → branch (the worked moment's lens_a argmax).
    let chain: Vec<&str> = b
        .parent_chain("corporate-founding-and-milestone")
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    assert_eq!(
        chain,
        vec![
            "corporate-founding-and-milestone",
            "enterprise-and-commerce",
            "economy-trade-and-labor"
        ]
    );

    // B-lens species → branch (the worked moment's lens_b argmax).
    let chain: Vec<&str> = b
        .parent_chain("negotiation-and-agreement")
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    assert_eq!(
        chain,
        vec!["negotiation-and-agreement", "communication-and-exchange"]
    );

    // A branch is its own chain; an unknown id yields an empty chain (representable absence).
    let chain: Vec<&str> = b
        .parent_chain("bonding-and-kinship")
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    assert_eq!(chain, vec!["bonding-and-kinship"]);
    assert!(b.parent_chain("no-such-node").is_empty());
}

#[test]
fn bridge_for_behaves_per_the_bundle() {
    let b = load_real();

    // The worked moment's lens_b argmax carries its own bridge in bundle v1.1.0.
    let br = b
        .bridge_for("negotiation-and-agreement")
        .expect("bridge exists");
    assert_eq!(br.relation, "scales-up-to");
    assert_eq!(
        br.event.as_deref(),
        Some("treaty-alliance-and-peace-accord")
    );

    // Its parent has no bridge row of its own.
    assert!(b.bridge_for("communication-and-exchange").is_none());

    // Every bridge event that exists is a Lens A node; every action a Lens B node.
    for br in &b.bridges {
        assert_eq!(b.node(&br.action).unwrap().lens, "B");
        if let Some(ev) = &br.event {
            assert_eq!(b.node(ev).unwrap().lens, "A");
        }
    }
}

#[test]
fn id_lookup_surface() {
    let b = load_real();
    assert!(b.is_valid_id("negotiation-and-agreement"));
    assert!(!b.is_valid_id("negotiation_and_agreement"));
    assert!(!b.is_valid_id(""));
    let n = b
        .node("politics-governance-and-law")
        .expect("branch exists");
    assert_eq!(n.lens, "A");
    assert_eq!(n.level, "branch");
    assert_eq!(n.parent, None);
    assert!(!n.label.is_empty());
    assert!(!n.definition.is_empty());
    assert!(b.node("no-such-node").is_none());
}

// ---- every check family rejects a broken bundle at load ----

#[test]
fn rejects_non_kebab_ids() {
    let mut v = raw_value();
    let i = node_index(&v, "journey-and-travel");
    v["nodes"][i]["id"] = json!("Journey_And_Travel");
    assert_invalid(&v, "kebab-case");
}

#[test]
fn rejects_duplicate_ids() {
    let mut v = raw_value();
    let node = v["nodes"][0].clone();
    v["nodes"].as_array_mut().unwrap().push(node);
    assert_invalid(&v, "unique");
}

#[test]
fn rejects_bad_lens_and_level() {
    let mut v = raw_value();
    let i = node_index(&v, "negotiation-and-agreement");
    v["nodes"][i]["lens"] = json!("C");
    assert_invalid(&v, "lens in {A,B}");

    let mut v = raw_value();
    let i = node_index(&v, "negotiation-and-agreement");
    v["nodes"][i]["level"] = json!("genus");
    assert_invalid(&v, "level valid");
}

#[test]
fn rejects_broken_parents() {
    let mut v = raw_value();
    let i = node_index(&v, "negotiation-and-agreement");
    v["nodes"][i]["parent"] = json!("no-such-parent");
    assert_invalid(&v, "parents exist");

    let mut v = raw_value();
    let i = node_index(&v, "politics-governance-and-law");
    v["nodes"][i]["parent"] = json!("bonding-and-kinship");
    assert_invalid(&v, "branches have null parent");

    // Re-parent an A-lens species under a B-lens branch: cross-lens.
    let mut v = raw_value();
    let i = node_index(&v, "enterprise-and-commerce");
    v["nodes"][i]["parent"] = json!("bonding-and-kinship");
    assert_invalid(&v, "parent shares lens");

    // Re-parent a subspecies under a branch: nesting broken.
    let mut v = raw_value();
    let i = node_index(&v, "corporate-founding-and-milestone");
    v["nodes"][i]["parent"] = json!("politics-governance-and-law");
    assert_invalid(&v, "level nesting");
}

#[test]
fn rejects_branch_count_and_lenses_block_drift() {
    // Demote a B branch to A: both branch counts break.
    let mut v = raw_value();
    let i = node_index(&v, "bonding-and-kinship");
    v["nodes"][i]["lens"] = json!("A");
    assert_invalid(&v, "branch count");

    let mut v = raw_value();
    v["lenses"]["A"]["branches"].as_array_mut().unwrap().pop();
    assert_invalid(&v, "lenses.A.branches matches nodes");

    let mut v = raw_value();
    v["lenses"]["B"]["branches"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert_invalid(&v, "lenses.B.branches matches nodes");
}

#[test]
fn rejects_broken_lateral_edges() {
    let mut v = raw_value();
    v["lateral_edges"][0]["a"] = json!("no-such-endpoint");
    assert_invalid(&v, "endpoints exist");

    let mut v = raw_value();
    let a = v["lateral_edges"][0]["a"].clone();
    v["lateral_edges"][0]["b"] = a;
    assert_invalid(&v, "self-loops");

    // Point edge 0's b at a B-lens node while a stays A-lens: cross-lens.
    let mut v = raw_value();
    v["lateral_edges"][0]["b"] = json!("negotiation-and-agreement");
    assert_invalid(&v, "within a lens");

    let mut v = raw_value();
    let edge = v["lateral_edges"][0].clone();
    v["lateral_edges"].as_array_mut().unwrap().push(edge);
    assert_invalid(&v, "duplicate lateral edges");

    // Reversed duplicate is still a duplicate (undirected).
    let mut v = raw_value();
    let mut edge = v["lateral_edges"][0].clone();
    let (a, b) = (edge["a"].clone(), edge["b"].clone());
    edge["a"] = b;
    edge["b"] = a;
    v["lateral_edges"].as_array_mut().unwrap().push(edge);
    assert_invalid(&v, "duplicate lateral edges");

    let mut v = raw_value();
    v["lateral_edges"][0]["weight"] = json!(1.0);
    assert_invalid(&v, "lateral weights == metric.lateral_edge_weight");
}

#[test]
fn rejects_broken_bridges() {
    // An A-lens action is not a Lens B action.
    let mut v = raw_value();
    v["bridges"][0]["action"] = json!("banking-and-exchange");
    assert_invalid(&v, "bridge actions exist in Lens B");

    // A B-lens event is not a Lens A event.
    let mut v = raw_value();
    v["bridges"][0]["event"] = json!("negotiation-and-agreement");
    assert_invalid(&v, "bridge events exist in Lens A");

    let mut v = raw_value();
    v["bridges"][0]["relation"] = json!("causes");
    assert_invalid(&v, "closed set");

    // Null event on a non-unrecorded bridge breaks the iff.
    let mut v = raw_value();
    v["bridges"][0]["event"] = Value::Null;
    assert_invalid(&v, "exactly unrecorded bridges have null event");
}

#[test]
fn rejects_kernel_drift() {
    let mut v = raw_value();
    v["kernel"].as_array_mut().unwrap().pop();
    assert_invalid(&v, "kernel == null-event bridge actions");

    let mut v = raw_value();
    v["kernel"]
        .as_array_mut()
        .unwrap()
        .push(json!("negotiation-and-agreement"));
    assert_invalid(&v, "kernel == null-event bridge actions");
}

#[test]
fn rejects_empty_definitions_and_labels() {
    let mut v = raw_value();
    let i = node_index(&v, "journey-and-travel");
    v["nodes"][i]["definition"] = json!("   ");
    assert_invalid(&v, "non-empty definition");

    let mut v = raw_value();
    let i = node_index(&v, "journey-and-travel");
    v["nodes"][i]["label"] = json!("");
    assert_invalid(&v, "non-empty label");
}

#[test]
fn parse_errors_are_typed_not_validation_failures() {
    match Bundle::load_from_str("{ not json") {
        Err(TtError::Parse(_)) => {}
        other => panic!("expected Parse error, got {other:?}"),
    }
    match Bundle::load_from_file("/no/such/path/taxonomy.json") {
        Err(TtError::Io(_)) => {}
        other => panic!("expected Io error, got {other:?}"),
    }
}

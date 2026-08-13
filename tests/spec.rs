//! TT-SPEC states numbers. This is what keeps them true.
//!
//! A specification that drifts from its implementation is worse than none: it
//! is a document people trust and act on while it quietly stops describing
//! anything. Every figure TT-SPEC prints about the shipped bundle is asserted
//! here against the bundle itself, so a release that moves one of them cannot
//! be published with a spec that still claims the old value.
//!
//! What is NOT checked here is prose. This catches drift in the facts, not in
//! the reasoning, and it is not a substitute for reading the document.

use std::collections::BTreeMap;

const BUNDLE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/bundle/taxonomy-v2.1.json");
const SPEC_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/TT-SPEC.md");

fn bundle() -> tt_core::Bundle {
    tt_core::Bundle::load_from_file(BUNDLE_PATH).expect("bundle loads")
}
fn spec() -> String {
    std::fs::read_to_string(SPEC_PATH).expect("TT-SPEC.md exists — it is cited normatively")
}

#[test]
fn the_spec_names_the_release_it_describes() {
    let b = bundle();
    let s = spec();
    assert!(
        s.contains(&b.version_string()),
        "TT-SPEC must name the release it describes ({})",
        b.version_string()
    );
    if let Some(prev) = &b.supersedes {
        assert!(s.contains(prev.as_str()), "the lineage step is stated: {prev}");
    }
}

#[test]
fn the_counts_the_spec_prints_are_the_bundles_counts() {
    let b = bundle();
    let s = spec();
    let count = |lens: &str, level: &str| {
        b.nodes.iter().filter(|n| n.lens == lens && n.level == level).count()
    };
    let per_level = |level: &str| b.nodes.iter().filter(|n| n.level == level).count();

    // Every figure below appears in TT-SPEC §2 and §5. If a release moves one,
    // this fails and the document must be updated in the same change.
    for (what, n) in [
        ("nodes total", b.nodes.len()),
        ("branches", per_level("branch")),
        ("species", per_level("species")),
        ("subspecies", per_level("subspecies")),
        ("lens A nodes", b.nodes.iter().filter(|n| n.lens == "A").count()),
        ("lens B nodes", b.nodes.iter().filter(|n| n.lens == "B").count()),
        ("lens A branches", count("A", "branch")),
        ("lens B branches", count("B", "branch")),
        ("lateral edges", b.lateral_edges.len()),
    ] {
        assert!(
            s.contains(&n.to_string()),
            "TT-SPEC does not state the bundle's {what} ({n}) — the spec has drifted"
        );
    }
}

#[test]
fn the_bridge_table_in_the_spec_matches_the_bundle() {
    let b = bundle();
    let s = spec();
    let mut by_relation: BTreeMap<&str, usize> = BTreeMap::new();
    for br in &b.bridges {
        *by_relation.entry(br.relation.as_str()).or_default() += 1;
    }
    for (relation, n) in &by_relation {
        assert!(
            s.contains(&format!("`{relation}` | {n}")),
            "TT-SPEC's bridge table is missing or wrong for `{relation}` ({n})"
        );
    }
}

#[test]
fn the_kernel_the_spec_lists_is_the_kernel() {
    let b = bundle();
    let s = spec();
    for action in b.kernel() {
        assert!(
            s.contains(action.as_str()),
            "TT-SPEC does not name kernel member `{action}` — the kernel is the \
             model's sharpest claim and the spec must state it exactly"
        );
    }
    // And nothing else is presented as kernel: the count is stated too.
    assert!(s.contains(&format!("| {} |", b.kernel().len())), "the kernel count is stated");
}

#[test]
fn every_cited_section_exists() {
    // 39 citations across two repos point at §2, §3, §4 and §5. A citation to a
    // section that does not exist is the state this document was written to end.
    let s = spec();
    for section in ["## §2.", "## §3.", "## §4.", "## §5."] {
        assert!(s.contains(section), "TT-SPEC has no {section} — but the code cites it");
    }
}

#[test]
fn the_spec_states_the_hash_coverage_exactly() {
    // The load-bearing decision of the format: a moment's identity is its claim.
    let s = spec();
    for field in tt_core::CONTENT_HASH_FIELDS {
        assert!(s.contains(field), "TT-SPEC must state that `{field}` is hash-covered");
    }
    assert!(
        s.contains("classification") && s.contains("provenance"),
        "and it must state what is deliberately OUTSIDE the hash"
    );
}

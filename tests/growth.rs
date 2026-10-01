//! taxonomy v2.2.0 is Growth: two Lens A species and nothing else.
//!
//! The release notes claim three things a reader could otherwise only take on
//! trust, so they are checked here mechanically (GOVERNANCE §7):
//!
//! 1. every v2.1.0 node is still present with the same lens, level, parent,
//!    label, definition and retirement fields (no id changed meaning, none
//!    deleted);
//! 2. every distance between two v2.1.0 nodes is unchanged (the new species are
//!    leaves with only a parent edge, so they cannot shorten any existing path);
//! 3. the only additions are the two species, with no lateral edge, bridge or
//!    kernel change.
//!
//! The goldens below were computed from `bundle/taxonomy-v2.1.json` at tag
//! v2.2.0 (sha256 31ed385e…). The old file is no longer in the tree, so the
//! digests stand in for it. To recompute them from that file:
//!
//! ```text
//! git show v2.2.0:bundle/taxonomy-v2.1.json > /tmp/taxonomy-v2.1.json
//! TT_GOLDEN_BUNDLE=/tmp/taxonomy-v2.1.json cargo test --test growth -- --ignored --nocapture
//! ```

use sha2::{Digest, Sha256};
use tt_core::Bundle;

const NEW_IDS: [&str; 2] = ["procurement-and-contract-award", "public-program-adoption"];

/// The 149 v2.1.0 node records, canonical (JCS) and sorted by id.
const V2_1_NODES_SHA256: &str = "88c541ca4bd21339219c6b347617c9a098d074ef37dfb4558499494d6b38ea33";
/// Every unordered pair of v2.1.0 ids with its shortest-path distance.
const V2_1_DISTANCES_SHA256: &str = "3e111e6eaee7f5d6a1df0f0931c5ad0e652968bdfa8e9fe3fb403754b462ca91";
const V2_1_NODE_COUNT: usize = 149;

fn shipped() -> Bundle {
    Bundle::load_from_file(concat!(env!("CARGO_MANIFEST_DIR"), "/bundle/taxonomy-v2.2.json"))
        .expect("the shipped bundle loads")
}

fn sorted_ids(b: &Bundle, skip: &[&str]) -> Vec<String> {
    let mut ids: Vec<String> = b
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .filter(|id| !skip.contains(&id.as_str()))
        .collect();
    ids.sort();
    ids
}

fn nodes_digest(b: &Bundle, ids: &[String]) -> String {
    let mut h = Sha256::new();
    for id in ids {
        let node = b.node(id).expect("id present");
        let value = serde_json::to_value(node).expect("node serializes");
        h.update(serde_json_canonicalizer::to_string(&value).expect("canonical json"));
        h.update(b"\n");
    }
    hex::encode(h.finalize())
}

fn distances_digest(b: &Bundle, ids: &[String]) -> String {
    let index = b.distances();
    let mut h = Sha256::new();
    for (i, a) in ids.iter().enumerate() {
        for c in &ids[i + 1..] {
            let d = index
                .node_distance(a, c)
                .map_or_else(|| "none".to_string(), |d| format!("{d:.9}"));
            h.update(format!("{a}\t{c}\t{d}\n"));
        }
    }
    hex::encode(h.finalize())
}

#[test]
fn every_v2_1_node_keeps_its_meaning_and_none_was_deleted() {
    let b = shipped();
    let old = sorted_ids(&b, &NEW_IDS);
    assert_eq!(old.len(), V2_1_NODE_COUNT);
    assert_eq!(nodes_digest(&b, &old), V2_1_NODES_SHA256);
}

#[test]
fn every_distance_between_v2_1_nodes_is_unchanged() {
    let b = shipped();
    let old = sorted_ids(&b, &NEW_IDS);
    assert_eq!(distances_digest(&b, &old), V2_1_DISTANCES_SHA256);
}

#[test]
fn the_only_additions_are_two_leaf_lens_a_species() {
    let b = shipped();
    assert_eq!(b.version_string(), "tt-ontology/1.0 v2.2.0");
    assert_eq!(b.supersedes.as_deref(), Some("tt-ontology/1.0 v2.1.0"));
    for (id, parent) in [
        ("procurement-and-contract-award", "economy-trade-and-labor"),
        ("public-program-adoption", "politics-governance-and-law"),
    ] {
        let n = b.node(id).expect("new node present");
        assert_eq!((n.lens.as_str(), n.level.as_str()), ("A", "species"), "{id}");
        assert_eq!(n.parent.as_deref(), Some(parent), "{id}");
        assert!(n.deprecated_in.is_none(), "{id}");
        assert!(b.nodes.iter().all(|c| c.parent.as_deref() != Some(id)), "{id} is a leaf");
        assert!(
            b.lateral_edges.iter().all(|e| e.a != id && e.b != id),
            "{id} has no lateral edge"
        );
        assert!(
            b.bridges.iter().all(|br| br.event.as_deref() != Some(id) && br.action != id),
            "{id} has no bridge"
        );
        assert!(!b.kernel.iter().any(|k| k == id), "{id} is not kernel");
        // Reachable from the rest of Lens A, so a reading on it always has a distance.
        assert!(b.distances().node_distance(id, "treaty-alliance-and-peace-accord").is_some());
    }
    assert_eq!(b.lateral_edges.len(), 151);
    assert_eq!(b.bridges.len(), 26);
    assert_eq!(b.kernel.len(), 3);
}

/// Prints the goldens for the bundle at `TT_GOLDEN_BUNDLE` over ALL its ids.
#[test]
#[ignore]
fn print_goldens_for_a_bundle_file() {
    let path = std::env::var("TT_GOLDEN_BUNDLE").expect("TT_GOLDEN_BUNDLE names a bundle file");
    let b = Bundle::load_from_file(&path).expect("bundle loads");
    let ids = sorted_ids(&b, &[]);
    println!("count {}", ids.len());
    println!("nodes {}", nodes_digest(&b, &ids));
    println!("distances {}", distances_digest(&b, &ids));
}

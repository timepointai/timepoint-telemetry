//! taxonomy v2.3.0 is Growth: two Lens A subspecies and nothing else. It
//! follows v2.2.0, which was Growth too (two Lens A species).
//!
//! The release notes claim three things a reader could otherwise only take on
//! trust, so they are checked here mechanically (GOVERNANCE §7), against BOTH
//! earlier releases:
//!
//! 1. every v2.2.0 node (and so every v2.1.0 node) is still present with the
//!    same lens, level, parent, label, definition and retirement fields (no id
//!    changed meaning, none deleted);
//! 2. every distance between two v2.2.0 nodes (and so between two v2.1.0 nodes)
//!    is unchanged (the new subspecies are leaves with only a parent edge, so
//!    they cannot shorten any existing path);
//! 3. the only additions are the two subspecies, with no lateral edge, bridge or
//!    kernel change.
//!
//! The v2.1.0 goldens were computed from `bundle/taxonomy-v2.1.json` at tag
//! v2.2.0 (sha256 31ed385e…); the v2.2.0 goldens from `bundle/taxonomy-v2.2.json`
//! at tag v2.3.0 (sha256 d7081258…). Neither file is in the tree any more, so
//! the digests stand in for them. To recompute either:
//!
//! ```text
//! git show v2.3.0:bundle/taxonomy-v2.2.json > /tmp/taxonomy-v2.2.json
//! TT_GOLDEN_BUNDLE=/tmp/taxonomy-v2.2.json cargo test --test growth -- --ignored --nocapture
//! ```

use sha2::{Digest, Sha256};
use tt_core::Bundle;

/// Added in v2.3.0.
const NEW_IDS: [&str; 2] = ["monetary-policy-decision", "court-ruling"];
/// Added in v2.2.0; together with NEW_IDS, everything added since v2.1.0.
const V2_2_IDS: [&str; 2] = ["procurement-and-contract-award", "public-program-adoption"];

/// The 149 v2.1.0 node records, canonical (JCS) and sorted by id.
const V2_1_NODES_SHA256: &str = "88c541ca4bd21339219c6b347617c9a098d074ef37dfb4558499494d6b38ea33";
/// Every unordered pair of v2.1.0 ids with its shortest-path distance.
const V2_1_DISTANCES_SHA256: &str = "3e111e6eaee7f5d6a1df0f0931c5ad0e652968bdfa8e9fe3fb403754b462ca91";
const V2_1_NODE_COUNT: usize = 149;

/// The 151 v2.2.0 node records, canonical (JCS) and sorted by id.
const V2_2_NODES_SHA256: &str = "4e0b085cdefc05c0559bf87e30a774a1ae6eb493db3b32874df99129e1d5764b";
/// Every unordered pair of v2.2.0 ids with its shortest-path distance.
const V2_2_DISTANCES_SHA256: &str = "2f108df3a44ba2a22c136cf7434302f64e200952b604b3ee7865862f4a381dbc";
const V2_2_NODE_COUNT: usize = 151;

fn since_v2_1() -> Vec<&'static str> {
    NEW_IDS.iter().chain(V2_2_IDS.iter()).copied().collect()
}

fn shipped() -> Bundle {
    Bundle::load_from_file(concat!(env!("CARGO_MANIFEST_DIR"), "/bundle/taxonomy-v2.3.json"))
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
fn every_v2_2_node_keeps_its_meaning_and_none_was_deleted() {
    let b = shipped();
    let old = sorted_ids(&b, &NEW_IDS);
    assert_eq!(old.len(), V2_2_NODE_COUNT);
    assert_eq!(nodes_digest(&b, &old), V2_2_NODES_SHA256);
}

#[test]
fn every_distance_between_v2_2_nodes_is_unchanged() {
    let b = shipped();
    let old = sorted_ids(&b, &NEW_IDS);
    assert_eq!(distances_digest(&b, &old), V2_2_DISTANCES_SHA256);
}

#[test]
fn every_v2_1_node_keeps_its_meaning_and_none_was_deleted() {
    let b = shipped();
    let old = sorted_ids(&b, &since_v2_1());
    assert_eq!(old.len(), V2_1_NODE_COUNT);
    assert_eq!(nodes_digest(&b, &old), V2_1_NODES_SHA256);
}

#[test]
fn every_distance_between_v2_1_nodes_is_unchanged() {
    let b = shipped();
    let old = sorted_ids(&b, &since_v2_1());
    assert_eq!(distances_digest(&b, &old), V2_1_DISTANCES_SHA256);
}

#[test]
fn the_only_additions_are_two_leaf_lens_a_subspecies() {
    let b = shipped();
    assert_eq!(b.version_string(), "tt-ontology/1.0 v2.3.0");
    assert_eq!(b.supersedes.as_deref(), Some("tt-ontology/1.0 v2.2.0"));
    assert_eq!(b.nodes.len(), V2_2_NODE_COUNT + NEW_IDS.len());
    for (id, parent) in [
        ("monetary-policy-decision", "money-and-finance"),
        ("court-ruling", "law-and-justice"),
    ] {
        let n = b.node(id).expect("new node present");
        assert_eq!((n.lens.as_str(), n.level.as_str()), ("A", "subspecies"), "{id}");
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

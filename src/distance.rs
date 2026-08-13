//! TT distance — the query utility of TT-SPEC §5, made operative (PLAN B2.5d).
//! OWNED BY: TT-surfaces agent. Contract: CONTRACTS.md §11.6 "TT surfaces".
//!
//! "How alike are two moments? Weighted shortest path in the bundle graph (hierarchy
//! 1.0, lateral 1.6) between their mass-weighted profiles." The graph is the loaded
//! bundle's: an undirected edge per parent link (weight `metric.hierarchy_edge_weight`)
//! and per lateral edge (weight `metric.lateral_edge_weight`) — the weights come from
//! the LOADED bundle, never compiled in. Node-to-node distance is Dijkstra over that
//! graph.
//!
//! Between profiles the metric is the symmetric mass-weighted chamfer distance:
//! normalize each profile's masses to sum 1, then average the two directed terms
//! `Σᵢ âᵢ · minⱼ δ(i,j)` and `Σⱼ b̂ⱼ · minᵢ δ(i,j)`. This keeps the properties the
//! contract names: identical profiles (single- OR multi-node) are at distance 0,
//! single-node profiles reduce to the plain shortest path (parent–child = 1.0,
//! lateral pair = 1.6), and mass weights each node's contribution.
//!
//! Absence is typed, never 0.0: an empty profile, a weightless profile (masses sum
//! to ≤ 0), an id unknown to the loaded bundle, or an unreachable pair (the two
//! lenses are disjoint components — the bundle has no metric edges between them)
//! all return `None`.

use std::cmp::{Ordering, Reverse};
use std::collections::{BTreeMap, BinaryHeap, HashMap};

use crate::bundle::Bundle;

/// f64 path cost with a total order (`total_cmp`) so it can ride a `BinaryHeap`.
#[derive(PartialEq)]
struct Cost(f64);

impl Eq for Cost {}

impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cost {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

/// Mass-weighted distance between two classification profiles (id → mass maps, the
/// TT-SPEC §4 lens shape) over the bundle graph. See the module doc for the exact
/// metric. `None` is typed absence: empty or weightless profile, unknown id, or no
/// path between the profiles' supports (e.g. profiles from different lenses).
pub fn distance(
    bundle: &Bundle,
    a: &BTreeMap<String, f64>,
    b: &BTreeMap<String, f64>,
) -> Option<f64> {
    bundle.distances().between(a, b)
}

/// The bundle graph with every node-to-node shortest path already solved.
///
/// WHY THIS EXISTS. `distance()` rebuilt the index, the adjacency and one
/// Dijkstra per call. That is correct and it is why nothing ever used it: the
/// metric the taxonomy defines — and the 151 lateral edges that exist only to
/// feed it — sat unreachable behind a per-call cost nobody wanted to pay in a
/// request path. Solving all pairs once is 149 Dijkstras over ~300 edges and
/// about 178 KB held, so the metric becomes something a surface can ask
/// thousands of times while assembling a document.
///
/// Build it once (the server does, at boot, beside the Bundle) and share it.
#[derive(Debug)]
pub struct DistanceIndex {
    index: HashMap<String, usize>,
    /// Flat n×n matrix of shortest-path costs; `f64::INFINITY` where no path
    /// exists — the two lenses are disjoint components, which is a real answer.
    dist: Vec<f64>,
    n: usize,
}

impl DistanceIndex {
    pub fn build(bundle: &Bundle) -> Self {
        let index: HashMap<&str, usize> = bundle
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.as_str(), i))
            .collect();
        let adjacency = build_adjacency(bundle, &index);
        let n = bundle.nodes.len();
        let mut dist = vec![f64::INFINITY; n * n];
        for src in 0..n {
            let row = dijkstra(&adjacency, src);
            dist[src * n..(src + 1) * n].copy_from_slice(&row);
        }
        Self {
            index: index.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            dist,
            n,
        }
    }

    /// Shortest path between two node ids. `None` for an id the bundle does not
    /// carry or a pair with no path — never a silent 0.0.
    pub fn node_distance(&self, a: &str, b: &str) -> Option<f64> {
        let (i, j) = (*self.index.get(a)?, *self.index.get(b)?);
        let d = self.dist[i * self.n + j];
        d.is_finite().then_some(d)
    }

    /// The symmetric mass-weighted chamfer distance of the module doc. This is
    /// the ONE implementation of the metric; `distance()` calls it.
    pub fn between(&self, a: &BTreeMap<String, f64>, b: &BTreeMap<String, f64>) -> Option<f64> {
        if a.is_empty() || b.is_empty() {
            return None;
        }
        // Unknown id → None (absence typed, never a silent 0.0 or a skipped entry).
        let idx = |id: &String| self.index.get(id.as_str()).copied();
        let a_idx: Vec<usize> = a.keys().map(idx).collect::<Option<_>>()?;
        let b_idx: Vec<usize> = b.keys().map(idx).collect::<Option<_>>()?;

        let a_total: f64 = a.values().sum();
        let b_total: f64 = b.values().sum();
        let has_mass = |total: f64| total.is_finite() && total > 0.0;
        if !has_mass(a_total) || !has_mass(b_total) {
            // Weightless (or NaN-weighted) profiles carry no mass to weight — absence.
            return None;
        }

        // Directed term a→b: each a-node contributes its normalized mass times
        // the shortest path to the NEAREST b-node.
        let mut a_to_b = 0.0;
        for (&i, mass) in a_idx.iter().zip(a.values()) {
            let nearest = b_idx
                .iter()
                .map(|&j| self.dist[i * self.n + j])
                .fold(f64::INFINITY, f64::min);
            if nearest.is_infinite() {
                return None; // no path at all between the profiles — typed absence
            }
            a_to_b += (mass / a_total) * nearest;
        }
        // Directed term b→a over the transpose.
        let mut b_to_a = 0.0;
        for (&j, mass) in b_idx.iter().zip(b.values()) {
            let nearest = a_idx
                .iter()
                .map(|&i| self.dist[i * self.n + j])
                .fold(f64::INFINITY, f64::min);
            if nearest.is_infinite() {
                return None;
            }
            b_to_a += (mass / b_total) * nearest;
        }

        Some((a_to_b + b_to_a) / 2.0)
    }
}

/// Undirected weighted adjacency over node indices: hierarchy edges (child↔parent)
/// at `metric.hierarchy_edge_weight`, lateral edges at `metric.lateral_edge_weight`.
/// Weights are read from the loaded bundle (TT-SPEC: no consumer compiles them in).
fn build_adjacency(bundle: &Bundle, index: &HashMap<&str, usize>) -> Vec<Vec<(usize, f64)>> {
    let mut adjacency = vec![Vec::new(); bundle.nodes.len()];
    let hier = bundle.metric.hierarchy_edge_weight;
    let lateral = bundle.metric.lateral_edge_weight;
    for (i, node) in bundle.nodes.iter().enumerate() {
        if let Some(parent) = node.parent.as_deref()
            && let Some(&p) = index.get(parent)
        {
            adjacency[i].push((p, hier));
            adjacency[p].push((i, hier));
        }
    }
    for edge in &bundle.lateral_edges {
        if let (Some(&x), Some(&y)) = (index.get(edge.a.as_str()), index.get(edge.b.as_str())) {
            adjacency[x].push((y, lateral));
            adjacency[y].push((x, lateral));
        }
    }
    adjacency
}

/// Textbook Dijkstra from `source`; unreachable nodes stay `f64::INFINITY`.
fn dijkstra(adjacency: &[Vec<(usize, f64)>], source: usize) -> Vec<f64> {
    let mut dist = vec![f64::INFINITY; adjacency.len()];
    dist[source] = 0.0;
    let mut heap = BinaryHeap::new();
    heap.push(Reverse((Cost(0.0), source)));
    while let Some(Reverse((Cost(d), u))) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &(v, w) in &adjacency[u] {
            let nd = d + w;
            if nd < dist[v] {
                dist[v] = nd;
                heap.push(Reverse((Cost(nd), v)));
            }
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::distance;
    use crate::bundle::Bundle;
    use std::collections::BTreeMap;

    const EPS: f64 = 1e-9;

    fn real_bundle() -> Bundle {
        Bundle::load_from_file(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/bundle/taxonomy-v2.1.json"
        ))
        .expect("vendored bundle loads")
    }

    fn profile(entries: &[(&str, f64)]) -> BTreeMap<String, f64> {
        entries.iter().map(|(id, m)| (id.to_string(), *m)).collect()
    }

    #[test]
    fn identical_profiles_are_at_distance_zero() {
        let bundle = real_bundle();
        // Single-node.
        let single = profile(&[("marriage-and-union", 1.0)]);
        assert_eq!(distance(&bundle, &single, &single), Some(0.0));
        // Multi-node with uneven masses — still exactly 0 (every node's nearest
        // counterpart is itself).
        let multi = profile(&[
            ("marriage-and-union", 0.6),
            ("courtship-and-falling-in-love", 0.3),
        ]);
        assert_eq!(distance(&bundle, &multi, &multi), Some(0.0));
    }

    #[test]
    fn parent_child_is_one_hierarchy_edge() {
        let bundle = real_bundle();
        // bonding-and-kinship (branch) is the parent of marriage-and-union (species).
        let parent = profile(&[("bonding-and-kinship", 1.0)]);
        let child = profile(&[("marriage-and-union", 1.0)]);
        let d = distance(&bundle, &parent, &child).expect("path exists");
        assert!((d - 1.0).abs() < EPS, "parent-child = 1.0, got {d}");
    }

    #[test]
    fn lateral_pair_is_one_lateral_edge() {
        let bundle = real_bundle();
        // courtship-and-falling-in-love ~ intimacy-and-sex is a lateral edge (1.6).
        let a = profile(&[("courtship-and-falling-in-love", 1.0)]);
        let b = profile(&[("intimacy-and-sex", 1.0)]);
        let d = distance(&bundle, &a, &b).expect("path exists");
        assert!((d - 1.6).abs() < EPS, "lateral pair = 1.6, got {d}");
    }

    #[test]
    fn cross_branch_value_computed_by_hand() {
        let bundle = real_bundle();
        // homemaking-and-dwelling (species, branch movement-and-dwelling) to the
        // BRANCH making-and-cultivating. By hand: no single edge joins them (branches
        // have no laterals; hierarchy stays within a branch). A two-edge path costs
        // 2.0 (impossible here: 1.0+1.0 needs a shared hierarchy neighbor and the two
        // branches share none), 2.6, or 3.2; the 2.6 path EXISTS:
        //   homemaking-and-dwelling ~(lateral 1.6)~ building-and-construction
        //   building-and-construction —(hierarchy 1.0)→ making-and-cultivating.
        // Three or more edges cost ≥ 3.0 > 2.6, so the shortest path is exactly 2.6.
        let a = profile(&[("homemaking-and-dwelling", 1.0)]);
        let b = profile(&[("making-and-cultivating", 1.0)]);
        let d = distance(&bundle, &a, &b).expect("path exists");
        assert!((d - 2.6).abs() < EPS, "hand-computed 2.6, got {d}");

        // Sibling species under one branch: courtship → bonding-and-kinship →
        // marriage is 2.0; every lateral detour costs more (1.6+1.6 = 3.2, and any
        // mixed two-edge path ≥ 2.6). Exactly 2.0.
        let a = profile(&[("courtship-and-falling-in-love", 1.0)]);
        let b = profile(&[("marriage-and-union", 1.0)]);
        let d = distance(&bundle, &a, &b).expect("path exists");
        assert!((d - 2.0).abs() < EPS, "hand-computed 2.0, got {d}");
    }

    #[test]
    fn masses_weight_the_contribution_and_normalize() {
        let bundle = real_bundle();
        // a→b: bonding (0.5 of the mass) is 1.0 from marriage, marriage (0.5) is 0.
        // b→a: marriage's nearest a-node is itself (0). Chamfer = (0.5 + 0)/2 = 0.25.
        let a = profile(&[("bonding-and-kinship", 0.5), ("marriage-and-union", 0.5)]);
        let b = profile(&[("marriage-and-union", 1.0)]);
        let d = distance(&bundle, &a, &b).expect("path exists");
        assert!((d - 0.25).abs() < EPS, "hand-computed 0.25, got {d}");

        // Masses are normalized per profile: scaling every mass leaves d unchanged.
        let a_scaled = profile(&[("bonding-and-kinship", 0.2), ("marriage-and-union", 0.2)]);
        let b_scaled = profile(&[("marriage-and-union", 0.4)]);
        let d2 = distance(&bundle, &a_scaled, &b_scaled).expect("path exists");
        assert!(
            (d - d2).abs() < EPS,
            "normalization invariance: {d} vs {d2}"
        );
    }

    #[test]
    fn distance_is_symmetric() {
        let bundle = real_bundle();
        let a = profile(&[
            ("homemaking-and-dwelling", 0.7),
            ("journey-and-travel", 0.3),
        ]);
        let b = profile(&[("making-and-cultivating", 1.0)]);
        let ab = distance(&bundle, &a, &b).expect("path exists");
        let ba = distance(&bundle, &b, &a).expect("path exists");
        assert!((ab - ba).abs() < EPS, "symmetry: {ab} vs {ba}");
    }

    #[test]
    fn unknown_id_and_empty_profiles_are_typed_absence() {
        let bundle = real_bundle();
        let known = profile(&[("marriage-and-union", 1.0)]);
        let unknown = profile(&[("not-a-bundle-id", 1.0)]);
        assert_eq!(distance(&bundle, &unknown, &known), None, "unknown id in a");
        assert_eq!(distance(&bundle, &known, &unknown), None, "unknown id in b");
        assert_eq!(distance(&bundle, &BTreeMap::new(), &known), None, "empty a");
        assert_eq!(distance(&bundle, &known, &BTreeMap::new()), None, "empty b");
        // Weightless profile: present ids, no mass to weight — absence, never 0.0.
        let weightless = profile(&[("marriage-and-union", 0.0)]);
        assert_eq!(distance(&bundle, &weightless, &known), None, "weightless a");
    }

    #[test]
    fn cross_lens_profiles_have_no_path_and_return_none() {
        let bundle = real_bundle();
        // The metric graph has hierarchy + lateral edges only; the two lenses are
        // disjoint components (bridges are NOT metric edges).
        let lens_b = profile(&[("bonding-and-kinship", 1.0)]);
        let lens_a = profile(&[("politics-governance-and-law", 1.0)]);
        assert_eq!(distance(&bundle, &lens_b, &lens_a), None);
    }
}

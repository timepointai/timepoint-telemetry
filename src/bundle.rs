//! The ontology bundle: parse + validate at load. All check families from
//! genesis/snag/validate.py are ported as loader-time assertions — loading an invalid
//! bundle is an error and MUST fail boot. Contract: CONTRACTS.md §6, TT-SPEC §2.

use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::TtError;

/// The closed set of bridge relations (TT-SPEC §2 / validate.py check 5).
pub const RELATIONS: [&str; 4] = ["constitutes", "scales-up-to", "recorded-as", "unrecorded"];

const LEVELS: [&str; 3] = ["branch", "species", "subspecies"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub lens: String,
    pub level: String,
    pub parent: Option<String>,
    pub label: String,
    pub definition: String,
    // ── RETIREMENT (GOVERNANCE.md §3) ───────────────────────────────────────
    //
    // An id never changes meaning and is never deleted. Without a way to retire
    // one, the taxonomy could only ever grow, and would eventually collapse
    // under its own history. A deprecated node stays in the bundle, stays
    // readable forever, and stops being a target for new classification.
    //
    // All three default to absent, so every bundle written before this existed
    // still parses unchanged.
    /// The version that retired this node. Its presence IS the deprecation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated_in: Option<String>,
    /// The node that replaces it. Absent only when nothing does — and then
    /// `deprecation_note` has to say why.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    /// One sentence a stranger can act on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecation_note: Option<String>,
}

impl Node {
    pub fn is_deprecated(&self) -> bool {
        self.deprecated_in.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bridge {
    pub action: String,
    pub relation: String,
    pub event: Option<String>,
    /// Free-text annotation; eight bridge rows carry `""` by design (ERRATA #7).
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LateralEdge {
    pub a: String,
    pub b: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lens {
    pub label: String,
    pub question: String,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lenses {
    #[serde(rename = "A")]
    pub a: Lens,
    #[serde(rename = "B")]
    pub b: Lens,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub hierarchy_edge_weight: f64,
    pub lateral_edge_weight: f64,
    pub distance: String,
}

#[derive(Debug, Deserialize)]
pub struct Bundle {
    pub schema: String,
    pub version: String,
    /// The release immediately behind this one, `"<schema> v<version>"`, e.g.
    /// `"snag-ontology/1.0 v1.1.0"`. `None` on the first release of a chain.
    ///
    /// This was carried in `raw` only and modelled nowhere, which made the
    /// supersession chain — the mechanism that lets a record written under an
    /// older schema id still be read — invisible to every consumer of this
    /// crate. It went unnoticed while the chain had only ever been walked by a
    /// person reading the JSON. Renaming the format to TT is the second link
    /// (Clockchain → SNAG → TT), so it is now load-bearing.
    ///
    /// A bundle names ONE step back, so resolving a record older than one
    /// release means walking a version at a time. That is a deliberate
    /// limitation of the format and not an oversight here.
    #[serde(default)]
    pub supersedes: Option<String>,
    pub metric: Metric,
    pub lenses: Lenses,
    pub nodes: Vec<Node>,
    pub lateral_edges: Vec<LateralEdge>,
    pub bridges: Vec<Bridge>,
    pub kernel: Vec<String>,
    /// The full bundle document as parsed, kept verbatim so a server can hand it out
    /// unchanged (fields the typed struct does not model — governance,
    /// design_principles — are not lost). `Value::Null` only on hand-built bundles.
    #[serde(skip)]
    pub raw: Value,
    /// All-pairs shortest paths over this bundle's graph, solved on first use.
    ///
    /// The metric TT-SPEC §5 defines was implemented and never called, because
    /// `distance()` rebuilt the whole graph per call — too expensive to put in a
    /// request path, so the 151 lateral edges that feed it stayed dead. Solving
    /// once and hanging the result off the bundle makes it free to ask.
    #[serde(skip)]
    distances: std::sync::OnceLock<crate::distance::DistanceIndex>,
}

impl Bundle {
    /// Load and validate a bundle from a file path. An invalid bundle is an error.
    pub fn load_from_file(path: &str) -> Result<Self, TtError> {
        let raw = std::fs::read_to_string(path)?;
        Self::load_from_str(&raw)
    }

    /// Load and validate a bundle from raw JSON text. An invalid bundle is an error.
    pub fn load_from_str(raw: &str) -> Result<Self, TtError> {
        let raw_value: Value = serde_json::from_str(raw)?;
        let mut bundle: Bundle = serde_json::from_value(raw_value.clone())?;
        bundle.raw = raw_value;
        bundle.validate()?;
        Ok(bundle)
    }

    /// The bundle graph with every shortest path solved. Built on first call
    /// (149 Dijkstras, ~178 KB) and shared thereafter.
    pub fn distances(&self) -> &crate::distance::DistanceIndex {
        self.distances
            .get_or_init(|| crate::distance::DistanceIndex::build(self))
    }

    /// `"<schema> v<version>"`, e.g. `"tt-ontology/1.0 v2.0.0"`. Doubles as the ETag.
    pub fn version_string(&self) -> String {
        format!("{} v{}", self.schema, self.version)
    }

    /// Whether `version_string` names this release or any release it supersedes,
    /// one step back. A record stamped with the previous schema id still belongs
    /// to this bundle; one stamped two releases back does not, and the caller
    /// needs the intervening bundle to say so rather than being told "no".
    pub fn is_this_release(&self, version_string: &str) -> bool {
        version_string == self.version_string()
            || self.supersedes.as_deref() == Some(version_string)
    }

    pub fn is_valid_id(&self, id: &str) -> bool {
        self.nodes.iter().any(|n| n.id == id)
    }

    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Follow supersession to the node that should be used TODAY.
    ///
    /// Reading a retired id must still give an answer — that is the whole point
    /// of never deleting one. `Ok(None)` means the id is unknown; a retired id
    /// with no successor resolves to ITSELF, because "this was retired and
    /// nothing replaces it" is a real answer and must not be confused with
    /// "never existed".
    ///
    /// Cycles are DETECTED rather than survived. A chain that loops is a broken
    /// bundle, and looping quietly would hang whatever asked.
    pub fn resolve(&self, id: &str) -> Result<Option<&Node>, TtError> {
        let mut seen: Vec<&str> = Vec::new();
        let mut cur = match self.node(id) {
            Some(n) => n,
            None => return Ok(None),
        };
        loop {
            if seen.contains(&cur.id.as_str()) {
                seen.push(&cur.id);
                return Err(TtError::Invalid(format!(
                    "supersession cycle: {}",
                    seen.join(" -> ")
                )));
            }
            seen.push(&cur.id);
            match cur
                .superseded_by
                .as_deref()
                .and_then(|next| self.node(next))
            {
                Some(next) => cur = next,
                None => return Ok(Some(cur)),
            }
        }
    }

    /// Every node retired as of this bundle.
    pub fn deprecated(&self) -> Vec<&Node> {
        self.nodes.iter().filter(|n| n.is_deprecated()).collect()
    }

    pub fn bridge_for(&self, action_id: &str) -> Option<&Bridge> {
        self.bridges.iter().find(|b| b.action == action_id)
    }

    /// The node itself, then each parent up to the branch root. Unknown id → empty.
    pub fn parent_chain(&self, id: &str) -> Vec<&Node> {
        let mut out = Vec::new();
        let mut cur = self.node(id);
        while let Some(n) = cur {
            out.push(n);
            if out.len() > LEVELS.len() {
                // A validated bundle nests at most branch→species→subspecies; a hand-built
                // cyclic bundle must not loop us forever.
                break;
            }
            cur = n.parent.as_deref().and_then(|p| self.node(p));
        }
        out
    }

    pub fn kernel(&self) -> &[String] {
        &self.kernel
    }

    /// Every check family from genesis/snag/validate.py, all failures collected — the
    /// error message lists each failed check by its validate.py name.
    pub fn validate(&self) -> Result<(), TtError> {
        let mut failures: Vec<String> = Vec::new();

        // Retirement rules (GOVERNANCE.md §3). Checked at LOAD, because a
        // dangling successor is the kind of thing that reads fine in a diff and
        // strands every record pointing at it.
        {
            let ids: std::collections::HashSet<&str> =
                self.nodes.iter().map(|n| n.id.as_str()).collect();
            for n in self.nodes.iter().filter(|n| n.is_deprecated()) {
                match n.superseded_by.as_deref() {
                    Some(sup) if !ids.contains(sup) => {
                        failures.push(format!("node {} superseded_by unknown id {sup}", n.id))
                    }
                    Some(sup) if sup == n.id => {
                        failures.push(format!("node {} supersedes itself", n.id))
                    }
                    // Retired with no successor is legal — a node that should
                    // never have existed has none — but it must say why.
                    None if n.deprecation_note.is_none() => failures.push(format!(
                        "node {} is deprecated with neither superseded_by nor deprecation_note",
                        n.id
                    )),
                    _ => {}
                }
            }
            // A successor that is itself retired is fine (chains are allowed);
            // a chain that closes is not.
            for n in self.nodes.iter().filter(|n| n.is_deprecated()) {
                if let Err(TtError::Invalid(e)) = self.resolve(&n.id) {
                    failures.push(e);
                }
            }
        }

        // Index + duplicate detection.
        let mut by_id: HashMap<&str, &Node> = HashMap::with_capacity(self.nodes.len());
        let mut dupes: Vec<&str> = Vec::new();
        for n in &self.nodes {
            if by_id.insert(n.id.as_str(), n).is_some() {
                dupes.push(n.id.as_str());
            }
        }

        // 1. kebab-case + unique.
        let bad_kebab: Vec<&str> = self
            .nodes
            .iter()
            .filter(|n| !is_kebab(&n.id))
            .map(|n| n.id.as_str())
            .collect();
        if !bad_kebab.is_empty() {
            failures.push(format!("node ids kebab-case: {}", bad_kebab.join(", ")));
        }
        if !dupes.is_empty() {
            failures.push(format!("node ids unique: {}", dupes.join(", ")));
        }

        // 1b. lens/level sanity.
        let bad_lens: Vec<&str> = self
            .nodes
            .iter()
            .filter(|n| n.lens != "A" && n.lens != "B")
            .map(|n| n.id.as_str())
            .collect();
        if !bad_lens.is_empty() {
            failures.push(format!("node lens in {{A,B}}: {}", bad_lens.join(", ")));
        }
        let bad_level: Vec<&str> = self
            .nodes
            .iter()
            .filter(|n| !LEVELS.contains(&n.level.as_str()))
            .map(|n| n.id.as_str())
            .collect();
        if !bad_level.is_empty() {
            failures.push(format!("node level valid: {}", bad_level.join(", ")));
        }

        // 2. parents resolve; branches have null parent; parent shares lens.
        // 2b. level nesting: species→branch, subspecies→species.
        let mut missing_parent: Vec<String> = Vec::new();
        let mut bad_branch_parent: Vec<&str> = Vec::new();
        let mut cross_lens_parent: Vec<&str> = Vec::new();
        let mut bad_nesting: Vec<&str> = Vec::new();
        for n in &self.nodes {
            if n.level == "branch" {
                if n.parent.is_some() {
                    bad_branch_parent.push(n.id.as_str());
                }
                continue;
            }
            let parent_node = match &n.parent {
                None => {
                    missing_parent.push(format!("{} -> null", n.id));
                    None
                }
                Some(p) => match by_id.get(p.as_str()) {
                    None => {
                        missing_parent.push(format!("{} -> {}", n.id, p));
                        None
                    }
                    Some(pn) => {
                        if pn.lens != n.lens {
                            cross_lens_parent.push(n.id.as_str());
                        }
                        Some(*pn)
                    }
                },
            };
            let parent_level = parent_node.map(|p| p.level.as_str());
            if n.level == "species" && parent_level != Some("branch") {
                bad_nesting.push(n.id.as_str());
            }
            if n.level == "subspecies" && parent_level != Some("species") {
                bad_nesting.push(n.id.as_str());
            }
        }
        if !missing_parent.is_empty() {
            failures.push(format!("parents exist: {}", missing_parent.join("; ")));
        }
        if !bad_branch_parent.is_empty() {
            failures.push(format!(
                "branches have null parent: {}",
                bad_branch_parent.join(", ")
            ));
        }
        if !cross_lens_parent.is_empty() {
            failures.push(format!(
                "parent shares lens: {}",
                cross_lens_parent.join(", ")
            ));
        }
        if !bad_nesting.is_empty() {
            failures.push(format!("level nesting correct: {}", bad_nesting.join(", ")));
        }

        // 3. branch counts + lenses block match (order-sensitive, like validate.py).
        let a_branches: Vec<&str> = self
            .nodes
            .iter()
            .filter(|n| n.lens == "A" && n.level == "branch")
            .map(|n| n.id.as_str())
            .collect();
        let b_branches: Vec<&str> = self
            .nodes
            .iter()
            .filter(|n| n.lens == "B" && n.level == "branch")
            .map(|n| n.id.as_str())
            .collect();
        // THE COUNTS ARE OVER LIVE BRANCHES, NOT ALL OF THEM.
        //
        // Retirement (GOVERNANCE.md §3) keeps a node in the bundle forever, so
        // deprecating a branch and adding its successor leaves SEVEN Lens-A
        // branch nodes even though six are in service. Counting rows would
        // therefore refuse to load — and because loading a bundle is how a
        // consumer starts, that is not a degraded surface, it is a process that
        // will not boot. Retirement shipped without this and would have bricked
        // the first Structure release that touched a branch.
        //
        // The frozen numbers stay frozen: adding a branch to SERVICE is still a
        // code change here as well as a bundle change, which is exactly why
        // branches sit in the slowest window.
        let live = |ids: &[&str]| -> usize {
            ids.iter()
                .filter(|id| self.node(id).is_none_or(|n| !n.is_deprecated()))
                .count()
        };
        let (live_a, live_b) = (live(&a_branches), live(&b_branches));
        if live_a != 6 {
            failures.push(format!("Lens A live branch count == 6: got {live_a}"));
        }
        if live_b != 9 {
            failures.push(format!("Lens B live branch count == 9: got {live_b}"));
        }
        if self.lenses.a.branches != a_branches {
            failures.push("lenses.A.branches matches nodes".to_owned());
        }
        if self.lenses.b.branches != b_branches {
            failures.push("lenses.B.branches matches nodes".to_owned());
        }

        // 4. lateral edges: endpoints exist, no self-loops, within a lens, no dupes,
        // weight == metric.lateral_edge_weight.
        let mut bad_edges: Vec<String> = Vec::new();
        let mut self_edges: Vec<&str> = Vec::new();
        let mut cross_lens_edges: Vec<String> = Vec::new();
        let mut seen_edges: BTreeSet<(&str, &str)> = BTreeSet::new();
        let mut dupe_edges: Vec<String> = Vec::new();
        let mut bad_weight: Vec<usize> = Vec::new();
        for (i, e) in self.lateral_edges.iter().enumerate() {
            for (end, name) in [(&e.a, "a"), (&e.b, "b")] {
                if !by_id.contains_key(end.as_str()) {
                    bad_edges.push(format!("{end} ({name})"));
                }
            }
            if e.a == e.b {
                self_edges.push(e.a.as_str());
            }
            if let (Some(na), Some(nb)) = (by_id.get(e.a.as_str()), by_id.get(e.b.as_str()))
                && na.lens != nb.lens
            {
                cross_lens_edges.push(format!("{}~{}", e.a, e.b));
            }
            let key = if e.a.as_str() <= e.b.as_str() {
                (e.a.as_str(), e.b.as_str())
            } else {
                (e.b.as_str(), e.a.as_str())
            };
            if !seen_edges.insert(key) {
                dupe_edges.push(format!("{}~{}", key.0, key.1));
            }
            if e.weight.to_bits() != self.metric.lateral_edge_weight.to_bits() {
                bad_weight.push(i);
            }
        }
        if !bad_edges.is_empty() {
            failures.push(format!(
                "lateral edge endpoints exist: {}",
                bad_edges.join(", ")
            ));
        }
        if !self_edges.is_empty() {
            failures.push(format!("no self-loops: {}", self_edges.join(", ")));
        }
        if !cross_lens_edges.is_empty() {
            failures.push(format!(
                "lateral edges stay within a lens: {}",
                cross_lens_edges.join(", ")
            ));
        }
        if !dupe_edges.is_empty() {
            failures.push(format!(
                "no duplicate lateral edges: {}",
                dupe_edges.join(", ")
            ));
        }
        if !bad_weight.is_empty() {
            failures.push(format!(
                "lateral weights == metric.lateral_edge_weight: {bad_weight:?}"
            ));
        }

        // 5/6. bridges: action in Lens B, event (when present) in Lens A, relation in the
        // closed set, null event iff relation == "unrecorded".
        let mut bad_action: Vec<&str> = Vec::new();
        let mut bad_event: Vec<&str> = Vec::new();
        let mut bad_relation: Vec<&str> = Vec::new();
        let mut null_event_actions: Vec<&str> = Vec::new();
        let mut unrecorded_actions: Vec<&str> = Vec::new();
        for br in &self.bridges {
            match by_id.get(br.action.as_str()) {
                Some(n) if n.lens == "B" => {}
                _ => bad_action.push(br.action.as_str()),
            }
            if !RELATIONS.contains(&br.relation.as_str()) {
                bad_relation.push(br.relation.as_str());
            }
            match &br.event {
                None => null_event_actions.push(br.action.as_str()),
                Some(ev) => match by_id.get(ev.as_str()) {
                    Some(n) if n.lens == "A" => {}
                    _ => bad_event.push(ev.as_str()),
                },
            }
            if br.relation == "unrecorded" {
                unrecorded_actions.push(br.action.as_str());
            }
        }
        if !bad_action.is_empty() {
            failures.push(format!(
                "bridge actions exist in Lens B: {}",
                bad_action.join(", ")
            ));
        }
        if !bad_event.is_empty() {
            failures.push(format!(
                "bridge events exist in Lens A: {}",
                bad_event.join(", ")
            ));
        }
        if !bad_relation.is_empty() {
            failures.push(format!(
                "bridge relations in closed set: {}",
                bad_relation.join(", ")
            ));
        }
        let mut null_sorted = null_event_actions.clone();
        null_sorted.sort_unstable();
        let mut unrecorded_sorted = unrecorded_actions.clone();
        unrecorded_sorted.sort_unstable();
        if null_sorted != unrecorded_sorted {
            failures.push(format!(
                "exactly unrecorded bridges have null event: null={null_sorted:?} unrecorded={unrecorded_sorted:?}"
            ));
        }

        // 7. kernel == null-event bridge actions.
        let mut kernel_sorted: Vec<&str> = self.kernel.iter().map(String::as_str).collect();
        kernel_sorted.sort_unstable();
        if kernel_sorted != null_sorted {
            failures.push(format!(
                "kernel == null-event bridge actions: kernel={kernel_sorted:?} null={null_sorted:?}"
            ));
        }

        // 8. non-empty definitions and labels.
        let no_def: Vec<&str> = self
            .nodes
            .iter()
            .filter(|n| n.definition.trim().is_empty())
            .map(|n| n.id.as_str())
            .collect();
        if !no_def.is_empty() {
            failures.push(format!(
                "every node has a non-empty definition: {}",
                no_def.join(", ")
            ));
        }
        let no_label: Vec<&str> = self
            .nodes
            .iter()
            .filter(|n| n.label.trim().is_empty())
            .map(|n| n.id.as_str())
            .collect();
        if !no_label.is_empty() {
            failures.push(format!(
                "every node has a non-empty label: {}",
                no_label.join(", ")
            ));
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(TtError::Invalid(failures.join("; ")))
        }
    }
}

/// `^[a-z0-9]+(-[a-z0-9]+)*$` without a regex dependency.
fn is_kebab(id: &str) -> bool {
    !id.is_empty()
        && id.split('-').all(|seg| {
            !seg.is_empty()
                && seg
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

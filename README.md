# Timepoint Telemetry

**A shared, frozen-identity vocabulary for recording what happened — and an
honest account of what the record cannot see.**

Most systems that record events invent their categories as they go. The
categories then mean whatever the code meant that week, and two records can
never be compared — not across time, and certainly not across organisations.

Timepoint Telemetry is a fixed, versioned taxonomy with **frozen identity**: a
node id never changes meaning, structure may grow, and a structural change is a
semantic version bump. That is what makes a moment recorded today comparable
with one recorded next year, or by somebody else.

---

## Start with the thing it says out loud

The taxonomy reads every moment through **two lenses** — *what did the record
keep?* and *what were people doing?* — and it ships **bridges** carrying an
action into the event it becomes.

Three actions have a bridge that says, explicitly, that **no event exists**:

```json
{"action": "migration-and-resettlement",     "relation": "unrecorded", "event": null,
 "note": "no event node exists for ordinary migration"}
{"action": "journey-and-travel",             "relation": "unrecorded", "event": null,
 "note": "travel is nearly invisible to the events graph"}
{"action": "courtship-and-falling-in-love",  "relation": "unrecorded", "event": null,
 "note": "the private core of a life leaves no public event"}
```

These are the **kernel**, and they are the point. History's record is
structurally blind to migration, to travel, and to falling in love — to most of
what a life actually consists of. A vocabulary that can *state* that blindness
is worth more than one that quietly drops it.

Which is why one of the design principles is **a missing bridge is a finding,
not a gap**. "The record does not keep this kind of thing" and "we have not
mapped this yet" are opposite claims, and a system that collapses them is
lying in the more flattering direction.

## What is here

| | |
|---|---|
| `bundle/taxonomy-v1.1.json` | the versioned taxonomy — 149 nodes, 2 lenses, 151 lateral edges, 26 bridges, the kernel, metric weights, design principles |
| `src/bundle.rs` | load + validate a bundle; parent chains, bridge lookup, id validity |
| `src/envelope.rs` | RFC 8785 canonicalisation, `content_hash`, `provenance_hash` |
| `src/distance.rs` | node-to-node and distribution-to-distribution distance |
| `vectors/` | conformance vectors — the hashes an implementation must reproduce |
| `tests/` | the suite the reference implementation passes (33 tests) |

The crate is named `snag-core` — SNAG is the ontology's schema id
(`snag-ontology/1.0`) and the name the reference implementation has carried
since before this repository existed. Keeping it means the code here is
*literally* the code Timepoint runs, not a re-typing of it.

Its lineage is **Clockchain**; the bundle declares the succession itself:

```json
"schema":     "snag-ontology/1.0",
"version":    "1.1.0",
"supersedes": "clockchain-taxonomy/1.0 v1.1.0-alpha.1",
"governance": "structure fluid, identity frozen — ids never change; structural change = semver bump"
```

## The model in five minutes

### Two lenses

| Lens | Asks | Branches | Nodes |
|---|---|---|---|
| **A — Recorded Public Events** | what did the record keep? | 6 | 79 |
| **B — Human Action & Behavior** | what were people doing? | 9 | 70 |

The lenses are disjoint components of the graph. There is no path between them,
and the distance function returns *no answer* rather than a convenient number
when asked for one.

### Distributions, not labels

A classification is a mass distribution over each lens, not a category:

```json
{
  "lens_b": { "negotiation-and-agreement": 0.55, "deciding-and-judging": 0.25 },
  "lens_a": { "corporate-founding-and-milestone": 0.6 },
  "abstain": false
}
```

Validity is **rejected, never repaired**: at most 3 entries per lens, every mass
in (0, 1], each lens summing to at most 1.0, every id present in the loaded
bundle under the correct lens. Two consequences worth stating:

- **Mass may sit on a branch.** "Somewhere in `bonding-and-kinship`, 0.6" is a
  real answer, not a failure to be precise.
- **`abstain: true` is publishable.** Never guess — untagged is valid.

### Bridges

| Relation | Count | Reading |
|---|---|---|
| `recorded-as` | 14 | the record keeps this as that |
| `scales-up-to` | 6 | at scale, this becomes that |
| `constitutes` | 3 | this is part of that |
| `unrecorded` | 3 | **no public event exists** — the kernel |

Walking an action up its parent chain to the first bridge is deterministic code
over the loaded bundle. A model may propose a classification; it does not get to
invent what that classification implies.

### The metric

Weighted shortest path over the bundle graph — a hierarchy edge costs **1.0**, a
lateral edge (the taxonomy asserting two nodes are adjacent in meaning) costs
**1.6**. Between two distributions it is the symmetric mass-weighted chamfer
distance.

Absence stays typed: an empty profile, a weightless one, an unknown id, or an
unreachable pair return `None`, never `0.0`.

### A claim is its content

```
content_hash    = sha256(canonical(payload))      // RFC 8785
provenance_hash = sha256(canonical(provenance))
```

There is no join table for "the same thing said twice". Two moments sharing a
`content_hash` **are** the same claim, told again — by a different system, from
a different source, possibly by a different party. Provenance is hashed
separately, so who said it never changes what was said.

## Use it

```toml
[dependencies]
snag-core = { git = "https://github.com/timepointai/timepoint-telemetry" }
```

```rust
let bundle = snag_core::Bundle::load_from_file("bundle/taxonomy-v1.1.json")?;

// Is this id real, and what does it mean?
let node = bundle.node("courtship-and-falling-in-love").unwrap();

// Where does this action land in the events graph?
match bundle.bridge_for(&node.id) {
    Some(b) if b.event.is_none() => { /* the kernel: no public event exists */ }
    Some(b) => { /* carried into b.event by b.relation */ }
    None    => { /* unmapped — a gap in the map, not a finding */ }
}

// How near are two classifications?
let d = bundle.distances().between(&profile_a, &profile_b); // Option<f64>
```

```
cargo test    # 33 tests, including every conformance vector
```

## Being compatible

A system is compatible when:

1. it classifies against a **published bundle version**, and cites it;
2. its classifications pass the validator unmodified — including abstention as a
   legitimate result;
3. its moments carry a `content_hash` computed the same way, so the same claim
   from two systems collides **on purpose**;
4. it derives shadows from the bundle's bridges rather than asserting
   relationships of its own, and reports the kernel and an unmapped node as the
   different things they are.

## What is deliberately not here

The product. Rooms, pricing, ledgers, simulation pipelines, anything belonging
to anyone's record. Timepoint is built on this; this is not Timepoint.

## Status

**Early and honest about it.** The taxonomy is at `1.1.0` and its identity
guarantee is real — ids will not change meaning. The surrounding spec prose is
still being written down; this README and the tests are currently the
normative description, and where they disagree, **the tests and the bundle
win**.

Issues and disagreements are welcome, particularly about the taxonomy itself. A
vocabulary is worth exactly as much as the number of parties willing to
classify against it.

## Licence

Apache-2.0. See [LICENSE](LICENSE).

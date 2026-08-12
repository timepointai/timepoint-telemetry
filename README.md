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
| `bundle/taxonomy-v2.0.json` | the versioned taxonomy — 149 nodes, 2 lenses, 151 lateral edges, 26 bridges, the kernel, metric weights, design principles |
| `src/bundle.rs` | load + validate a bundle; parent chains, bridge lookup, id validity |
| `src/envelope.rs` | RFC 8785 canonicalisation, `content_hash`, `provenance_hash` |
| `src/distance.rs` | node-to-node and distribution-to-distribution distance |
| `TT-SPEC.md` | **the normative description** — what a conforming implementation must do. Written from this implementation, not ahead of it |
| `GOVERNANCE.md` | how this changes — classes, pacing, retirement, migrations |
| `vectors/` | conformance vectors — the hashes an implementation must reproduce |
| `tests/` | the suite the reference implementation passes (51 tests) |
| `.github/workflows/ci.yml` | build · tests · clippy `-D warnings` · conformance as its own job, on every push |

**Where TT-SPEC and the vectors disagree, the vectors win.** A specification can
be read two ways; a hash cannot. Every figure TT-SPEC prints about the bundle is
asserted by `tests/spec.rs`, so the document cannot drift from the thing it
describes without the suite going red.

The crate is named `tt-core`, after the ontology's schema id
(`tt-ontology/1.0`). It is *literally* the code Timepoint runs, not a
re-typing of it.

Its lineage is **Clockchain → SNAG → TT**, and the bundle declares the step
immediately behind it:

```json
"schema":     "tt-ontology/1.0",
"version":    "2.0.0",
"supersedes": "snag-ontology/1.0 v1.1.0",
"governance": "structure fluid, identity frozen — ids never change; structural change = semver bump"
```

**v2.0.0 renames the format and changes nothing else.** The counts are
identical to v1.1.0 — 149 nodes, 151 lateral edges, 26 bridges, the same three
kernel members — and **no node id moved**, which is the guarantee that makes a
rename safe to ship at all. It is a major bump because every consumer keying on
the schema string has to change, not because any record changed meaning.

The format was called **SNAG** through v1.1.0. That name is retired; the chain
records it so nothing written under it is orphaned. A bundle names one step
back, so reading a record more than one release old means walking the chain a
version at a time — `Bundle::is_this_release` answers for one step, and
resolving further needs the intervening bundle.

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
tt-core = { git = "https://github.com/timepointai/timepoint-telemetry" }
```

```rust
let bundle = tt_core::Bundle::load_from_file("bundle/taxonomy-v2.0.json")?;

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
cargo test    # 40 tests, including every conformance vector
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

## How this changes

The update process is part of the format. People trust the format because they
trust the process, and a vocabulary with excellent structure and unpredictable
governance is not something anyone can build on.

**[GOVERNANCE.md](GOVERNANCE.md)** is the whole of it. In short:

- Changes come in three classes — **Correction**, **Growth**, **Structure** —
  decided by *effect*, not intent: could this cause a record to be read
  differently than it was written?
- There are **two release windows and no others**, both at the close of the New
  York Stock Exchange: a **daily** one carrying Corrections and Growth, and a
  **Friday** one carrying Structure with a full week of announced notice. An
  exchange close is a real instant the world already coordinates on, and it
  resolves holidays and half-days without anyone keeping a calendar.
- **An id never changes meaning and is never deleted.** Retirement happens
  through deprecation: the id stays readable forever and resolution carries you
  to whatever replaces it.
- **A proposal carries its solution** — the edit, the evidence, and a migration
  that reconciles records already classified under the old shape, declaring
  Prune / Synthesize / **Store** (the default) for every divergence it creates
  or exposes.
- Timepoint decides today. The triggers for opening that up are written down.

Proposals use [the change request template](.github/CHANGE_REQUEST.md).

## Status

**Early and honest about it.** The taxonomy is at `2.0.0` and its identity
guarantee is real — ids will not change meaning.

[TT-SPEC.md](TT-SPEC.md) is the normative description, written from the
implementation rather than ahead of it. Where it and the committed conformance
vectors disagree, **the vectors win and the spec is a bug** — and a test suite
asserts every figure the spec prints about the bundle, so it cannot drift
silently.

Issues and disagreements are welcome, particularly about the taxonomy itself. A
vocabulary is worth exactly as much as the number of parties willing to
classify against it.

This is not only a taxonomy. It is a reference system for the real world — the
shared ground that lets independent agents, platforms and parties coordinate
about what happened without sharing a database or an employer. That is why
identity is frozen, why the process is written down before it is needed, and
why the kernel is stated rather than hidden.

## Licence

**Business Source License 1.1** — see [LICENSE](LICENSE).

Source-available, not open source, and the difference is worth stating plainly:

- **You may use it in production.** Classify your own records against the
  taxonomy, store the classifications, exchange them with anyone. That is the
  point of the thing.
- **You may not** offer it to third parties as a hosted or managed service whose
  primary value is classification against, resolution of, or distribution of
  this taxonomy, in competition with Timepoint.
- **Publishing a bundle version, an implementation, or conformance results is
  never a competing offering.**
- **On the Change Date — four years after publication — each version converts
  to Apache-2.0** automatically and permanently. Every version carries its own
  clock, so what you adopt today is permissively licensed on a date you can
  read off the calendar.

### If you adopted this when it was Apache-2.0

The format was published as [`snag-core`](https://github.com/timepointai/snag-core)
under Apache-2.0 through v1.1.0. That repository is archived and frozen, and
**everything released there stays Apache-2.0** — the grant is irrevocable and
nothing here changes it. v1.1.0 remains available on those terms.

This repository, from v2.0.0, is BSL 1.1. Said plainly rather than left to be
discovered: the licence changed at the same moment the name did. The taxonomy
itself did not — no node id moved between v1.1.0 and v2.0.0.

# Timepoint Telemetry

**A reference system for the real world — the shared ground that lets
independent agents, platforms and parties coordinate about what happened
without sharing a database, a schema, or an employer.**

A community resource for the ML community: a shared, versioned vocabulary for
recording what happened, maintained on a tightly controlled, published update
cycle.

Two disambiguations, up front. This is **not OpenTelemetry** — nothing here is
a span, a trace or a metrics pipeline; "telemetry" here means records of
real-world events. And it is **source-available, not open source**: BSL 1.1,
with each version converting to Apache-2.0 four years after its publication,
and the one reserved right being competing hosted classification.
[Details below](#licence) — stated here so you don't discover it at the bottom.

## The failure it exists for

Three agents watch the same meeting happen. One records `"meeting"`. One
records `"sync"`. One records `"negotiation"`. Nothing downstream can now
deduplicate the three records, compare them, or measure how far apart they are
— not because any agent was wrong, but because there is no shared ground to be
wrong *against*.

Most systems that record events invent their categories as they go, so the
categories mean whatever the code meant that week. Timepoint Telemetry
replaces that with three fixed artifacts that ship and version together: a
**taxonomy** whose node ids never change meaning, an **identity rule** that
makes the same claim hash to the same digest in any implementation, and a
**metric** that says how near two records are in meaning. Records made by
strangers, years apart, become comparable — or provably the same claim.

## Where to start

| You want to | Read | Minutes |
|---|---|---|
| know what this is | this page, down through the worked moment | 3 |
| see the whole mechanism run on one object | [docs/WORKED-EXAMPLE.md](docs/WORKED-EXAMPLE.md) | 5 |
| ship a compatible system | [Being compatible](#being-compatible) + the [glossary](#glossary) | 15 |
| implement it from scratch | [TT-SPEC.md](TT-SPEC.md) | 60 |
| prove your implementation conforms | [vectors/](vectors/) + `tests/vectors.rs` | 30 |

You do not need the spec to use this. Classify against a published bundle,
pass the validator, hash claims the same way — that is the whole contract.

## The worked moment

One object, end to end — [docs/WORKED-EXAMPLE.md](docs/WORKED-EXAMPLE.md)
carries it through every step; this is the shape of it. On 9 June 1815, seven
powers sign the Final Act of the Congress of Vienna:

```json
{
  "label": "Final Act of the Congress of Vienna signed",
  "occurs_at": "1815-06-09",
  "participants": ["austria", "britain", "france", "portugal", "prussia", "russia", "sweden"],
  "classification": {
    "lens_b": { "negotiation-and-agreement": 0.7, "deciding-and-judging": 0.2 },
    "lens_a": { "treaty-alliance-and-peace-accord": 0.85 },
    "abstain": false
  }
}
```

- The first three fields are **the claim**. Canonicalised (RFC 8785) and
  hashed: `content_hash = sha256:6cdce3f7…`. A second system recording the
  same claim from a different source produces the **same hash** — same claim,
  told twice, recognised with no coordination. Re-classifying the moment does
  not change it: the classification sits outside the hash.
- The classification is a **mass distribution per lens**, not a category.
- Deterministic code walks the action to its implied event:
  `negotiation-and-agreement → scales-up-to → treaty-alliance-and-peace-accord`
  — *"a treaty is a handshake, at scale."*
- Against the Treaty of Paris (five months later, classified independently):
  distance **0.21** on Lens A. Against Mount Tambora erupting the same year:
  **8.8**, on a scale whose farthest pair is 10.4 — and `None` on Lens B,
  because the eruption's record honestly abstains there.

## What is here

| | |
|---|---|
| `bundle/taxonomy-v2.1.json` | the versioned taxonomy — 149 nodes, 2 lenses, 151 lateral edges, 26 bridges, the kernel, metric weights, design principles |
| `docs/WORKED-EXAMPLE.md` | one moment carried end to end — every number computed, none illustrative |
| `docs/how-tt-works.svg` | the model in one picture |
| `docs/CONSUMERS.md` | the consumer contract — five obligations, and the stricter-not-looser rule |
| `vectors/classification-verdicts.json` | 39 §4 verdict vectors — codes normative, detail strings advisory |
| `src/bundle.rs` | load + validate a bundle; parent chains, bridge lookup, id validity |
| `src/envelope.rs` | RFC 8785 canonicalisation, `content_hash`, `provenance_hash` |
| `src/distance.rs` | node-to-node and distribution-to-distribution distance |
| `TT-SPEC.md` | **the normative description** — what a conforming implementation must do. Written from this implementation, not ahead of it |
| `GOVERNANCE.md` | how this changes — classes, pacing, retirement, migrations |
| `vectors/` | 10 conformance vectors — the hashes an implementation must reproduce |
| `tests/` | the suite the reference implementation passes (51 tests) |
| `.github/workflows/ci.yml` | build · tests · clippy `-D warnings` · conformance as its own job, on every push |

**Where TT-SPEC and the vectors disagree, the vectors win.** A specification
can be read two ways; a hash cannot. Every figure TT-SPEC prints about the
bundle is asserted by `tests/spec.rs`, so the document cannot drift from the
thing it describes without the suite going red.

The crate is named `tt-core`, after the ontology's schema id
(`tt-ontology/1.0`). It is *literally* the code Timepoint runs, not a
re-typing of it. Its lineage is **Clockchain → SNAG → TT**, and the bundle
declares the step immediately behind it:

```json
"schema":     "tt-ontology/1.0",
"version":    "2.0.0",
"supersedes": "snag-ontology/1.0 v1.1.0",
"governance": "structure fluid, identity frozen — ids never change; structural change = semver bump"
```

**v2.1.0 is the current release and the first Structure release**: it retires
`everyday-movement-and-commute` into `journey-and-travel` — the id stays in the
bundle forever, stops being a target for new work, and `resolve()` carries every
stored reading onto the successor. Counts are unchanged. **v2.0.0 renamed the
format from SNAG and changed nothing else** — no node id moved, which is the
guarantee that made the rename safe to ship. The name SNAG is retired; the
chain records it so nothing written under it is orphaned. A bundle names one
step back, so reading a record more than one release old means walking the
chain a version at a time.

## Start with the thing it says out loud

Three actions have a bridge that says, explicitly, that **no event node
exists**:

```json
{"action": "migration-and-resettlement",     "relation": "unrecorded", "event": null,
 "note": "no event node exists for ordinary migration"}
{"action": "journey-and-travel",             "relation": "unrecorded", "event": null,
 "note": "travel is nearly invisible to the events graph"}
{"action": "courtship-and-falling-in-love",  "relation": "unrecorded", "event": null,
 "note": "the private core of a life leaves no public event"}
```

These are the **kernel**. The claim is precise, so state it precisely:
migration is among the most *documented* of human activities — ship manifests,
censuses, naturalization files — and courtship surfaces in marriage registers.
What the record keeps is paperwork: traces, not events. There is no headline,
no dated public happening, that an ordinary migration, journey or courtship
*becomes* — and so there is no Lens A node for one. The kernel asserts that
absence in data instead of letting it pass as an oversight.

Which is why one of the bundle's design principles reads, verbatim, **"a
missing bridge is a finding, not a gap"** — the kernel's explicit `null` is
the missing bridge it means, stated rather than absent. An action with no
bridge entry at all is the other case: a gap, unmapped so far. "The record
does not keep this kind of thing" and "we have not mapped this yet" are
opposite claims, and a system that collapses them is lying in the more
flattering direction.

## What this is not

The prior art is real and mostly orthogonal. If you know these systems, here
is the fastest way to locate TT among them:

| System | What it does | Why TT isn't it |
|---|---|---|
| [OpenTelemetry](https://opentelemetry.io) | traces, metrics and logs for running software | name collision only — nothing here is a span. TT records world events, not systems |
| [PROV-O](https://www.w3.org/TR/prov-o/) | provenance graphs for digital artifacts (entity, activity, agent) | TT hashes provenance but its subject is the world event, not the derivation history of data |
| [CIDOC-CRM](https://www.cidoc-crm.org) | an extensible ontology for cultural-heritage documentation | a framework for building vocabularies; TT is one small finished vocabulary with claim identity and a metric, neither of which CRM defines |
| [schema.org/Event](https://schema.org/Event) | event markup for search engines | no frozen ids, no versioned releases, no distributions, no distance, no conformance vectors |
| [Wikidata](https://www.wikidata.org) | an open, continuously edited knowledge base | TT is deliberately closed and small, with scheduled releases a record can cite exactly |
| [OWL-Time](https://www.w3.org/TR/owl-time/) | temporal relations — intervals, before/after | orthogonal: TT says *what* happened; it takes no position on temporal algebra |
| [C2PA](https://c2pa.org) | signed provenance for media files | authenticates an asset's editing chain; TT identifies a claim about the world |
| [Dublin Core](https://www.dublincore.org) | metadata fields for documents | describes documents; TT describes happenings |

The one deliberate overlap: like several of these, TT believes vocabulary
outlives software. Unlike most of them, it ships a distance metric, a claim
identity rule, and byte-level conformance vectors — the parts you need for two
*systems* to agree, not just two catalogues.

## The model in five minutes

![How Timepoint Telemetry works — a moment is hashed into a content/provenance
pair and classified as a mass distribution over two disjoint lenses; bridges
carry actions into the events they become, three of them stating that no event
exists; distance is a weighted shortest path over the bundle
graph.](docs/how-tt-works.svg)

### Two lenses

| Lens | Asks | Branches | Nodes |
|---|---|---|---|
| **A — Recorded Public Events** | what did the record keep? | 6 | 79 |
| **B — Human Action & Behavior** | what were people doing? | 9 | 70 |

The lenses are disjoint components of the graph. Within a lens, every pair of
nodes is reachable — all 3,081 Lens A pairs and 2,415 Lens B pairs have a
finite distance, verified. Across lenses there is no path at all, and the
distance function returns *no answer* rather than a convenient number.

### Distributions, not labels

Two independent systems classify the Vienna signing. One reads it
`{negotiation-and-agreement: 0.7, deciding-and-judging: 0.2}`; the other
`{negotiation-and-agreement: 0.6, persuasion-and-rhetoric: 0.25}`. Neither is
wrong. Their distance is **0.47** — near, on a lens whose farthest nodes sit
9.2 apart — and *that number*, not agreement on a single label, is what two
honest readings of one event look like.

So a classification is a mass distribution over each lens, not a category, and
validity is **rejected, never repaired**: at most 3 entries per lens, every
mass in (0, 1], each lens summing to at most 1.0, every id present in the
loaded bundle under the correct lens. Two consequences worth stating:

- **Mass may sit on a branch.** "Somewhere in `bonding-and-kinship`, 0.6" is a
  real answer, not a failure to be precise.
- **`abstain: true` is publishable.** Never guess — untagged is valid.

### Bridges

`negotiation-and-agreement` carries a bridge:
`scales-up-to → treaty-alliance-and-peace-accord` — *"a treaty is a handshake,
at scale."* Walking an action up its parent chain to the first bridge is
deterministic code over the loaded bundle; the derived `{relation, event}`
pair is the moment's **shadow**. A model may propose a classification; it does
not get to invent what that classification implies.

| Relation | Count | Reading |
|---|---|---|
| `recorded-as` | 14 | the record keeps this as that |
| `scales-up-to` | 6 | at scale, this becomes that |
| `constitutes` | 3 | this is part of that |
| `unrecorded` | 3 | **no public event exists** — the kernel |

### The metric, anchored

1.0 and 1.6 are the two edge costs — hierarchy and lateral — but bare
constants mean nothing, so here is the ruler, computed from the shipped
bundle:

| Pair | Distance | Reading |
|---|---|---|
| `negotiation-and-agreement` ↔ `communication-and-exchange` | 1.0 | child to its branch |
| `war-declaration-and-outbreak` ↔ `treaty-alliance-and-peace-accord` | 1.6 | a lateral edge — war and peace, adjacent in meaning by assertion |
| `negotiation-and-agreement` ↔ `conversation-and-storytelling` | 2.0 | siblings under one branch |
| `treaty-alliance-and-peace-accord` ↔ `pitched-battle` | 3.2 | neighbouring neighbourhoods |
| `negotiation-and-agreement` ↔ `migration-and-resettlement` | 5.2 | same lens, distant branches |
| `treaty-alliance-and-peace-accord` ↔ `volcanic-and-geological` | 8.8 | opposite ends of the recorded world |
| `earthquake-and-seismic` ↔ `spaceflight-and-frontier` | 10.4 | the farthest pair in Lens A |
| any Lens B node ↔ any Lens A node | `None` | no path exists, and none is invented |

Between distributions it is the symmetric mass-weighted chamfer distance over
these node distances (0.21 for the two treaty readings above). Absence stays
typed: an empty profile, a weightless one, an unknown id, or an unreachable
pair return `None`, never `0.0`.

One honest caveat: the 1.0/1.6 weights are design choices, not values fitted
to human similarity judgments. They are consistent, published, and so far
unvalidated — if you run a study against them, the tracker wants to hear from
you.

### A claim is its content

```
content_hash    = sha256(canonical(payload))      // RFC 8785, over {label, occurs_at, participants} only
provenance_hash = sha256(canonical(provenance))
```

There is no join table for "the same thing said twice". Two moments sharing a
`content_hash` **are** the same claim, told again — by a different system,
from a different source, possibly by a different party. Provenance is hashed
separately, so who said it never changes what was said — and because the
classification sits outside the hash, re-classifying a moment keeps its
identity. The [worked example](docs/WORKED-EXAMPLE.md) shows both hashes
computed on the Vienna moment, twice.

## What it refuses, verbatim

Constraints are learned fastest from violations. Five things you will try, and
what actually comes back — every message below is real output from this crate:

**1. Hash a payload missing a claim field** (no `occurs_at`):

```
payload missing required claim field `occurs_at`
```

**2. Verify a stored moment whose payload was edited** (one character added to
the label):

```
content_hash mismatch: stored sha256:6cdce3f7…, recomputed sha256:8f879240…
```

**3. Ask for distance to an id that does not exist.** While drafting this
document we invented `battle-and-campaign` — plausible, wrong (the real node
is `pitched-battle`). The metric returned `None`, not a small number. So does
a typo'd id, an empty profile, and an abstained lens. Absence is an answer,
never a zero.

**4. Ask for a cross-lens distance** — `negotiation-and-agreement` (B) to
`treaty-alliance-and-peace-accord` (A):

```
None
```

They are one bridge apart and *no* distance apart; those are different facts,
and the metric refuses to conflate them.

**5. Load a bundle with a branch quietly removed** — the loader refuses it at
the door, before any record can be read against the mutilated shape:

```
bundle invalid: lenses.A.branches matches nodes
```

And the boundary cases that are *not* errors, because they are representable
states: `resolve("everyday-movement-and-commute")` returns its successor
`journey-and-travel` (retired, carried forward); `resolve("horse-trading")`
returns `None` (never existed); `bridge_for` on an unmapped action returns
`None` (a gap), while a kernel action returns a bridge whose event is `null`
(a finding).

## Glossary

One sentence each, in dependency order.

- **bundle** — one published release of the taxonomy: nodes, lenses, lateral
  edges, bridges, kernel and metric weights in a single versioned JSON file.
- **node** — one meaning with a permanent id, a label, a definition, a lens
  and a parent; the things classifications put mass on.
- **lens** — one of the two disjoint trees a moment is read through: A (what
  did the record keep?) and B (what were people doing?).
- **branch** — a root node of a lens; mass may sit on one when nothing more
  specific is honest.
- **lateral edge** — the taxonomy asserting two same-lens nodes are adjacent
  in meaning beyond the hierarchy; costs 1.6 in the metric.
- **moment** — one recorded event: a claim (`label`, `occurs_at`,
  `participants`) plus classification and provenance, identified by its hashes.
- **profile** — the per-lens mass distribution inside a classification (at
  most 3 entries, masses in (0, 1], sum ≤ 1.0).
- **bridge** — a directed mapping from a Lens B action to the Lens A event it
  becomes when the record keeps it.
- **kernel** — the three bridges whose event is `null`: the taxonomy stating
  the record keeps no event for these actions.
- **shadow** — the `{relation, event}` a stored classification implies,
  derived by walking the bridge — recorded with the moment so the same
  classification always yields the same shadow.
- **resolve** — following a retired id to its successor, so old records read
  correctly under new bundles.
- **vector** — a committed conformance case: input, expected canonical bytes,
  expected hash; reproduce all 10 byte-for-byte and your implementation
  conforms.

## Use it

```toml
[dependencies]
tt-core = { git = "https://github.com/timepointai/timepoint-telemetry" }
```

```rust
let bundle = tt_core::Bundle::load_from_file("bundle/taxonomy-v2.1.json")?;

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
cargo test    # 51 tests, including every conformance vector
```

**Not a Rust shop?** `python/` is the stdlib-only on-ramp — no pip, no venv:

- `tt_validate.py` — the §4 classification validator: reject-never-repair,
  typed rejections (a retired id names its successor; an unknown id never
  existed), abstention first-class, bundle string stamped on accept.
- `tt_envelope.py` — RFC 8785 canonicalisation and both hashes, written fresh
  against the committed vectors and passing all 10 byte-for-byte in CI.
- `classification.schema.json` — the shape, for toolchains that speak JSON
  Schema; the validator remains the authority on bundle-dependent truth.

A second independent Python port of the envelope also runs the vectors in
another repo's CI (TT-SPEC §6) — written blind to this one, which is the
point.

## Being compatible

A system is compatible when:

1. it classifies against a **published bundle version**, and cites it;
2. its classifications pass the validator unmodified — including abstention as
   a legitimate result;
3. its moments carry a `content_hash` computed the same way, so the same claim
   from two systems collides **on purpose**;
4. it derives [shadows](#glossary) from the bundle's bridges rather than
   asserting relationships of its own, and reports the kernel and an unmapped
   node as the different things they are.

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

**Early and honest about it.** The taxonomy is at `2.1.0` and its identity
guarantee is real — ids will not change meaning, and change arrives only
through the published update cycle.

[TT-SPEC.md](TT-SPEC.md) is the normative description, written from the
implementation rather than ahead of it. Where it and the committed conformance
vectors disagree, **the vectors win and the spec is a bug** — and a test suite
asserts every figure the spec prints about the bundle, so it cannot drift
silently.

Two measurements this repo owes and has not yet made, said before someone else
says them: the metric's edge weights are unvalidated against human similarity
judgments, and the claim that a shared vocabulary beats free-text tags for
inter-system agreement is — so far — an argument, not a benchmark. A
vocabulary is worth exactly as much as the number of parties willing to
classify against it, and that is a measurable quantity.

Issues and disagreements are welcome, particularly about the taxonomy itself.

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
  never a competing offering.** Contributing to the taxonomy means contributing
  to a vocabulary Timepoint currently stewards and commercially hosts — that
  trade is stated here rather than left to be discovered.
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

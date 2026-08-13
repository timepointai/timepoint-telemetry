# TT-SPEC — Timepoint Telemetry, normatively

**Applies to:** `tt-ontology/1.0 v2.1.0` · crate `tt-core` 2.1.0
**Status:** normative. Where this document and the code disagree, **the committed
conformance vectors win** — see §6.

---

## §0. What this is, and why it was missing

This document was cited normatively in 39 places across two repositories before
it existed. `bundle.rs`, `envelope.rs`, `distance.rs` and the beta classifier all
said "TT-SPEC §2", "TT-SPEC §3", "TT-SPEC §4", "TT-SPEC §5" — pointing at a
document that lived in neither repo.

That was survivable while there was one implementation, because the code *was*
the spec and nothing could disagree with it. It stopped being survivable the
moment a second implementation appeared: an independent Python port of the
envelope hashing, written against the committed vectors, which found a real bug
on its own side (`Decimal(repr(f))` retaining a trailing zero, rendering
integers as `3.0`). A specification with one reader has been tested for
self-consistency, never for ambiguity.

**This document describes what the code does, not what it should do.** Every
claim below was read out of the implementation or the vectors. Where it says a
number, that number was counted.

---

## §1. The shape of the thing

TT is three artifacts that ship together and version together:

| | |
|---|---|
| **The bundle** | a versioned taxonomy — nodes, two lenses, lateral edges, bridges, the kernel, metric weights |
| **The envelope** | how one recorded moment is identified: canonicalisation and two hashes |
| **The metric** | how near two records are in meaning |

A release is identified by `"<schema> v<version>"` — for this one,
`tt-ontology/1.0 v2.1.0`. That string is the ETag a server serves the bundle
under and the value a record cites to say which vocabulary made it.

**Lineage is one step back.** A bundle names the release it supersedes and no
further:

```json
{ "schema": "tt-ontology/1.0", "version": "2.1.0",
  "supersedes": "tt-ontology/1.0 v2.0.0" }
```

Reading a record more than one release old therefore means walking the chain a
version at a time. That is a deliberate limitation, not an oversight:
`Bundle::is_this_release` answers for one step and refuses two, because claiming
a reach the format does not have would be worse than saying no.

---

## §2. The taxonomy

### §2.1 Two lenses, disjoint

Every moment is read twice, because "what happened" and "what people were doing"
are different questions and neither reduces to the other.

| Lens | Asks | Branches | Nodes |
|---|---|---|---|
| **A — Recorded Public Events** | what did the record keep? | 6 | 79 |
| **B — Human Action & Behavior** | what were people doing? | 9 | 70 |

149 nodes total: 15 branches, 87 species, 47 subspecies. **The two lenses are
disjoint components of the graph.** There is no path from one to the other
except a bridge (§2.4), and a node id belongs to exactly one lens — an id
offered under the wrong lens is not a mistake to be corrected, it is unknown
(§4.2).

The branch counts are **frozen**, and a loader must refuse a bundle that
violates them. This is why adding a branch is a code change as well as a bundle
change, and belongs in the slowest release window.

### §2.2 Levels and the parent chain

Every node carries `lens`, `level` ∈ {`branch`, `species`, `subspecies`},
`parent`, `label`, `definition`.

- A `branch` has `parent: null`. Everything else has a parent that exists in the
  same bundle and the same lens.
- The parent chain from any node terminates at a branch. A cycle is a broken
  bundle, detected at load, not survived at read.

### §2.3 Identity is frozen; structure is not

The governance line the bundle states about itself:

> structure fluid, identity frozen — ids never change; structural change = semver bump

**An id never changes meaning and is never deleted.** That is the guarantee the
whole format rests on: a record classified in 2026 must still mean the same
thing in 2036. Structure — parents, lateral edges, which nodes exist — is
expected to move, and a move is a version bump.

### §2.4 Bridges, and the kernel

A bridge carries an **action** (Lens B) into an **event** (Lens A). The relation
set is closed — exactly four values:

| Relation | Count | Reading |
|---|---|---|
| `recorded-as` | 14 | the record keeps this as that |
| `scales-up-to` | 6 | at scale, this becomes that |
| `constitutes` | 3 | this is part of that |
| `unrecorded` | 3 | **no public event exists for this** |

`event` is present for the first three and **null exactly for `unrecorded`**.
Every bridge event is a Lens-A node; a consumer may rely on that.

Those three `unrecorded` rows are the **kernel**:

- `migration-and-resettlement`
- `journey-and-travel`
- `courtship-and-falling-in-love`

**A missing bridge is a finding, not a gap.** The kernel says the public record
is structurally blind to some of the most human things. That is a claim about
the world. It must never be confused with a node nobody has mapped yet, which
is a claim about us. An implementation that reports both as "no event" has
destroyed the format's sharpest statement — and this is not hypothetical: it
happened, and 44 unmapped nodes were reported as kernel for weeks.

### §2.5 Retirement

An id is never deleted. To retire one:

| Field | Meaning |
|---|---|
| `deprecated_in` | the version that retired it. **Its presence IS the deprecation.** |
| `superseded_by` | the node that replaces it. Absent only when nothing does |
| `deprecation_note` | required when there is no successor: one sentence a stranger can act on |

Rules a loader enforces:

- `superseded_by` must name a node that exists in the same bundle.
- A node may not supersede itself, and a supersession chain that loops is an
  error — detected, never followed.
- Retired with **no** successor is legal, and must carry a note saying why.

**Resolution.** `resolve(id)` follows supersession to the node in service today.
Three outcomes, and the distinctions matter:

- an id the bundle does not carry → **nothing**. Never a plausible neighbour.
- a retired id with a successor → the successor.
- a retired id with **no** successor → **itself**. "This was retired and nothing
  replaces it" is a real answer and must not collapse into "never existed".

**Branch counts are checked over LIVE branches**, not all rows. A retired branch
stays in the bundle, so counting rows would make the first release that retires
one fail to load — and since loading is how a consumer starts, that is a process
that will not boot rather than a degraded surface.

---

## §3. The envelope

### §3.1 Canonicalisation

**RFC 8785 (JCS)**. Keys sorted by UTF-16 code unit; numbers serialized per
ECMAScript `Number::toString`. Integers beyond 2⁵³ lose precision exactly as
IEEE-754 doubles do — the vectors freeze that behaviour rather than hiding it.

### §3.2 The two hashes

```
content_hash    = "sha256:" + hex(sha256(canon({label, occurs_at, participants})))
provenance_hash = "sha256:" + hex(sha256(canon(<the whole provenance value>)))
```

**`content_hash` covers exactly three payload fields and nothing else:**

| Field | In `content_hash`? |
|---|---|
| `label` | ✅ |
| `occurs_at` | ✅ |
| `participants` | ✅ |
| `classification` | ❌ |
| `grounding` | ❌ |
| `basis_note` | ❌ |
| everything else in the payload | ❌ |
| the provenance object | ❌ (hashed separately) |

This is the load-bearing decision of the whole format. **A moment's identity is
its claim**, so re-classifying or re-grounding a moment does not change what it
is. Two moments sharing a `content_hash` **are** the same claim told again — by
a different run, from a different source, possibly by a different party. There
is no join table; the hash is the link.

### §3.3 Absence is typed

A hash-covered field that is **present but null** hashes as null — a
representable absence. A field that is **missing entirely** is a typed error,
not a hole hashed over. A claim needs all three.

---

## §4. The classification contract

A classification is **not a category**. It is a mass distribution over each lens.

```json
{ "lens_b": { "negotiation-and-agreement": 0.55, "deciding-and-judging": 0.25 },
  "lens_a": { "corporate-founding-and-milestone": 0.6 },
  "abstain": false,
  "bundle": "tt-ontology/1.0 v2.1.0" }
```

### §4.1 The rules

1. At most **3 entries per lens**.
2. Every mass in **(0, 1]** — zero, negative and >1 all reject.
3. Each lens's masses sum to **at most 1.0** (+1e-9 float epsilon).
4. Every id must exist in the loaded bundle **under the lens it is offered
   under**.
5. No id twice in a lens.
6. Unknown top-level keys reject. Missing lens keys mean empty maps.

### §4.2 Reject, never repair

A classification that violates the contract is **thrown back whole**. It is
never silently corrected, clamped, or partially accepted. Silent correction
produces a record that looks clean and is wrong, which is worse than a refusal
because nobody goes looking for it.

Rejections are typed and distinguish cases a caller must handle differently —
in particular, **a retired id and an unknown id are different rejections**. "This
was retired in 2.1.0; use X" and "this never existed" are different facts about
the world, and collapsing them makes a retry prompt useless.

### §4.3 Mass at a branch is legal

When the species is uncertain, putting mass on the **branch** is a real answer,
not a failure to be precise. "Somewhere in bonding-and-kinship, 0.6" is
information.

### §4.4 Abstention is a result, not an error

`abstain: true` with empty lenses is **valid and publishable**. A reader that
declines to label something is telling the truth about what it could see.

Two things follow, and both have been got wrong in practice:

- An abstention is a reading of a **specific vocabulary** — "this bundle had
  nothing for it" — so it cites its bundle like any other classification.
- **"Declined" and "never asked" are different.** An integrity statistic that
  divides abstentions by *all* moments rather than by moments *actually put to a
  reader* is reporting a number that is not about the reader at all.

### §4.5 Provenance of the reading

A stored classification names the release that produced it (`bundle`). Without
it, a node id is a string rather than a reading: the taxonomy is versioned
precisely because structure moves, so "which vocabulary was this classified
under" must have an answer.

Accepted but never required: a producer does not send it — the validator stamps
it. A stored classification must therefore round-trip: **what the validator
emits, the validator accepts.**

### §4.6 Shadow derivation is code, never model

A model proposes a classification. It does **not** decide what that
classification implies. Walking the Lens-B argmax up its parent chain to the
first bridge is deterministic code over the loaded bundle. The same
classification always yields the same shadow, and a model cannot invent a
relationship the taxonomy does not assert.

The walk yields exactly one of three states, and they must stay distinct
(§2.4): an **event**, the **kernel**, or **unmapped**.

---

## §5. The metric

"How near are these two records?" — weighted shortest path over the bundle
graph.

| Edge | Weight |
|---|---|
| hierarchy (parent ↔ child) | **1.0** |
| lateral (the taxonomy asserting two nodes are adjacent in meaning) | **1.6** |

The bundle ships **151 lateral edges**. They are undirected.

**Between two distributions**, the metric is the symmetric mass-weighted chamfer
distance: normalise each side, then average the two directed terms. Identical
profiles are exactly 0. Single-node profiles reduce to the plain shortest path.

**Absence stays typed rather than becoming a convenient zero.** An empty
profile, a weightless one, an unknown id, or an unreachable pair all return *no
answer*. The two lenses are disjoint components (§2.1), so there is no path
between them — and that is a correct result, not an error.

---

## §6. Conformance

**The committed vectors are normative.** `vectors/*.json` carry, for each case,
the input document, the expected canonical bytes, and the expected hash. An
implementation conforms when it reproduces all of them byte-for-byte.

They cover the cases that actually break implementations: the worked moment,
unicode, UTF-16 key ordering, ES6 number formatting, precision at the 2⁵³ edge,
nested arrays, empty participants, escaping, and a hash-coverage twin — two
documents differing only outside the covered fields, which must hash the same.

Two implementations exist today: this crate, and an independent stdlib-only
Python port. Both run in CI, in their respective repositories, on every push.

**If this document and the vectors disagree, the vectors are right and this
document is a bug.**

---

## §7. What is deliberately not here

- **Signing, ordering, and cross-party conflict.** TT identifies and classifies;
  it does not adjudicate. A ledger built on TT supplies those, and one exists
  (`cc-core` in the Clockchain project) with its own envelope and its own
  identity. **Neither adopts the other's canon** — they are two objects for two
  roles, and conflating them would give one claim two identities.
- **A coordinate.** `occurs_at` is an opaque string in the payload. A ledger
  needing a frame-independent integer coordinate defines its own.
- **Place.** Participants are registry paths; there is no location hierarchy.
- **Extent.** Moments are points. A process with duration, recurrence or spread
  is not expressible today.
- **Colour.** The bundle carries no presentation. A renderer supplies its own
  palette and may name branches; naming anything below a branch means it has
  written down structure the bundle already states.

---

## §8. How this changes

See [GOVERNANCE.md](GOVERNANCE.md) for change classes, release windows,
retirement and migrations. In short: **Correction** (text only) and **Growth**
(new surface) ship daily; **Structure** (existing meaning moves) and
**Identity** (the schema string changes) ship weekly with a full week's notice
and a major bump.

Every proposal carries its own migration, and every migration gives each
affected record one of three verdicts — **Prune**, **Synthesize**, or **Store**,
which is the default. A migration that rewrites history to look tidy has
destroyed the evidence it was meant to carry forward.

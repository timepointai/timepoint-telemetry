# TT-SPEC — Timepoint Telemetry, normatively

**Applies to:** `tt-ontology/1.0 v2.1.0` and `tt-relations/1.0 v1.0.0` · crate `tt-core` 2.2.0
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

TT is four artifacts that ship together:

| | |
|---|---|
| **The bundle** | a versioned taxonomy — nodes, two lenses, lateral edges, bridges, the kernel, metric weights |
| **The envelope** | how one recorded moment is identified: canonicalisation and two hashes |
| **The metric** | how near two records are in meaning |
| **The relations vocabulary** | entity kinds, relation kinds between entities, and the edge-statement contract (§9) |

A crate release (`v2.2.0`) versions the set it ships. Each versioned artifact
carries its own version string, and a record cites the artifact it was
validated against: crate 2.2.0 ships the taxonomy at `2.1.0`, byte for byte
the file 2.1.x shipped, and the relations vocabulary at `1.0.0`.

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

**That directory level holds envelope vectors and nothing else.** Consumers
glob `vectors/*.json` and read every file as one (timepoint-beta's
`conformance/python/check_vectors.py` does), so it is an interface, not just a
folder, and CI checks the layout. Corpora of other shapes live below it, in
`vectors/verdicts/`.

They cover the cases that actually break implementations: the worked moment,
unicode, UTF-16 key ordering, ES6 number formatting, precision at the 2⁵³ edge,
nested arrays, empty participants, escaping, and a hash-coverage twin — two
documents differing only outside the covered fields, which must hash the same.

Three implementations exist today: this crate; an independent stdlib-only
Python port in another repository; and the stdlib-only Python on-ramp in
`python/`, written blind against the vectors. All three run in CI on every
push, two of them in this repository. Two of the three were written without
sight of the others' code, which is what lets this document claim to be
unambiguous rather than merely self-consistent.

**Classification verdicts have their own corpus** —
`vectors/verdicts/classification-verdicts.json`, 39 cases over the §4 contract, with a
two-tier rule stated in the file: the verdict, the normalized form, and the
multiset of rejection codes are normative in any language; the rejection
detail strings are an advisory byte-for-byte tier that the reference
implementation passes and a port may opt into. The corpus began as a
downstream consumer's differential harness against `python/tt_validate.py`;
adopting it upstream converted the reference implementation's rendering
habits from silent law into a documented, separately-checkable tier. Inputs
stay within IEEE-754 doubles — wider integer literals are parser territory,
not §4 contract.

**Relation verdicts have a third corpus** — `vectors/verdicts/relation-verdicts.json`,
over the §9 edge contract and load rules, generated by
`python/gen_relation_vectors.py` and never written by hand. The same two tiers
apply. Both reference implementations, `src/relations.rs` and
`python/tt_relations.py`, pass the advisory tier too, with one exception
stated in the file: the wording of a `malformed` load failure is each parser's
own.

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

---

## §9. The relations vocabulary

`bundle/relations-v1.0.json`, schema `tt-relations/1.0`, version `1.0.0`,
version string `tt-relations/1.0 v1.0.0`. It is a separate artifact, and it
never goes inside the taxonomy: the taxonomy's bytes and version string are
what every stored classification cites, so adding a vocabulary to them would
leave every published reading citing a release that is no longer the loaded
one. Proposal: `docs/proposals/ENTITY-RELATIONS.md`.

**Relation kinds are not lateral edges.** They connect *entities* in a
consumer's registry, not taxonomy *nodes*. The §5 graph has no edge from them,
and no distance moves because they exist.

### §9.1 What it contains

Four **entity kinds**, flat, with no parent field: `person`, `org`, `market`
(a publicly priced tradable, not a sector or sales territory) and `role` (a
formal position in exactly one organization, existing apart from its holder
and possibly vacant).

Ten **relation kinds**, each with a direction, allowed endpoint pairs, a
closed attribute schema and a nature:

| id | direction | endpoints (from → to) | attributes (`!` = required) | nature |
|---|---|---|---|---|
| `holds-office` | directed | person → role | valid_from, valid_to | structural |
| `office-of` | directed | role → org | none | structural |
| `member-of` | directed | person → org; org → org | valid_from, valid_to | structural |
| `reports-to` | directed | role → role | valid_from, valid_to | structural |
| `governs` | directed | org → org | valid_from, valid_to | structural |
| `vendor-to` | directed | org → org | subject, valid_from, valid_to | structural |
| `knows` | symmetric | person, person | none | social |
| `allied-with` | symmetric | person, person; org, org; person, org | **subject!**, valid_from, valid_to | social |
| `opposes` | directed | person or org → person or org | **subject!**, valid_from, valid_to | social |
| `competes-with` | symmetric | person, person; org, org | subject, valid_from, valid_to | social |

A directed relation has an `inverse_label` (`held by`, `customer of`) for
display. It is never an id and is never aliased.

Two **attribute types**. A `date` is `YYYY`, `YYYY-MM` or `YYYY-MM-DD`,
proleptic Gregorian, year ≥ 1, naming a real calendar day. Reduced precision is
kept, never padded. A `text` value is a JSON string, non-empty after trimming
Unicode White_Space, at most 200 Unicode scalar values (not bytes, not UTF-16
units), with no control characters (Unicode Cc).

**What the `text` rule does not catch.** Format characters (Unicode Cf) are
neither White_Space nor controls, so they pass: a `subject` that is only a
zero-width space (U+200B), or one that carries bidirectional controls
(U+202A–U+202E, U+2066–U+2069), is valid TT text. TT checks shape, not
meaning or rendering. Refusing invisible or direction-changing text is the
consumer's write-time lint (beta: the K2 rubric), and two vectors pin that TT
accepts both.

`valid_from` and `valid_to` are the period
*in the world* during which the relation held. **An absent `valid_to` means
"not stated", never "ongoing".**

### §9.2 Load rules

An invalid artifact fails boot, as an invalid taxonomy does. Every failure is
collected, each under a stable rule code: ids kebab-case, unique and disjoint
across the two collections (`id-not-kebab`, `duplicate-id`, `ids-not-disjoint`);
non-empty labels and definitions (`empty-label`, `empty-definition`);
`direction` and `nature` from their sets, `inverse_label` present if and only if
directed (`bad-direction`, `bad-nature`, `inverse-label-mismatch`); endpoints
naming existing kinds, a live relation naming no retired kind, no duplicate
pair, unordered when symmetric (`unknown-endpoint-kind`,
`retired-endpoint-kind`, `duplicate-endpoint-pair`); attribute names
`^[a-z][a-z0-9_]*$` and types from `attribute_types`, which may only name types
an implementation can check (`bad-attribute-name`, `unknown-attribute-type`,
`unsupported-attribute-type`); retirement as §2.5, for kinds and relations
alike (`retired-without-successor-or-note`, `unknown-successor`,
`self-supersession`, `supersession-cycle`); and the schema, version and
one-step-back lineage (`bad-schema`, `bad-version`, `bad-supersedes`). A
document of the wrong shape fails with `malformed` alone.

### §9.3 The edge-statement contract

TT validates what a relation **says**, and nothing about who said it or how.

```json
{ "relation": "holds-office",
  "from_kind": "person",
  "to_kind": "role",
  "attributes": { "valid_from": "2022-07-01" },
  "vocabulary": "tt-relations/1.0 v1.0.0" }
```

1. Top-level keys ⊆ {`relation`, `from_kind`, `to_kind`, `attributes`,
   `vocabulary`} (`unknown-key`). `basis`, `score`, `sources` and `observed_at`
   are unknown keys: the consumer strips its provenance before asking.
2. `relation` exists (`unknown-relation`) and is live (`retired-relation`,
   naming the successor, or the note when there is none). `from_kind` and
   `to_kind` exist (`unknown-kind`) and are live (`retired-kind`). A missing
   field is `missing-field`, and a non-string one is `field-not-string`.
3. The pair is allowed (`endpoint-pair-not-allowed`). Order is significant for
   a directed relation. A symmetric one accepts either order and is **not
   reordered**.
4. `attributes` is an object (`attributes-not-object`); absent means `{}`, and
   `null` is not absent. Keys ⊆ the relation's schema (`unknown-attribute`),
   required ones present (`missing-attribute`), values well-typed
   (`attribute-type`). With both dates present, the earliest day of
   `valid_from` is on or before the latest day of `valid_to` (`tenure-order`).
5. `vocabulary`, if present, equals the loaded version string
   (`vocabulary-mismatch`). `null` is a mismatch, not absence. The validator
   stamps it on output, and **what the validator emits, the validator
   accepts.**
6. Every failure is reported, not only the first, and the statement is thrown
   back whole (§4.2). A value is never trimmed, padded or reordered.

TT never sees entity **ids**. That the endpoints exist, that their stored kinds
match `from_kind` and `to_kind`, and that they are two different entities are
the consumer's checks (docs/CONSUMERS.md).

### §9.4 What it will not hold

No entity kind, relation kind or attribute encodes a person's protected traits
or any inference about behavior drawn from them. The rule is written into the
artifact (`respectful_modeling`), and a proposal that breaks it is refused
under that rule, not weighed as a trade-off. So there are no kinship,
household, romantic or intimate relations; no disposition, closeness or
influence attributes; and no stance on a `role`, because a stance attached to
an office would pass from one holder to the next.

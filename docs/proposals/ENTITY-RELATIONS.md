# Proposal: entity kinds and relation kinds (`tt-relations/1.0`)

Status: **draft for review** (Client Readiness wave, node T1). Nothing here is
built, tagged or merged. T2 implements it, runs the gate and cuts `v2.2.0` only
after L0 confirms and Sean approves. This file is the only thing this change
request commits.

Release target: **`v2.2.0`** (Growth, minor). Window: the **daily** window, at
the first NYSE close after the three preconditions in section 9 are met. It does
not need the Friday window, and section 1 explains why.

---

## 0. What this proposes, in one paragraph

A new, separately versioned vocabulary artifact, `bundle/relations-v1.0.json`
(schema `tt-relations/1.0`, version `1.0.0`), shipped by the `v2.2.0` release.
It defines four **entity kinds** (`person`, `org`, `market`, `role`) and ten
**relation kinds** between entities, each with a direction, allowed endpoint
kinds, a closed attribute schema and a `nature` (`structural` or `social`). It
also defines an **edge-statement contract** that works the way the
classification contract does (TT-SPEC §4): a validator that accepts or rejects
whole and never repairs. The taxonomy file `bundle/taxonomy-v2.1.json` is **not
touched, byte for byte**. It stays `tt-ontology/1.0 v2.1.0` with sha256
`31ed385e26522a5b548f7404f7757ee370ed9783dbd550b05cd69e89e9462113`. Basis,
score, sources and `observed_at` stay with the consumer (section 2.6).

---

## 1. Class

- [ ] **Correction**: text only; cannot move a single record between nodes
- [x] **Growth**: new node / bridge / lateral edge. Here: new ids in a new artifact
- [ ] **Structure**: existing meaning moves; re-parenting, deprecation, metric weights, a branch, the kernel

**Why Growth and not something lighter.** It adds 14 new ids (4 entity kinds and
10 relation kinds), and a new id is new surface. Correction covers text only, so
it does not apply.

**Why not Structure.** The effect test asks whether an existing record could be
read differently than it was written. It cannot:

- Every taxonomy node, lateral edge, bridge, kernel entry and metric weight is
  unchanged, because the taxonomy file's bytes are unchanged. So every
  classification, every shadow and every TT-SPEC §5 distance comes out identical.
- **Relation kinds are not lateral edges and never enter the metric.** A reviewer
  will reasonably ask whether "a new relation between things" changes a
  distance, as a lateral edge does. It does not. Relation kinds connect
  *entities* in a consumer's registry. They are not edges between taxonomy
  *nodes*, the TT-SPEC §5 graph has no edge from them, and anyone thresholding
  on nearness gets exactly today's answer. Any future proposal that turns a
  relation kind into a taxonomy edge is a separate change, and at least Growth
  with a metric migration.
- The envelope is unchanged. `participants` stays an opaque array of registry
  paths (TT-SPEC §7), and TT does not start validating it. A consumer may later
  write `/role/<slug>` into a participant list. That is a new value in an opaque
  field, and no existing `content_hash` moves.

**Why not Identity.** No existing schema id or envelope shape changes. A new
schema id for a new artifact orphans nothing written under `tt-ontology/1.0`.
A consumer that never loads the new file is unaffected.

**Does a consumer have to change? No.** Keeping the vocabulary *out of* the
taxonomy file is what guarantees this, and the choice was deliberate:

- Beta's `publications/telemetry.rs` compares each stored reading's `bundle`
  string to `bundle.version_string()` for **equality**, and reports any mismatch
  as `bundle_unavailable`. If the vocabulary had been added to the taxonomy, its
  version would become `v2.2.0`. Every previously published research reading
  that cites `tt-ontology/1.0 v2.1.0` would then render as unavailable after the
  upgrade. The records would stay the same, but they would be read differently.
  That is exactly what Growth must not do.
- Beta's `tt-bundle-pin` workflow, beta's `tt.bundles` projection
  (`0027_tt_bundle.sql`, keyed by version, sha-checked), the Clockchain's vendored
  byte-check and every classification's `bundle` stamp are all keyed on the
  taxonomy's bytes or version string. None of them move.

Release numbering already allows this: crate `2.1.2` ships taxonomy `2.1.0`
today. The release tag versions the crate and the set of artifacts it ships.
Each artifact carries its own version string, and a record cites the artifact
it was validated against.

**Utilization does not push the class up** (section 4): no taxonomy node is
touched, and nothing existing is rewritten (section 5).

---

## 2. The edit

### 2.1 Files T2 adds or changes (TT repository)

| Path | Change |
|---|---|
| `bundle/relations-v1.0.json` | **new**: the vocabulary below, verbatim |
| `bundle/taxonomy-v2.1.json` | **unchanged**. The gate checks its sha256 |
| `src/relations.rs` | **new**: `Vocabulary::load_from_file/str` (load rules, 2.4) and `validate_edge` (contract, 2.5); re-exported from `lib.rs` |
| `python/tt_relations.py` | **new**: stdlib-only on-ramp for the same contract, written against the vectors |
| `vectors/relation-verdicts.json` | **new**: section 8 |
| `tests/relations.rs`, `python/test_relation_vectors.py` | **new**: load-rule negatives plus the vector walker, in both languages |
| `Cargo.toml` | `version = "2.2.0"` |
| `TT-SPEC.md`, `README.md`, `docs/CONSUMERS.md` | prose (Timepoint's, not governed): §1 artifact list, a new "Relations vocabulary" section, counts, consumer obligations 1 to 5 extended to the new artifact |

### 2.2 `bundle/relations-v1.0.json`

```json
{
  "schema": "tt-relations/1.0",
  "version": "1.0.0",
  "supersedes": null,
  "governance": "identity frozen — ids never change meaning and are never deleted; governed by GOVERNANCE.md exactly as the taxonomy is",
  "respectful_modeling": "No entity kind, relation kind or attribute here encodes a person's protected traits or any inference about behavior drawn from them. A proposal adding one is refused under this rule, not weighed.",
  "attribute_types": ["date", "text"],
  "entity_kinds": [
    {"id": "person", "label": "Person",
     "definition": "An individual human being, identified as a named individual."},
    {"id": "org", "label": "Organization",
     "definition": "A body of people with a collective identity that can act, hold offices and have members: a company, public agency, school district, board, association, coalition or party."},
    {"id": "market", "label": "Market (priced tradable)",
     "definition": "A publicly priced tradable treated as an actor because its price or implied probability is observed: a trading pair, instrument or prediction-market contract. Not an industry, sector or geographic sales territory."},
    {"id": "role", "label": "Role (office)",
     "definition": "A formally defined position within exactly one organization that exists independently of whoever holds it and may be vacant, such as the superintendent of a school district or a numbered board seat. Not an informal function, disposition or characterization of a person."}
  ],
  "relation_kinds": [
    {"id": "holds-office", "label": "holds office", "inverse_label": "held by",
     "direction": "directed", "nature": "structural",
     "endpoints": [["person", "role"]],
     "attributes": {"valid_from": {"type": "date", "required": false}, "valid_to": {"type": "date", "required": false}},
     "definition": "The person occupies the role for a period. Tenure is the claim's own period; absence of valid_to means not stated, never ongoing."},
    {"id": "office-of", "label": "office of", "inverse_label": "has office",
     "direction": "directed", "nature": "structural",
     "endpoints": [["role", "org"]],
     "attributes": {},
     "definition": "The role is a position within the organization."},
    {"id": "member-of", "label": "member of", "inverse_label": "has member",
     "direction": "directed", "nature": "structural",
     "endpoints": [["person", "org"], ["org", "org"]],
     "attributes": {"valid_from": {"type": "date", "required": false}, "valid_to": {"type": "date", "required": false}},
     "definition": "The person or organization belongs to the organization as a member, or as staff when no position is recorded. A position that persists independently of its holder is a role, stated with holds-office."},
    {"id": "reports-to", "label": "reports to", "inverse_label": "has report",
     "direction": "directed", "nature": "structural",
     "endpoints": [["role", "role"]],
     "attributes": {"valid_from": {"type": "date", "required": false}, "valid_to": {"type": "date", "required": false}},
     "definition": "The first role is formally accountable to the second. Reporting lines join offices, not people; who reports to whom follows through holds-office."},
    {"id": "governs", "label": "governs", "inverse_label": "governed by",
     "direction": "directed", "nature": "structural",
     "endpoints": [["org", "org"]],
     "attributes": {"valid_from": {"type": "date", "required": false}, "valid_to": {"type": "date", "required": false}},
     "definition": "The first organization holds formal authority, conferred by law, charter or bylaws, to direct or oversee the second."},
    {"id": "vendor-to", "label": "vendor to", "inverse_label": "customer of",
     "direction": "directed", "nature": "structural",
     "endpoints": [["org", "org"]],
     "attributes": {"subject": {"type": "text", "required": false}, "valid_from": {"type": "date", "required": false}, "valid_to": {"type": "date", "required": false}},
     "definition": "The first organization supplies goods or services to the second under an agreement."},
    {"id": "knows", "label": "knows",
     "direction": "symmetric", "nature": "social",
     "endpoints": [["person", "person"]],
     "attributes": {},
     "definition": "The two people are acquainted. Says nothing about the nature, closeness or privacy of the acquaintance."},
    {"id": "allied-with", "label": "allied with",
     "direction": "symmetric", "nature": "social",
     "endpoints": [["person", "person"], ["org", "org"], ["person", "org"]],
     "attributes": {"subject": {"type": "text", "required": true}, "valid_from": {"type": "date", "required": false}, "valid_to": {"type": "date", "required": false}},
     "definition": "The two parties are aligned on a named public matter."},
    {"id": "opposes", "label": "opposes", "inverse_label": "opposed by",
     "direction": "directed", "nature": "social",
     "endpoints": [["person", "person"], ["person", "org"], ["org", "person"], ["org", "org"]],
     "attributes": {"subject": {"type": "text", "required": true}, "valid_from": {"type": "date", "required": false}, "valid_to": {"type": "date", "required": false}},
     "definition": "The first party takes a stated position against the second on a named public matter. Not personal animosity."},
    {"id": "competes-with", "label": "competes with",
     "direction": "symmetric", "nature": "social",
     "endpoints": [["person", "person"], ["org", "org"]],
     "attributes": {"subject": {"type": "text", "required": false}, "valid_from": {"type": "date", "required": false}, "valid_to": {"type": "date", "required": false}},
     "definition": "The two parties contend for the same contract, office, customers or other contested outcome."}
  ]
}
```

### 2.3 The vocabulary at a glance

**Entity kinds**: flat, four siblings, no parent field.

| id | status | meaning, briefly |
|---|---|---|
| `person` | adopted from beta, meaning unchanged | a named individual |
| `org` | adopted from beta, meaning unchanged | a body that acts, holds offices and has members |
| `market` | adopted from beta, meaning unchanged | a publicly priced tradable (pair, instrument, prediction-market contract). **Not** a sector or sales territory |
| `role` | **new** | a formal position in exactly one org, existing apart from its holder and possibly vacant |

**Relation kinds**:

| id | direction | inverse (a reading, not an id) | endpoints (from → to) | attributes (`!` = required) | nature |
|---|---|---|---|---|---|
| `holds-office` | directed | held by | person → role | valid_from, valid_to | structural |
| `office-of` | directed | has office | role → org | none | structural |
| `member-of` | directed | has member | person → org; org → org | valid_from, valid_to | structural |
| `reports-to` | directed | has report | role → role | valid_from, valid_to | structural |
| `governs` | directed | governed by | org → org | valid_from, valid_to | structural |
| `vendor-to` | directed | customer of | org → org | subject, valid_from, valid_to | structural |
| `knows` | symmetric | none | person, person | none | social |
| `allied-with` | symmetric | none | person, person; org, org; person, org | **subject!**, valid_from, valid_to | social |
| `opposes` | directed | opposed by | person or org → person or org | **subject!**, valid_from, valid_to | social |
| `competes-with` | symmetric | none | person, person; org, org | subject, valid_from, valid_to | social |

Attribute types (a closed set of two):

- **`date`**: ISO 8601 calendar date at year, month or day precision: `YYYY`,
  `YYYY-MM` or `YYYY-MM-DD`, proleptic Gregorian, year ≥ 1, and a real calendar
  day (`2023-02-29` rejects). Reduced precision is required, not tolerated,
  because public records often give a year only. Padding `2019` to `2019-01-01`
  would invent a precision nobody recorded.
- **`text`**: a JSON string, non-empty after trimming, at most 200 Unicode
  scalar values, with no control characters. It names a public matter (2.7).

The shared attributes mean the same thing on every relation that carries them:

- `valid_from` / `valid_to`: the period **in the world** during which the stated
  relation held. This is the claim's own time. It is not when anyone saw
  evidence of it (that is the consumer's `observed_at`, 2.6). **An absent
  `valid_to` means "not stated", never "ongoing".** Reading "currently holds"
  from an absent `valid_to` plus a recent `observed_at` is a consumer inference,
  and it must be labeled as one.
- `subject`: the public matter, contest, market, product or service the relation
  concerns ("District A bond measure", "assessment platform"). It is required
  where a relation without one would describe a personal disposition instead of
  a public position (`allied-with`, `opposes`).

### 2.4 Load rules (`Vocabulary::load_*`; an invalid artifact fails boot, as the taxonomy does)

1. Entity kind ids and relation ids are kebab-case, unique, and disjoint from
   each other.
2. Every kind and relation has a non-empty `label` and `definition`.
3. `direction` ∈ {`directed`, `symmetric`}. `nature` ∈ {`structural`, `social`}.
   `inverse_label` is present **if and only if** the relation is directed.
4. Every endpoint names an existing kind. A **live** relation may not name a
   **retired** kind. No duplicate endpoint pair: for symmetric relations,
   `[a,b]` and `[b,a]` count as the same pair.
5. Attribute names match `^[a-z][a-z0-9_]*$`. `type` is in `attribute_types`.
   `required` is a boolean.
6. Retirement follows GOVERNANCE §3, for kinds and relations alike:
   `deprecated_in` plus `superseded_by` (same collection, not itself, no cycle)
   or a `deprecation_note`. Neither is refused.
7. `schema == "tt-relations/1.0"`. The version string is
   `"tt-relations/1.0 v<version>"`, and `supersedes` names one step back
   (null for 1.0.0).

### 2.5 The edge-statement contract (`validate_edge`; reject, never repair)

TT validates what a relation **says**, and nothing about who said it or how.
This mirrors the envelope, where the claim is hashed and provenance is not.

```json
{ "relation": "holds-office",
  "from_kind": "person",
  "to_kind": "role",
  "attributes": { "valid_from": "2022-07-01" },
  "vocabulary": "tt-relations/1.0 v1.0.0" }
```

1. Top-level keys ⊆ {`relation`, `from_kind`, `to_kind`, `attributes`,
   `vocabulary`}. Any other key rejects `unknown-key`. **This includes `basis`,
   `score`, `sources` and `observed_at`**: the consumer strips its provenance
   before asking TT. A vector pins this boundary.
2. `relation` exists (`unknown-relation`) and is live (`retired-relation`,
   naming the successor). Inverse labels (`customer-of`) and non-kebab spellings
   (`holds_office`) are unknown. They are never aliased.
3. `from_kind` and `to_kind` exist (`unknown-kind`) and are live (`retired-kind`).
4. The pair is allowed (`endpoint-pair-not-allowed`). For **directed** relations
   the order is significant. For **symmetric** ones either order is accepted, and
   the validator **does not reorder** the pair.
5. Attribute keys ⊆ the relation's schema (`unknown-attribute`). Required
   attributes are present (`missing-attribute`). Values are well-typed
   (`attribute-type`). `attributes` absent means `{}`.
6. If both dates are present, the earliest day of `valid_from` must be on or
   before the latest day of `valid_to` (`tenure-order`). So `2020-06` → `2020`
   is valid, and `2021` → `2020-12` is not.
7. `vocabulary`, if present, equals the loaded version string
   (`vocabulary-mismatch`). The validator stamps it on output. **What the
   validator emits, the validator accepts.**
8. Every failure is reported, not only the first. The verdict, normalized form
   and multiset of codes are normative. Detail strings are advisory, as in
   `classification-verdicts.json`.

TT never sees entity **ids**, so the consumer owns these checks: that the
endpoints exist, that their stored kinds match `from_kind`/`to_kind` (CONSUMERS
obligation 5: a declared kind is checked, not believed), and that the two
endpoints are not the same entity.

### 2.6 Basis, score, sources, `observed_at`: left to the consumer, on purpose

TT does **not** define these fields. The reasons:

1. **It is TT's existing boundary.** TT identifies and classifies. It does not
   adjudicate (TT-SPEC §7). Grounding sits outside `content_hash` (TT-SPEC §3.2).
   Provenance completeness and storage schemas are listed in CONSUMERS.md under
   "What TT has no opinion about". A relation's claim is TT's. How somebody came
   to believe it is theirs.
2. **Two consumers already disagree, legitimately.** Beta uses
   `GROUNDED | GENERATED | ACCUMULATED` plus source strings (`0007_entity.sql`,
   `entities.rs`). The Clockchain uses `cc.source-evidence.v1` evidence classes
   (docs/SOURCE-EVIDENCE.md). Making beta's enum normative would make the
   Clockchain non-conforming and gain nothing.
3. **TT already has the cross-consumer provenance primitive.** At the consumer
   baseline `v2.1.2` (7afbc3b), `SourceAttribution` / `timepoint-source/1`
   (`src/provenance.rs`) refuses a model-generated attribution in the `Report`
   or `Observation` roles. That is TT's form of "a simulation's output can never
   become evidence". `v2.2.0` must keep it (section 9).

**Hard policy, which beta enforces at write and TT restates.** Simulations
never write `GROUNDED`. A `GROUNDED` edge carries at least one source URL. A
relation's `nature` does not set its basis. A structural relation may be
`GENERATED` (a model guessed the reporting line), and a social one may be
`GROUNDED` (a published coalition statement).

**Recommendations to beta E1** (the consumer's call, not TT law):

- `score` means **belief that the edge holds as stated**, never strength,
  closeness or intensity. Use `(0, 1]` at write: zero is absence, which matches
  the assertion gate's rule 3 and TT's mass rule.
- Keep `observed_at` (when the evidence was seen) distinct from
  `valid_from`/`valid_to` (when the relation held) and `created_at` (when the row
  was written).
- **Do not store tenure in SQL `date` columns.** They cannot hold `2019` without
  inventing `2019-01-01`. Store the validated `attributes` jsonb, and if a column
  is needed for queries, derive earliest/latest bounds from it (never declare
  them separately).
- Do not store `nature` per edge as a declared field. Derive it from the pinned
  vocabulary, or check it against the vocabulary on write.
- Store each symmetric edge **once**, in canonical endpoint order, with a unique
  constraint, as `tt.lateral_edges` does with `a < b`. Store a directed edge in
  its forward direction only. Inverse labels are for display.
- A **structural path** is a path whose every edge has a relation of nature
  `structural`. A generated `knows` edge can never be part of one, whatever its
  score. "Grounded structural path" is a second, separate filter on basis.

### 2.7 Respectful modeling: a hard rule of this vocabulary

The vocabulary models **role, public record, stated pressures and incentives,
and public positions**. It must never contain a relation, attribute or entity
kind that encodes a person's protected traits (for example race, ethnicity,
national origin, religion, sex, gender identity, sexual orientation, age,
disability or health, pregnancy, familial or marital status, genetic
information, veteran status), or any inference about behavior drawn from them.
This rule is written into the artifact (`respectful_modeling`). A future
proposal that breaks it is refused under this rule, not weighed as a trade-off.

What that means concretely in `1.0.0`:

- **No kinship, household, romantic or intimate relation kinds.** They are
  refused, not deferred. `knows` states acquaintance and nothing about its
  nature, and it has no attributes that could carry one. (Lens B's
  `bonding-and-kinship` classifies *moments*. It is untouched and out of scope.)
- **No disposition, personality or psychometric attributes**, and no
  "influence" or "closeness" weight. `score` is belief, not intensity (2.6).
- **Stances attach to people and orgs, never to a `role`.** Otherwise one
  holder's position would pass to the next holder through the office.
  `allied-with` and `opposes` require a `subject`, so every stance names a
  public matter.
- **`role` is a formal position** ("Superintendent of District A"), never a
  characterization ("decision-maker", "gatekeeper", "influencer").
- **Membership that reveals a protected trait.** `member-of` a religious body,
  political party, union or patient/health association may be recorded **only as
  `GROUNDED` from a record the member made public in a public capacity** (for
  example a candidate's ballot designation). It must never be `GENERATED` or
  inferred. TT cannot tell what kind of body an org is, so the consumer enforces
  this at write (beta: the K2 rubric, `docs/policy/ENTITY-LANGUAGE.md`).
- `text` values are free text. TT cannot check their meaning, so the same
  consumer write-time lint applies to `subject`.

### 2.8 Placing `role`, and the other candidate decisions

**The existing kind hierarchy is flat.** TT has had no entity kinds until now.
Beta's registry has three sibling kinds with no subsumption (`0007_entity.sql`
and `0021_tge_ownership.sql` CHECK `person|org|market`). `role` joins as a
**fourth top-level sibling**:

- **Not a child of `person`.** A role outlives its holders, can be vacant and
  has a succession of occupants. A subtype of person would inherit person
  semantics: a registry path per human, participation as a human. A role is
  what a person *holds*.
- **Not a child of `org`.** An org has members and offices. A role has neither,
  and it acts only through its holder. As a sub-org, "Superintendent of
  District A" would become something that can have members, which is wrong.
- **The connection is a relation, not a hierarchy.** `person —holds-office→ role
  —office-of→ org`. Tenure lives on `holds-office`, so a change of
  superintendent is one new edge. The office entity and its reporting lines
  (`reports-to` joins roles) stay exactly as they were. That stability is the
  reason for G6.
- **The hierarchy stays flat.** Adding one later means giving existing kinds a
  parent. That can change how existing edges are read, which makes it Structure,
  and it needs evidence first.

**Adopting `person`, `org`, `market` with their existing meanings.** Beta
records already use these ids, and they appear inside hash-covered
`participants` as `/person/…`, `/org/…`, `/market/…`. TT must use the same ids
with the same meanings, or it would silently re-read those records. `market` is
defined as what beta's records mean: `btc-usd` in frames, and Polymarket
markets and events from `storysim.rs`.

| Candidate | Decision | Reason |
|---|---|---|
| all ten | **renamed** to kebab-case (`holds_office` → `holds-office`, …) | TT ids are kebab-case (node ids, bridge relations such as `scales-up-to`), and beta's attribute grammar is kebab-case. Ids are frozen, so this is the last chance to choose |
| `holds_office` | **kept**, person → role | G6. Tenure attributes |
| `office_of` | **kept**, role → org | Without it a role belongs to no organization. It is not the inverse of `holds-office` |
| `member_of` | **kept**, person → org, org → org | Membership without a seat (association, consortium). Staff with no known position are included by definition, so no `employed-by` id is needed (open question 3) |
| `reports_to` | **kept, narrowed** to role → role | Reporting lines belong to offices. Person-to-person reporting follows through `holds-office`, so a personnel change does not rewrite the org chart |
| `governs` | **kept, narrowed** to org → org | Formal authority (law, charter, bylaws). An office's authority is already expressed by `office-of` plus `reports-to`. Allowing role → org would create two paths for one fact |
| `knows` | **kept**, symmetric, no attributes | The canonical social, usually generated, edge that beta must keep apart from structural paths |
| `allied_with` | **kept**, symmetric, `subject` required | Public alignment on a named matter. Not merged with `knows`: acquaintance and public alignment are independent facts |
| `opposes` | **kept, made directed**, `subject` required | A opposing B does not imply B opposes A. Anchored to a public matter |
| `competes_with` | **kept**, symmetric, same-kind pairs only | Vendors for contracts, candidates for an office. Not merged with `opposes`: rivalry for an outcome is not a stated position |
| `vendor_to` | **kept**, org → org | Procurement is public record (board minutes, contracts). The inverse is read as "customer of" |
| inverse ids (`has-office`, `customer-of`, …) | **not minted** | Two ids for one fact would give it two identities (CONSUMERS: identity is exact). Inverses are `inverse_label`, for display only |

---

## 3. Evidence

A node earns its place by being needed. The evidence below is **record shapes
that beta writes in production today**, read from beta's code on `cr/coord` (its
code equals master). It is not row counts: those are section 4's job, and
nobody has measured them yet. None of it comes from a client engagement. All
examples use synthetic names.

1. **An office cannot be recorded, so it is written as a string on a person.**
   `account.rs::post_onboarding` mints a `person` and an `org`, then writes
   `current-role: "<text>"` as an assertion **on the person**. There is no org
   and no tenure. When the holder changes, the office has no identity to carry
   its history, and two people who held "Superintendent of District A" in turn
   share nothing addressable. `dossier.rs`'s import path extracts
   `current-role` the same way (its test uses "Chair of …").
2. **A relationship between two entities that both exist is stored as a string.**
   The same onboarding call holds the minted `org_id` and still writes
   `works-at: "<company name>"` on the person. With no relation vocabulary, the
   link to an entity already in the registry had to be flattened into text.
3. **A document's statement that someone holds a role loses the org.**
   `uploads.rs` writes `named-in-uploaded-document: {"role": "<text>"}`,
   `GROUNDED`, after a person approves it. The source is real and named, but
   which organization's office it is cannot be expressed.
4. **The only entity-to-entity links are run-scoped and generated.** Frame
   `ties` (`quicksim.rs`: `{a, b, branch, w, grounding: g|a|i}`) live inside
   `run.artifacts`. They are classified by a Lens-B branch and declared by the
   model. There is nowhere to put a grounded, typed, dated fact such as "Person A
   holds the office of Superintendent of District A, since 2022-07". Nor can a
   generated acquaintance be told apart from a structural path, which the
   wave's PLAN hard policy 3 and gate G5 require.
5. **The kind set cannot express the thing G6 needs.** `ENTITY_KINDS` and both
   CHECK constraints allow only `person|org|market`, so an office has no kind.
6. **A meaning collision is on the way unless TT fixes `market` now.** The wave's
   drill-down (WORKGRAPH D1) calls its top level "satellite (market)", meaning a
   go-to-market territory. Beta's existing `market` records mean a priced
   tradable. Without a published definition, the first territory written as
   `/market/<slug>` would silently give an id a second meaning. This is evidence
   for *defining* `market`, not for changing it.

What is **not** evidence, and is not used: that other ontologies have
"employment" or "family" edges, symmetry with the candidate list, or
completeness. That is why `employed-by`, an `acting`/interim flag,
office-existence dates and a `place` kind are deferred (section 11), not
proposed.

---

## 4. Utilization

**Taxonomy nodes touched: 0 of 149, by construction.** The taxonomy file is
byte-identical, and the gate proves this by sha256, not by reading. No
classification, shadow, forecast anchor or published reading can be affected.

The new artifact has no prior records: no record anywhere cites
`tt-relations/1.0`. The table below covers the **consumer records whose
meaning this change fixes or exposes** (section 5). **None of these figures has
been measured.** Per GOVERNANCE §5 that means *nobody told us*, never *nobody
uses it*. T2 runs `tt-utilization` for them, read-only, against beta
production, reporting aggregate counts only (no display names, no values), with
each query printed next to its number:

| Touched | Records | Of those, load-bearing | Source |
|---|---|---|---|
| taxonomy nodes (all 149) | 0 touched (sha256 unchanged) | 0 | by construction; gate check 4 |
| `entity.entities` by `kind` (`person`/`org`/`market`) | not measured | rows in `participants` of moments that anchor forecasts or publications | beta prod, T2 |
| `entity.assertions` with `attribute IN ('current-role','works-at','named-in-uploaded-document')`, by `basis` | not measured | those injected as KNOWN context (GROUNDED/ACCUMULATED) | beta prod, T2 |
| frame/cockpit `ties` in `run.artifacts` | not measured | none can be: artifacts are hashed history | beta prod, T2 |
| `run.moments.participants` paths by `/kind/` prefix | not measured | paths on moments anchoring a forecast | beta prod, T2 |
| Clockchain | 0 known | nobody told us | Clockchain has no entity registry that uses these kinds, as far as this author could see |

Queries for T2 (read-only; adjust to the schema as found, since it has moved
before):

```sql
SELECT kind, count(*) FROM entity.entities GROUP BY kind ORDER BY kind;
SELECT attribute, basis, count(*) FROM entity.assertions
 WHERE attribute IN ('current-role','works-at','named-in-uploaded-document')
 GROUP BY 1,2 ORDER BY 1,2;
SELECT kind, sum(jsonb_array_length(content->'ties')) FROM run.artifacts
 WHERE jsonb_typeof(content->'ties') = 'array' GROUP BY kind ORDER BY kind;
SELECT split_part(p, '/', 2) AS kind, count(*)
  FROM run.moments m, jsonb_array_elements_text(m.participants) p
 GROUP BY 1 ORDER BY 1;
```

**Pacing.** Every verdict is Store and nothing is rewritten, so no figure can
push this change into the weekly window. The figures price the *consumer's*
later, opt-in adoption work (a human-approved conversion of `current-role`
strings into role entities, if Sean ever wants one). They do not price this
release.

---

## 5. The migration

**Iterator (specified here, built by T2 in beta as a report-only pass beside
`ttmigrate.rs`; nothing exists yet).** It is deterministic: every read has a
total `ORDER BY`, and it reads only the pinned artifacts. It is idempotent: it
never writes to entity, run or TT tables, and its only optional write is one
pass-log row. It is report-first: the default mode prints what it *would* do,
and `--apply` has nothing to apply because every verdict below is Store. Two
runs on the same database print byte-identical reports.

For each divergence the change creates or exposes:

| # | Divergence | Verdict | What the iterator does |
|---|---|---|---|
| D1 | beta `entity.entities.kind ∈ {person, org, market}` versus the new TT kinds | **Store** | Counts rows per kind. Reports any kind outside the TT set (CHECK makes this impossible today, and a count of 0 is still printed). Rewrites nothing: the ids and meanings are identical |
| D2 | `current-role` / `works-at` / `named-in-uploaded-document` string assertions versus the new `role` kind and `holds-office`/`office-of`/`member-of` | **Store** | Counts per attribute and basis. Creates no role entity and no edge |
| D3 | frame and cockpit `ties` versus the new relation kinds | **Store** | Counts ties. Never converts one into an edge |
| D4 | `participants` paths versus the new `role` kind | **Store** | Counts per `/kind/` prefix. Hash-covered, so never touched |
| D5 | stored readings citing `tt-ontology/1.0 v2.1.0` | **Store**, and no divergence | Confirms that the loaded taxonomy version is still `v2.1.0` (the release is safe only if this holds) |

**Argument.** Store is the default. Prune and Synthesize are not argued for,
because each would destroy or invent information:

- **D2 cannot be synthesized into edges without fabrication.** `current-role`
  carries no org, and turning it into `office-of` would guess one. It carries no
  tenure, and turning it into `holds-office` would imply one. Its basis is often
  `console_manual` (GROUNDED because a person typed it, not because a public
  record says so), and re-expressing it as a structural, public-record edge
  would inflate its evidential standing. A conversion, if wanted, is a
  **proposal a person approves**, one row at a time (PLAN hard policy 5). It is
  not a migration.
- **D3 must never be synthesized.** Ties are model-declared. Converting a tie
  with `grounding: "g"` into an edge would turn simulation output into a
  GROUNDED record, which is exactly what hard policy 3 forbids. Ties are also
  inside `run.artifacts`, which `ttmigrate.rs` rules Store forever because their
  `content_hash` covers the whole document.
- **Nothing is pruned.** No id is retired.

**Dry-run output:** **not run.** The iterator does not exist yet (T2 builds it),
and this author did not query any beta database. The report T2 must produce
looks like this. The numbers are placeholders, **not results**:

```
TT RELATIONS v1.0.0 ADOPTION REPORT — REPORT-ONLY (nothing written)
taxonomy loaded: tt-ontology/1.0 v2.1.0 sha256 31ed385e… (unchanged: yes)
D1 entities by kind: person=<n> org=<n> market=<n> outside-TT-kinds=<n> → STORE
D2 string role/link assertions: current-role/<basis>=<n> works-at/<basis>=<n> named-in-uploaded-document/<basis>=<n> → STORE
D3 ties in run.artifacts: <kind>=<n> → STORE (always: hashed artifacts)
D4 participant paths: /person=<n> /org=<n> /market=<n> /other=<n> → STORE (hash-covered)
D5 readings citing a taxonomy version other than the loaded one: <n> → STORE
rows that would change on --apply: 0
```

- [ ] **Prune**
- [ ] **Synthesize**
- [x] **Store**: all five divergences above

---

## 6. Compatibility for existing consumers

Baseline: **`v2.1.2` = GitHub tag at `7afbc3b`** (`refs/remotetags/v2.1.2`),
which includes `src/provenance.rs`. This is what beta compiles against.

| Consumer | Pinned today | Effect of `v2.2.0` | Must change? |
|---|---|---|---|
| beta, never upgrades | tag `v2.1.2` | none | no |
| beta, upgrades (T3) | → `v2.2.0` | the taxonomy bytes and version string are identical, so `tt-bundle-pin`, `tt.bundles`, classifier ETag, classification stamps and publication readings are unchanged. `SourceAttribution` and the rest of the provenance API stay available **only if** section 9 holds | no (edges are opt-in, work for E1/E2) |
| Clockchain (crate by rev; wasm vendors the taxonomy) | its own revs | the vendored taxonomy hash is unchanged. The new artifact is unused unless adopted | no |
| independent Python ports | vectors | the existing vectors are unchanged, and there is a new, optional corpus | no |

Lineage notes for T3:

- `timepoint-beta/Cargo.lock` resolves `tag=v2.1.2` to **`5f9295f`**, but the
  GitHub tag now points at **`7afbc3b`**. The two commits have the identical
  tree (`80825a5…`), and `5f9295f` is on no `origin` branch. T3's bump to
  `v2.2.0` re-resolves the lock. T3 should confirm that a clean fetch works.
- `tt-bundle-pin.yml` fetches the published taxonomy from the **`v2.1.0`** path
  while Cargo pins `v2.1.2`. That still passes because the bytes are the same.
  T3 should point it at the tag it depends on, and optionally add a byte-check
  of `relations-v1.0.json` when beta starts reading it (CONSUMERS obligation 1
  applies to the new artifact).

---

## 7. Utilization plan: what T2's `tt-utilization` must measure

1. That taxonomy nodes touched = 0: compare the sha256 of `bundle/taxonomy-v2.1.json`
   at the `v2.2.0` candidate with the one at `7afbc3b` and `16227a9`.
2. The four aggregate queries in section 4 against beta production, with each
   query printed next to its figure. The deployment is named, and zeros are
   reported as "nobody told us".
3. How many D2 assertions are load-bearing, meaning they enter
   `read_context_assertions` (GROUNDED or ACCUMULATED).
4. A single sentence on pacing. The expected answer is "no pressure toward the
   weekly window". If the numbers say otherwise, that is a finding.

---

## 8. Conformance vectors T2 adds (Rust and Python)

`vectors/relation-verdicts.json` uses the two-tier rule of
`classification-verdicts.json`: the verdict, the normalized form and the
multiset of codes are normative, and the detail strings are advisory. Both
`tests/relations.rs` and `python/test_relation_vectors.py` walk it. Each walker
claims only its own corpus, as `tests/vectors.rs` does today.

A throwaway prototype of sections 2.4 and 2.5 (not tt-core, not committed) loads
the artifact above cleanly and gives the specified verdict on all 28 cases below.
In every accepted case the output validates again (round-trip).

Accept (the normalized form is the input plus the `vocabulary` stamp):

1. `holds-office` person→role `{valid_from: "2022-07-01"}`
2. `holds-office` `{valid_from: "2019", valid_to: "2019"}` (same-year tenure)
3. `holds-office` `{valid_to: "2024-06"}` (end known, start not stated)
4. `office-of` role→org, attributes absent → normalized to `{}`
5. `member-of` org→org
6. `knows` person–person
7. `allied-with` **org→person** (reverse of the declared `[person, org]`, symmetric) with `subject`
8. `opposes` org→org with `subject`
9. `vendor-to` org→org with `subject`
10. `{valid_from: "2020-06", valid_to: "2020"}` (overlapping precisions)
11. `vocabulary: "tt-relations/1.0 v1.0.0"` present and matching

Reject (with the code multiset):

12. `holds_office` → `unknown-relation` (snake case is not aliased)
13. `customer-of` → `unknown-relation` (an inverse label is not an id)
14. `holds-office` role→person → `endpoint-pair-not-allowed` (directed order)
15. `reports-to` person→person → `endpoint-pair-not-allowed`
16. `allied-with` role→org → `endpoint-pair-not-allowed` (stances never attach to a role)
17. `to_kind: "office"` → `unknown-kind`
18. `opposes` without `subject` → `missing-attribute`
19. `knows` with `closeness` → `unknown-attribute`
20. `valid_from: "2022-13-01"` → `attribute-type`
21. `valid_from: "July 2022"` → `attribute-type`
22. `valid_from: "2023-02-29"` → `attribute-type`
23. `valid_from: "2021", valid_to: "2020-12"` → `tenure-order`
24. `subject: "   "` → `attribute-type`
25. `subject` of 201 scalar values → `attribute-type`
26. top-level `basis: "GENERATED"` → `unknown-key` (**TT stays out of provenance**)
27. `holds-office` role→person with `closeness` → `endpoint-pair-not-allowed` + `unknown-attribute` (all failures reported)
28. `vocabulary: "tt-relations/1.0 v0.9.0"` → `vocabulary-mismatch`

T2 should also add: a malformed `attributes` value (not an object), a
`retired-relation` / `retired-kind` pair using a hand-built vocabulary fixture,
and a 200-scalar `subject` containing multibyte characters (which is accepted).
The last one pins "scalar values, not bytes" across languages.

Load-rule negatives (Rust and Python tests, on hand-mutated copies): a duplicate
relation id; an endpoint naming an unknown kind; `inverse_label` on a symmetric
relation or missing on a directed one; retirement with neither successor nor
note; a live relation naming a retired kind; a duplicate symmetric pair written
in both orders; an attribute type outside `attribute_types`.

Unchanged, and must still pass byte for byte: the 10 existing
envelope/provenance vector files, the 39 classification verdicts, and the
taxonomy loader counts (149/151/26, 6/9 branches). This author ran the baseline
suites on `16227a9` with the pinned toolchain 1.97.1, and they are green: Rust
6 suites and 51 tests, Python 35 tests. That is the baseline, not the candidate.

---

## 9. Release lineage for v2.2.0

**`v2.2.0` must descend from both `v2.1.2` and `main`.** Today they have
diverged:

- GitHub tag **`v2.1.2` → `7afbc3b`**, which lives only on
  `origin/fix/source-provenance-v1` and was never merged. It carries
  `src/provenance.rs`, `schema/source-attribution-v1*`, `tests/source_vectors.rs`
  and `Cargo.toml 2.1.2`. **Beta compiles against it**, using
  `SourceAttribution`, `SOURCE_CONTRACT`, `ClaimRole`, `ProductionMethod`,
  `EvidenceRef` and `source_from_provenance`, in 16 files under `crates/`.
- **`origin/main` (`16227a9`)** forked at `v2.1.0` (`fc92e61`). It has the
  Python on-ramp, the classification verdicts and the media and source-evidence
  docs, but **no `src/provenance.rs`**, and its `Cargo.toml` still says `2.1.0`.
- A `v2.2.0` cut from `main` alone would drop an API beta uses, and T3's pin bump
  would fail to compile. A `v2.2.0` cut from the provenance branch alone would
  drop main's Python on-ramp and verdict corpus.

**Recommended path (Sean and L0 decide):** a TT PR that merges
`origin/fix/source-provenance-v1` into `main`, landed before this proposal's
implementation. Since `fc92e61`, the only file both lines changed is
**`Cargo.toml`**: the provenance side changed `version`, and `main` changed
`description`, which are adjacent lines. The provenance side otherwise added
only new files plus two lines in `src/lib.rs` (`mod provenance;` and its
re-export), so the merge should be mechanical.
Then T2 implements this proposal on top and sets `version = "2.2.0"`.

**Preconditions for tagging `v2.2.0`:** (1) the lineage merge is on `main`; (2)
T2's gate is green (section 10); (3) L0 confirms and Sean approves. Then it goes
out at the next daily close. Nothing ships between windows.

---

## 10. Gate (GOVERNANCE §7)

None of these can be ticked from a docs-only change request. T2 runs each one
as a command:

- [ ] **the bundle validates**: `Bundle::load_from_file` on the unchanged taxonomy, **and** `Vocabulary::load_from_file` on `relations-v1.0.json` (the loader does not exist yet; only a prototype has loaded it)
- [ ] **every existing conformance vector still produces its recorded hash**: `cargo test` and the Python suites on the post-merge candidate (the baseline at `16227a9` is green, but it is not the candidate)
- [ ] **new surface has new vectors**: section 8
- [ ] **no id changed meaning; none were deleted**: taxonomy sha256 equal to `v2.1.2`'s (`31ed385e…`), plus a mechanical id-set diff. The new artifact has no predecessor, and T2 records that as the reason there is no diff for it
- [ ] **the migration ran on a rehearsal copy and reported what it would do**: section 5 iterator, run twice with identical output
- [ ] **Growth: everything valid under the previous version is still valid**: all 39 classification verdicts plus the envelope vectors, unchanged

Exceptions stated in advance: none of the six checks were run by the author;
utilization was not measured; the dry run was not run. The reasons are given in
sections 4, 5 and 8.

---

## 11. Open questions for Sean (each with a recommended default)

1. **Keep the kind id `role`, or use `office`?** The id is frozen forever.
   "role" is overloaded in TT and beta (`ClaimRole`, workspace roles, model
   roles), and in plain English it invites informal characterizations.
   **Default: keep `role`.** G6, E2 and D1 already use it, the definitions in 2.3
   and 2.7 confine it to formal positions, and K2 enforces that at write.
2. **The drill-down's "satellite (market)".** **Default: D1 must not write
   territories as kind `market`.** In this wave, treat the satellite as a query
   scope, not an entity. If territories need to be entities, that is a later
   Growth proposal for a new kind, with its own evidence.
3. **Employment when the position is unknown.** **Default: `member-of` covers
   staff (as its definition says). No `employed-by` id** until real records show
   `member-of` is overloaded.
4. **Separate artifact versus inside the taxonomy.** **Default: separate** (the
   `bundle_unavailable` argument in section 1). The alternative bumps the
   taxonomy to `v2.2.0` and requires a beta change to the publication reader
   first.
5. **Lineage merge path.** **Default: a PR merging
   `fix/source-provenance-v1` into `main`**, before T2 (section 9).
6. **Should `opposes` and `allied-with` require `subject`?** **Default: yes.** It
   is the vocabulary's main guard against modeling personal animosity instead of
   public positions.

---

## 12. Deferred ideas (not proposed; each needs its own evidence)

- An `acting`/interim flag or a `selection` attribute (elected, appointed, hired) on `holds-office`
- Office lifetime (`valid_from`/`valid_to` on `office-of`, or on the role) for offices created or abolished
- `employed-by`, only if `member-of` proves overloaded
- A `place` or `territory` kind for sourced context about a place (the PLAN permits this context, and this vocabulary does not model it)
- An edge content identity (a hash over `{relation, from, to, attributes}`), so two parties stating the same edge collide. This needs cross-consumer entity identity, which TT does not have
- Relation-aware distance, or bridges from relation kinds to Lens-A events (for example `holds-office` start → `appointments-and-government-formation`). Either would touch the metric, so it would be at least Growth with a metric migration
- Graph-level constraints (a role is `office-of` exactly one org; one holder per role at a time). These are consumer-side for now, and TT cannot see entity ids

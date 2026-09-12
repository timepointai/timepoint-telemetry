# Proposal: typed media absence for Clockchain

Status: updated for review following Sean's 2026-09-12 instruction:
"proceed with the plan - update telemetry and clockchain as needed."
That instruction authorizes preparing this contract and its consumer together;
this PR is not merged and does not claim independent Telemetry verification.

## Problem and evidence

The accepted [generated-media contract](../GENERATED-MEDIA.md) defines missing
media as `no_image`. Clockchain `b1e3328`, `crates/cc-node/src/media.rs`, returns
that same value for every empty attachment list. Neither the record nor the
response distinguishes an explicit choice to leave a claim unillustrated from
an absence of recorded generation. Clockchain's PLAN item 5 requires that
distinction, and requires upstream review before building it.

Renaming every empty list to `not_yet_generated` would infer intent from missing
data. The state must be grounded in a recorded decision. This proposal makes no
claim about the number or intent of currently unillustrated claims.

## Proposed boundary

Keep generated attachments and their signatures unchanged. Introduce a separate
signed absence decision, outside the TT envelope, classification, claim hash,
identity, dedup, near-match, feasibility and historical ledger anchors.

The decision binds an exact `source_entity_id` and `source_body_hash`, writer,
reason and decision timestamp. Its explicit type is `deliberately_unillustrated`.
The entity ID uses canonical decimal text, as existing image manifests do, to
preserve all bits under JSON canonicalization. Admission records its own time
and verifies the signature and current entity/body projection before admitting
it. The decision has its own versioned schema and signature domain; it cannot
be replayed as an image attachment or a historical attestation.

TT is asked to confirm this belongs entirely to consumer provenance and requires
no `ALLOWED_FIELDS` change. If an envelope field is required instead, TT must
publish that contract before the consumer implements it. No TT field, bundle,
vector, admission rule or identifier is changed by this proposal.

## Proposed read semantics

Report state per exact entity/body reading, considering only records visible
at the requested `as_of`. Do not collapse an entity's multiple readings.

| Visible records for the reading | Proposed state | Meaning |
|---|---|---|
| Neither an image nor an absence decision | `no_generation_recorded` | No applicable media record is visible; no inference about off-system attempts or intent. |
| Explicit absence decision(s), no image | `deliberately_unillustrated` | The response includes the signed decision(s) and their reasons. |
| Image attachment(s), no absence decision | `generated` | The response includes the existing generated manifests. |
| Both image and absence decision(s) | `conflicting_media_records` | Expose both records; do not select a winner. |

A signature proves authorship of the decision, not consensus that the claim
should remain unillustrated. Multiple decisions remain individually inspectable.
Queue, running and failure states, if implemented later, belong to generation
jobs and are never inferred from these states.

Preserve the distinction between admission visibility at `as_of` and source
binding against the current projection. Label the latter as current, not as a
reconstruction of historical projection state. Return stale decisions as stale
provenance, just as withdrawn image sources remain inspectable.

A correction or delete-and-re-mint cannot transfer an absence decision to a new
body. An old image or decision must not set the new body's state. A body still
projected by another entity does not make a decision applicable to that entity.

For the initial contract, conflicting records remain explicit and immutable.
No automatic latest-writer rule, implicit revocation or supersession is proposed.
Telemetry must accept this initial conflict behavior, or define a different
explicit resolution contract, before an absence writer is built.

## Compatibility and migration

Existing claims, image manifests, hashes and signatures retain their exact bytes.
Do not backfill absence decisions from existing empty lists. A lack of a media
record maps to `no_generation_recorded` only on the new, explicitly versioned
read contract; it does not assert a historical decision.

Preserve the existing response contract for existing callers. In particular,
`no_image` retains its coarse meaning on the legacy surface; it must not acquire
the meaning "deliberately unillustrated." Introduce the richer per-reading state
through an explicitly versioned consumer read contract. Use `GET /v2/media?entity_id=...&as_of=...`; do not silently replace the v1 enum.
The legacy surface cannot satisfy the typed-absence requirement on its own.

An absence writer is enabled only alongside the accepted richer read surface,
so the decision is inspectable when it can first be recorded. Disabling that
writer must not remove the ability to verify decisions already admitted.

## Required consumer acceptance cases

- No records produces unknown intent; explicit signed absence produces the
  deliberate state; a generated attachment produces the generated state.
- Image and absence for one reading expose conflict and both signed records,
  regardless of arrival order. No record is silently discarded.
- An `as_of` before admission cannot see the decision. Current source binding
  is separately labeled and never presented as an as-of reconstruction.
- A corrected or re-minted body does not inherit an old image or absence;
  old records remain inspectable as stale provenance.
- Multiple readings, including a shared body on another entity, do not leak
  illustration decisions between entity/body pairs.
- Tampered signatures, substituted hashes/IDs, cross-domain replay and an
  unprojected source are rejected before a write. Entity IDs retain all bits.
- Historical claim hashes, signatures, event counts, admission behavior and
  TT conformance vectors remain unchanged.
- Legacy callers retain their current response meaning; the versioned contract
  exposes all four states without fabricating decisions for old missing media.

## Requested upstream ruling

Please accept or amend the provenance boundary, four-state vocabulary, explicit
conflict behavior and versioned compatibility approach. Confirm whether any TT
admission field change is needed. Acceptance here authorizes subsequent consumer
implementation of the agreed contract, not a production deploy or bulk decisions.

The separate image-orphan assertion is consumer integrity work: every image's
body must remain projected by some moment. It does not resolve the typed-absence
question and must not be treated as satisfying this proposal.

## Concrete wire contract for the proposed consumer

`POST /v2/media/absence-decisions` accepts `manifest`, `author`, `signature`.
The manifest has exactly these fields:

| Field | Value |
|---|---|
| `schema` | `cc.media-absence.v1` |
| `kind` | `deliberately_unillustrated` |
| `source_entity_id` | Canonical decimal i64 string |
| `source_body_hash` | Lowercase SHA-256 hex |
| `writer` | Lowercase Ed25519 public-key hex, equal to `author` |
| `reason` | Nonblank text, at most 4096 UTF-8 bytes |
| `decided_at_ticks` | Canonical decimal i64 string, whole ticks since J2000 |

Decision time is writer-declared provenance, not admission time. The decision
ID is SHA-256 of `cc.media-absence.v1` followed by one NUL byte and the UTF-8
JCS manifest. Ed25519 signs that 32-byte digest. The server records its own
admission coordinate and deduplicates identical decisions by ID. Admission
locks the current source entity/body projection through commit. Replays after
source withdrawal are refused, like existing image replays.

The read route requires a full/read credential and mandatory `as_of`. The write
route requires a full credential and a live posture. Narrow credentials gain
no access. The response has schema `cc.media-readings.v2`,
`projection_basis: current`, `historical_evidence: false` and a `readings` array.
Each reading carries `source_body_hash`, the four-state `state`, signed `images`
and `absence_decisions`, and `source_binding` (`currently_projected` or
`stale_source`). Each signed record includes its admission coordinate.

Readings comprise current moment bodies with event coordinate at or before
`as_of`, plus bodies named by media records admitted at or before `as_of`.
Source binding uses the current projection independently of that visibility
filter. A query before all applicable moments/records returns an empty readings
array. It does not invent a reading or reconstruct withdrawn projections.
Results must not silently truncate records before computing state: a hidden
record must not turn conflict into an apparently uncontested decision.

This consumer-only addition does not add TT `ALLOWED_FIELDS` entries or permit
extra claim-envelope fields. No claims, image manifests, TT classifications or
historical signatures are rewritten. Revocation/resolution remains outside this
initial contract; coexisting records expose conflict regardless of arrival order.

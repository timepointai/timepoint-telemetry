# Clockchain source evidence and human-directed contributions v1

This consumer profile adds provenance and operator workflow. It changes no TT
ontology, classification contract, canonicalization, envelope identity or metric.
TT-SPEC and the five obligations in CONSUMERS.md continue to apply in full.
Clockchain owns source admission, causal relations, jobs and approval policy.

## Profile: cc.source-evidence.v1

A source-backed generated claim carries the profile in nested provenance, never
in TT's hash-covered `label`, `occurs_at` or `participants` fields.
`prov_measured.source_evidence_schema` names `cc.source-evidence.v1`.
`prov_measured.source_evidence` is a nonempty array. Each source records an HTTPS
`url`, `retrieved_at` UTC timestamp, `content_sha256` of the captured bytes,
`excerpt`, and `supports` field names. Sources collectively support `title`,
`year` and `summary`. Operational capture records additionally retain
`capture_path` or durable object reference, `publisher`, `license` and `locator`.
A hash identifies captured bytes; it does not prove publisher reliability or
passage support. Capture failure is a failure, never evidence of absence.
Captures remain available for audit; URLs alone are insufficient. Source
licensing is recorded per source; model licensing does not confer document rights.

`prov_asserted.source_support` records `schema: cc.source-support.v1`, a
`claim` string, nonempty `source_urls`, and `support_kind` of `observed` or
`attributed_announcement`. Every URL resolves to a captured source in the
measured record. `rationale` states what the cited passage supports and what it
does not. Announcement claims attribute the announcement; forecasts, promises
and promotional claims are not silently promoted into accomplished events.
Measured provider/model/run information remains separate from this asserted
historical interpretation. Existing model-asserted records are not retroactively
relabelled source-backed.

## Causal contributions

Clockchain relation names and evidence classes remain Clockchain vocabulary,
not TT bridges. A proposed edge explicitly supplies endpoints, relation,
evidence class and an `evidence` array in the same source shape, with
`supports: ["relation"]`, plus a `rationale`.
Unknown relations, unresolved references and unsupported declarations reject;
none defaults to influence. Chronological succession, topical similarity, shared
sources, model agreement and distinct signer keys do not alone establish cause.
A source asserting causation remains an attributed assertion; an inference is
labelled inference. The complete evidence record is bound to the exact edge event
or a signed manifest targeting it, never silently discarded during publication.

A claim with occurrence evidence but without causal evidence can be published
without causal edges. This is useful coverage, not an invalid partial answer.
Feasibility remains a result about the recorded graph, not verification of history.
Source support, signature integrity and independent corroboration are separate
reported properties.

## Profile: cc.human-directed-contribution.v1

A human approves a bounded topic brief before any metered generation. The brief
records its id, exact text, allowed sources, maximum entries, budget and model
policy. Initial operation permits at most five entries, forty text calls per UTC
day, and five USD per UTC day across text and image generation. Reservations
precede calls; crashes retain reservations until reconciled. Provider failures,
timeouts and unknown costs do not reset or evade the limit. Workers do not
choose new topics, recursively expand the graph or publish on their own.

The completed proposal includes the exact entries, edges and image manifests
with original image byte hashes, plus its brief id and capture references.
A human reviews the finished prose and actual images, then approves the SHA-256
of the entire canonical proposal. Canonicalization for this workflow is named by
the producer and publisher and tested byte-for-byte; it is not a change to TT
canonicalization. Any change to entries, edges, image bytes or manifests after
approval requires a new digest and new approval. Approval records identify the
human operator and approval timestamp. A model, worker or service account may
not self-approve. A typed actor name in a file is attribution, not authentication:
operator authentication is supplied by the controlled operator environment.

A separate controlled publisher verifies the brief approval, proposal approval,
digest, source bindings and admission before signing and committing. The worker
has no ledger signing key. Publication is idempotent for an approved digest and
must expose pending, failed and completed states. Jobs use durable leases and
fencing so stale workers cannot replace a newer result. Kill switches are checked
at startup and each cycle. One worker is the initial deployment; more workers
reuse the same limits and approval contract, not independent per-process budgets.

Images retain GENERATED-MEDIA.md's independent identity and provenance boundary.
Only the selected pinned model profile is accepted; a human reviews generated
images before approval. An image is an interpretation and never source evidence.
Missing, unrequested and failed media remain distinct operational states.

## Conformance and scope

Tests exercise rejected TT classifications, unresolved evidence, unknown edge
relations, changed approved bytes, duplicate publication, stale leases, exhausted
reservations and kill switches. Re-grounding or reclassification preserves TT
content identity; changing a hash-covered claim field yields a new identity.
Near matches remain ranked candidates or explicit recorded resolution decisions.
Human approval authorizes publication; it does not certify empirical truth.

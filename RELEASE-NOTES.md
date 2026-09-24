# Release notes

## v2.2.0 (not yet tagged)

Class: **Growth**, daily window. Proposal:
[docs/proposals/ENTITY-RELATIONS.md](docs/proposals/ENTITY-RELATIONS.md).

### What ships

- **A second vocabulary, beside the taxonomy.** `bundle/relations-v1.0.json`,
  `tt-relations/1.0 v1.0.0`, sha256
  `23b0dc5627301d19f128d96152a84fed87000954e1d9fff571f8658ee81b73bc`: 4 entity
  kinds and 10 relation kinds between entities, with load rules and an
  edge-statement contract (TT-SPEC §9). Loaded by `Vocabulary` in
  `src/relations.rs` and by `python/tt_relations.py`, which agree on every
  vector.
- **The taxonomy is untouched.** `bundle/taxonomy-v2.1.json` is byte-identical
  to v2.1.2's (sha256 `31ed385e…`, pinned by a test). No classification, shadow,
  distance or published reading moves.
- **Both lines of history.** v2.2.0 descends from v2.1.2 (`7afbc3b`, the
  source-provenance API beta compiles against) and from `main` (the Python
  on-ramp and the classification corpus).
- **Vectors layout.** Top-level `vectors/*.json` holds the same 10 envelope
  vectors, byte for byte, and only those: consumers glob that level. Verdict
  corpora live in `vectors/verdicts/`: `classification-verdicts.json`, tagged
  here for the first time, and the new `relation-verdicts.json`.

### For consumers

Nothing is required. A consumer that bumps to v2.2.0 changes the pinned tag
wherever it appears. Beta bumps the Cargo dependency and its CI's checkout
`ref` together. Beta's `conformance/python/check_vectors.py` passes unmodified
against this tree (10/10, exit 0). The five consumer obligations extend to the
new artifact (docs/CONSUMERS.md).

### The migration: Store only

Every divergence (D1–D5 in the proposal, §5) is **Store**. Nothing is pruned,
nothing is synthesized, and no row anywhere is rewritten.

- **The adoption pass lives in TT, not in beta.** The proposal (§5) says it is
  built in beta beside `ttmigrate.rs`. It was built here instead, at
  `tools/relations-adoption/relations_adoption.py`, because the node building
  this release could write only to TT. **A beta port is owed.** It belongs
  beside `ttmigrate.rs`, which owns `tt.migrations` and so the optional
  pass-log row this tool does not write.
- **Where it was rehearsed.** A disposable local PostgreSQL 18.4 on
  127.0.0.1, built from timepoint-beta's migrations as found on `cr/coord`
  (the 52 files numbered 0001–0053; there is no 0048; unchanged between beta
  `d6f5d0f` and `be0fd07`), seeded with
  `tools/relations-adoption/rehearsal_seed.sql`. That seed is synthetic and
  includes hostile values. Every figure matched the expectations at the end of
  the seed. Two runs printed byte-identical reports, and the database recorded
  no writes. That is a rehearsal, not a measurement.

### Not yet done

- **Utilization is not measured.** Nobody has run the adoption pass against
  beta production, so the proposal's §4 figures are still unknown. Per
  GOVERNANCE §5 that means "nobody told us", not "nobody uses it". It needs
  read-only production access, so L0 or Sean runs it:

  ```
  python3 tools/relations-adoption/relations_adoption.py \
      --db 'service=<beta production, read-only>' --deployment 'beta production'
  ```

  Use a pg_service.conf entry or `~/.pgpass`, not a password in `--db`. The
  report is aggregate counts only and prints each query next to its figure.
  Every verdict is Store, so no figure can move this release out of the daily
  window. The figures price the consumer's later, opt-in adoption work.
- **The lineage merge must be on `main` before tagging.** The candidate branch
  carries the merge of `fix/source-provenance-v1` (v2.1.2) into `main`'s line.
  It lands on `main` by pull request, and only then is `v2.2.0` tagged, at the
  first daily close after L0 confirms and Sean approves.

### Known limits

- **A new field in the relations artifact breaks consumers.** Its loader
  refuses unknown fields at every level (TT-SPEC §9.2), so any new field in a
  tt-relations artifact is a consumer-must-change release (Identity-class under
  GOVERNANCE §1). The taxonomy bundle is unlike it here, because its loader
  tolerates unknown fields. The strictness is the M1 default and Sean may
  reverse it before the tag.

- The `text` attribute type checks shape, not meaning or rendering. Text that
  is only format characters (a zero-width space) or that carries bidirectional
  controls passes TT, and two vectors pin that. Refusing it is the consumer's
  lint (TT-SPEC §9.1).

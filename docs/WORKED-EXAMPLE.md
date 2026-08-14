# One moment, carried end to end

The same object at every step — no section introduces a new example. Every
hash, distance and refusal below was computed by `tt-core` against
`bundle/taxonomy-v2.1.json`; nothing is illustrative or approximate. If you
recompute and get different bytes, one of us has a bug, and the
[vectors](../vectors/) decide who.

The moment: on 9 June 1815, seven powers signed the Final Act of the Congress
of Vienna.

## 1. The raw text

> "The Final Act, embodying all the separate treaties, was signed on 9 June
> 1815 by the plenipotentiaries of Austria, Britain, France, Portugal, Prussia,
> Russia and Sweden."

A sentence in a source. Two different systems reading it should end up holding
the same record — that is the entire problem.

## 2. The payload

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

Three of these fields are **the claim**: `label`, `occurs_at`, `participants`.
Everything else — the classification here, grounding and notes in a fuller
record — is interpretation layered on the claim, and the identity machinery
below never touches it.

## 3. The canonical form

`content_canonical` extracts exactly the claim fields and serialises them under
RFC 8785 (JCS): sorted keys, no whitespace, ES6 number formatting. These are
the exact bytes that get hashed:

```
{"label":"Final Act of the Congress of Vienna signed","occurs_at":"1815-06-09","participants":["austria","britain","france","portugal","prussia","russia","sweden"]}
```

Canonicalisation is why two implementations in two languages hash the same
claim to the same digest: there is only one byte sequence a conforming
implementation can produce.

## 4. The two hashes — and the same claim, told twice

```
content_hash = sha256:6cdce3f7563d74f6698b9b5ad6256a76fa16e2d6a9997781eebecbce6150f627
```

Now two systems record this moment independently. Telling one cites Wikipedia;
telling two cites a treaty corpus:

```json
{"system": "timepoint",   "source": "wikipedia:Congress_of_Vienna", "recorded_at": "2026-08-14"}
{"system": "another-lab", "source": "treaty-corpus-v3",             "recorded_at": "2026-09-02"}
```

```
provenance_hash (telling 1) = sha256:80682652212913c70adf4e2e1a9c1be97b624405824ef7001db4cbd43b07ea24
provenance_hash (telling 2) = sha256:6d74e475c3180a517fd946da5958c202ed679c41d1016698c3a30204ff38a1fa
```

Same `content_hash`, different `provenance_hash`: **the same claim, told
twice.** The collision is the point — no join table, no shared database, no
coordination between the two systems. And because the classification sits
outside the hash, a re-classified copy of this payload (say, someone who reads
the signing as `persuasion-and-rhetoric: 0.9`) still hashes to

```
sha256:6cdce3f7563d74f6698b9b5ad6256a76fa16e2d6a9997781eebecbce6150f627
```

— the identical claim. Interpretations differ; the moment keeps its identity.

## 5. The classification, read as a distribution

The record's reading of the moment, per lens:

- **Lens B** (what were people doing?): mostly `negotiation-and-agreement`
  (0.7), some `deciding-and-judging` (0.2). Mass need not sum to 1 — the
  remaining 0.1 is honest uncertainty.
- **Lens A** (what did the record keep?): `treaty-alliance-and-peace-accord`
  (0.85).

The format's validity rules (TT-SPEC §4): at most 3 entries per lens, every
mass in (0, 1], each lens summing to at most 1.0, every id present in the
bundle under the correct lens. An invalid distribution is rejected, never
repaired. `abstain: true` — no reading at all — is a publishable result.

## 6. The bridge walk

What event does the *action* imply? Deterministic code, no model involved: walk
`negotiation-and-agreement` up its parent chain to the first bridge.

```
negotiation-and-agreement  (chain: negotiation-and-agreement → communication-and-exchange)
  → bridge: relation = scales-up-to, event = treaty-alliance-and-peace-accord
    note: "a treaty is a handshake, at scale"
```

The derived pair `{relation, event}` is the moment's **shadow**, stored with
it. Here the shadow agrees with the Lens A classification — the record and the
bridge arrive at the same event independently, which is what you want to see.

Two contrasting walks, same code path:

```
friendship-and-companionship  → None            (no bridge anywhere on the chain: a gap — unmapped, so far)
courtship-and-falling-in-love → bridge: relation = unrecorded, event = null
                                ("the private core of a life leaves no public event" — the kernel: a finding)
```

## 7. Distance to a second moment — and a third

**Near:** the Treaty of Paris, signed 1815-11-20, classified by an independent
system as `lens_b: {negotiation-and-agreement: 0.6, persuasion-and-rhetoric:
0.25}`, `lens_a: {treaty-alliance-and-peace-accord: 0.7,
economic-summit-and-reparations: 0.25}`.

```
distance(vienna.lens_b, paris.lens_b) = 0.47
distance(vienna.lens_a, paris.lens_a) = 0.21
```

**Far:** Mount Tambora erupts, 1815-04-10, classified `lens_a:
{volcanic-and-geological: 0.9}` — and abstaining on Lens B, because "what were
people doing?" has no honest answer from the record of the eruption itself.

```
distance(vienna.lens_a, tambora.lens_a) = 8.8
distance(vienna.lens_b, tambora.lens_b) = None     (an abstained lens has no distance — absence, typed)
```

For scale: the farthest two nodes in Lens A sit 10.4 apart
(`earthquake-and-seismic` ↔ `spaceflight-and-frontier`). 0.21 is close kin;
8.8 is opposite ends of the recorded world. The full calibration table is in
the [README](../README.md#the-metric-anchored).

Same year, three moments: two treaties near each other on both lenses, one
volcano far away on the only lens it answers to. That is the whole system, run
once.

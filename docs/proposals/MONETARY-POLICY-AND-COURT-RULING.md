# Change request: monetary policy decisions and court rulings

Taxonomy `tt-ontology/1.0 v2.2.0` → `v2.3.0`, shipped as crate **v2.4.0** (tag
`v2.3.0` already exists and carries taxonomy 2.2.0). Follows
`.github/CHANGE_REQUEST.md` and GOVERNANCE §1–§7.

## 0. Base

| Copy | Taxonomy | sha256 |
|---|---|---|
| `main` @ `eb53032`, `bundle/taxonomy-v2.2.json` | 2.2.0 (151 nodes) | `d70812582ea080985a1f7778a67336b5c006bb2e9d7f77ae0c980f6193e52574` |
| tag `v2.3.0` (`4fe6d6a`) | 2.2.0 | identical |
| the copy timepoint-beta vendors and pins | 2.2.0 | identical |

The base on `main` is byte for byte the release consumers run, so this change
stacks on nothing unmerged.

## 1. Class

**Growth**, daily window. Two Lens A subspecies are added, each a leaf with
only a parent edge. No lateral edge, bridge, kernel, metric or lens change; no
id changes meaning; nothing is deleted or retired.

**Why not a lighter class:** none exists below Growth for an addition.
**Why not heavier:** every distance between v2.2.0 nodes is unchanged (§5), so
nothing a stored reading means can move.

### Options considered

- **Broaden existing definitions** (`currency-and-monetary-reform`,
  `law-and-justice`). Rejected: it changes what existing ids mean, which
  governance forbids outright.
- **A bridge.** Rejected: there is no Lens A node to bridge to, and bridges do
  not enter the distance metric.
- **One shared "scheduled decision" node.** Rejected: a rate hold, a court
  opinion and a statistics print would all sit at distance 0 from each other.
- **New leaves (this request).**

**Cut from the first draft:** `commodity-output-decision`,
`intergovernmental-resolution` and `official-economic-statistics-release`. No
abstained or misread record exists for any of them (§3), and a node without
evidence does not land.

## 2. The edit

| id | level | parent | label |
|---|---|---|---|
| `monetary-policy-decision` | subspecies | `money-and-finance` | Monetary Policy Decision |
| `court-ruling` | subspecies | `law-and-justice` | Court Ruling & Opinion |

**`monetary-policy-decision`:** A central bank or monetary authority formally
decides and announces its policy settings at a policy meeting or between
meetings: a policy interest rate such as a bank rate or target range, reserve
requirements, or asset purchases and sales, including a decision to leave them
unchanged. Setting a policy rate is always this event, even when the decision
is taken to stabilize prices or the currency. Not the introduction of a new
currency, and not a one-time package of currency or monetary stabilization
measures that replaces or revalues the money itself.

**`court-ruling`:** A court formally decides a case or a question of law and
issues its judgment or opinion: rulings of supreme, constitutional and
appellate courts, and judgments of international courts. A proceeding against
a person, with its verdict and penalty, is a trial, not this event, whoever
the defendant is; an appellate or constitutional ruling on such a case is this
event. Not the enactment of a law.

### Why these parents

- **`money-and-finance`** ("Banking institutions, exchanges, public offerings,
  currency introductions, and monetary reforms") already holds of a central
  bank's decision. Readers under v2.2.0 put rate decisions there or in its
  child `currency-and-monetary-reform` (§3). Placing the new leaf beside that
  child keeps an old misreading **near** (2.0) a new correct reading, rather
  than 4.0 apart, as the first draft's parent (`trade-policy-and-economic-governance`)
  would have done. The trade-policy label also hid a central bank from a reader
  scanning labels.
- **`law-and-justice`** ("the making of law and the administration of justice —
  legislation, constitutions, trials, and punishment") holds of a court's
  judgment.

### Boundaries: one rule each, stated the same way in the definition and here

| New node | Neighbour | Rule |
|---|---|---|
| `monetary-policy-decision` | `currency-and-monetary-reform` ("Introduction of new currencies and monetary stabilization measures, often responding to inflation or reparations crises") | **Setting a policy rate, reserve requirement or asset purchases is always `monetary-policy-decision`**, including an emergency hike against inflation. `currency-and-monetary-reform` keeps the introduction of a currency and one-time packages that replace or revalue the money (a redenomination, a new currency board, a currency reform). They are not split. |
| `court-ruling` | `trials-and-political-punishment` ("Judicial or extrajudicial proceedings and their penalties — trials, tribunals, proscriptions, executions, and exile of political figures") | **A proceeding against a person, with its verdict and penalty, is always `trials-and-political-punishment`**, whoever the defendant is. A court deciding a case or question of law and issuing a judgment or opinion is `court-ruling`, including an appellate or constitutional ruling on a trial. They are not split. |
| `court-ruling` | `legislation-edicts-and-decrees` | Enacting a law is legislation; a court deciding what a law means or whether it stands is a ruling. |

### Coverage left out by design

- Conditions ("Bank Rate is below 3% on a date") are states, not events, and
  must still abstain (§3: they do).
- Commodity output decisions, intergovernmental resolutions and statistics
  releases stay uncovered until a record shows the gap.

## 3. Evidence

**Stored abstentions** from the one consumer deployment (timepoint-beta,
portal endpoint readings), each `{abstain: true, lens_a: {}, lens_b: {}}`:

| Run | Endpoint | Node it is evidence for |
|---|---|---|
| `e5e38488-822f-400d-9521-b20d58fee6d9` | the Bank of England MPC publishes its November Bank Rate decision, 2026-11-05 | `monetary-policy-decision` |
| `a16cd5e6-c017-4618-9cba-8169e7a5cf50` | the ECB changes its main refinancing rate at least once between 2026-10-19 and 2027-12-31 | `monetary-policy-decision` |
| `3cd8d6e7-551c-4c11-a6a5-6749e4a9d6e7` | the US Supreme Court issues an opinion in an argued case, by 2027-07-31 | `court-ruling` |

**Controlled re-reading.** The consumer's exact reader prompt, node catalog and
context, run against the v2.2.0 bundle and against this release's
`taxonomy-v2.3.json` (sha256 `c84817144c6ed99431bcee63a7bbfe6338facf51a3573c0cda4562331df1e1c1`).
Model `anthropic/claude-haiku-5.5`, 2 runs per endpoint and bundle, $0.05:

| Endpoint | v2.2.0 (Lens A) | v2.3.0 (Lens A) |
|---|---|---|
| BoE Bank Rate decision on 2026-11-05 | `currency-and-monetary-reform` 0.6; `money-and-finance` 0.6 | `monetary-policy-decision` 0.9; 0.9 |
| ECB changes its rate at least once in a window | `money-and-finance` 0.5; `currency-and-monetary-reform` 0.6 | `monetary-policy-decision` 1.0; 0.9 |
| ECB deposit rate decision on 2026-10-29 | `currency-and-monetary-reform` 0.5; `money-and-finance` 0.6 | `monetary-policy-decision` 0.9; 0.9 |
| US Supreme Court opinion by 2027-07-31 | `law-and-justice` 0.8; 0.8 | `court-ruling` 0.9; 0.9 |
| A G20 country holds a national election (control) | `elections-and-official-selection` 1.0; 1.0 | same, 1.0; 0.9 |
| UK general election on 2029-05-03 (control) | `elections-and-official-selection` 0.8; 0.85 | same, 1.0; 0.9 |
| Bank Rate is below 3% on 2027-06-30 (condition) | abstain; abstain | abstain; abstain |

What this shows:

- **Under v2.2.0 a rate decision is misread**, never cleanly read: half the
  readings land on a currency *reform*. A reader that abstains (the stored
  records) and a reader that misreads (the re-reading) are both answering a
  question the vocabulary does not let them answer.
- **Under v2.3.0 both node types read cleanly and stably**, at 0.9–1.0.
- **Controls do not move**, and the condition still abstains, so the new
  definitions are not catch-alls.
- **What it does not show:** why the stored readings abstained rather than
  misread. The re-reading did not reproduce those abstentions on any endpoint,
  including the G20 control, whose node already exists. The consumer's
  production reader model or its frame summary differs from the re-reading.
  That is a consumer question (§8), not a vocabulary one: whichever way the
  reader fails today, it has no node that holds.

## 4. Utilization

**Unmeasured.** Reading the consumer's stored readings needs its database, and
no credential for it was used to prepare this release. Zero means "nobody told
us", not "no records". The structural answer does not depend on it: no
existing record can move, because no existing id or distance changes.

## 5. The migration: Store only

`tools/taxonomy-growth/check.py`, run twice on the rehearsal readings with
byte-identical output (`tools/taxonomy-growth/test_check.py` asserts it):

```text
nodes: 151 -> 153 (added 2, removed 0, meaning changed 0)
lateral_edges unchanged: yes
bridges unchanged: yes
kernel unchanged: yes
metric unchanged: yes
lenses unchanged: yes
pre-existing pairs compared: 11325; distance changed: 0 (yes)
  court-ruling: reaches 82 of 82 same-lens nodes (yes)
  monetary-policy-decision: reaches 82 of 82 same-lens nodes (yes)
content valid under old: 9; still valid under new: 9 (yes)
citations older than one step (resolved by walking the chain, not rewritten): 5
citations that would be rewritten: 0 (yes)
rows that would be rewritten: 0
growth-clean: yes
```

`tests/growth.rs` checks the same two claims against golden digests of **both**
earlier releases: every v2.2.0 node and every v2.1.0 node keeps its record, and
every distance between them is unchanged.

**Store is the default and the whole migration.** No Prune, no Synthesize. A
consumer may re-read endpoints that abstained as new readings citing v2.3.0,
keeping the old reading.

## 6. Gate

- [x] The bundle validates (`tests/bundle.rs`: 153 nodes, Lens A 6/28/49, Lens B 9/61/0, 151 lateral edges, 26 bridges, 3 kernel).
- [x] Every existing conformance vector still produces its recorded hash (`tests/vectors.rs`, `envelope.rs`, `source_vectors.rs`; Python envelope and relation suites).
- [x] New surface has new vectors: each new id accepted at mass 1, split with its parent, wrong-lens under B; v2.2.0 cited one step back accepted; v2.1.0 two steps back rejected (`vectors/verdicts/classification-verdicts.json`, 46 → 54). Each new id is `unknown-id` under v2.2.0 (`python/test_tt_validate.py`).
- [x] No id changed meaning, none deleted (`tests/growth.rs`, `check.py`).
- [x] Migration rehearsed, deterministic, zero rewrites (§5).
- [x] Everything valid under the previous version is still valid: **content** yes. **Citations:** see the exception in RELEASE-NOTES v2.4.0 (v2.1.0 stamps are now two steps back).
- [x] Class is right: Growth (§1).

## 7. Risk to blind-check honesty

**R1. Readers pushed to guess.** Lowered. Each definition requires a formal act
by a named kind of institution and names its neighbour's actual wording in a
"Not" clause. The condition control abstains under v2.3.0 as under v2.2.0.

**R2. Proximity guaranteed by the calendar. The taxonomy cannot fix this.**
Telemetry records the *type* of an event, not its outcome. A blind replay that
runs to a scheduled meeting date reads as `monetary-policy-decision` at
distance 0 whether the bank cut or held. Direction is a dimension, not a kind,
and will not be encoded as nodes. **Consumer condition for adoption:** a
consumer that scores replays by distance must check the outcome separately
before counting a distance-0 match on a scheduled event as reaching the
endpoint. timepoint-beta is adding that check alongside its adoption.

**R3. Nearness** (`check.py` distances, Lens A, ≤ 2.0):

| New node | Within 2.0 |
|---|---|
| `monetary-policy-decision` | 4: `money-and-finance` 1; `banking-and-exchange`, `currency-and-monetary-reform`, `economy-trade-and-labor` 2 |
| `court-ruling` | 5: `law-and-justice` 1; `constitutions-and-charters`, `legislation-edicts-and-decrees`, `politics-governance-and-law`, `trials-and-political-punishment` 2 |

The two new nodes are 5.6 apart.

## 8. Consumer notes for timepoint-beta (adoption, not part of this release)

- Bump the pin to tag `v2.4.0` and the vendored path to `bundle/taxonomy-v2.3.json`; re-check the sha256.
- `portal_no_route.rs` holds a deliberate tripwire for this gap; it becomes a positive test.
- R2 must be resolved before portal walks count these nodes as reached.
- The stored abstentions above stay as they are; a re-run is a new run.
- The stored abstentions were not reproduced by the controlled re-reading
  (§3). The consumer should compare its production reader model and endpoint
  summary with the re-reading.

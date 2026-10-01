# Change request: two Lens A species for institutional acquisition and program adoption

Taxonomy `tt-ontology/1.0 v2.1.0` → `v2.2.0`, shipped in crate `tt-core` 2.3.0.
Follows `.github/CHANGE_REQUEST.md` and GOVERNANCE §1–§7.

## 1. Class

**Growth**, daily window (the NYSE close). Taxonomy minor bump 2.1.0 → 2.2.0;
crate 2.2.0 → 2.3.0 (tag `v2.2.0` already exists and carries taxonomy 2.1.0, so
the new taxonomy ships under crate tag `v2.3.0`).

**Why this class and not a lighter one.** It adds ids, which is new surface, so
it is not a Correction. **Why not Structure.** The effect test — could any
existing record be read differently? — answers no:

- No existing id, parent, label, definition, lateral edge, bridge, kernel
  member or metric weight changes.
- The two new nodes are leaves with only a parent edge. A node of degree one
  cannot shorten a path between two other nodes, so every distance between
  existing nodes is unchanged. This is checked, not read: all 11,026 pairs of
  v2.1.0 nodes (3,081 of them Lens A) keep their distance
  (`tests/growth.rs`, `tools/taxonomy-growth/check.py`).
- **No lateral edge and no bridge is added, deliberately.** Either would move
  distances or derived shadows of existing readings, and would need its own
  argument.

**Identity question.** The schema id and the envelope shape are unchanged. The
file is renamed `bundle/taxonomy-v2.1.json` → `bundle/taxonomy-v2.2.json`, as at
every earlier bump (v1.1 → v2.0 → v2.1). Consumers who pin by tag (GOVERNANCE
§8) change the tag and the path together.

## 2. The edit

`version` "2.1.0" → "2.2.0"; `supersedes` → `"tt-ontology/1.0 v2.1.0"`; two
nodes appended to their branch groups:

```json
{"id": "procurement-and-contract-award", "lens": "A", "level": "species",
 "parent": "economy-trade-and-labor", "label": "Procurement & Contract Award",
 "definition": "A buyer — a government body, public institution or company — formally awards, signs or renews a contract with an outside supplier for goods, services or works: tender awards, supply and service contracts, and multi-year service agreements. Not the purchase of a company."}
{"id": "public-program-adoption", "lens": "A", "level": "species",
 "parent": "politics-governance-and-law", "label": "Public Program Adoption",
 "definition": "A government body or public institution formally adopts, funds, starts or ends a public program, service or initiative by its own administrative or budget decision, without enacting a law: program rollouts, budget decisions and grants that fund a program, and decisions to discontinue one."}
```

Result: 151 nodes; Lens A 81 (6 branches, 28 species, 47 subspecies); Lens B
70; lateral edges 151; bridges 26; kernel 3.

### Why two nodes, and where they sit

The two record types are different shapes. One is bilateral: a buyer and an
outside supplier sign a contract. The other is unilateral: a public body
decides, under authority it already holds, to adopt, fund, start or end a
program. A single node would need a disjunctive definition spanning two
branches, and would then not be true of every member. Each is a species
directly under its branch, as `games-and-sport` and `war-declaration-and-outbreak`
already are, because no existing species definition holds of its members
(`enterprise-and-commerce` is the founding and milestones of companies;
`governance-and-popular-politics` is elections, government formation and street
politics).

### Boundaries against the nearest existing nodes

| Existing node | Boundary |
|---|---|
| `legislation-edicts-and-decrees` | A law or decree changes the legal order. Program adoption is a decision under existing authority. A statute that creates a program is legislation (optionally plus program adoption at lower mass). |
| `enterprise-and-commerce`, `corporate-founding-and-milestone` | Founding, chartering and milestones of companies, including acquisitions. Buying a company is not procurement; the definition says so. |
| `trade-policy-and-economic-governance` | State or multilateral economic action (tariffs, trade pacts, summits). A state-to-state agreement is not a buyer–supplier contract. |
| `labor-reform-and-accord` | Employer–workforce settlements. Procurement is a supplier contract. |
| `appointments-and-government-formation` | Appointing an official or forming a body. Retaining a contractor is procurement. |
| `treaty-alliance-and-peace-accord` | Agreements between polities. |
| `learning-institutions` | Founding universities, academies, libraries. A school body adopting a program is program adoption; buying a supplier's curriculum or service is procurement. |
| `consumer-product-launch` | The seller-side market release. Procurement is the buyer-side award. |
| `trade-and-transport-infrastructure` | Opening a line or facility. Awarding its construction contract is procurement; adopting the program to build it is program adoption. |
| `money-and-finance` | Banking, currency, monetary reform. A budget decision funding a program is program adoption. |

### What each covers (generic, across sectors and eras)

**`procurement-and-contract-award`:** an air ministry's production order for
fighter aircraft; a space agency awarding a crewed-spacecraft contract; a
defense department awarding a development or cloud-computing contract; a dam
or subway construction contract award; a water or power concession to a
private operator; a bloc's advance purchase agreement for vaccine doses; an
airline's aircraft order; a hospital network renewing a managed-IT contract; a
school body signing a multi-year agreement for an outside provider's program.
*Not:* a bank acquiring a rival (corporate milestone), a trade pact (trade
policy), a transit line opening (infrastructure), a staff union accord (labor).

**`public-program-adoption`:** a health department starting a national
screening program; a treasury announcing and funding a demand-support scheme;
a budget cancelling a space program; a government launching school meals or
household electrification; a digital-residency program; a cabinet approving a
rail line; a city council budget that funds a pre-kindergarten pilot; a
research council funding a multi-year grant program. *Not:* the act creating a
national health service (legislation), the ministry's supply contract that
follows (procurement), a tram inauguration (infrastructure), appointing a
program director (appointments).

## 3. Evidence

Under v2.1.0 the Lens A vocabulary has no event type for an institution
formally acquiring something from a supplier or formally adopting a public
program. A consumer (a backward-search product that classifies modeled states
against the bundle) recorded **10 classifier readings of 10 distinct states
from two runs, and none had a Lens A reading**: 9 abstentions and 1 action-only
reading (Lens B `negotiation-and-agreement`, empty Lens A). The states were of
two types: a public body has signed a multi-year agreement to bring an outside
provider's program into its schools; a governing committee adopts a budget
that funds, defers or scales a program.

Two independent readers given the full classifier prompt and catalog both
judged that no Lens A definition holds: the treaty node requires polities,
`enterprise-and-commerce` is company founding and milestones, and the governance
nodes are elections, appointments, assemblies and protest. Abstention was the
correct answer under v2.1.0. The gap is vocabulary coverage, not model error.

**Limit of this evidence:** two runs of one product, one domain. The coverage
list above is how the definitions were tested for generality, not evidence of
demand from other consumers.

## 4. Utilization

| Node touched | Records classified against it | Of those, load-bearing | Source |
|---|---|---|---|
| `procurement-and-contract-award` (new) | 0 by construction | 0 | new id |
| `public-program-adoption` (new) | 0 by construction | 0 | new id |
| `economy-trade-and-labor`, `politics-governance-and-law` (parents) | **not measured** | **not measured** | see below |
| all other v2.1.0 nodes | 0 touched (no id, edge or text change) | — | by construction; gate checks |

**Not measured, and why.** The read-only utilization pass against the one known
consumer deployment was not run: its database read was not permitted in the
session that prepared this request. Per GOVERNANCE §5 that means "nobody told
us", never "nobody uses it". The parents' meanings do not change and no
reading moves, so the figure would not change the class; it is owed before the
release notes claim any count.

## 5. The migration

**Store only.** Nothing is pruned, nothing is synthesized, no row is rewritten.

| # | Divergence | Verdict |
|---|---|---|
| D1 | Readings citing v2.1.0 versus the v2.2.0 bundle | Store. Every v2.1.0-valid id is valid under v2.2.0 with the same lens and meaning. The citation stays `v2.1.0`, a true record of what produced it, and is accepted one step back (`Bundle::is_this_release`; the Python validator now agrees, see below). |
| D2 | Stored abstentions on states the new nodes would now cover | Store, untouched. An abstention is a true reading of the vocabulary it cites. Rewriting it would fabricate a reading no v2.1.0 reader produced. A consumer may re-read such a state as a new reading citing v2.2.0 and keep the old one; that is consumer work, not a TT migration. |
| D3 | Action-only readings with Lens B `negotiation-and-agreement`, whose shadow is the treaty node | Store, untouched. A bridge to the new nodes would change derived shadows and needs its own proposal. |

**The iterator.** `tools/taxonomy-growth/check.py OLD NEW [READINGS.jsonl]`:
report-only, stdlib, deterministic (two runs print the same bytes). It
compares the bundles node by node, edge by edge and pair by pair, and
re-validates a file of stored readings under both releases.

**Dry-run output** (old = `v2.2.0:bundle/taxonomy-v2.1.json`, sha256 `31ed385e…`;
readings = the synthetic rehearsal file `tools/taxonomy-growth/rehearsal_readings.jsonl`):

```
TAXONOMY GROWTH CHECK — REPORT-ONLY (nothing written)
old: tt-ontology/1.0 v2.1.0
new: tt-ontology/1.0 v2.2.0 (supersedes tt-ontology/1.0 v2.1.0)
lineage is one step: yes
nodes: 149 -> 151 (added 2, removed 0, meaning changed 0)
  added: procurement-and-contract-award (lens A, species, parent economy-trade-and-labor)
  added: public-program-adoption (lens A, species, parent politics-governance-and-law)
lateral_edges unchanged: yes
bridges unchanged: yes
kernel unchanged: yes
metric unchanged: yes
lenses unchanged: yes
pre-existing pairs compared: 11026; distance changed: 0 (yes)
  procurement-and-contract-award: reaches 80 of 80 same-lens nodes (yes)
  public-program-adoption: reaches 80 of 80 same-lens nodes (yes)
stored readings by cited bundle: <unstamped>=2 tt-ontology/1.0 v2.0.0=1 tt-ontology/1.0 v2.1.0=6
content valid under old: 7; still valid under new: 7 (yes)
citations older than one step (resolved by walking the chain, not rewritten): 1
citations that would be rewritten: 0 (yes)
verdict for every divergence: STORE
rows that would be rewritten: 0
growth-clean: yes
```

**Python validator, in the same release.** `python/tt_validate.py` rejected any
citation other than the loaded release (`bundle-mismatch`), while the Rust
`Bundle::is_this_release` accepts one step back. Under v2.2.0 that would have
made every reading citing v2.1.0 invalid to the Python reference, contradicting
gate item 6. It now accepts the loaded release or the one it supersedes, keeps
the citation as written, and still rejects two steps back. New vectors cover it.

## 6. Gate

- [x] the bundle validates (`cargo test --test bundle`, updated counts)
- [x] every existing conformance vector still produces its recorded hash: the 10
      envelope vectors are byte-identical; `classification-verdicts.json` is
      regenerated and differs only in version strings and added vectors
- [x] new surface has new vectors: both species accepted at mass 1, split, with
      their branch, and rejected under Lens B; a citation of the superseded
      release accepted and kept
- [x] no id changed meaning; none were deleted — checked mechanically against
      golden digests of the 149 v2.1.0 records and of every v2.1.0 pair distance
      (`tests/growth.rs`), and by `tools/taxonomy-growth/check.py`
- [x] the migration ran on a rehearsal copy and reported what it would do: on a
      synthetic readings file, twice, byte-identical. **Exception:** not run on a
      consumer's stored readings (the read was not permitted; §4)
- [x] Correction/Growth only: everything valid under the previous version is
      still valid — content of every v2.1.0-valid reading; citations one step
      back accepted by both tiers

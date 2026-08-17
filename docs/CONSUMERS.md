# The consumer contract

What TT asks of anything downstream of it. Two consumers exist today —
timepoint-beta and the Clockchain — and both were consulted in the writing;
every rule below is one that a real consumer either followed from the start or
broke, got caught breaking, and now tests for. Nothing here is hypothetical.

TT-SPEC governs the objects themselves (classifications, envelopes, the
bundle). This document governs the *relationship*: what a consumer must do to
be able to say its TT claims mean what TT says they mean.

## The five obligations

**1. Pin, and check the bytes.**
Consume a named release, pinned by tag or revision, and verify the bundle's
sha256 against the bytes you actually loaded — at build time or at startup,
but before the first read that depends on it. Two forms are equally
legitimate: depending on the `tt-core` crate at a pinned revision, or vendoring
the bundle artifact and hashing it (the Clockchain's wasm filter does the
latter deliberately; its minimal dependency tree is a load-bearing property).
What is not legitimate is a copy of the taxonomy that nothing byte-checks: an
unverified copy is a fork with extra steps.

**2. Validate against the whole bundle, not your subset.**
A consumer that adopts 91 of the 149 node ids still validates incoming ids
against all 149. Validity and declaration are different questions: an id
outside your subset but inside the bundle is *valid and undeclared* (answer
with silence, or adopt it); an id outside the bundle *does not exist* (answer
loudly). A boundary that cannot tell those two apart has collapsed a finding
into a gap, and no policy on top of it can recover the difference.

**3. Resolve retirement on read.**
Ids are never deleted and never change meaning, but a release may retire one
and name its successor. A stored classification citing a retired id is still a
true record of the reading that produced it — do not rewrite it — but a
*query* against it resolves through the successor chain (bounded, so a
defective chain cannot hang a read). The write path is stricter: new
classifications citing a retired id are rejected, and the rejection names the
successor (TT-SPEC §4).

**4. Reject, never repair.**
A classification that fails any §4 rule is thrown back whole, with every
failure named — not the first, and not a repaired version. A consumer that
"helpfully" fixes mass sums, drops surplus entries, or remaps a wrong-lens id
has manufactured a reading nobody produced. The same discipline applies one
level up: a batch containing a rejected entry is the consumer's own policy
call (the Clockchain refuses the whole batch; that is local, and fine), but
each *entry's* verdict is TT's and is not negotiable per entry.

**5. Derive bundle facts; check any you accept as input.**
The lens of an id, the ancestry of an id, a bridge's shape, whether an
alternative crosses lenses — these are facts *of the pinned bundle*, and a
consumer computes them from it. If your pipeline also carries them as declared
fields (a `lens` column, a `cross_lens` flag), the declaration is checked
against the derivation and a mismatch is rejected: a declared lens is checked,
not believed. A bundle-derivable field allowed to drift from the bundle is a
config echo — the value survives long after the source it was copied from has
moved.

## Stricter is allowed; looser is not

A consumer may refuse what TT permits. It may never accept what TT rejects.

The Clockchain refuses a `claim_type_alternatives` entry that is an ancestor
or descendant of the chosen type. TT *permits* mass on an ancestor and a
descendant in one classification — that is a legitimate expression of
specificity uncertainty, and the metric's hierarchy weights price it. Both
positions are correct, because they answer different questions: TT defines
what is *sayable*; a consumer's admission profile defines what it will
*store*. A local profile stays local: it binds no other consumer, it is not a
TT rule, and a consumer documents it as its own. The one-way rule is the
boundary — the moment a consumer accepts an unknown id, a fourth lens entry,
or a mass of 1.3, its "TT classifications" are no longer TT classifications,
whatever the column is named.

## What TT has no opinion about

Coordinates and calendars (`occurs_at` is an opaque string — TT-SPEC §7),
deduplication and merge policy, batch semantics, title hygiene, provenance
completeness rules, storage schemas, retry and queue behavior. Consumers own
these. TT's identity rule has one consequence worth naming here: two payloads
with different labels are two identities — a dedup policy that merges them
silently has erased a distinction the hash was built to keep. Refusing a
merge and leaving it for a human is a policy; merging silently is a loss.

## Abstention, at the boundary

`abstain: true` with empty lenses is valid and publishable (TT-SPEC §4.4).
Two consumer-side rules follow. An abstention cites its bundle — it is a
reading of a specific vocabulary, not a shrug. And an abstention *rate*
divides by moments actually put to a reader, never by all moments: "declined"
and "never asked" are different facts, and a denominator that pools them is
reporting on the corpus while claiming to report on the reader.

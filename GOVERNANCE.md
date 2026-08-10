# How Timepoint Telemetry changes

Timepoint Telemetry is a reference system for coordinating about the real
world — across agents, across platforms, across parties who share no database
and no employer. Its whole value is that a record classified here means the
same thing when read there, today and in four years.

That makes the **update process part of the format**. People trust the format
because they trust the process; they adopt it because they trust the format.
A vocabulary with excellent structure and unpredictable governance is not
something anyone can build on.

Two guarantees hold above everything else:

> **An id never changes meaning.** Not once, not ever, not "slightly".
>
> **A change carries its own solution.** A proposal that edits the taxonomy
> without saying what happens to records already classified under it is not a
> proposal; it is a wish.

---

## 1. Change classes

Every change is exactly one class. The class determines its pace, the work
required to propose it, and who has to agree.

| Class | What it is | Examples |
|---|---|---|
| **Correction** | Text only. Cannot move a single record between nodes. | a misspelled label, a typo in a definition, a broken note |
| **Growth** | New surface. Existing records keep their classification, but the graph changes shape. | a new node, a new bridge, a new lateral edge |
| **Structure** | Existing meaning moves. | re-parenting, deprecating a node, changing metric weights, touching a branch or the kernel |

**Never, under any circumstance:** reusing an id for a different meaning, or
deleting an id. Retirement happens through deprecation (§3), which keeps the id
readable forever.

### The class is decided by effect, not by intent

The test is mechanical: *could this change cause a record to be read
differently than it was written?*

- Fixing `recieved` to `received` in a definition — no. **Correction.**
- Adding a lateral edge — **yes.** It changes the distance between two nodes
  that already existed, so anyone thresholding on nearness gets a different
  answer. That is why lateral edges are Growth, not Correction, and why they
  carry a migration.

## 2. Release windows

There are **two windows and no others**, both anchored to the close of the New
York Stock Exchange:

| Window | When | Carries |
|---|---|---|
| **Daily** | every trading day, at the close | Correction, Growth |
| **Weekly** | Friday, at the close | Structure |

| Class | Window | Notice | Version |
|---|---|---|---|
| **Correction** | daily | none | patch — `1.1.0` → `1.1.1` |
| **Growth** | daily | in the release notes | minor — `1.1.0` → `1.2.0` |
| **Structure** | weekly | **announced at the previous weekly window — one full week** | major — `1.1.0` → `2.0.0` |

### Why an exchange close

Because it is a real instant the world already coordinates on, and this is a
reference system for the real world. It resolves holidays and half-days without
anyone maintaining a calendar — "the close" is whatever the close was that day,
including the early ones. A schedule that is itself a reference beats a schedule
somebody has to remember to keep.

Nothing ships between windows. An urgent correction waits for the next daily
close, which is never more than a day away; if something is too urgent for that,
the honest response is a security advisory, not a surprise release.

### Why the two are different

Corrections and Growth cannot make an existing record mean something new, so a
consumer who never upgrades loses nothing by them — they can move at the speed
of the work.

Structure can. It gets the weekly window and a full week of announced notice,
and the kernel — the part of the taxonomy that says which human actions leave no
public record — is the slowest thing inside the slowest window.

A consumer who pins a major version and ignores every window must still be
correct. That is the promise the windows exist to keep.

## 3. Deprecation, because identity is frozen

Ids can never be deleted and never change meaning. Without a retirement
primitive the taxonomy could only ever grow, and would eventually collapse
under its own history.

A deprecated node stays in the bundle, stays valid to read, and stops being a
target for new classification:

```json
{
  "id": "some-node",
  "deprecated_in": "2.0.0",
  "superseded_by": "the-node-that-replaces-it",
  "deprecation_note": "why, in one sentence a stranger can act on"
}
```

- **A deprecation is always Structure.** It never ships as Growth.
- **`superseded_by` is required unless the note explains why nothing replaces
  it** — a node deprecated because it should never have existed has no
  successor, and must say so.
- **Resolution follows the chain.** Reading a deprecated id gives you the node
  it points at; the chain is acyclic and implementations must detect cycles
  rather than loop.
- Old records are never rewritten by the bundle. They are rewritten, if at all,
  by the migration the change carried (§4).

## 4. A proposal carries its solution

This is the rule that makes the rest survivable.

A change request is not "here is the edit". It is **the edit, the evidence, and
the iterator that reconciles what already exists**.

### Every proposal states

1. **The class** (§1), and why that class and not a lighter one.
2. **The edit** — the exact bundle diff.
3. **The evidence** — for Growth, real records that could not be classified
   without it, or were classified wrong; for Correction, the error; for
   Structure, what is broken today.
4. **Utilization** (§5) — how much existing work this touches.
5. **The migration** — see below.

### The migration

A deterministic iterator over existing records that leaves them correct under
the new version. It must state, for every affected record, which of three
things happens:

| | |
|---|---|
| **Prune** | The branch was created in error or is redundant. It is removed, and the records under it move to the node that should always have held them. |
| **Synthesize** | Two or more nodes were the same thing wearing different names. They merge; one id survives, the others are deprecated pointing at it, and references rewrite. |
| **Store** | The divergence is real and worth keeping. Both nodes persist, the relationship between them is recorded, and nothing is rewritten. |

The default is **Store**. Prune and Synthesize destroy or move information and
must be argued for, not assumed — a taxonomy that tidies itself is a taxonomy
that quietly loses things.

A migration must be runnable more than once without changing its answer, and
must be able to report what it *would* do before it does anything.

## 5. Utilization is metadata, and it sets the price

Changing a node nobody has used is cheap. Changing a node that anchors a
hundred thousand classifications is expensive, and the process should say so
out loud rather than discovering it afterwards.

Every proposal reports, for each node it touches:

- how many records classify against it,
- how many of those are load-bearing (anchoring a forecast, a bridge, a
  published claim),
- and from which deployments the figures came.

Utilization does not change a proposal's class — effect does that. It changes
the **work required to land it**: a heavily-used node demands a migration with
a rehearsal, and — where the class allows a choice — more notice rather than
less. It can also push a change up a class: if reconciling the records is
substantial enough that a consumer could be surprised, that is Structure and
belongs in the weekly window.

Utilization is reported, not authoritative. No single deployment can see the
whole ecosystem, and a figure of zero means *nobody told us*, never *nobody
uses it*.

## 6. Who decides

**Today: Timepoint decides.** There is one steward, changes are made by
mandate, and pretending otherwise would be theatre.

This is stated plainly rather than dressed up, because the transition is the
part that matters:

| Stage | Decision rule | Trigger to leave it |
|---|---|---|
| **Steward** (now) | Timepoint decides; every change is public, with its reasoning | a second independent implementation passes the conformance vectors |
| **Steward + comment** | proposals open for a full week before they land; Timepoint still decides, and answers objections in writing | three or more parties classifying in production against a pinned version |
| **Stewards** | a small group; Growth needs a majority, Structure needs consensus or an explicit, recorded override | — |

Decision rights cover **the taxonomy**. The specification prose and the
reference implementation remain Timepoint's, and are not governed by this
document.

## 7. The gate

Deliberately light while this is young. A release is a release when it can
say yes to all of these, and the windows are where they are enforced:

- the bundle validates;
- every existing conformance vector still produces its recorded hash;
- new surface has new vectors;
- **no id changed meaning and none were deleted** — checked mechanically
  against the previous version, not by reading the diff;
- the migration ran on a rehearsal copy and reported what it would do;
- for Correction and Growth: everything valid under the previous version is
  still valid.

If a check cannot be run yet, the release notes say which one and why. An
unstated exception is the only kind that is not allowed.

## 8. Where versions live

GitHub. Every published version stays fetchable forever at a tag; `main` is
the current release; proposals are pull requests carrying their evidence and
migration.

Pin by tag. Verify with the bundle's own hash. Nothing here requires trusting
a running deployment.

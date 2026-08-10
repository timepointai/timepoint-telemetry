# Change request

> A proposal is **the edit, the evidence, and the iterator that reconciles what
> already exists**. A change that edits the taxonomy without saying what happens
> to records already classified under it is not a proposal.
>
> Read GOVERNANCE.md first. Delete the guidance lines as you fill this in.

## 1. Class

- [ ] **Correction** — text only; cannot move a single record between nodes
- [ ] **Growth** — new node / bridge / lateral edge
- [ ] **Structure** — existing meaning moves; re-parenting, deprecation, metric weights, a branch, the kernel

**Why this class and not a lighter one:**
<!-- The test is mechanical: could this cause a record to be read differently
     than it was written? Adding a lateral edge changes distances between nodes
     that already existed — that is Growth, never Correction. -->

## 2. The edit

<!-- The exact bundle diff. Ids, parents, lens, level, definitions. -->

```json
```

## 3. Evidence

<!-- Growth: real records that could not be classified without this, or were
     classified wrong. Correction: the error. Structure: what is broken today.
     A node earns its place by being needed. "It would be tidier" is not
     evidence. -->

## 4. Utilization

<!-- What existing work this touches. A figure of zero means NOBODY TOLD US,
     never "nobody uses it". Name the deployments the numbers came from. -->

| Node touched | Records classified against it | Of those, load-bearing | Source |
|---|---|---|---|
| | | | |

## 5. The migration

<!-- Required for Growth and Structure. A deterministic iterator that leaves
     existing records correct under the new version. It must be runnable more
     than once without changing its answer, and must be able to report what it
     WOULD do before doing anything. -->

**Dry-run output:**

```
```

For every divergence the change creates or exposes, one of:

- [ ] **Prune** — the branch was an error or is redundant; records move to the node that should always have held them
- [ ] **Synthesize** — two nodes were the same thing under different names; one id survives, the others are deprecated pointing at it
- [ ] **Store** — the divergence is real and worth keeping; both persist, the relationship is recorded, nothing is rewritten

**Store is the default.** Prune and Synthesize destroy or move information and
have to be argued for — a taxonomy that tidies itself is one that quietly loses
things.

**Argument:**

## 6. Gate

- [ ] the bundle validates
- [ ] every existing conformance vector still produces its recorded hash
- [ ] new surface has new vectors
- [ ] no id changed meaning; none were deleted (checked mechanically, not by reading the diff)
- [ ] the migration ran on a rehearsal copy and reported what it would do
- [ ] Correction/Growth only: everything valid under the previous version is still valid

<!-- If a check cannot be run yet, say which and why. An unstated exception is
     the only kind that is not allowed. -->

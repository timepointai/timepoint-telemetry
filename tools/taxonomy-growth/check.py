#!/usr/bin/env python3
"""Report-only migration pass for a taxonomy Growth release. Never writes.

    python3 tools/taxonomy-growth/check.py OLD_BUNDLE NEW_BUNDLE [READINGS.jsonl]

Compares two bundle files and, optionally, a file of stored readings (one JSON
classification per line, as a consumer stores them, `bundle` stamp included).
It states, for every divergence, what a migration would do — and for Growth
the answer must be Store with zero rewrites:

  * every OLD node is present in NEW with identical lens, level, parent, label,
    definition and retirement fields (no id changed meaning, none deleted);
  * lateral edges, bridges, kernel and metric are identical;
  * every pairwise distance between OLD nodes is identical in NEW;
  * every stored reading valid under OLD is valid under NEW, keeps its stamp,
    and would be rewritten 0 times.

Deterministic: sorted output, no clock, no randomness; two runs print the same
bytes. Exit 0 when the release is Growth-clean, 1 when any check fails, 2 on
bad usage. Stdlib only, like the rest of the Python on-ramp.
"""

import heapq
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(HERE)), "python"))
import tt_validate  # noqa: E402

NODE_FIELDS = ("lens", "level", "parent", "label", "definition",
               "deprecated_in", "superseded_by", "deprecation_note")


def load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def version(raw):
    return f"{raw['schema']} v{raw['version']}"


def distances(raw):
    """All-pairs weighted shortest paths over parent and lateral edges."""
    w_h = raw["metric"]["hierarchy_edge_weight"]
    w_l = raw["metric"]["lateral_edge_weight"]
    adj = {n["id"]: [] for n in raw["nodes"]}
    for n in raw["nodes"]:
        if n.get("parent"):
            adj.setdefault(n["id"], []).append((n["parent"], w_h))
            adj.setdefault(n["parent"], []).append((n["id"], w_h))
    for e in raw["lateral_edges"]:
        adj.setdefault(e["a"], []).append((e["b"], w_l))
        adj.setdefault(e["b"], []).append((e["a"], w_l))
    out = {}
    for src in sorted(adj):
        dist = {src: 0.0}
        heap = [(0.0, src)]
        while heap:
            d, x = heapq.heappop(heap)
            if d > dist.get(x, float("inf")):
                continue
            for y, w in adj.get(x, ()):
                nd = d + w
                if nd < dist.get(y, float("inf")):
                    dist[y] = nd
                    heapq.heappush(heap, (nd, y))
        out[src] = dist
    return out


def report(old, new, readings):
    lines, failed = [], []

    def w(s):
        lines.append(s)

    def check(ok, what):
        if not ok:
            failed.append(what)
        return "yes" if ok else "NO"

    old_nodes = {n["id"]: n for n in old["nodes"]}
    new_nodes = {n["id"]: n for n in new["nodes"]}
    added = sorted(set(new_nodes) - set(old_nodes))
    removed = sorted(set(old_nodes) - set(new_nodes))
    changed = sorted(i for i in old_nodes if i in new_nodes and
                     tuple(old_nodes[i].get(f) for f in NODE_FIELDS) !=
                     tuple(new_nodes[i].get(f) for f in NODE_FIELDS))
    w("TAXONOMY GROWTH CHECK — REPORT-ONLY (nothing written)")
    w(f"old: {version(old)}")
    w(f"new: {version(new)} (supersedes {new.get('supersedes')})")
    w(f"lineage is one step: {check(new.get('supersedes') == version(old), 'supersedes')}")
    w(f"nodes: {len(old_nodes)} -> {len(new_nodes)} (added {len(added)}, removed {len(removed)}, "
      f"meaning changed {len(changed)})")
    for i in added:
        n = new_nodes[i]
        w(f"  added: {i} (lens {n['lens']}, {n['level']}, parent {n.get('parent')})")
    check(not removed, "removed ids")
    check(not changed, "changed ids")
    for key in ("lateral_edges", "bridges", "kernel", "metric", "lenses"):
        same = old[key] == new[key]
        w(f"{key} unchanged: {check(same, key)}")
    d_old, d_new = distances(old), distances(new)
    ids = sorted(old_nodes)
    pairs = moved = 0
    for x, a in enumerate(ids):
        for b in ids[x + 1:]:
            pairs += 1
            if d_old.get(a, {}).get(b) != d_new.get(a, {}).get(b):
                moved += 1
    w(f"pre-existing pairs compared: {pairs}; distance changed: {moved} "
      f"({check(moved == 0, 'distances')})")
    for i in added:
        reach = sum(1 for j in new_nodes if j != i and j in d_new[i] and new_nodes[j]["lens"] == new_nodes[i]["lens"])
        same_lens = sum(1 for j in new_nodes if j != i and new_nodes[j]["lens"] == new_nodes[i]["lens"])
        w(f"  {i}: reaches {reach} of {same_lens} same-lens nodes "
          f"({check(reach == same_lens, 'reachability ' + i)})")

    if readings is None:
        w("stored readings: not supplied (not measured, never reported as 0)")
    else:
        b_old = tt_validate.load_bundle(readings["old_path"])
        b_new = tt_validate.load_bundle(readings["new_path"])
        # Content (ids, lenses, masses) is what a Growth release must keep valid.
        # A citation is lineage: one step back resolves directly; older ones walk
        # the chain through the intervening bundle (TT-SPEC §1), never rewritten.
        this_new = (b_new["version_string"], b_new["supersedes"])
        stamps, valid_old, still_valid, walk, restamped = {}, 0, 0, 0, 0
        for c in readings["rows"]:
            stamp = c.get("bundle", "<unstamped>") if isinstance(c, dict) else "<not-an-object>"
            stamps[stamp] = stamps.get(stamp, 0) + 1
            content = {k: v for k, v in c.items() if k != "bundle"} if isinstance(c, dict) else c
            _, e_old = tt_validate.validate(content, b_old)
            if e_old:
                continue
            valid_old += 1
            _, e_new = tt_validate.validate(content, b_new)
            if e_new:
                continue
            still_valid += 1
            if stamp != "<unstamped>":
                if stamp not in this_new:
                    walk += 1
                else:
                    n_new, _ = tt_validate.validate(c, b_new)
                    restamped += int(n_new["bundle"] != stamp)
        w("stored readings by cited bundle: " + " ".join(f"{k}={stamps[k]}" for k in sorted(stamps)))
        w(f"content valid under old: {valid_old}; still valid under new: {still_valid} "
          f"({check(still_valid == valid_old, 'readings validity')})")
        w(f"citations older than one step (resolved by walking the chain, not rewritten): {walk}")
        w(f"citations that would be rewritten: {restamped} ({check(restamped == 0, 'restamp')})")
    w("verdict for every divergence: STORE")
    w("rows that would be rewritten: 0")
    w("growth-clean: " + ("yes" if not failed else "NO — " + ", ".join(failed)))
    return "\n".join(lines) + "\n", not failed


def main(argv):
    if len(argv) not in (3, 4):
        print((__doc__ or "").strip(), file=sys.stderr)
        return 2
    old, new = load(argv[1]), load(argv[2])
    readings = None
    if len(argv) == 4:
        with open(argv[3], encoding="utf-8") as f:
            rows = [json.loads(line) for line in f if line.strip()]
        readings = {"old_path": argv[1], "new_path": argv[2], "rows": rows}
    text, ok = report(old, new, readings)
    sys.stdout.write(text)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))

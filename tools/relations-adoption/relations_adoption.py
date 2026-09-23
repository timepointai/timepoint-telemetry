#!/usr/bin/env python3
"""The tt-relations/1.0 adoption pass: REPORT-ONLY, and it never writes.

The migration the v2.2.0 change request carries (docs/proposals/ENTITY-RELATIONS.md
§5), built against timepoint-beta's schema as found. It reads a consumer
database and reports, for each divergence the new vocabulary creates or
exposes, how many records it touches and what happens to them. Every verdict is
Store, so there is nothing to apply: `--apply` is accepted, writes nothing, and
says so.

    python3 tools/relations-adoption/relations_adoption.py \
        --db 'host=/path/to/socket port=5432 dbname=beta' --deployment rehearsal

Properties the change request requires, and how this file keeps them:

  * DETERMINISTIC. Every query has a total ORDER BY and all of them read one
    REPEATABLE READ snapshot. Nothing time-dependent is printed. Two runs on the
    same database print byte-identical reports.
  * READ-ONLY. The session sets default_transaction_read_only and the snapshot
    is a READ ONLY transaction, so a write would fail rather than land. The
    pass-log row §5 allows is left to the port beside beta's ttmigrate.rs, which
    owns tt.migrations; this tool writes nothing at all.
  * PINNED. It reads only the two artifacts in this repository and refuses to
    run unless their sha256 are the released ones.
  * AGGREGATE ONLY. Counts, never values: no display names, no assertion
    values, no participant slugs. A kind, prefix or version string outside the
    expected set is counted under "(other)", never printed.
  * A ZERO IS MEASURED, AN ABSENT TABLE IS NOT. A table the database lacks is
    reported as absent, never as 0.

The connection string is passed explicitly with --db, handed to psql, and never
printed. No environment variable is read for it, so the tool cannot reach a
database nobody named. --deployment is the label printed in the report.
"""

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
TAXONOMY = os.path.join(REPO, "bundle", "taxonomy-v2.1.json")
RELATIONS = os.path.join(REPO, "bundle", "relations-v1.0.json")
TAXONOMY_SHA256 = "31ed385e26522a5b548f7404f7757ee370ed9783dbd550b05cd69e89e9462113"
RELATIONS_SHA256 = "23b0dc5627301d19f128d96152a84fed87000954e1d9fff571f8658ee81b73bc"

# The string assertions a role, an office or a link to an org is written as today (§3).
D2_ATTRIBUTES = ("current-role", "named-in-uploaded-document", "works-at")
# Bases that enter read_context_assertions as KNOWN context: load-bearing (§7.3).
LOAD_BEARING_BASES = ("ACCUMULATED", "GROUNDED")
BASES = ("ACCUMULATED", "GENERATED", "GROUNDED")
VERSION_RE = re.compile(r"^[a-z][a-z-]*/[0-9]+\.[0-9]+ v[0-9]+\.[0-9]+\.[0-9]+$")
TABLES = ("entity.assertions", "entity.entities", "run.artifacts", "run.moments", "tt.verdicts")

NULL = "<null>"


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def load_artifacts():
    for path, want in ((TAXONOMY, TAXONOMY_SHA256), (RELATIONS, RELATIONS_SHA256)):
        got = sha256(path)
        if got != want:
            raise SystemExit(f"refusing to run: {os.path.relpath(path, REPO)} has sha256 {got}, "
                             f"not the released {want}")
    with open(TAXONOMY, encoding="utf-8") as f:
        tax = json.load(f)
    with open(RELATIONS, encoding="utf-8") as f:
        rel = json.load(f)
    return (f"{tax['schema']} v{tax['version']}", f"{rel['schema']} v{rel['version']}",
            [k["id"] for k in rel["entity_kinds"]])


def sql_list(items):
    # Every item is a constant of this file or a validated kebab-case id.
    for i in items:
        assert re.fullmatch(r"[A-Za-z0-9 ./-]+", i), i
    return ", ".join(f"'{i}'" for i in items)


def queries(present, kinds, taxonomy_vs):
    """(label, sql) for each figure, over the tables the database has."""
    k = sql_list(kinds)
    out = []
    if "entity.entities" in present:
        out.append(("D1", f"""SELECT CASE WHEN kind IN ({k}) THEN kind ELSE '(other)' END AS kind, count(*)
  FROM entity.entities GROUP BY 1 ORDER BY 1"""))
    if "entity.assertions" in present:
        out.append(("D2", f"""SELECT attribute,
       CASE WHEN basis IN ({sql_list(BASES)}) THEN basis ELSE '(other)' END AS basis, count(*)
  FROM entity.assertions
 WHERE attribute IN ({sql_list(D2_ATTRIBUTES)})
 GROUP BY 1, 2 ORDER BY 1, 2"""))
    if "run.artifacts" in present:
        out.append(("D3", """SELECT kind, count(*) AS documents, sum(jsonb_array_length(content->'ties')) AS ties
  FROM run.artifacts
 WHERE jsonb_typeof(content->'ties') = 'array'
 GROUP BY kind ORDER BY kind"""))
    if "run.moments" in present:
        out.append(("D4", f"""SELECT CASE WHEN split_part(p, '/', 2) IN ({k}) AND p LIKE '/%' THEN split_part(p, '/', 2)
            ELSE '(other)' END AS kind, count(*)
  FROM run.moments m, jsonb_array_elements_text(m.participants) p
 WHERE jsonb_typeof(m.participants) = 'array'
 GROUP BY 1 ORDER BY 1"""))
        out.append(("D4-not-array", """SELECT count(*) FROM run.moments
 WHERE jsonb_typeof(participants) IS DISTINCT FROM 'array'"""))
        out.append(("D5-moments", """SELECT coalesce(classification->>'bundle', '<unstamped>') AS bundle, count(*)
  FROM run.moments WHERE classification IS NOT NULL
 GROUP BY 1 ORDER BY 1"""))
    if "run.artifacts" in present:
        out.append(("D5-readings", """SELECT coalesce(r.value->>'bundle', '<unstamped>') AS bundle, count(*)
  FROM run.artifacts a, jsonb_each(a.content #> '{appendix,moment_readings}') r
 WHERE jsonb_typeof(a.content #> '{appendix,moment_readings}') = 'object'
 GROUP BY 1 ORDER BY 1"""))
    if "tt.verdicts" in present:
        out.append(("D5-verdicts", """SELECT coalesce(bundle_version, '<unstamped>') AS bundle, count(*)
  FROM tt.verdicts GROUP BY 1 ORDER BY 1"""))
    return out


def run_psql(db, script):
    cmd = ["psql", "-X", "-q", "-A", "-t", "-F", "\t", "-v", "ON_ERROR_STOP=1", "-d", db, "-f", "-"]
    proc = subprocess.run(cmd, input=script, capture_output=True, text=True)
    if proc.returncode != 0:
        # psql's own message; it names the failing statement, never the password.
        raise SystemExit(f"psql failed (exit {proc.returncode}):\n{proc.stderr.strip()}")
    sections, cur = {}, None
    for line in proc.stdout.splitlines():
        if line.startswith("@@"):
            cur = line[2:]
            sections[cur] = []
        elif cur is not None and line != "":
            sections[cur].append(line.split("\t"))
    return sections


PREAMBLE = f"""\\pset null '{NULL}'
SET default_transaction_read_only = on;
BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;
"""


def measure(db, kinds, taxonomy_vs):
    probe = PREAMBLE + "\\echo @@present\n" + (
        "SELECT t FROM unnest(ARRAY[" + sql_list(TABLES) + "]) t "
        "WHERE to_regclass(t) IS NOT NULL ORDER BY t;\nCOMMIT;\n")
    present = {row[0] for row in run_psql(db, probe).get("present", [])}
    qs = queries(present, kinds, taxonomy_vs)
    script = PREAMBLE + "".join(f"\\echo @@{label}\n{sql};\n" for label, sql in qs) + "COMMIT;\n"
    return present, qs, run_psql(db, script)


def version_label(v):
    if v == "<unstamped>" or VERSION_RE.match(v):
        return v
    return "(other)"


def fold(rows):
    """[[key, n]] -> {key: n}, keys passed through version_label."""
    out = {}
    for key, n in rows:
        key = version_label(key)
        out[key] = out.get(key, 0) + int(n)
    return out


def report(deployment, taxonomy_vs, relations_vs, kinds, present, qs, res, apply):
    lines = []
    w = lines.append
    absent = [t for t in TABLES if t not in present]
    w(f"TT RELATIONS v{relations_vs.split(' v')[1]} ADOPTION REPORT — REPORT-ONLY (nothing written)")
    w(f"deployment: {deployment}")
    w(f"taxonomy loaded: {taxonomy_vs} sha256 {TAXONOMY_SHA256[:8]}… (unchanged: yes)")
    w(f"relations loaded: {relations_vs} sha256 {RELATIONS_SHA256[:8]}…")
    w("tables absent (not measured, never reported as 0): " + (", ".join(absent) if absent else "none"))

    if "D1" in res:
        d1 = {row[0]: int(row[1]) for row in res["D1"]}
        parts = [f"{k}={d1.get(k, 0)}" for k in kinds] + [f"outside-TT-kinds={d1.get('(other)', 0)}"]
        w("D1 entities by kind: " + " ".join(parts) + " → STORE")
    else:
        w("D1 entities by kind: not measured (entity.entities absent)")

    if "D2" in res:
        cells = {(a, b): int(n) for a, b, n in res["D2"]}
        parts, bearing = [], 0
        for a in D2_ATTRIBUTES:
            for b in BASES + ("(other)",):
                n = cells.get((a, b), 0)
                if b != "(other)" or n:
                    parts.append(f"{a}/{b}={n}")
                if b in LOAD_BEARING_BASES:
                    bearing += n
        w("D2 string role/link assertions: " + " ".join(parts) + " → STORE")
        w(f"D2 of those, load-bearing (GROUNDED or ACCUMULATED, read as KNOWN context): {bearing}")
    else:
        w("D2 string role/link assertions: not measured (entity.assertions absent)")

    if "D3" in res:
        parts = [f"{kind}={ties} (in {docs} doc{'' if docs == '1' else 's'})" for kind, docs, ties in res["D3"]]
        w("D3 ties in run.artifacts: " + (" ".join(parts) if parts else "none")
          + " → STORE (always: hashed artifacts)")
    else:
        w("D3 ties in run.artifacts: not measured (run.artifacts absent)")

    if "D4" in res:
        d4 = {row[0]: int(row[1]) for row in res["D4"]}
        parts = [f"/{k}={d4.get(k, 0)}" for k in kinds] + [f"/other={d4.get('(other)', 0)}"]
        not_array = int(res["D4-not-array"][0][0])
        w("D4 participant paths: " + " ".join(parts) + " → STORE (hash-covered)")
        w(f"D4 moments whose participants are not an array (not counted above): {not_array}")
    else:
        w("D4 participant paths: not measured (run.moments absent)")

    other_total, unstamped_total = 0, 0
    for label, source in (("D5-moments", "run.moments.classification"),
                          ("D5-readings", "run.artifacts appendix.moment_readings"),
                          ("D5-verdicts", "tt.verdicts.bundle_version")):
        if label not in res:
            continue
        counts = fold(res[label])
        parts = [f"{k}={n}" for k, n in sorted(counts.items())]
        w(f"D5 {source} by cited bundle: " + (" ".join(parts) if parts else "none"))
        other_total += sum(n for k, n in counts.items() if k not in (taxonomy_vs, "<unstamped>"))
        unstamped_total += counts.get("<unstamped>", 0)
    if any(label in res for label in ("D5-moments", "D5-readings", "D5-verdicts")):
        w(f"D5 readings citing a taxonomy version other than the loaded one: {other_total} → STORE")
        w(f"D5 readings with no bundle stamp (declared, never back-filled): {unstamped_total} → STORE")
    else:
        w("D5 readings: not measured (run.moments, run.artifacts and tt.verdicts absent)")
    w("rows that would change on --apply: 0")
    if apply:
        w("--apply: every verdict is Store; nothing was applied and nothing was written")
    w("")
    w("QUERIES (read-only, one REPEATABLE READ snapshot, each labeled with the figure it produced)")
    for label, sql in qs:
        w(f"-- {label}")
        w(sql + ";")
    return "\n".join(lines) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=(__doc__ or "").split("\n")[0])
    ap.add_argument("--db", required=True, help="psql connection string; passed to psql, never printed")
    ap.add_argument("--deployment", required=True, help="the name printed in the report")
    ap.add_argument("--apply", action="store_true",
                    help="accepted for symmetry with ttmigrate; every verdict is Store, so it writes nothing")
    args = ap.parse_args(argv)
    taxonomy_vs, relations_vs, kinds = load_artifacts()
    present, qs, res = measure(args.db, kinds, taxonomy_vs)
    sys.stdout.write(report(args.deployment, taxonomy_vs, relations_vs, kinds, present, qs, res, args.apply))
    return 0


if __name__ == "__main__":
    sys.exit(main())

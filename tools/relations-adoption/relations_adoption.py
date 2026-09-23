#!/usr/bin/env python3
"""The tt-relations/1.0 adoption pass: REPORT-ONLY, and it never writes.

The migration the v2.2.0 change request carries (docs/proposals/ENTITY-RELATIONS.md
§5), built against timepoint-beta's schema as found. It reads a consumer
database and reports, for each divergence the new vocabulary creates or
exposes, how many records it touches and what happens to them. Every verdict is
Store, so there is nothing to apply: `--apply` is accepted, writes nothing, and
says so. It lives in TT for now; a port beside beta's ttmigrate.rs is owed.

    python3 tools/relations-adoption/relations_adoption.py \
        --db 'service=beta-readonly' --deployment 'beta production'

CONNECTING. --db is a libpq connection string handed to psql and never
printed. Keep the password out of it: use a service entry (pg_service.conf,
`service=NAME`) or a password file (~/.pgpass), so the secret never sits in
shell history or a process listing. No environment variable is read for the
target, so the tool cannot reach a database nobody named.

Properties the change request requires, and how this file keeps them:

  * DETERMINISTIC. All figures come from one REPEATABLE READ snapshot, rows
    are sorted before printing, and nothing time-dependent is printed. Two runs
    on the same database print byte-identical reports.
  * READ-ONLY. The session sets default_transaction_read_only and the snapshot
    is a READ ONLY transaction, so a write would fail rather than land. The
    pass-log row §5 allows is left to the beta port, which owns tt.migrations.
  * PINNED. It reads only the two artifacts in this repository and refuses to
    run unless their sha256 are the released ones.
  * AGGREGATE ONLY. Counts, never values: no display names, no assertion
    values, no participant slugs. Every label printed comes from a fixed
    allow-list, applied in SQL and checked again here; anything else is
    counted as "(other)".
  * FAIL CLOSED. psql returns ONE JSON document per call. Stored values are
    inside JSON strings, so no value can forge a row or a section. Output that
    is not exactly the expected shape stops the run with exit 3 and no report.
  * A ZERO IS MEASURED, AN ABSENT TABLE IS NOT. A table the database lacks is
    reported as not measured, never as 0.
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
# The run.artifacts kinds whose documents carry top-level `ties` in beta.
D3_KINDS = ("cockpit_doc", "frame")
VERSION_RE = r"^[a-z][a-z-]*/[0-9]+\.[0-9]+ v[0-9]+\.[0-9]+\.[0-9]+$"
TABLES = ("entity.assertions", "entity.entities", "run.artifacts", "run.moments", "tt.verdicts")
OTHER, UNSTAMPED = "(other)", "<unstamped>"
EXIT_REFUSED = 3


class Refused(Exception):
    """psql's output was not exactly what the queries can produce."""


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
            tuple(k["id"] for k in rel["entity_kinds"]))


def sql_list(items):
    # Every item is a constant of this file or a validated kebab-case id.
    for i in items:
        assert re.fullmatch(r"[A-Za-z0-9 ./_-]+", i), i
    return ", ".join(f"'{i}'" for i in items)


def version_case(expr):
    return (f"CASE WHEN {expr} IS NULL THEN '{UNSTAMPED}' "
            f"WHEN {expr} ~ '{VERSION_RE}' THEN {expr} ELSE '{OTHER}' END")


# --------------------------------------------------------------------------
# The figures: label, the query shown in the report, and each row's columns.

def figures(present, kinds):
    k = sql_list(kinds)
    out = []
    if "entity.entities" in present:
        out.append(("D1", ("kind", "n"), f"""SELECT CASE WHEN kind IN ({k}) THEN kind ELSE '{OTHER}' END AS kind, count(*) AS n
  FROM entity.entities GROUP BY 1 ORDER BY 1"""))
    if "entity.assertions" in present:
        out.append(("D2", ("attribute", "basis", "n"), f"""SELECT attribute,
       CASE WHEN basis IN ({sql_list(BASES)}) THEN basis ELSE '{OTHER}' END AS basis, count(*) AS n
  FROM entity.assertions
 WHERE attribute IN ({sql_list(D2_ATTRIBUTES)})
 GROUP BY 1, 2 ORDER BY 1, 2"""))
    if "run.artifacts" in present:
        out.append(("D3", ("kind", "documents", "ties"), f"""SELECT CASE WHEN kind IN ({sql_list(D3_KINDS)}) THEN kind ELSE '{OTHER}' END AS kind,
       count(*) AS documents, sum(jsonb_array_length(content->'ties')) AS ties
  FROM run.artifacts
 WHERE jsonb_typeof(content->'ties') = 'array'
 GROUP BY 1 ORDER BY 1"""))
    if "run.moments" in present:
        out.append(("D4", ("kind", "n"), f"""SELECT CASE WHEN p LIKE '/%' AND split_part(p, '/', 2) IN ({k}) THEN split_part(p, '/', 2)
            ELSE '{OTHER}' END AS kind, count(*) AS n
  FROM run.moments m, jsonb_array_elements_text(m.participants) p
 WHERE jsonb_typeof(m.participants) = 'array'
 GROUP BY 1 ORDER BY 1"""))
        out.append(("D4-not-array", ("n",), """SELECT count(*) AS n FROM run.moments
 WHERE jsonb_typeof(participants) IS DISTINCT FROM 'array'"""))
        out.append(("D5-moments", ("bundle", "n"), f"""SELECT {version_case("classification->>'bundle'")} AS bundle, count(*) AS n
  FROM run.moments WHERE classification IS NOT NULL
 GROUP BY 1 ORDER BY 1"""))
    if "run.artifacts" in present:
        out.append(("D5-readings", ("bundle", "n"), f"""SELECT {version_case("r.value->>'bundle'")} AS bundle, count(*) AS n
  FROM run.artifacts a, jsonb_each(a.content #> '{{appendix,moment_readings}}') r
 WHERE jsonb_typeof(a.content #> '{{appendix,moment_readings}}') = 'object'
 GROUP BY 1 ORDER BY 1"""))
    if "tt.verdicts" in present:
        out.append(("D5-verdicts", ("bundle", "n"), f"""SELECT {version_case("bundle_version")} AS bundle, count(*) AS n
  FROM tt.verdicts GROUP BY 1 ORDER BY 1"""))
    return out


PREAMBLE = """SET default_transaction_read_only = on;
BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;
"""


def presence_script():
    return PREAMBLE + (
        "SELECT jsonb_build_object('present', coalesce(jsonb_agg(t ORDER BY t), '[]'::jsonb))\n"
        f"  FROM unnest(ARRAY[{sql_list(TABLES)}]) t WHERE to_regclass(t) IS NOT NULL;\nCOMMIT;\n")


def figures_script(figs):
    """One SELECT returning one jsonb object: {label: [row, ...]} for every figure."""
    if not figs:
        return PREAMBLE + "SELECT '{}'::jsonb;\nCOMMIT;\n"
    parts = ",\n".join(
        f"  '{label}', (SELECT coalesce(jsonb_agg(to_jsonb(q)), '[]'::jsonb) FROM (\n{sql}\n) q)"
        for label, _, sql in figs)
    return PREAMBLE + f"SELECT jsonb_build_object(\n{parts});\nCOMMIT;\n"


# --------------------------------------------------------------------------
# Parsing: exactly one JSON document of exactly the expected shape, or refuse.

COUNT_COLUMNS = ("n", "documents", "ties")


def one_json_line(stdout):
    """psql's whole output: exactly one line, holding exactly one JSON value.
    jsonb text never contains a raw newline, so a second line can only mean
    output nobody asked for."""
    if not stdout.endswith("\n") or stdout.count("\n") != 1:
        raise Refused("psql output is not exactly one line")
    try:
        return json.loads(stdout)
    except ValueError as e:
        raise Refused(f"psql output is not one JSON document: {e}") from None


def parse_presence(stdout):
    doc = one_json_line(stdout)
    if not isinstance(doc, dict) or set(doc) != {"present"} or not isinstance(doc["present"], list):
        raise Refused("presence probe: not {\"present\": [...]}")
    names = doc["present"]
    if not all(isinstance(t, str) and t in TABLES for t in names) or len(set(names)) != len(names):
        raise Refused("presence probe: a table name outside the list asked about")
    return set(names)


def parse_document(stdout, expected):
    """stdout must be one line holding one JSON object whose keys are `expected`
    ({label: columns}) and whose values are lists of rows with exactly those
    columns. Strings are strings, counts are non-negative integers."""
    doc = one_json_line(stdout)
    if not isinstance(doc, dict) or set(doc) != set(expected):
        raise Refused("psql output does not have exactly the expected sections")
    for label, columns in expected.items():
        rows = doc[label]
        if not isinstance(rows, list):
            raise Refused(f"{label}: not a list of rows")
        for row in rows:
            if not isinstance(row, dict) or set(row) != set(columns):
                raise Refused(f"{label}: a row does not have exactly the columns {list(columns)}")
            for c in columns:
                v = row[c]
                if c in COUNT_COLUMNS:
                    if isinstance(v, bool) or not isinstance(v, int) or v < 0:
                        raise Refused(f"{label}.{c}: not a non-negative integer")
                elif not isinstance(v, str):
                    raise Refused(f"{label}.{c}: not a string")
    return doc


def check_labels(res, kinds):
    """Every label printed must be one the queries can produce. A value that is
    not is refused, never printed: the SQL allow-lists should have folded it."""
    version = re.compile(VERSION_RE)
    allowed = {
        "D1": {"kind": set(kinds) | {OTHER}},
        "D2": {"attribute": set(D2_ATTRIBUTES), "basis": set(BASES) | {OTHER}},
        "D3": {"kind": set(D3_KINDS) | {OTHER}},
        "D4": {"kind": set(kinds) | {OTHER}},
    }
    for label, rows in res.items():
        keys = set()
        for row in rows:
            key = tuple(v for c, v in row.items() if isinstance(v, str))
            if key in keys:
                raise Refused(f"{label}: a label appears twice")
            keys.add(key)
            for c, v in row.items():
                if not isinstance(v, str):
                    continue
                if label.startswith("D5-"):
                    ok = v in (UNSTAMPED, OTHER) or version.match(v)
                else:
                    ok = v in allowed[label][c]
                if not ok:
                    raise Refused(f"{label}.{c}: a label outside the allow-list")
    if "D4-not-array" in res and len(res["D4-not-array"]) != 1:
        raise Refused("D4-not-array: expected exactly one row")


def run_psql(db, script):
    cmd = ["psql", "-X", "-q", "-A", "-t", "-v", "ON_ERROR_STOP=1", "-d", db, "-f", "-"]
    proc = subprocess.run(cmd, input=script, capture_output=True, text=True)
    if proc.returncode != 0:
        # psql's own message; it names the failing statement, never the password.
        raise SystemExit(f"psql failed (exit {proc.returncode}):\n{proc.stderr.strip()}")
    return proc.stdout


def measure(db, kinds):
    present = parse_presence(run_psql(db, presence_script()))
    figs = figures(present, kinds)
    res = parse_document(run_psql(db, figures_script(figs)), {label: cols for label, cols, _ in figs})
    check_labels(res, kinds)
    return present, figs, res


# --------------------------------------------------------------------------
# The report.

def report(deployment, taxonomy_vs, relations_vs, kinds, present, figs, res, apply):
    lines = []
    w = lines.append
    absent = [t for t in TABLES if t not in present]
    # Rows keyed by their label columns in declared order (jsonb reorders
    # object keys, so the row's own order is not the query's).
    columns = {label: cols for label, cols, _ in figs}
    by = {label: {tuple(row[c] for c in columns[label] if c not in COUNT_COLUMNS): row
                  for row in rows} for label, rows in res.items()}

    def n(label, *key):
        row = by.get(label, {}).get(key)
        return row["n"] if row else 0

    w(f"TT RELATIONS v{relations_vs.split(' v')[1]} ADOPTION REPORT — REPORT-ONLY (nothing written)")
    w(f"deployment: {deployment}")
    w(f"taxonomy loaded by this tool: {taxonomy_vs} sha256 {TAXONOMY_SHA256[:8]}… "
      "(unchanged: TT's copy is byte-identical to v2.1.2's)")
    w(f"relations loaded by this tool: {relations_vs} sha256 {RELATIONS_SHA256[:8]}…")
    w("tables absent (not measured, never reported as 0): " + (", ".join(absent) if absent else "none"))

    if "D1" in res:
        parts = [f"{k}={n('D1', k)}" for k in kinds] + [f"outside-TT-kinds={n('D1', OTHER)}"]
        w("D1 entities by kind: " + " ".join(parts) + " → STORE")
    else:
        w("D1 entities by kind: not measured (entity.entities absent)")

    if "D2" in res:
        parts, bearing = [], 0
        for a in D2_ATTRIBUTES:
            for b in BASES + (OTHER,):
                c = n("D2", a, b)
                if b != OTHER or c:
                    parts.append(f"{a}/{b}={c}")
                if b in LOAD_BEARING_BASES:
                    bearing += c
        w("D2 string role/link assertions: " + " ".join(parts) + " → STORE")
        w(f"D2 of those, load-bearing (GROUNDED or ACCUMULATED, read as KNOWN context): {bearing}")
    else:
        w("D2 string role/link assertions: not measured (entity.assertions absent)")

    if "D3" in res:
        parts = []
        for kind in D3_KINDS + (OTHER,):
            row = by["D3"].get((kind,))
            if row:
                docs = row["documents"]
                parts.append(f"{kind}={row['ties']} (in {docs} doc{'' if docs == 1 else 's'})")
        w("D3 ties in run.artifacts: " + (" ".join(parts) if parts else "none")
          + " → STORE (always: hashed artifacts)")
    else:
        w("D3 ties in run.artifacts: not measured (run.artifacts absent)")

    if "D4" in res:
        parts = [f"/{k}={n('D4', k)}" for k in kinds] + [f"/other={n('D4', OTHER)}"]
        w("D4 participant paths: " + " ".join(parts) + " → STORE (hash-covered)")
        w(f"D4 moments whose participants are not an array (not counted above): "
          f"{res['D4-not-array'][0]['n']}")
    else:
        w("D4 participant paths: not measured (run.moments absent)")

    measured_d5 = False
    other_total, unstamped_total = 0, 0
    for label, source in (("D5-moments", "run.moments.classification"),
                          ("D5-readings", "run.artifacts appendix.moment_readings"),
                          ("D5-verdicts", "tt.verdicts.bundle_version")):
        if label not in res:
            continue
        measured_d5 = True
        rows = sorted((r["bundle"], r["n"]) for r in res[label])
        parts = [f"{b}={c}" for b, c in rows]
        w(f"D5 pre-existing {source} by cited bundle: " + (" ".join(parts) if parts else "none"))
        other_total += sum(c for b, c in rows if b not in (taxonomy_vs, UNSTAMPED))
        unstamped_total += sum(c for b, c in rows if b == UNSTAMPED)
    if measured_d5:
        w(f"D5 pre-existing readings citing a taxonomy version other than the loaded one: "
          f"{other_total} → STORE (this release does not change which version they cite)")
        w(f"D5 pre-existing readings with no bundle stamp (declared, never back-filled): "
          f"{unstamped_total} → STORE")
    else:
        w("D5 readings: not measured (run.moments, run.artifacts and tt.verdicts absent)")
    w("rows that would change on --apply: 0")
    if apply:
        w("--apply: every verdict is Store; nothing was applied and nothing was written")
    w("")
    w("QUERIES (read-only, one REPEATABLE READ snapshot, each labeled with the figure it produced;")
    w("each is wrapped in jsonb_agg so psql returns one JSON document)")
    for label, _, sql in figs:
        w(f"-- {label}")
        w(sql + ";")
    return "\n".join(lines) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=(__doc__ or "").split("\n")[0])
    ap.add_argument("--db", required=True,
                    help="libpq connection string, passed to psql and never printed; "
                         "prefer service=NAME or ~/.pgpass to a password in it")
    ap.add_argument("--deployment", required=True, help="the name printed in the report")
    ap.add_argument("--apply", action="store_true",
                    help="accepted for symmetry with ttmigrate; every verdict is Store, so it writes nothing")
    args = ap.parse_args(argv)
    taxonomy_vs, relations_vs, kinds = load_artifacts()
    try:
        present, figs, res = measure(args.db, kinds)
    except Refused as e:
        print(f"refusing to report: {e}", file=sys.stderr)
        return EXIT_REFUSED
    sys.stdout.write(report(args.deployment, taxonomy_vs, relations_vs, kinds, present, figs, res,
                            args.apply))
    return 0


if __name__ == "__main__":
    sys.exit(main())

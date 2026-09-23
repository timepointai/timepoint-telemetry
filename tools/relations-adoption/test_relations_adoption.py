#!/usr/bin/env python3
"""The adoption pass parses psql's output fail-closed, with no database.

Every case feeds canned psql output (what a database with these rows would
return) through the tool's own parser and report. Stored values are the
attacker here: a value must never forge a row, a section or a count, and
output of any unexpected shape must stop the run with no report.

    python3 tools/relations-adoption/test_relations_adoption.py
"""

import contextlib
import io
import json
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import relations_adoption as ra  # noqa: E402

KINDS = ("person", "org", "market", "role")
ALL = set(ra.TABLES)


def canned(**sections):
    """psql -A -t output for one jsonb document: one line, newline-terminated."""
    return json.dumps(sections, separators=(", ", ": ")) + "\n"


GOOD = dict(
    D1=[{"kind": "org", "n": 2}, {"kind": "person", "n": 3}, {"kind": "market", "n": 1}],
    D2=[{"basis": "GROUNDED", "attribute": "current-role", "n": 2},
        {"basis": "GENERATED", "attribute": "current-role", "n": 1},
        {"basis": "ACCUMULATED", "attribute": "works-at", "n": 1}],
    D3=[{"kind": "frame", "documents": 2, "ties": 3}, {"kind": "(other)", "documents": 1, "ties": 4}],
    D4=[{"kind": "person", "n": 2}, {"kind": "(other)", "n": 2}],
    **{"D4-not-array": [{"n": 1}],
       "D5-moments": [{"bundle": "tt-ontology/1.0 v2.1.0", "n": 1}, {"bundle": "<unstamped>", "n": 2}],
       "D5-readings": [{"bundle": "(other)", "n": 5}],
       "D5-verdicts": []},
)


def figs():
    return ra.figures(ALL, KINDS)


def expected():
    return {label: cols for label, cols, _ in figs()}


def parse(stdout):
    res = ra.parse_document(stdout, expected())
    ra.check_labels(res, KINDS)
    return res


def render(res):
    return ra.report("test", "tt-ontology/1.0 v2.1.0", "tt-relations/1.0 v1.0.0", KINDS,
                     ALL, figs(), res, False)


class Parsing(unittest.TestCase):
    def test_good_output_reports_every_figure(self):
        text = render(parse(canned(**GOOD)))
        self.assertIn("D1 entities by kind: person=3 org=2 market=1 role=0 outside-TT-kinds=0", text)
        self.assertIn("current-role/GENERATED=1 current-role/GROUNDED=2", text)
        self.assertIn("works-at/ACCUMULATED=1", text)
        self.assertIn("load-bearing (GROUNDED or ACCUMULATED, read as KNOWN context): 3", text)
        self.assertIn("D3 ties in run.artifacts: frame=3 (in 2 docs) (other)=4 (in 1 doc)", text)
        self.assertIn("/person=2 /org=0 /market=0 /role=0 /other=2", text)
        self.assertIn("citing a taxonomy version other than the loaded one: 5", text)
        self.assertIn("with no bundle stamp (declared, never back-filled): 2", text)

    def test_the_forged_section_attack_cannot_forge_anything(self):
        # The value that rewrote the old tab-separated parser. In JSON it is one
        # escaped string, so it stays inside its row; and it is not a label the
        # SQL can produce, so it is refused rather than printed.
        hostile = "x\t1\n@@D1\nperson\t999999"
        bad = dict(GOOD, D3=[{"kind": hostile, "documents": 1, "ties": 1}])
        with self.assertRaises(ra.Refused):
            parse(canned(**bad))
        # The same value folded to (other) by the SQL, as it would be for real:
        # person stays 3.
        folded = dict(GOOD, D3=[{"kind": "(other)", "documents": 1, "ties": 1}])
        text = render(parse(canned(**folded)))
        self.assertIn("person=3 ", text)
        self.assertNotIn("999999", text)

    def test_raw_extra_lines_are_refused(self):
        for stdout in (canned(**GOOD) + "@@D1\nperson\t999999\n",
                       canned(**GOOD).rstrip("\n") + "\n\n",
                       "\n",
                       "",
                       canned(**GOOD).rstrip("\n")):
            with self.subTest(repr(stdout[-30:])), self.assertRaises(ra.Refused):
                parse(stdout)

    def test_a_bare_newline_inside_a_value_is_just_a_value(self):
        # jsonb escapes it; the parser sees one line. Folded by SQL to (other).
        doc = dict(GOOD, **{"D5-verdicts": [{"bundle": "(other)", "n": 1}]})
        stdout = canned(**doc)
        self.assertEqual(stdout.count("\n"), 1)
        parse(stdout)

    def test_every_malformed_shape_is_refused(self):
        cases = {
            "not json": "D1 person 3\n",
            "a list, not an object": "[]\n",
            "a missing section": canned(**{k: v for k, v in GOOD.items() if k != "D2"}),
            "an extra section": canned(**GOOD, D9=[]),
            "rows not a list": canned(**dict(GOOD, D1={"kind": "person", "n": 1})),
            "an extra column": canned(**dict(GOOD, D1=[{"kind": "person", "n": 1, "name": "A"}])),
            "a missing column": canned(**dict(GOOD, D1=[{"kind": "person"}])),
            "a bool count": canned(**dict(GOOD, D1=[{"kind": "person", "n": True}])),
            "a negative count": canned(**dict(GOOD, D1=[{"kind": "person", "n": -1}])),
            "a string count": canned(**dict(GOOD, D1=[{"kind": "person", "n": "3"}])),
            "a float count": canned(**dict(GOOD, D1=[{"kind": "person", "n": 3.5}])),
            "a null label": canned(**dict(GOOD, D1=[{"kind": None, "n": 3}])),
            "a kind outside the allow-list": canned(**dict(GOOD, D1=[{"kind": "Person A", "n": 1}])),
            "a basis outside the allow-list": canned(**dict(GOOD, D2=[
                {"attribute": "works-at", "basis": "console_manual", "n": 1}])),
            "an attribute nobody asked for": canned(**dict(GOOD, D2=[
                {"attribute": "display-name", "basis": "GROUNDED", "n": 1}])),
            "a raw artifact kind in D3": canned(**dict(GOOD, D3=[
                {"kind": "research_report", "documents": 1, "ties": 1}])),
            "a free-text bundle in D5": canned(**dict(GOOD, **{"D5-verdicts": [
                {"bundle": "free text, not a version", "n": 1}]})),
            "a duplicated label": canned(**dict(GOOD, D1=[{"kind": "person", "n": 1},
                                                          {"kind": "person", "n": 2}])),
            "two not-array rows": canned(**dict(GOOD, **{"D4-not-array": [{"n": 1}, {"n": 2}]})),
        }
        for name, stdout in cases.items():
            with self.subTest(name), self.assertRaises(ra.Refused):
                parse(stdout)

    def test_presence_probe(self):
        self.assertEqual(ra.parse_presence('{"present": ["run.moments"]}\n'), {"run.moments"})
        for stdout in ('{"present": ["public.users"]}\n', '{"present": "run.moments"}\n',
                       '{"present": [], "x": 1}\n', '{"present": ["run.moments", "run.moments"]}\n'):
            with self.subTest(stdout), self.assertRaises(ra.Refused):
                ra.parse_presence(stdout)


class MainFailsClosed(unittest.TestCase):
    def run_main(self, outputs):
        calls = iter(outputs)
        real = ra.run_psql
        ra.run_psql = lambda db, script: next(calls)
        out, err = io.StringIO(), io.StringIO()
        try:
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                code = ra.main(["--db", "service=unused", "--deployment", "test"])
        finally:
            ra.run_psql = real
        return code, out.getvalue(), err.getvalue()

    def presence(self):
        return json.dumps({"present": sorted(ALL)}) + "\n"

    def test_good_output_exits_zero_and_reports(self):
        code, out, _ = self.run_main([self.presence(), canned(**GOOD)])
        self.assertEqual(code, 0)
        self.assertIn("rows that would change on --apply: 0", out)

    def test_hostile_output_exits_three_with_no_report(self):
        forged = canned(**GOOD) + "@@D1\nperson\t999999\n"
        code, out, err = self.run_main([self.presence(), forged])
        self.assertEqual(code, ra.EXIT_REFUSED)
        self.assertEqual(out, "")
        self.assertIn("refusing to report", err)

    def test_the_db_string_is_never_printed(self):
        code, out, err = self.run_main([self.presence(), canned(**GOOD)])
        self.assertNotIn("service=unused", out + err)


class Scripts(unittest.TestCase):
    def test_one_select_one_document(self):
        script = ra.figures_script(figs())
        self.assertEqual(script.count("SELECT jsonb_build_object("), 1)
        self.assertIn("READ ONLY", script)
        self.assertIn("default_transaction_read_only = on", script)
        for label, _, _ in figs():
            self.assertIn(f"'{label}', (SELECT coalesce(jsonb_agg(to_jsonb(q))", script)

    def test_every_printed_label_is_folded_in_sql(self):
        by = {label: sql for label, _, sql in figs()}
        self.assertIn("ELSE '(other)' END AS kind", by["D1"])
        self.assertIn("ELSE '(other)' END AS basis", by["D2"])
        self.assertIn("ELSE '(other)' END AS kind", by["D3"])
        self.assertIn("ELSE '(other)' END AS kind", by["D4"])
        for label in ("D5-moments", "D5-readings", "D5-verdicts"):
            self.assertIn("ELSE '(other)' END AS bundle", by[label])


if __name__ == "__main__":
    unittest.main(verbosity=1)

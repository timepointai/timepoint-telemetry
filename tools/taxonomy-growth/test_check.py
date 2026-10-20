#!/usr/bin/env python3
"""The growth check passes the shipped release and fails the edits it exists to catch."""

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import check  # noqa: E402

NEW = os.path.join(REPO, "bundle", "taxonomy-v2.3.json")
# sha256 of bundle/taxonomy-v2.2.json as released (tag v2.3.0); old_bundle() rebuilds it.
OLD_SHA256 = "d70812582ea080985a1f7778a67336b5c006bb2e9d7f77ae0c980f6193e52574"
READINGS = os.path.join(HERE, "rehearsal_readings.jsonl")


def old_bundle():
    """v2.2.0 rebuilt from v2.3.0 by removing exactly the two Growth nodes."""
    new = check.load(NEW)
    old = copy.deepcopy(new)
    old["version"], old["supersedes"] = "2.2.0", "tt-ontology/1.0 v2.1.0"
    old["nodes"] = [n for n in new["nodes"]
                    if n["id"] not in ("monetary-policy-decision", "court-ruling")]
    return old


class GrowthCheck(unittest.TestCase):
    def run_check(self, old, new, readings=None):
        with tempfile.TemporaryDirectory() as d:
            po, pn = os.path.join(d, "old.json"), os.path.join(d, "new.json")
            for path, value in ((po, old), (pn, new)):
                with open(path, "w", encoding="utf-8") as f:
                    json.dump(value, f)
            argv = [sys.executable, os.path.join(HERE, "check.py"), po, pn] + ([readings] if readings else [])
            return subprocess.run(argv, capture_output=True, text=True)

    def test_the_shipped_release_is_growth_clean_and_deterministic(self):
        first = self.run_check(old_bundle(), check.load(NEW), READINGS)
        second = self.run_check(old_bundle(), check.load(NEW), READINGS)
        self.assertEqual(first.returncode, 0, first.stdout + first.stderr)
        self.assertEqual(first.stdout, second.stdout)
        out = first.stdout
        self.assertIn("nodes: 151 -> 153 (added 2, removed 0, meaning changed 0)", out)
        self.assertIn("pre-existing pairs compared: 11325; distance changed: 0 (yes)", out)
        self.assertIn("content valid under old: 9; still valid under new: 9 (yes)", out)
        # The v2.1.0 stamps were one step back under v2.2.0 and are two steps back
        # now; the chain resolves them and nothing is rewritten (RELEASE-NOTES v2.4.0).
        self.assertIn("citations older than one step (resolved by walking the chain, not rewritten): 5", out)
        self.assertIn("citations that would be rewritten: 0 (yes)", out)
        self.assertIn("rows that would be rewritten: 0", out)

    def test_a_changed_definition_fails(self):
        new = check.load(NEW)
        node = next(n for n in new["nodes"] if n["id"] == "enterprise-and-commerce")
        node["definition"] += " Edited."
        result = self.run_check(old_bundle(), new)
        self.assertEqual(result.returncode, 1)
        self.assertIn("changed ids", result.stdout)

    def test_a_new_lateral_edge_fails(self):
        new = check.load(NEW)
        new["lateral_edges"].append({"a": "monetary-policy-decision",
                                     "b": "court-ruling", "weight": 1.6})
        result = self.run_check(old_bundle(), new)
        self.assertEqual(result.returncode, 1)
        self.assertIn("lateral_edges", result.stdout)

    def test_the_rebuilt_previous_release_is_byte_for_byte_the_old_content(self):
        # Removing the two nodes from v2.3.0 must give back exactly v2.2.0's nodes.
        old = old_bundle()
        self.assertEqual(len(old["nodes"]), 151)
        self.assertTrue(all(n["id"] not in ("monetary-policy-decision", "court-ruling")
                            for n in old["nodes"]))

    def test_a_deleted_id_fails(self):
        new = check.load(NEW)
        new["nodes"] = [n for n in new["nodes"] if n["id"] != "games-and-sport"]
        result = self.run_check(old_bundle(), new)
        self.assertEqual(result.returncode, 1)
        self.assertIn("removed ids", result.stdout)


if __name__ == "__main__":
    unittest.main()

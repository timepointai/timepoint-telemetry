#!/usr/bin/env python3
"""The reference validator reproduces every classification vector — both tiers.

Normative tier: accepted flag, normalized form on accept, multiset of
rejection codes on reject. Advisory tier: detail strings byte for byte.
The reference implementation must pass both; a port conforms on the
normative tier alone (vectors/classification-verdicts.json, `conformance`).
"""

import json
import os
import unittest

from tt_validate import load_bundle, validate

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
VECTORS = os.path.join(REPO, "vectors", "classification-verdicts.json")
BUNDLE = os.path.join(REPO, "bundle", "taxonomy-v2.1.json")


class ClassificationVectors(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with open(VECTORS, encoding="utf-8") as f:
            cls.doc = json.load(f)
        cls.bundle = load_bundle(BUNDLE)

    def test_bundle_matches(self):
        self.assertEqual(self.doc["bundle"], self.bundle["version_string"])

    def test_every_vector_both_tiers(self):
        for v in self.doc["vectors"]:
            with self.subTest(v["name"]):
                normalized, errors = validate(v["input"], self.bundle)
                expect = v["expect"]
                self.assertEqual(not errors, expect["accepted"], v["name"])
                if expect["accepted"]:
                    self.assertEqual(normalized, expect["normalized"])
                else:
                    # normative: the multiset of codes
                    self.assertEqual(
                        sorted(e["code"] for e in errors),
                        sorted(e["code"] for e in expect["rejections"]),
                    )
                    # advisory tier, which the reference itself must pass
                    self.assertEqual(errors, expect["rejections"])

    def test_round_trip_of_every_accepted_vector(self):
        # §4.5: what the validator emits, it accepts.
        for v in self.doc["vectors"]:
            if not v["expect"]["accepted"]:
                continue
            with self.subTest(v["name"]):
                again, errors = validate(v["expect"]["normalized"], self.bundle)
                self.assertEqual(errors, [])
                self.assertEqual(again, v["expect"]["normalized"])


if __name__ == "__main__":
    unittest.main(verbosity=2)

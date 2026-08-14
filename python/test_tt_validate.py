#!/usr/bin/env python3
"""Tests for tt_validate against the shipped bundle — stdlib only.

Run from the repo root or this directory:
    python3 python/test_tt_validate.py
"""

import copy
import pathlib
import unittest

import tt_validate

BUNDLE_PATH = pathlib.Path(__file__).resolve().parent.parent / "bundle" / "taxonomy-v2.1.json"
BUNDLE = tt_validate.load_bundle(str(BUNDLE_PATH))

VIENNA = {
    "lens_b": {"negotiation-and-agreement": 0.7, "deciding-and-judging": 0.2},
    "lens_a": {"treaty-alliance-and-peace-accord": 0.85},
    "abstain": False,
}


def codes(errors):
    return sorted(e["code"] for e in errors)


class Accepts(unittest.TestCase):
    def test_worked_moment_classification(self):
        normalized, errors = tt_validate.validate(VIENNA, BUNDLE)
        self.assertEqual(errors, [])
        self.assertEqual(normalized["bundle"], "tt-ontology/1.0 v2.1.0")

    def test_abstention_is_publishable(self):
        normalized, errors = tt_validate.validate({"abstain": True}, BUNDLE)
        self.assertEqual(errors, [])
        self.assertEqual(normalized["lens_a"], {})
        self.assertEqual(normalized["lens_b"], {})

    def test_mass_at_branch_is_legal(self):
        c = {"lens_b": {"bonding-and-kinship": 0.6}}
        normalized, errors = tt_validate.validate(c, BUNDLE)
        self.assertEqual(errors, [])

    def test_missing_lens_keys_mean_empty_maps(self):
        normalized, errors = tt_validate.validate({"lens_a": {"pitched-battle": 0.5}}, BUNDLE)
        self.assertEqual(errors, [])
        self.assertEqual(normalized["lens_b"], {})

    def test_round_trip_what_it_emits_it_accepts(self):
        emitted, errors = tt_validate.validate(VIENNA, BUNDLE)
        self.assertEqual(errors, [])
        again, errors = tt_validate.validate(emitted, BUNDLE)
        self.assertEqual(errors, [])
        self.assertEqual(emitted, again)

    def test_matching_bundle_citation_accepted(self):
        c = dict(VIENNA, bundle="tt-ontology/1.0 v2.1.0")
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertEqual(errors, [])


class Rejects(unittest.TestCase):
    def test_four_entries_in_a_lens(self):
        c = {"lens_b": {"negotiation-and-agreement": 0.2, "deciding-and-judging": 0.2,
                        "trade-and-barter": 0.2, "gift-and-reciprocity": 0.2}}
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertIn("too-many-entries", codes(errors))

    def test_zero_negative_and_over_one_masses(self):
        for bad in (0.0, -0.1, 1.5):
            c = {"lens_b": {"negotiation-and-agreement": bad}}
            _, errors = tt_validate.validate(c, BUNDLE)
            self.assertIn("mass-out-of-range", codes(errors), msg=f"mass {bad}")

    def test_lens_sum_over_one(self):
        c = {"lens_b": {"negotiation-and-agreement": 0.7, "deciding-and-judging": 0.7}}
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertIn("lens-sum-exceeded", codes(errors))

    def test_sum_epsilon_is_honored(self):
        c = {"lens_b": {"negotiation-and-agreement": 0.7, "deciding-and-judging": 0.3}}
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertEqual(errors, [], "sum of exactly 1.0 must not reject on float noise")

    def test_unknown_id_never_existed(self):
        c = {"lens_a": {"battle-and-campaign": 0.5}}
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertEqual(codes(errors), ["unknown-id"])

    def test_retired_id_names_its_successor(self):
        c = {"lens_b": {"everyday-movement-and-commute": 0.5}}
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertEqual(codes(errors), ["retired-id"])
        self.assertIn("journey-and-travel", errors[0]["detail"])

    def test_retired_and_unknown_are_different_rejections(self):
        c = {"lens_b": {"everyday-movement-and-commute": 0.3,
                        "horse-trading": 0.3}}
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertEqual(codes(errors), ["retired-id", "unknown-id"])

    def test_right_id_wrong_lens(self):
        c = {"lens_b": {"accession-and-coronation": 0.5}}  # a real Lens A node
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertEqual(codes(errors), ["wrong-lens"])

    def test_unknown_top_level_key(self):
        c = dict(VIENNA, confidence=0.9)
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertIn("unknown-key", codes(errors))

    def test_abstain_with_mass(self):
        c = copy.deepcopy(VIENNA)
        c["abstain"] = True
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertIn("abstain-with-mass", codes(errors))

    def test_bundle_mismatch(self):
        c = dict(VIENNA, bundle="tt-ontology/1.0 v2.0.0")
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertIn("bundle-mismatch", codes(errors))

    def test_boolean_mass_is_not_a_number(self):
        c = {"lens_b": {"negotiation-and-agreement": True}}
        _, errors = tt_validate.validate(c, BUNDLE)
        self.assertIn("mass-not-number", codes(errors))

    def test_rejected_whole_never_partially(self):
        c = {"lens_b": {"negotiation-and-agreement": 0.5, "horse-trading": 0.2}}
        normalized, errors = tt_validate.validate(c, BUNDLE)
        self.assertIsNone(normalized, "a rejected classification must not be partially accepted")


if __name__ == "__main__":
    unittest.main(verbosity=2)

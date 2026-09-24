#!/usr/bin/env python3
"""Run every committed conformance vector against tt_envelope, byte-for-byte.

The vectors are normative (TT-SPEC §6): an implementation conforms when it
reproduces all of them exactly. This is the proof this module ships under.

    python3 python/test_tt_envelope.py
"""

import json
import pathlib
import unittest

import tt_envelope

VECTORS_DIR = pathlib.Path(__file__).resolve().parent.parent / "vectors"


class Vectors(unittest.TestCase):
    pass


def _make_test(path):
    def test(self):
        vector = json.loads(path.read_text(encoding="utf-8"))
        if "payload" in vector["input"]:
            canonical = tt_envelope.content_canonical(vector["input"]["payload"])
            digest = tt_envelope.content_hash(vector["input"]["payload"])
        else:
            value = vector["input"]["provenance"]
            canonical = tt_envelope.canonicalize(value)
            digest = tt_envelope.provenance_hash(value)
        self.assertEqual(canonical, vector["expected_canonical"], "canonical bytes differ")
        self.assertEqual(digest, vector["expected_hash"], "hash differs")
    return test


# Top-level vectors/*.json are envelope vectors and nothing else: consumers
# glob that directory and read every file as one (TopLevelLayout below). The
# verdict corpora live in vectors/verdicts/ with their own walkers.
_paths = sorted(VECTORS_DIR.glob("*.json"))
assert len(_paths) == 10, f"expected 10 envelope vectors, found {len(_paths)}"
for _p in _paths:
    setattr(Vectors, f"test_{_p.stem.replace('-', '_')}", _make_test(_p))


class TopLevelLayout(unittest.TestCase):
    """Top-level vectors/*.json are a published interface, not just our files.

    Consumers glob that directory and read every file as an envelope vector:
    timepoint-beta's conformance/python/check_vectors.py reads v["name"],
    v["input"]["payload" | "provenance"], v["expected_canonical"] and
    v["expected_hash"] from each. Any other shape at this level crashes them,
    so other corpora live in subdirectories (vectors/verdicts/).
    """

    def test_every_top_level_file_is_an_envelope_vector(self):
        self.assertEqual(len(_paths), 10)
        for path in _paths:
            with self.subTest(path.name):
                v = json.loads(path.read_text(encoding="utf-8"))
                self.assertIsInstance(v, dict)
                self.assertIsInstance(v.get("name"), str)
                self.assertIsInstance(v.get("input"), dict)
                self.assertEqual(len(set(v["input"]) & {"payload", "provenance"}), 1,
                                 "input holds exactly one of payload | provenance")
                self.assertIsInstance(v.get("expected_canonical"), str)
                self.assertIsInstance(v.get("expected_hash"), str)
                self.assertTrue(v["expected_hash"].startswith("sha256:"))

    def test_verdict_corpora_are_below_the_top_level(self):
        verdicts = VECTORS_DIR / "verdicts"
        self.assertEqual(sorted(p.name for p in verdicts.glob("*.json")),
                         ["classification-verdicts.json", "relation-verdicts.json"])


class TypedErrors(unittest.TestCase):
    def test_missing_claim_field_is_typed(self):
        with self.assertRaises(tt_envelope.MissingPayloadField) as ctx:
            tt_envelope.content_hash({"label": "x", "participants": []})
        self.assertEqual(ctx.exception.field, "occurs_at")

    def test_payload_not_object(self):
        with self.assertRaises(tt_envelope.PayloadNotObject):
            tt_envelope.content_hash(["not", "an", "object"])

    def test_null_claim_field_is_representable_absence(self):
        canonical = tt_envelope.content_canonical(
            {"label": "x", "occurs_at": None, "participants": []})
        self.assertIn('"occurs_at":null', canonical)


if __name__ == "__main__":
    unittest.main(verbosity=1)

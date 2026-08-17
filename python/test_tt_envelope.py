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


# Envelope vectors only — classification-verdicts.json is the §4 corpus with
# its own shape and its own test (test_classification_vectors.py).
_paths = sorted(p for p in VECTORS_DIR.glob("*.json")
                if p.name != "classification-verdicts.json")
assert len(_paths) == 10, f"expected 10 envelope vectors, found {len(_paths)}"
for _p in _paths:
    setattr(Vectors, f"test_{_p.stem.replace('-', '_')}", _make_test(_p))


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

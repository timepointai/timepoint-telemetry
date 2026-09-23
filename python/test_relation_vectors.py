#!/usr/bin/env python3
"""The reference validator reproduces every relation vector, on both tiers.

Normative tier: accepted flag, normalized form on accept, and the multiset of
rejection codes on reject; for load cases, whether the patched artifact loads
and the multiset of failure rules. Advisory tier: detail strings byte for
byte, in order, except a `malformed` load failure's wording. The reference
implementation passes both; a port conforms on the normative tier alone
(vectors/verdicts/relation-verdicts.json, `conformance`).

    python3 python/test_relation_vectors.py
"""

import hashlib
import json
import os
import unittest

from gen_relation_vectors import apply_patch
from tt_relations import VocabularyInvalid, check_vocabulary, load_vocabulary, validate_edge

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
VECTORS = os.path.join(REPO, "vectors", "verdicts", "relation-verdicts.json")
ARTIFACT = os.path.join(REPO, "bundle", "relations-v1.0.json")
RELATIONS_SHA256 = "23b0dc5627301d19f128d96152a84fed87000954e1d9fff571f8658ee81b73bc"


def codes(items, key):
    return sorted(i[key] for i in items)


class RelationVectors(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with open(VECTORS, encoding="utf-8") as f:
            cls.doc = json.load(f)
        with open(ARTIFACT, "rb") as f:
            cls.bytes = f.read()
        cls.raw = json.loads(cls.bytes.decode("utf-8"))
        cls.vocab = load_vocabulary(ARTIFACT)

    def test_artifact_is_the_pinned_one(self):
        digest = hashlib.sha256(self.bytes).hexdigest()
        self.assertEqual(digest, RELATIONS_SHA256)
        self.assertEqual(self.doc["vocabulary_sha256"], RELATIONS_SHA256)
        self.assertEqual(self.doc["vocabulary"], self.vocab["version_string"])
        self.assertEqual(self.vocab["version_string"], "tt-relations/1.0 v1.0.0")
        self.assertEqual(len(self.vocab["entity_kinds"]), 4)
        self.assertEqual(len(self.vocab["relation_kinds"]), 10)

    def walk(self, vectors, vocab):
        for v in vectors:
            with self.subTest(v["name"]):
                normalized, errors = validate_edge(v["input"], vocab)
                expect = v["expect"]
                self.assertEqual(not errors, expect["accepted"], errors)
                if expect["accepted"]:
                    self.assertEqual(normalized, expect["normalized"])
                    again, errors = validate_edge(normalized, vocab)
                    self.assertEqual(errors, [])
                    self.assertEqual(again, normalized)
                else:
                    self.assertEqual(codes(errors, "code"), codes(expect["rejections"], "code"))
                    self.assertEqual(errors, expect["rejections"])

    def test_every_edge_vector_both_tiers(self):
        names = [v["name"] for v in self.doc["edges"]]
        for i in range(1, 29):
            self.assertTrue(any(n.startswith(f"cr-{i:02d} ") for n in names),
                            f"change request case {i} is present")
        self.walk(self.doc["edges"], self.vocab)

    def test_every_fixture_edge_vector_both_tiers(self):
        fixture, failures = check_vocabulary(apply_patch(self.raw, self.doc["fixture"]["patch"]))
        self.assertEqual(failures, [])
        self.walk(self.doc["fixture_edges"], fixture)

    def test_every_load_vector(self):
        for v in self.doc["loads"]:
            with self.subTest(v["name"]):
                _, failures = check_vocabulary(apply_patch(self.raw, v["patch"]))
                expect = v["expect"]
                self.assertEqual(not failures, expect["loads"], failures)
                self.assertEqual(codes(failures, "rule"), codes(expect["failures"], "rule"))
                if not any(f["rule"] == "malformed" for f in failures):
                    self.assertEqual(failures, expect["failures"])

    def test_invalid_artifact_raises(self):
        broken = apply_patch(self.raw, [{"op": "replace", "path": "/relation_kinds/6/nature",
                                         "value": "personal"}])
        _, failures = check_vocabulary(broken)
        with self.assertRaises(VocabularyInvalid) as ctx:
            raise VocabularyInvalid(failures)
        self.assertTrue(str(ctx.exception).startswith("bad-nature: "))

    def test_the_corpus_is_what_the_generator_writes(self):
        # The file is generated, never hand-edited: regenerating reproduces it.
        import subprocess
        import sys
        out = subprocess.run(
            [sys.executable, os.path.join(HERE, "gen_relation_vectors.py"), ARTIFACT],
            check=True, capture_output=True, cwd=HERE).stdout
        with open(VECTORS, "rb") as f:
            self.assertEqual(out, f.read())


class PythonOnlyInputs(unittest.TestCase):
    """Inputs Rust never sees, because its JSON parser refuses them first."""

    @classmethod
    def setUpClass(cls):
        cls.vocab = load_vocabulary(ARTIFACT)

    def test_a_lone_surrogate_in_text_is_an_attribute_type_rejection(self):
        for text in ("\ud800", "bond \udfff measure", "\udc00" * 3):
            with self.subTest(repr(text)):
                statement = json.loads(json.dumps(
                    {"relation": "vendor-to", "from_kind": "org", "to_kind": "org",
                     "attributes": {"subject": text}}))
                normalized, errors = validate_edge(statement, self.vocab)
                self.assertIsNone(normalized)
                self.assertEqual(errors, [{"code": "attribute-type",
                                           "detail": "`subject`: text must not contain unpaired surrogates"}])

    def test_a_surrogate_pair_is_one_scalar_and_accepts(self):
        statement = json.loads('{"relation": "vendor-to", "from_kind": "org", "to_kind": "org",'
                               ' "attributes": {"subject": "platform \\ud83d\\ude42"}}')
        normalized, errors = validate_edge(statement, self.vocab)
        self.assertEqual(errors, [])
        self.assertEqual(normalized["attributes"]["subject"], "platform \U0001F642")


if __name__ == "__main__":
    unittest.main(verbosity=1)

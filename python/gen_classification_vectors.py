#!/usr/bin/env python3
"""Regenerate vectors/classification-verdicts.json from the reference validator.

Every expected value in the vector file is computed by running tt_validate
against the shipped bundle — never written by hand. To regenerate after a
spec Correction or a bundle release:

    python3 python/gen_classification_vectors.py bundle/taxonomy-v2.1.json \
        > vectors/classification-verdicts.json

The case corpus began life as the Clockchain's differential harness
(ops/tt-differential.py in their repo), which ported tt_validate rule for
rule and then diffed the two implementations verbatim. That diff surfaced
seven rendering divergences a human review had passed — including two whole
classes (integer literals, exponent spellings) invisible until the suite
stopped using only float literals. The numeric cases below exist because a
conformance corpus naturally omits them.

Inputs stay within IEEE-754 double range. Integer literals wider than a
double's exact range are deliberately absent: how a JSON parser treats them
is parser territory, not §4 contract, and a vector asserting Python's
arbitrary-precision rendering would make every non-Python implementation
non-conformant on input no sane producer emits.
"""

import json
import sys

from tt_validate import load_bundle, validate

VERSION_STRING = "tt-ontology/1.0 v2.1.0"

CASES = [
    ("abstention", {"lens_a": {}, "lens_b": {}, "abstain": True, "bundle": VERSION_STRING}),
    ("abstention, bundle unsent", {"lens_a": {}, "lens_b": {}, "abstain": True}),
    ("single A type at mass 1", {"lens_a": {"conflict-and-warfare": 1.0}, "lens_b": {}}),
    ("single B type at mass 1", {"lens_a": {}, "lens_b": {"bonding-and-kinship": 1.0}}),
    ("split mass, both lenses", {"lens_a": {"governance-and-popular-politics": 0.6},
                                 "lens_b": {"reconciliation-and-forgiveness": 0.4}}),
    ("ancestor+descendant (TT permits)", {"lens_a": {"politics-governance-and-law": 0.3,
                                                     "rulership-and-succession": 0.7}}),
    ("three entries, at the cap", {"lens_a": {"conflict-and-warfare": 0.3,
                                              "religious-life": 0.3, "siege-and-sack": 0.4}}),
    ("abstain with mass in A", {"lens_a": {"conflict-and-warfare": 1.0}, "abstain": True}),
    ("abstain with mass in B", {"lens_b": {"bonding-and-kinship": 0.5}, "abstain": True}),
    ("abstain with mass in both", {"lens_a": {"conflict-and-warfare": 0.5},
                                   "lens_b": {"bonding-and-kinship": 0.5}, "abstain": True}),
    ("unknown id", {"lens_a": {"not-a-real-node": 1.0}}),
    ("typo'd id", {"lens_b": {"courtship-and-fallng-in-love": 1.0}}),
    ("retired id", {"lens_b": {"everyday-movement-and-commute": 1.0}}),
    ("wrong lens", {"lens_b": {"conflict-and-warfare": 1.0}}),
    ("mass zero", {"lens_a": {"conflict-and-warfare": 0.0}}),
    ("mass above one", {"lens_a": {"conflict-and-warfare": 1.5}}),
    ("mass is a bool", {"lens_a": {"conflict-and-warfare": True}}),
    ("mass is a string", {"lens_a": {"conflict-and-warfare": "1.0"}}),
    ("lens sum exceeded", {"lens_a": {"conflict-and-warfare": 0.6,
                                      "religious-life": 0.6}}),
    ("four entries in one lens", {"lens_a": {"conflict-and-warfare": 0.1,
                                             "religious-life": 0.1,
                                             "siege-and-sack": 0.1,
                                             "economy-trade-and-labor": 0.1}}),
    ("unknown top-level key", {"lens_a": {}, "lens_b": {}, "surprise": 1}),
    ("abstain not a bool", {"lens_a": {}, "lens_b": {}, "abstain": "yes"}),
    ("lens not an object", {"lens_a": [], "lens_b": {}}),
    ("bundle names another release", {"lens_a": {}, "lens_b": {}, "abstain": True,
                                      "bundle": "tt-ontology/1.0 v2.0.0"}),
    ("not an object", [1, 2, 3]),
    ("several failures at once", {"lens_a": {"not-a-node": 5.0}, "abstain": True, "surprise": 1}),
    ("integer mass, out of range", {"lens_a": {"conflict-and-warfare": 5}}),
    ("integer zero mass", {"lens_a": {"conflict-and-warfare": 0}}),
    ("float zero mass", {"lens_a": {"conflict-and-warfare": 0.0}}),
    ("exponent form, large", {"lens_a": {"conflict-and-warfare": 1e20}}),
    ("just below Python's sci threshold", {"lens_a": {"conflict-and-warfare": 1e15}}),
    ("at Python's sci threshold", {"lens_a": {"conflict-and-warfare": 1e16}}),
    ("very large float", {"lens_a": {"conflict-and-warfare": 1.5e300}}),
    ("float needing full precision", {"lens_a": {"conflict-and-warfare": 1.0000000000000002}}),
    ("negative mass", {"lens_a": {"conflict-and-warfare": -0.5}}),
    ("sum in float-error territory", {"lens_a": {"conflict-and-warfare": 0.1,
                                                 "religious-life": 0.2}}),
    ("null mass", {"lens_a": {"conflict-and-warfare": None}}),
    ("nested object as mass", {"lens_a": {"conflict-and-warfare": {"a": 1}}}),
    ("array as mass", {"lens_a": {"conflict-and-warfare": [1, 2]}}),
]


def main(argv):
    if len(argv) != 2:
        print(__doc__.strip() if __doc__ else "", file=sys.stderr)
        return 2
    bundle = load_bundle(argv[1])
    vectors = []
    for name, payload in CASES:
        normalized, errors = validate(payload, bundle)
        expect: dict = {"accepted": not errors}
        if errors:
            expect["rejections"] = errors
        else:
            expect["normalized"] = normalized
        vectors.append({"name": name, "input": payload, "expect": expect})
    doc = {
        "what": "TT classification conformance vectors — TT-SPEC §4 verdicts",
        "bundle": bundle["version_string"],
        "conformance": {
            "normative": "accepted, normalized (on accept), and the multiset of "
                         "rejection codes (on reject). An implementation matching "
                         "these in any language conforms.",
            "advisory": "rejection detail strings, byte for byte. The reference "
                        "implementation passes this tier; ports may opt in. The "
                        "details render numbers as Python's repr() does — that is "
                        "reference-implementation habit made visible, not design.",
        },
        "regenerate": "python3 python/gen_classification_vectors.py bundle/taxonomy-v2.1.json",
        "vectors": vectors,
    }
    json.dump(doc, sys.stdout, indent=2)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

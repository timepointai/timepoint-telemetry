#!/usr/bin/env python3
"""tt_validate — the TT classification validator, stdlib only.

The Python on-ramp for the classification contract (TT-SPEC §4). Validates a
classification against a loaded bundle: reject, never repair. What this file
emits, this file accepts (§4.5 round-trip).

Not here: envelope hashing (content_hash/provenance_hash need RFC 8785
canonicalisation — see the conformance vectors) and the distance metric. The
Rust crate remains the reference implementation; where this file and the
vectors disagree, the vectors win.

Usage:
    python3 tt_validate.py <bundle.json> <classification.json>
    python3 tt_validate.py <bundle.json> -   # classification on stdin

Exit 0 and the normalized classification (bundle string stamped) on stdout, or
exit 1 with one typed rejection per line on stderr. Rejections are typed
because a caller must handle them differently: a retired id names its
successor; an unknown id never existed (§4.2).

Two behaviours TT-SPEC §4 does not pin down, resolved conservatively here and
flagged for the spec:
  * `abstain: true` alongside non-empty lens masses rejects (`abstain-with-mass`)
    — §4.4 blesses abstention "with empty lenses" and says nothing else.
  * a `bundle` citation naming a different release than the loaded bundle
    rejects (`bundle-mismatch`) — §4.5 requires the citation to mean something.
"""

import json
import sys

SUM_EPSILON = 1e-9  # TT-SPEC §4.1 rule 3
MAX_ENTRIES_PER_LENS = 3
ALLOWED_KEYS = {"lens_a", "lens_b", "abstain", "bundle"}
LENS_KEYS = {"lens_a": "A", "lens_b": "B"}


def load_bundle(path):
    """Load a bundle file into the index this validator needs.

    Trusts the bundle's internal consistency — full structural validation
    (counts, parent chains, bridge targets) is the loader's job in tt-core.
    """
    with open(path, encoding="utf-8") as f:
        raw = json.load(f)
    nodes = {}
    for n in raw["nodes"]:
        nodes[n["id"]] = {
            "lens": n["lens"],
            "deprecated_in": n.get("deprecated_in"),
            "superseded_by": n.get("superseded_by"),
        }
    return {
        "version_string": f"{raw['schema']} v{raw['version']}",
        "nodes": nodes,
    }


def validate(classification, bundle):
    """Validate one classification. Returns (normalized, errors).

    Exactly one of the pair is meaningful: errors == [] means normalized is
    the accepted, bundle-stamped form; any errors mean the classification is
    thrown back whole (§4.2) and normalized is None.
    """
    errors = []

    def reject(code, detail):
        errors.append({"code": code, "detail": detail})

    if not isinstance(classification, dict):
        return None, [{"code": "not-an-object", "detail": "classification must be a JSON object"}]

    for key in classification:
        if key not in ALLOWED_KEYS:
            reject("unknown-key", f"unknown top-level key `{key}`")

    abstain = classification.get("abstain", False)
    if not isinstance(abstain, bool):
        reject("abstain-not-bool", f"abstain must be true or false, got {abstain!r}")
        abstain = False

    cited = classification.get("bundle")
    if cited is not None and cited != bundle["version_string"]:
        reject("bundle-mismatch",
               f"classification cites `{cited}`, loaded bundle is `{bundle['version_string']}`")

    total_mass_entries = 0
    lenses = {}
    for key, lens_letter in LENS_KEYS.items():
        profile = classification.get(key, {})
        if not isinstance(profile, dict):
            reject("lens-not-object", f"{key} must be an object of id: mass")
            continue
        if len(profile) > MAX_ENTRIES_PER_LENS:
            reject("too-many-entries",
                   f"{key} has {len(profile)} entries; at most {MAX_ENTRIES_PER_LENS}")
        lens_sum = 0.0
        for node_id, mass in profile.items():
            if isinstance(mass, bool) or not isinstance(mass, (int, float)):
                reject("mass-not-number", f"{key}.{node_id}: mass {mass!r} is not a number")
                continue
            if not (0.0 < mass <= 1.0):
                reject("mass-out-of-range", f"{key}.{node_id}: mass {mass} outside (0, 1]")
            lens_sum += mass
            node = bundle["nodes"].get(node_id)
            if node is None:
                reject("unknown-id", f"{key}.{node_id}: no such id in the bundle")
            elif node["deprecated_in"] is not None:
                successor = node["superseded_by"] or node_id
                reject("retired-id",
                       f"{key}.{node_id}: retired in {node['deprecated_in']}; use `{successor}`")
            elif node["lens"] != lens_letter:
                reject("wrong-lens",
                       f"{key}.{node_id}: id is lens {node['lens']}, offered under lens {lens_letter}")
        if lens_sum > 1.0 + SUM_EPSILON:
            reject("lens-sum-exceeded", f"{key} masses sum to {lens_sum}; at most 1.0")
        total_mass_entries += len(profile)
        lenses[key] = profile

    if abstain and total_mass_entries > 0:
        reject("abstain-with-mass",
               "abstain is true but lens masses are present; abstention is empty lenses (§4.4)")

    if errors:
        return None, errors

    normalized = {
        "lens_b": dict(lenses["lens_b"]),
        "lens_a": dict(lenses["lens_a"]),
        "abstain": abstain,
        "bundle": bundle["version_string"],  # stamped, whether or not it was sent (§4.5)
    }
    return normalized, []


def main(argv):
    if len(argv) != 3:
        print((__doc__ or "").strip(), file=sys.stderr)
        return 2
    bundle = load_bundle(argv[1])
    source = sys.stdin if argv[2] == "-" else open(argv[2], encoding="utf-8")
    with source:
        try:
            classification = json.load(source)
        except json.JSONDecodeError as e:
            print(f"rejected: not-json: {e}", file=sys.stderr)
            return 1
    normalized, errors = validate(classification, bundle)
    if errors:
        for e in errors:
            print(f"rejected: {e['code']}: {e['detail']}", file=sys.stderr)
        return 1
    json.dump(normalized, sys.stdout, indent=2)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

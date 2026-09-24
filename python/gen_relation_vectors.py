#!/usr/bin/env python3
"""Regenerate vectors/verdicts/relation-verdicts.json from the reference validator.

Every expected value in the vector file is computed by running tt_relations
against the shipped artifact, never written by hand:

    python3 python/gen_relation_vectors.py bundle/relations-v1.0.json \
        > vectors/verdicts/relation-verdicts.json

Three corpora, one file:

  * `edges`: edge statements against the shipped artifact. Cases 01-28 are
    the change request's own list (docs/proposals/ENTITY-RELATIONS.md §8), in
    its order and with its numbers; the rest pin what the list left open.
  * `fixture` + `fixture_edges`: retirement, which the shipped 1.0.0 artifact
    cannot show because nothing in it is retired. The fixture is the shipped
    artifact plus an RFC 6902 patch. It is a test document, not a release.
  * `loads`: load rules (§2.4). Each case is an RFC 6902 patch of the shipped
    artifact and the multiset of rules the loader must fail with. The patch
    subset is add, remove and replace, with JSON Pointer paths.
"""

import hashlib
import json
import sys

from tt_relations import check_vocabulary, validate_edge

VS = "tt-relations/1.0 v1.0.0"


def edge(relation, from_kind, to_kind, attributes=None, **extra):
    s = {"relation": relation, "from_kind": from_kind, "to_kind": to_kind}
    if attributes is not None:
        s["attributes"] = attributes
    s.update(extra)
    return s


# 200 Unicode scalar values that are 399 UTF-8 bytes and 201 UTF-16 code
# units: accepted, which pins "scalar values, not bytes, not UTF-16 units".
MULTIBYTE_200 = "é" * 100 + "日" * 99 + "\U0001F642"
assert len(MULTIBYTE_200) == 200

EDGES = [
    # The change request's list, accept (§8, 1-11).
    ("cr-01 holds-office person -> role, day precision",
     edge("holds-office", "person", "role", {"valid_from": "2022-07-01"})),
    ("cr-02 holds-office same-year tenure at year precision",
     edge("holds-office", "person", "role", {"valid_from": "2019", "valid_to": "2019"})),
    ("cr-03 holds-office end known, start not stated",
     edge("holds-office", "person", "role", {"valid_to": "2024-06"})),
    ("cr-04 office-of role -> org, attributes absent, normalized to {}",
     edge("office-of", "role", "org")),
    ("cr-05 member-of org -> org",
     edge("member-of", "org", "org")),
    ("cr-06 knows person -- person",
     edge("knows", "person", "person")),
    ("cr-07 allied-with org -- person, reverse of the declared pair, not reordered",
     edge("allied-with", "org", "person", {"subject": "District A bond measure"})),
    ("cr-08 opposes org -> org with subject",
     edge("opposes", "org", "org", {"subject": "District A bond measure"})),
    ("cr-09 vendor-to org -> org with subject",
     edge("vendor-to", "org", "org", {"subject": "assessment platform"})),
    ("cr-10 overlapping precisions, 2020-06 to 2020",
     edge("holds-office", "person", "role", {"valid_from": "2020-06", "valid_to": "2020"})),
    ("cr-11 vocabulary cited and matching",
     edge("holds-office", "person", "role", {"valid_from": "2022-07-01"}, vocabulary=VS)),
    # The change request's list, reject (§8, 12-28).
    ("cr-12 snake case is not aliased",
     edge("holds_office", "person", "role")),
    ("cr-13 an inverse label is not an id",
     edge("customer-of", "org", "org")),
    ("cr-14 holds-office role -> person, directed order matters",
     edge("holds-office", "role", "person")),
    ("cr-15 reports-to person -> person, reporting lines join roles",
     edge("reports-to", "person", "person")),
    ("cr-16 allied-with role -- org, stances never attach to a role",
     edge("allied-with", "role", "org", {"subject": "District A bond measure"})),
    ("cr-17 to_kind office is not a kind",
     edge("holds-office", "person", "office")),
    ("cr-18 opposes without subject",
     edge("opposes", "person", "org")),
    ("cr-19 knows with closeness",
     edge("knows", "person", "person", {"closeness": 0.9})),
    ("cr-20 month 13",
     edge("holds-office", "person", "role", {"valid_from": "2022-13-01"})),
    ("cr-21 prose date",
     edge("holds-office", "person", "role", {"valid_from": "July 2022"})),
    ("cr-22 no such calendar day, 2023-02-29",
     edge("holds-office", "person", "role", {"valid_from": "2023-02-29"})),
    ("cr-23 tenure out of order, 2021 to 2020-12",
     edge("holds-office", "person", "role", {"valid_from": "2021", "valid_to": "2020-12"})),
    ("cr-24 subject empty after trimming",
     edge("opposes", "person", "org", {"subject": "   "})),
    ("cr-25 subject of 201 scalar values",
     edge("competes-with", "org", "org", {"subject": "a" * 201})),
    ("cr-26 basis at top level, TT stays out of provenance",
     edge("holds-office", "person", "role", {"valid_from": "2022-07-01"}, basis="GENERATED")),
    ("cr-27 every failure reported: wrong order and unknown attribute",
     edge("holds-office", "role", "person", {"closeness": 0.9})),
    ("cr-28 vocabulary names another release",
     edge("holds-office", "person", "role", vocabulary="tt-relations/1.0 v0.9.0")),

    # The additions §8 asks for.
    ("attributes is an array, not an object",
     edge("holds-office", "person", "role", ["2022-07-01"])),
    ("attributes is null, which is not absence",
     edge("holds-office", "person", "role", None) | {"attributes": None}),
    ("subject of 200 multibyte scalar values (399 bytes, 201 UTF-16 units) accepts",
     edge("allied-with", "person", "org", {"subject": MULTIBYTE_200})),
    ("subject of 201 multibyte scalar values rejects",
     edge("allied-with", "person", "org", {"subject": MULTIBYTE_200 + "日"})),
    ("vocabulary null is a mismatch, not absence",
     edge("holds-office", "person", "role", vocabulary=None)),

    # The provenance boundary, whole.
    ("all four consumer provenance keys, each an unknown key",
     edge("knows", "person", "person", basis="GROUNDED", score=0.8,
          sources=["https://example.org/minutes"], observed_at="2026-09-01")),

    # Shape.
    ("not an object", ["holds-office", "person", "role"]),
    ("empty object: three fields missing", {}),
    ("relation is not a string", edge(7, "person", "role")),
    ("to_kind missing", {"relation": "office-of", "from_kind": "role"}),
    ("from_kind null", edge("office-of", None, "org")),

    # Identity is exact: no case folding, no label lookup.
    ("relation in title case", edge("Holds-Office", "person", "role")),
    ("relation spelled as its label", edge("holds office", "person", "role")),
    ("kind in title case", edge("office-of", "Role", "org")),
    ("unknown relation: attributes are not checked against a schema it lacks",
     edge("holds_office", "person", "role", {"closeness": 0.9})),
    ("several unknowns at once",
     edge("holds_office", "human", "office", surprise=1)),

    # Endpoint pairs.
    ("allied-with person -- org, the declared order",
     edge("allied-with", "person", "org", {"subject": "District A bond measure"})),
    ("competes-with person -- org, same-kind pairs only",
     edge("competes-with", "person", "org")),
    ("knows person -- org",
     edge("knows", "person", "org")),
    ("member-of role -> org, a role is not a member",
     edge("member-of", "role", "org")),
    ("governs role -> org, authority of an office is office-of plus reports-to",
     edge("governs", "role", "org")),
    ("opposes org -> person",
     edge("opposes", "org", "person", {"subject": "District A bond measure"})),
    ("reports-to role -> role",
     edge("reports-to", "role", "role", {"valid_from": "2023"})),
    ("market is a kind no 1.0.0 relation allows",
     edge("competes-with", "market", "market")),

    # Dates.
    ("leap day in a leap year", edge("holds-office", "person", "role", {"valid_from": "2024-02-29"})),
    ("2000 is a leap year", edge("holds-office", "person", "role", {"valid_from": "2000-02-29"})),
    ("1900 is not a leap year", edge("holds-office", "person", "role", {"valid_from": "1900-02-29"})),
    ("year zero", edge("holds-office", "person", "role", {"valid_from": "0000"})),
    ("year one", edge("holds-office", "person", "role", {"valid_from": "0001"})),
    ("unpadded month", edge("holds-office", "person", "role", {"valid_from": "2022-7-01"})),
    ("day 31 in a 30-day month", edge("holds-office", "person", "role", {"valid_from": "2022-06-31"})),
    ("a timestamp is not a date",
     edge("holds-office", "person", "role", {"valid_from": "2022-07-01T00:00:00Z"})),
    ("fullwidth digits are not ASCII digits",
     edge("holds-office", "person", "role", {"valid_from": "\uff12\uff10\uff12\uff12"})),
    ("five-digit year", edge("holds-office", "person", "role", {"valid_from": "20220"})),
    ("a date as a JSON number", edge("holds-office", "person", "role", {"valid_from": 2022})),
    ("tenure of one day", edge("holds-office", "person", "role",
                               {"valid_from": "2022-07-01", "valid_to": "2022-07-01"})),
    ("tenure ending before it starts, same precision",
     edge("holds-office", "person", "role", {"valid_from": "2022-07-02", "valid_to": "2022-07-01"})),
    ("tenure 2021-01 to 2020", edge("holds-office", "person", "role",
                                    {"valid_from": "2021-01", "valid_to": "2020"})),
    ("tenure 2020 to 2020-01, the year covers the month",
     edge("holds-office", "person", "role", {"valid_from": "2020", "valid_to": "2020-01"})),
    ("an ill-typed date skips the order check",
     edge("holds-office", "person", "role", {"valid_from": "2021", "valid_to": "2020-13"})),

    # Text.
    ("text is kept as sent, surrounding spaces included",
     edge("vendor-to", "org", "org", {"subject": "  assessment platform "})),
    ("text containing a newline", edge("vendor-to", "org", "org", {"subject": "line one\nline two"})),
    ("text containing C1 control U+0085", edge("vendor-to", "org", "org", {"subject": "a\u0085b"})),
    ("text of only no-break spaces", edge("vendor-to", "org", "org", {"subject": "\u00a0\u00a0"})),
    ("text of only an ideographic space", edge("vendor-to", "org", "org", {"subject": "\u3000"})),
    ("text as a JSON number", edge("vendor-to", "org", "org", {"subject": 42})),
    # Format characters (Unicode Cf) are neither White_Space nor controls, so
    # TT's text rule lets them through. Judging them is the consumer's lint
    # (TT-SPEC §9.1; beta's K2). These pin that TT accepts, not that it should.
    ("text of only a zero-width space passes TT (a consumer lint's job)",
     edge("vendor-to", "org", "org", {"subject": "\u200b"})),
    ("text with a right-to-left override passes TT (a consumer lint's job)",
     edge("vendor-to", "org", "org", {"subject": "bond \u202emeasure"})),
    ("text of exactly 200 ASCII scalar values",
     edge("competes-with", "person", "person", {"subject": "a" * 200})),
    ("required subject present but empty string", edge("allied-with", "person", "person", {"subject": ""})),
]

# A hand-built vocabulary with something retired in each collection.
FIXTURE_PATCH = [
    {"op": "replace", "path": "/version", "value": "1.1.0"},
    {"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v1.0.0"},
    {"op": "add", "path": "/entity_kinds/-", "value": {
        "id": "office", "label": "Office (retired, fixture only)",
        "definition": "A retired entity kind in a test fixture.",
        "deprecated_in": "1.1.0", "superseded_by": "role"}},
    {"op": "add", "path": "/relation_kinds/-", "value": {
        "id": "holds-post", "label": "holds post", "inverse_label": "post held by",
        "direction": "directed", "nature": "structural",
        "endpoints": [["person", "office"], ["person", "role"]], "attributes": {},
        "definition": "A retired relation kind in a test fixture.",
        "deprecated_in": "1.1.0", "superseded_by": "holds-office"}},
    {"op": "add", "path": "/relation_kinds/-", "value": {
        "id": "acquainted-with", "label": "acquainted with",
        "direction": "symmetric", "nature": "social",
        "endpoints": [["person", "person"]], "attributes": {},
        "definition": "A retired relation kind in a test fixture.",
        "deprecated_in": "1.1.0",
        "deprecation_note": "Retired in a test fixture; nothing replaces it."}},
]

FIXTURE_EDGES = [
    ("retired relation names its successor", edge("holds-post", "person", "role")),
    ("retired relation and retired kind together", edge("holds-post", "person", "office")),
    ("retired kind under a live relation", edge("holds-office", "person", "office")),
    ("retired relation with no successor gives its note", edge("acquainted-with", "person", "person")),
    ("the fixture's live surface still accepts", edge("knows", "person", "person")),
]

LOADS = [
    ("the shipped artifact loads", []),
    ("duplicate relation id",
     [{"op": "add", "path": "/relation_kinds/-", "value": {
         "id": "knows", "label": "knows", "direction": "symmetric", "nature": "social",
         "endpoints": [["person", "person"]], "attributes": {},
         "definition": "A second knows."}}]),
    ("endpoint naming an unknown kind",
     [{"op": "replace", "path": "/relation_kinds/0/endpoints/0/1", "value": "office"}]),
    ("inverse_label on a symmetric relation",
     [{"op": "add", "path": "/relation_kinds/6/inverse_label", "value": "known by"}]),
    ("inverse_label missing on a directed relation",
     [{"op": "remove", "path": "/relation_kinds/0/inverse_label"}]),
    ("retirement with neither successor nor note",
     [{"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "1.0.0"}]),
    ("live relations naming a retired kind",
     [{"op": "add", "path": "/entity_kinds/3/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/entity_kinds/3/deprecation_note", "value": "Fixture."}]),
    ("duplicate symmetric pair written in both orders",
     [{"op": "add", "path": "/relation_kinds/7/endpoints/-", "value": ["org", "person"]}]),
    ("duplicate directed pair",
     [{"op": "add", "path": "/relation_kinds/0/endpoints/-", "value": ["person", "role"]}]),
    ("attribute type outside attribute_types",
     [{"op": "replace", "path": "/relation_kinds/0/attributes/valid_from/type", "value": "datetime"}]),
    ("attribute_types naming a type nobody can check",
     [{"op": "add", "path": "/attribute_types/-", "value": "number"}]),
    ("relation id not kebab-case",
     [{"op": "replace", "path": "/relation_kinds/0/id", "value": "holds_office"}]),
    ("entity kind id not kebab-case",
     [{"op": "add", "path": "/entity_kinds/-", "value": {
         "id": "Place", "label": "Place", "definition": "Fixture."}}]),
    ("relation id equal to a kind id",
     [{"op": "replace", "path": "/relation_kinds/6/id", "value": "role"}]),
    ("blank entity kind label",
     [{"op": "replace", "path": "/entity_kinds/0/label", "value": " \u3000 "}]),
    ("empty relation definition",
     [{"op": "replace", "path": "/relation_kinds/1/definition", "value": ""}]),
    ("empty inverse_label",
     [{"op": "replace", "path": "/relation_kinds/1/inverse_label", "value": " "}]),
    ("direction outside the set",
     [{"op": "replace", "path": "/relation_kinds/6/direction", "value": "undirected"}]),
    ("nature outside the set",
     [{"op": "replace", "path": "/relation_kinds/6/nature", "value": "personal"}]),
    ("attribute name not snake_case",
     [{"op": "add", "path": "/relation_kinds/5/attributes/Subject",
       "value": {"type": "text", "required": False}}]),
    ("self-supersession",
     [{"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/entity_kinds/2/superseded_by", "value": "market"}]),
    ("unknown successor",
     [{"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/entity_kinds/2/superseded_by", "value": "territory"}]),
    ("successor in the other collection",
     [{"op": "add", "path": "/relation_kinds/6/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/6/superseded_by", "value": "person"}]),
    ("supersession cycle, reported once",
     [{"op": "add", "path": "/relation_kinds/6/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/6/superseded_by", "value": "competes-with"},
      {"op": "add", "path": "/relation_kinds/9/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/9/superseded_by", "value": "knows"}]),
    ("a retired relation may name a retired kind",
     [{"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/entity_kinds/2/deprecation_note", "value": "Fixture."},
      {"op": "add", "path": "/relation_kinds/-", "value": {
          "id": "prices", "label": "prices", "inverse_label": "priced by",
          "direction": "directed", "nature": "structural",
          "endpoints": [["market", "org"]], "attributes": {}, "definition": "Fixture.",
          "deprecated_in": "1.0.0", "deprecation_note": "Fixture."}}]),
    ("wrong schema",
     [{"op": "replace", "path": "/schema", "value": "tt-relations/2.0"}]),
    ("version not MAJOR.MINOR.PATCH, and supersedes nothing",
     [{"op": "replace", "path": "/version", "value": "1.0"}]),
    ("1.0.0 superseding something",
     [{"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v0.9.0"}]),
    ("1.1.0 superseding nothing",
     [{"op": "replace", "path": "/version", "value": "1.1.0"}]),
    ("1.1.0 superseding itself",
     [{"op": "replace", "path": "/version", "value": "1.1.0"},
      {"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v1.1.0"}]),
    ("1.1.0 one step back loads",
     [{"op": "replace", "path": "/version", "value": "1.1.0"},
      {"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v1.0.0"}]),
    ("several failures at once",
     [{"op": "replace", "path": "/relation_kinds/6/nature", "value": "personal"},
      {"op": "replace", "path": "/relation_kinds/7/id", "value": "knows"}]),
    ("malformed: required is not a boolean",
     [{"op": "replace", "path": "/relation_kinds/7/attributes/subject/required", "value": "yes"}]),
    ("malformed: an endpoint of three kinds",
     [{"op": "replace", "path": "/relation_kinds/0/endpoints/0", "value": ["person", "role", "org"]}]),
    ("malformed: relation_kinds missing",
     [{"op": "remove", "path": "/relation_kinds"}]),
    # F2: supersession on live items, and cycles through them.
    ("a live kind with a deprecation_note",
     [{"op": "add", "path": "/entity_kinds/2/deprecation_note", "value": "Fixture."}]),
    ("a live relation with superseded_by",
     [{"op": "add", "path": "/relation_kinds/6/superseded_by", "value": "competes-with"}]),
    ("a cycle through a live item: retired knows -> live competes-with -> knows",
     [{"op": "add", "path": "/relation_kinds/6/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/6/superseded_by", "value": "competes-with"},
      {"op": "add", "path": "/relation_kinds/9/superseded_by", "value": "knows"}]),
    ("a three-item cycle, reported once",
     [{"op": "add", "path": "/relation_kinds/6/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/6/superseded_by", "value": "competes-with"},
      {"op": "add", "path": "/relation_kinds/9/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/9/superseded_by", "value": "allied-with"},
      {"op": "add", "path": "/relation_kinds/7/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/7/superseded_by", "value": "knows"}]),
    ("a chain into a cycle is reported once, from inside the cycle",
     [{"op": "add", "path": "/relation_kinds/7/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/7/superseded_by", "value": "knows"},
      {"op": "add", "path": "/relation_kinds/6/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/6/superseded_by", "value": "competes-with"},
      {"op": "add", "path": "/relation_kinds/9/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/relation_kinds/9/superseded_by", "value": "knows"}]),

    # F6: closed shapes and tighter retirement.
    ("unknown top-level field",
     [{"op": "add", "path": "/extensions", "value": {}}]),
    ("governance missing",
     [{"op": "remove", "path": "/governance"}]),
    ("unknown field on an entity kind",
     [{"op": "add", "path": "/entity_kinds/0/parent", "value": "org"}]),
    ("unknown field on a relation kind",
     [{"op": "add", "path": "/relation_kinds/6/weight", "value": 1}]),
    ("unknown field in an attribute spec",
     [{"op": "add", "path": "/relation_kinds/5/attributes/subject/enum", "value": ["a"]}]),
    ("a relation with no endpoint pairs",
     [{"op": "replace", "path": "/relation_kinds/1/endpoints", "value": []}]),
    ("deprecated_in not MAJOR.MINOR.PATCH",
     [{"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "v1"},
      {"op": "add", "path": "/entity_kinds/2/deprecation_note", "value": "Fixture."}]),
    ("deprecated_in later than the release",
     [{"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "1.0.10"},
      {"op": "add", "path": "/entity_kinds/2/deprecation_note", "value": "Fixture."}]),
    ("deprecated_in earlier than the release loads",
     [{"op": "replace", "path": "/version", "value": "1.10.0"},
      {"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v1.9.0"},
      {"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "1.9.0"},
      {"op": "add", "path": "/entity_kinds/2/deprecation_note", "value": "Fixture."}]),
    ("a blank deprecation_note",
     [{"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "1.0.0"},
      {"op": "add", "path": "/entity_kinds/2/deprecation_note", "value": " \u00a0 "}]),
    ("attribute_types listing a type twice",
     [{"op": "add", "path": "/attribute_types/-", "value": "date"}]),

    # R3: the artifact's statements are not blank; versions are SemVer 2.0.0.
    ("blank governance",
     [{"op": "replace", "path": "/governance", "value": "  "}]),
    ("blank respectful_modeling",
     [{"op": "replace", "path": "/respectful_modeling", "value": ""}]),
    ("version with a leading zero",
     [{"op": "replace", "path": "/version", "value": "1.01.0"},
      {"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v1.0.0"}]),
    ("deprecated_in with a leading zero",
     [{"op": "add", "path": "/entity_kinds/2/deprecated_in", "value": "1.00.0"},
      {"op": "add", "path": "/entity_kinds/2/deprecation_note", "value": "Fixture."}]),
    ("supersedes with a leading zero",
     [{"op": "replace", "path": "/version", "value": "1.1.0"},
      {"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v01.0.0"}]),
    ("supersedes naming another schema",
     [{"op": "replace", "path": "/version", "value": "1.1.0"},
      {"op": "replace", "path": "/supersedes", "value": "tt-ontology/1.0 v2.1.0"}]),
    ("supersedes naming a later release",
     [{"op": "replace", "path": "/version", "value": "1.1.0"},
      {"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v1.2.0"}]),
    ("a part that is exactly 0 is not a leading zero",
     [{"op": "replace", "path": "/version", "value": "1.0.10"},
      {"op": "replace", "path": "/supersedes", "value": "tt-relations/1.0 v1.0.9"}]),
]


# --------------------------------------------------------------------------
# RFC 6902, the subset used here. tests/relations.rs has the same function.

def _tokens(pointer):
    if pointer == "":
        return []
    if not pointer.startswith("/"):
        raise ValueError(f"bad JSON Pointer {pointer!r}")
    return [t.replace("~1", "/").replace("~0", "~") for t in pointer[1:].split("/")]


def apply_patch(doc, patch):
    doc = json.loads(json.dumps(doc))  # deep copy
    for op in patch:
        *parent_path, last = _tokens(op["path"])
        parent = doc
        for t in parent_path:
            parent = parent[int(t)] if isinstance(parent, list) else parent[t]
        kind = op["op"]
        if isinstance(parent, list):
            if kind == "add":
                if last == "-":
                    parent.append(op["value"])
                else:
                    parent.insert(int(last), op["value"])
            elif kind == "remove":
                del parent[int(last)]
            elif kind == "replace":
                parent[int(last)] = op["value"]
            else:
                raise ValueError(f"unsupported op {kind}")
        else:
            if kind == "add":
                parent[last] = op["value"]
            elif kind == "remove":
                del parent[last]
            elif kind == "replace":
                if last not in parent:
                    raise KeyError(f"replace of absent {op['path']}")
                parent[last] = op["value"]
            else:
                raise ValueError(f"unsupported op {kind}")
    return doc


# The change request's §8 list, verdict by verdict: [] accepts, otherwise the
# multiset of codes. The generator refuses to write a corpus that disagrees.
CR_EXPECT = {
    1: [], 2: [], 3: [], 4: [], 5: [], 6: [], 7: [], 8: [], 9: [], 10: [], 11: [],
    12: ["unknown-relation"], 13: ["unknown-relation"],
    14: ["endpoint-pair-not-allowed"], 15: ["endpoint-pair-not-allowed"],
    16: ["endpoint-pair-not-allowed"], 17: ["unknown-kind"], 18: ["missing-attribute"],
    19: ["unknown-attribute"], 20: ["attribute-type"], 21: ["attribute-type"],
    22: ["attribute-type"], 23: ["tenure-order"], 24: ["attribute-type"],
    25: ["attribute-type"], 26: ["unknown-key"],
    27: ["endpoint-pair-not-allowed", "unknown-attribute"], 28: ["vocabulary-mismatch"],
}


def _check_cr_cases(vectors):
    seen = set()
    for v in vectors:
        if not v["name"].startswith("cr-"):
            continue
        n = int(v["name"][3:5])
        got = sorted(r["code"] for r in v["expect"].get("rejections", []))
        if got != sorted(CR_EXPECT[n]):
            raise SystemExit(f"{v['name']}: the change request expects {CR_EXPECT[n]}, got {got}")
        if (not got) and v["input"] != {k: v["expect"]["normalized"][k] for k in v["input"]}:
            raise SystemExit(f"{v['name']}: normalized form is not the input plus the stamp")
        seen.add(n)
    if seen != set(CR_EXPECT):
        raise SystemExit(f"change request cases missing: {sorted(set(CR_EXPECT) - seen)}")


def _edge_vectors(cases, vocab):
    out = []
    for name, statement in cases:
        normalized, errors = validate_edge(statement, vocab)
        expect = {"accepted": not errors}
        if errors:
            expect["rejections"] = errors
        else:
            expect["normalized"] = normalized
        out.append({"name": name, "input": statement, "expect": expect})
    return out


def main(argv):
    if len(argv) != 2:
        print((__doc__ or "").strip(), file=sys.stderr)
        return 2
    with open(argv[1], "rb") as f:
        data = f.read()
    raw = json.loads(data.decode("utf-8"))
    vocab, failures = check_vocabulary(raw)
    if failures:
        print(f"the artifact does not load: {failures}", file=sys.stderr)
        return 1
    fixture, fixture_failures = check_vocabulary(apply_patch(raw, FIXTURE_PATCH))
    if fixture_failures:
        print(f"the fixture does not load: {fixture_failures}", file=sys.stderr)
        return 1

    edges = _edge_vectors(EDGES, vocab)
    _check_cr_cases(edges)

    loads = []
    for name, patch in LOADS:
        _, fails = check_vocabulary(apply_patch(raw, patch))
        loads.append({"name": name, "patch": patch,
                      "expect": {"loads": not fails, "failures": fails}})

    doc = {
        "what": "TT relations conformance vectors: tt-relations/1.0 edge-statement "
                "verdicts and load rules (docs/proposals/ENTITY-RELATIONS.md §2.4, §2.5, §8)",
        "vocabulary": vocab["version_string"],
        "vocabulary_file": "bundle/relations-v1.0.json",
        "vocabulary_sha256": hashlib.sha256(data).hexdigest(),
        "conformance": {
            "normative": "edges and fixture_edges: accepted, normalized (on accept), and the "
                         "multiset of rejection codes (on reject). loads: whether the patched "
                         "artifact loads, and the multiset of failure rules. An implementation "
                         "matching these in any language conforms.",
            "advisory": "detail strings, byte for byte, in order. Both reference "
                        "implementations (src/relations.rs and python/tt_relations.py) pass this "
                        "tier, except the detail of a `malformed` load failure, which is the "
                        "parser's own wording.",
        },
        "patches": "RFC 6902 subset: add, remove, replace; paths are JSON Pointers into the "
                   "shipped artifact. The fixture and every load case are the shipped artifact "
                   "plus a patch.",
        "regenerate": "python3 python/gen_relation_vectors.py bundle/relations-v1.0.json "
                      "> vectors/verdicts/relation-verdicts.json",
        "edges": edges,
        "fixture": {
            "what": "the shipped artifact as a 1.1.0 document, plus a retired entity kind (office -> role), a retired "
                    "relation with a successor (holds-post -> holds-office) and one with only a "
                    "note (acquainted-with). A test document, not a release.",
            "patch": FIXTURE_PATCH,
        },
        "fixture_edges": _edge_vectors(FIXTURE_EDGES, fixture),
        "loads": loads,
    }
    json.dump(doc, sys.stdout, indent=2)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

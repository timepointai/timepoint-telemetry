#!/usr/bin/env python3
"""tt_relations — the TT relations vocabulary and edge-statement contract, stdlib only.

The Python on-ramp for `tt-relations/1.0` (bundle/relations-v1.0.json):
entity kinds, relation kinds between entities, and the edge-statement
contract. It mirrors `src/relations.rs` rule for rule: the same rule and
rejection codes, the same detail strings, in the same order. Where this file
and the vectors disagree, the vectors win (vectors/verdicts/relation-verdicts.json).

Two surfaces, both reject-never-repair:

  * load_vocabulary / check_vocabulary — the artifact's load rules. An invalid
    artifact is an error and must fail boot, as an invalid taxonomy does.
  * validate_edge — one edge statement. TT validates what a relation SAYS
    (relation, endpoint kinds, attributes) and nothing about who said it or
    how: `basis`, `score`, `sources` and `observed_at` are unknown keys, so the
    consumer strips its provenance before asking. TT never sees entity ids;
    that the endpoints exist, that their stored kinds match, and that they are
    two different entities are the consumer's checks.

Usage:
    python3 tt_relations.py <relations.json> <edge.json>
    python3 tt_relations.py <relations.json> -   # edge statement on stdin

Exit 0 and the normalized statement (vocabulary stamped) on stdout, or exit 1
with one typed rejection per line on stderr.
"""

import json
import sys

RELATIONS_SCHEMA = "tt-relations/1.0"
ATTRIBUTE_TYPES = ("date", "text")
EDGE_KEYS = ("relation", "from_kind", "to_kind", "attributes", "vocabulary")
TEXT_MAX_SCALARS = 200
DIRECTIONS = ("directed", "symmetric")

# The closed key set at every level of the artifact. An unknown field is
# `malformed`, as Rust's deny_unknown_fields makes it: a field nobody reads is
# a rule nobody enforces.
TOP_KEYS = ("schema", "version", "supersedes", "governance", "respectful_modeling",
            "attribute_types", "entity_kinds", "relation_kinds")
RETIRE_KEYS = ("deprecated_in", "superseded_by", "deprecation_note")
KIND_KEYS = ("id", "label", "definition") + RETIRE_KEYS
RELATION_KEYS = ("id", "label", "inverse_label", "direction", "nature", "endpoints",
                 "attributes", "definition") + RETIRE_KEYS
SPEC_KEYS = ("type", "required")
NATURES = ("structural", "social")

# The Unicode White_Space property, spelled out: Rust's str::trim uses exactly
# this set, and Python's str.strip() uses a slightly different one.
WHITE_SPACE = ("\t\n\x0b\x0c\r \x85\xa0\u1680"
               "\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a"
               "\u2028\u2029\u202f\u205f\u3000")

DATE_REJECTION = ("not a date at year, month or day precision (YYYY, YYYY-MM or "
                  "YYYY-MM-DD) naming a real calendar day with year >= 1")


class VocabularyInvalid(Exception):
    """An artifact that fails its load rules. `failures` is every failure."""

    def __init__(self, failures):
        self.failures = failures
        super().__init__("; ".join(f"{f['rule']}: {f['detail']}" for f in failures))


# --------------------------------------------------------------------------
# Loading

def _malformed(detail):
    return None, [{"rule": "malformed", "detail": detail}]


def _shape_error(raw):
    """The first way `raw` fails the artifact's shape, or None.

    Mirrors what the Rust loader's typed deserialization refuses. Only the rule
    (`malformed`) is shared across implementations; the wording is local.
    """
    def is_opt_str(v):
        return v is None or isinstance(v, str)

    def unknown(obj, allowed, where):
        extra = sorted(k for k in obj if k not in allowed)
        return f"{where}: unknown field `{extra[0]}`" if extra else None

    if not isinstance(raw, dict):
        return "the artifact must be a JSON object"
    why = unknown(raw, TOP_KEYS, "artifact")
    if why:
        return why
    for key in ("schema", "version", "governance", "respectful_modeling"):
        if not isinstance(raw.get(key), str):
            return f"`{key}` must be a string"
    if not is_opt_str(raw.get("supersedes")):
        return "`supersedes` must be a string or null"
    types = raw.get("attribute_types")
    if not isinstance(types, list) or not all(isinstance(t, str) for t in types):
        return "`attribute_types` must be an array of strings"
    for coll in ("entity_kinds", "relation_kinds"):
        if not isinstance(raw.get(coll), list):
            return f"`{coll}` must be an array"
    retire_keys = ("deprecated_in", "superseded_by", "deprecation_note")
    for i, k in enumerate(raw["entity_kinds"]):
        where = f"entity_kinds[{i}]"
        if not isinstance(k, dict):
            return f"{where} must be an object"
        why = unknown(k, KIND_KEYS, where)
        if why:
            return why
        for key in ("id", "label", "definition"):
            if not isinstance(k.get(key), str):
                return f"{where}.{key} must be a string"
        for key in retire_keys:
            if not is_opt_str(k.get(key)):
                return f"{where}.{key} must be a string or null"
    for i, r in enumerate(raw["relation_kinds"]):
        where = f"relation_kinds[{i}]"
        if not isinstance(r, dict):
            return f"{where} must be an object"
        why = unknown(r, RELATION_KEYS, where)
        if why:
            return why
        for key in ("id", "label", "direction", "nature", "definition"):
            if not isinstance(r.get(key), str):
                return f"{where}.{key} must be a string"
        for key in ("inverse_label",) + retire_keys:
            if not is_opt_str(r.get(key)):
                return f"{where}.{key} must be a string or null"
        endpoints = r.get("endpoints")
        if not isinstance(endpoints, list) or not all(
                isinstance(p, list) and len(p) == 2 and all(isinstance(e, str) for e in p)
                for p in endpoints):
            return f"{where}.endpoints must be an array of [from_kind, to_kind] pairs"
        attrs = r.get("attributes")
        if not isinstance(attrs, dict):
            return f"{where}.attributes must be an object"
        for name, spec in attrs.items():
            if isinstance(spec, dict) and unknown(spec, SPEC_KEYS, f"{where}.attributes.{name}"):
                return unknown(spec, SPEC_KEYS, f"{where}.attributes.{name}")
            if not (isinstance(spec, dict) and isinstance(spec.get("type"), str)
                    and isinstance(spec.get("required"), bool)):
                return f"{where}.attributes.{name} must be {{type: string, required: bool}}"
    return None


def check_vocabulary(raw):
    """Check every load rule (proposal §2.4). Returns (vocab, failures).

    Exactly one is meaningful: failures == [] means vocab is loaded; any
    failures mean the artifact is refused whole and vocab is None. A document
    of the wrong shape fails with the single rule `malformed`.
    """
    why = _shape_error(raw)
    if why is not None:
        return _malformed(why)

    out = []

    def fail(rule, detail):
        out.append({"rule": rule, "detail": detail})

    schema, version = raw["schema"], raw["version"]
    supersedes = raw.get("supersedes")
    version_string = f"{schema} v{version}"
    kinds_list = raw["entity_kinds"]
    rels_list = raw["relation_kinds"]
    kinds = {}
    for k in kinds_list:
        kinds.setdefault(k["id"], k)  # the first of a duplicated id answers, as in Rust
    rels = {}
    for r in rels_list:
        rels.setdefault(r["id"], r)

    def retired(item):
        return item.get("deprecated_in") is not None

    # Rule 7: schema, version, one step back.
    if schema != RELATIONS_SCHEMA:
        fail("bad-schema", f"schema is `{schema}`, expected `{RELATIONS_SCHEMA}`")
    if not _is_semver(version):
        fail("bad-version", f"version `{version}` is not MAJOR.MINOR.PATCH")
    if version == "1.0.0":
        if supersedes is not None:
            fail("bad-supersedes",
                 f"1.0.0 is the first release and supersedes nothing, not `{supersedes}`")
    elif supersedes is None:
        fail("bad-supersedes", f"version {version} must name the release it supersedes")
    elif supersedes == version_string:
        fail("bad-supersedes", f"`{supersedes}` supersedes itself")

    types_seen = set()
    for t in raw["attribute_types"]:
        if t not in ATTRIBUTE_TYPES:
            fail("unsupported-attribute-type", f"attribute type `{t}` is not one of date, text")
        if t in types_seen:
            fail("duplicate-attribute-type", f"attribute type `{t}` is listed twice")
        types_seen.add(t)

    # Rule 1: kebab-case, unique, disjoint.
    for what, items in (("entity kind", kinds_list), ("relation kind", rels_list)):
        seen = set()
        for it in items:
            i = it["id"]
            if not _is_kebab(i):
                fail("id-not-kebab", f"{what} id `{i}` is not kebab-case")
            if i in seen:
                fail("duplicate-id", f"{what} id `{i}` appears twice")
            seen.add(i)
    for r in rels_list:
        if r["id"] in kinds:
            fail("ids-not-disjoint", f"`{r['id']}` is both an entity kind and a relation kind")

    # Rule 2: non-empty label and definition.
    for k in kinds_list:
        if _trim(k["label"]) == "":
            fail("empty-label", f"entity kind `{k['id']}` has no label")
        if _trim(k["definition"]) == "":
            fail("empty-definition", f"entity kind `{k['id']}` has no definition")
    for r in rels_list:
        if _trim(r["label"]) == "":
            fail("empty-label", f"relation kind `{r['id']}` has no label")
        if r.get("inverse_label") is not None and _trim(r["inverse_label"]) == "":
            fail("empty-label", f"relation kind `{r['id']}` has an empty inverse_label")
        if _trim(r["definition"]) == "":
            fail("empty-definition", f"relation kind `{r['id']}` has no definition")

    declared_types = set(raw["attribute_types"])
    for r in rels_list:
        rid = r["id"]
        inverse = r.get("inverse_label")
        # Rule 3: direction, nature, inverse_label iff directed.
        if r["direction"] not in DIRECTIONS:
            fail("bad-direction",
                 f"relation kind `{rid}`: direction `{r['direction']}` is not directed or symmetric")
        elif r["direction"] == "directed" and inverse is None:
            fail("inverse-label-mismatch", f"relation kind `{rid}` is directed and has no inverse_label")
        elif r["direction"] == "symmetric" and inverse is not None:
            fail("inverse-label-mismatch", f"relation kind `{rid}` is symmetric and has an inverse_label")
        if r["nature"] not in NATURES:
            fail("bad-nature",
                 f"relation kind `{rid}`: nature `{r['nature']}` is not structural or social")

        # Rule 4: endpoints name existing kinds; a live relation names no
        # retired kind; no duplicate pair (unordered when symmetric).
        if not r["endpoints"]:
            fail("no-endpoints", f"relation kind `{rid}` allows no endpoint pair")
        symmetric = r["direction"] == "symmetric"
        pairs = set()
        retired_named = set()
        for a, b in r["endpoints"]:
            for end in (a, b):
                k = kinds.get(end)
                if k is None:
                    fail("unknown-endpoint-kind",
                         f"relation kind `{rid}` names unknown entity kind `{end}`")
                elif retired(k) and not retired(r):
                    retired_named.add(end)
            key = (b, a) if symmetric and b < a else (a, b)
            if key in pairs:
                fail("duplicate-endpoint-pair", f"relation kind `{rid}` lists {a} -> {b} twice")
            pairs.add(key)
        for k in sorted(retired_named):
            fail("retired-endpoint-kind", f"live relation kind `{rid}` names retired entity kind `{k}`")

        # Rule 5: attribute names and types.
        for name in sorted(r["attributes"]):
            spec = r["attributes"][name]
            if not _is_attribute_name(name):
                fail("bad-attribute-name",
                     f"relation kind `{rid}`: attribute name `{name}` does not match ^[a-z][a-z0-9_]*$")
            if spec["type"] not in declared_types:
                fail("unknown-attribute-type",
                     f"relation kind `{rid}`: attribute `{name}` has type `{spec['type']}`, "
                     "not in attribute_types")

    # Rule 6: retirement, GOVERNANCE §3, for both collections alike.
    for what, items, index in (("entity kind", kinds_list, kinds),
                               ("relation kind", rels_list, rels)):
        for it in items:
            s, note, dep = it.get("superseded_by"), it.get("deprecation_note"), it.get("deprecated_in")
            if dep is None:
                if s is not None or note is not None:
                    fail("successor-without-retirement",
                         f"{what} `{it['id']}` has superseded_by or deprecation_note but no deprecated_in")
                continue
            if not _is_semver(dep):
                fail("bad-deprecated-in",
                     f"{what} `{it['id']}`: deprecated_in `{dep}` is not MAJOR.MINOR.PATCH")
            elif _is_semver(version) and _semver_key(dep) > _semver_key(version):
                fail("bad-deprecated-in",
                     f"{what} `{it['id']}`: deprecated_in {dep} is later than this release, {version}")
            if s is not None and s == it["id"]:
                fail("self-supersession", f"{what} `{it['id']}` supersedes itself")
            elif s is not None and s not in index:
                fail("unknown-successor", f"{what} `{it['id']}` is superseded_by unknown `{s}`")
            elif s is None and note is None:
                fail("retired-without-successor-or-note",
                     f"{what} `{it['id']}` is retired with neither superseded_by nor deprecation_note")
            if note is not None and _trim(note) == "":
                fail("empty-deprecation-note", f"{what} `{it['id']}` has a blank deprecation_note")
        # No supersession chain may close, whether it runs through retired or
        # live items. Walked from every item that names a successor; each
        # cycle is reported once, from its smallest id. A one-item loop is
        # `self-supersession`, already reported above.
        for it in items:
            if it.get("superseded_by") is None:
                continue
            seen = []
            cur = it
            while cur is not None:
                if cur["id"] in seen:
                    p = seen.index(cur["id"])
                    if p == 0 and len(seen) > 1 and all(x >= it["id"] for x in seen):
                        seen.append(cur["id"])
                        fail("supersession-cycle", f"{what} supersession cycle: {' -> '.join(seen)}")
                    break
                seen.append(cur["id"])
                nxt = cur.get("superseded_by")
                cur = index.get(nxt) if nxt is not None else None

    if out:
        return None, out
    return {
        "raw": raw,
        "version_string": version_string,
        "entity_kinds": kinds,
        "relation_kinds": rels,
    }, []


def load_vocabulary(path):
    """Load and check an artifact file. Raises VocabularyInvalid; never repairs."""
    with open(path, encoding="utf-8") as f:
        raw = json.load(f)
    vocab, failures = check_vocabulary(raw)
    if failures:
        raise VocabularyInvalid(failures)
    return vocab


# --------------------------------------------------------------------------
# Edge statements

def validate_edge(statement, vocab):
    """Validate one edge statement (proposal §2.5). Returns (normalized, errors).

    errors == [] means normalized is the accepted form: the input with
    `attributes` defaulted to {} and the `vocabulary` stamp set, which
    validates again unchanged. Any errors mean the statement is thrown back
    whole and normalized is None. Every failure is reported, never only the
    first.
    """
    vs = vocab["version_string"]
    if not isinstance(statement, dict):
        return None, [{"code": "not-an-object", "detail": "edge statement must be a JSON object"}]
    errors = []

    def reject(code, detail):
        errors.append({"code": code, "detail": detail})

    # Rule 1: the closed key set. Provenance is the consumer's.
    for k in sorted(k for k in statement if k not in EDGE_KEYS):
        reject("unknown-key", f"unknown top-level key `{k}`")

    # Rule 2: the relation exists and is live.
    relation = None
    if "relation" not in statement:
        reject("missing-field", "`relation` is required")
    elif not isinstance(statement["relation"], str):
        reject("field-not-string", "`relation` must be a string")
    else:
        rid = statement["relation"]
        relation = vocab["relation_kinds"].get(rid)
        if relation is None:
            reject("unknown-relation", f"no relation kind `{rid}` in {vs}")
        elif relation.get("deprecated_in") is not None:
            reject("retired-relation",
                   f"relation `{rid}` retired in {relation['deprecated_in']}; "
                   f"{_successor_text(relation)}")

    # Rule 3: both kinds exist and are live.
    ends = [None, None]
    for slot, field in enumerate(("from_kind", "to_kind")):
        if field not in statement:
            reject("missing-field", f"`{field}` is required")
        elif not isinstance(statement[field], str):
            reject("field-not-string", f"`{field}` must be a string")
        else:
            kid = statement[field]
            kind = vocab["entity_kinds"].get(kid)
            if kind is None:
                reject("unknown-kind", f"`{field}`: no entity kind `{kid}` in {vs}")
            else:
                if kind.get("deprecated_in") is not None:
                    reject("retired-kind",
                           f"`{field}`: entity kind `{kid}` retired in {kind['deprecated_in']}; "
                           f"{_successor_text(kind)}")
                ends[slot] = kid

    # Rule 4: the pair is allowed, checked only when all three are known.
    if relation is not None and ends[0] is not None and ends[1] is not None:
        frm, to = ends
        if not _allows(relation, frm, to):
            arrow = "--" if relation["direction"] == "symmetric" else "->"
            allowed = ", ".join(f"{a} {arrow} {b}" for a, b in relation["endpoints"])
            reject("endpoint-pair-not-allowed",
                   f"`{relation['id']}` does not allow {frm} {arrow} {to}; allowed: {allowed}")

    # Rules 5 and 6: attributes against the relation's closed schema.
    attributes = statement.get("attributes", {})  # absent means {}; null is not absent
    if not isinstance(attributes, dict):
        reject("attributes-not-object", "`attributes` must be a JSON object")
        attributes = None
    if relation is not None and attributes is not None:
        schema = relation["attributes"]
        dates = {}
        for name in sorted(attributes):
            value = attributes[name]
            spec = schema.get(name)
            if spec is None:
                reject("unknown-attribute", f"`{relation['id']}` has no attribute `{name}`")
                continue
            why, bound = _check_value(spec["type"], value)
            if why is not None:
                reject("attribute-type", f"`{name}`: {why}")
            elif bound is not None:
                dates[name] = (bound, value)
        for name in sorted(schema):
            if schema[name]["required"] and name not in attributes:
                reject("missing-attribute", f"`{relation['id']}` requires attribute `{name}`")
        if "valid_from" in dates and "valid_to" in dates:
            (frm_bound, frm_raw), (to_bound, to_raw) = dates["valid_from"], dates["valid_to"]
            if frm_bound[0] > to_bound[1]:
                reject("tenure-order",
                       f"`valid_from` {frm_raw} begins after `valid_to` {to_raw} ends")

    # Rule 7: a cited vocabulary must be this one. null is not absence.
    if "vocabulary" in statement:
        cited = statement["vocabulary"]
        if not isinstance(cited, str):
            reject("vocabulary-mismatch", f"`vocabulary` must be the string `{vs}`")
        elif cited != vs:
            reject("vocabulary-mismatch", f"edge cites `{cited}`, loaded vocabulary is `{vs}`")

    if errors:
        return None, errors
    normalized = dict(statement)
    normalized["attributes"] = dict(attributes)
    normalized["vocabulary"] = vs
    return normalized, []


# --------------------------------------------------------------------------
# Helpers — each mirrors the function of the same name in src/relations.rs.

def _successor_text(item):
    s, note = item.get("superseded_by"), item.get("deprecation_note")
    if s is not None:
        return f"use `{s}`"
    if note is not None:
        return f"no successor: {note}"
    return "no successor"


def _allows(relation, frm, to):
    symmetric = relation["direction"] == "symmetric"
    return any((a == frm and b == to) or (symmetric and a == to and b == frm)
               for a, b in relation["endpoints"])


def _check_value(ty, value):
    """(why, bound): why is None when the value is well-typed; bound is set for a date."""
    if ty == "date":
        if not isinstance(value, str):
            return "a date must be a JSON string", None
        bound = _parse_date(value)
        if bound is None:
            return DATE_REJECTION, None
        return None, bound
    if ty == "text":
        if not isinstance(value, str):
            return "text must be a JSON string", None
        if any(0xD800 <= ord(c) <= 0xDFFF for c in value):
            # json.loads accepts "\ud800"; Rust's parser refuses it before the
            # validator runs, and a lone surrogate is no Unicode scalar value.
            return "text must not contain unpaired surrogates", None
        if any(_is_control(c) for c in value):
            return "text must not contain control characters", None
        if _trim(value) == "":
            return "text must not be empty after trimming", None
        n = len(value)  # code points, which are scalar values once surrogates are refused
        if n > TEXT_MAX_SCALARS:
            return f"text is {n} Unicode scalar values; at most {TEXT_MAX_SCALARS}", None
        return None, None
    return f"attribute type `{ty}` cannot be checked", None


def _is_control(c):
    """Unicode general category Cc."""
    o = ord(c)
    return o <= 0x1F or 0x7F <= o <= 0x9F


def _trim(s):
    return s.strip(WHITE_SPACE)


def _digits(p, n):
    if len(p) == n and all(c in "0123456789" for c in p):
        return int(p)
    return None


def _days_in_month(year, month):
    leap = (year % 4 == 0 and year % 100 != 0) or year % 400 == 0
    if month in (1, 3, 5, 7, 8, 10, 12):
        return 31
    if month in (4, 6, 9, 11):
        return 30
    if month == 2:
        return 29 if leap else 28
    return None


def _parse_date(s):
    """((y, m, d) earliest, (y, m, d) latest), or None. Precision is kept, never padded."""
    parts = s.split("-")
    year = _digits(parts[0], 4)
    if year is None or year < 1:
        return None
    if len(parts) == 1:
        return (year, 1, 1), (year, 12, 31)
    if len(parts) == 2:
        month = _digits(parts[1], 2)
        last = _days_in_month(year, month) if month is not None else None
        if last is None:
            return None
        return (year, month, 1), (year, month, last)
    if len(parts) == 3:
        month, day = _digits(parts[1], 2), _digits(parts[2], 2)
        if month is None or day is None:
            return None
        last = _days_in_month(year, month)
        if last is None or not 1 <= day <= last:
            return None
        return (year, month, day), (year, month, day)
    return None


def _semver_key(v):
    """Numeric order without int(): (digit count without leading zeros, digits)."""
    return [(len(p.lstrip("0")), p.lstrip("0")) for p in v.split(".")]


def _is_semver(v):
    parts = v.split(".")
    return len(parts) == 3 and all(p != "" and all(c in "0123456789" for c in p) for p in parts)


def _is_attribute_name(name):
    lower = "abcdefghijklmnopqrstuvwxyz"
    return (name != "" and name[0] in lower
            and all(c in lower or c in "0123456789" or c == "_" for c in name))


def _is_kebab(i):
    ok = "abcdefghijklmnopqrstuvwxyz0123456789"
    return i != "" and all(seg != "" and all(c in ok for c in seg) for seg in i.split("-"))


def main(argv):
    if len(argv) != 3:
        print((__doc__ or "").strip(), file=sys.stderr)
        return 2
    try:
        vocab = load_vocabulary(argv[1])
    except VocabularyInvalid as e:
        for f in e.failures:
            print(f"vocabulary invalid: {f['rule']}: {f['detail']}", file=sys.stderr)
        return 2
    source = sys.stdin if argv[2] == "-" else open(argv[2], encoding="utf-8")
    with source:
        try:
            statement = json.load(source)
        except json.JSONDecodeError as e:
            print(f"rejected: not-json: {e}", file=sys.stderr)
            return 1
    normalized, errors = validate_edge(statement, vocab)
    if errors:
        for e in errors:
            print(f"rejected: {e['code']}: {e['detail']}", file=sys.stderr)
        return 1
    json.dump(normalized, sys.stdout, indent=2, ensure_ascii=False)
    print()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))

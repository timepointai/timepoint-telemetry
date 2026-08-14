#!/usr/bin/env python3
"""tt_envelope — TT envelope hashing, stdlib only.

RFC 8785 (JCS) canonicalisation and the two hashes (TT-SPEC §3):

    content_hash    = "sha256:" + hex(sha256(canonical(claim)))
    provenance_hash = "sha256:" + hex(sha256(canonical(provenance)))

where the claim is EXACTLY {label, occurs_at, participants} extracted from the
payload. classification, grounding, basis_note and the whole provenance object
sit outside content_hash, so a re-classified or re-grounded moment keeps its
identity. A payload missing a claim field is a typed error, never a silently
hashed hole; a field present with value null is representable absence and
hashes as null.

Written fresh against the committed conformance vectors (vectors/*.json),
which are normative: run python/test_tt_envelope.py to reproduce every one
byte-for-byte. The three classically-wrong corners, handled explicitly:

  * object keys sort by UTF-16 code unit, not code point — a supplementary
    character (surrogate pair) sorts below U+E000..U+FFFF;
  * numbers serialise per ECMAScript Number::toString over IEEE-754 doubles —
    18446744073709551615 hashes as the double 18446744073709552000, decimal
    form, because integers below 1e21 never take exponent form;
  * strings escape only what JSON.stringify escapes: ", \\, and C0 controls
    (shortcuts for \\b \\t \\n \\f \\r), everything else literal UTF-8.
"""

import hashlib
import json
import math

CONTENT_HASH_FIELDS = ("label", "occurs_at", "participants")

_ESCAPES = {
    '"': '\\"', "\\": "\\\\",
    "\b": "\\b", "\t": "\\t", "\n": "\\n", "\f": "\\f", "\r": "\\r",
}


class MissingPayloadField(ValueError):
    def __init__(self, field):
        self.field = field
        super().__init__(f"payload missing required claim field `{field}`")


class PayloadNotObject(ValueError):
    def __init__(self):
        super().__init__("payload is not a JSON object")


def _es6_number(value):
    """Serialise a JSON number as ECMAScript Number::toString would.

    Every JSON number is treated as an IEEE-754 double (JCS rule); Python's
    repr supplies the shortest round-trip digits and this reformats them to
    the ES6 grammar."""
    f = float(value)
    if math.isnan(f) or math.isinf(f):
        raise ValueError("NaN and Infinity are not JSON numbers")
    if f == 0.0:
        return "0"  # covers -0.0, per ES6 String(-0)
    neg = f < 0.0
    mant = repr(abs(f))
    if "e" in mant:
        mant, exp_str = mant.split("e")
        exp = int(exp_str)
    else:
        exp = 0
    int_part, _, frac_part = mant.partition(".")
    combined = int_part + frac_part
    leading = len(combined) - len(combined.lstrip("0"))
    digits = combined.strip("0")
    k = len(digits)
    # decimal point position: value = 0.<digits> * 10**n
    n = len(int_part) - leading + exp
    if k <= n <= 21:
        s = digits + "0" * (n - k)
    elif 0 < n <= 21:
        s = digits[:n] + "." + digits[n:]
    elif -6 < n <= 0:
        s = "0." + "0" * (-n) + digits
    else:
        e = n - 1
        s = digits[0] + ("." + digits[1:] if k > 1 else "")
        s += "e" + ("+" if e >= 0 else "-") + str(abs(e))
    return "-" + s if neg else s


def _quote(s):
    out = ['"']
    for ch in s:
        if ch in _ESCAPES:
            out.append(_ESCAPES[ch])
        elif ch < "\x20":
            out.append(f"\\u{ord(ch):04x}")
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def _utf16_key(s):
    return s.encode("utf-16-be")


def canonicalize(value):
    """The RFC 8785 canonical serialisation of a JSON value, as a str."""
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, str):
        return _quote(value)
    if isinstance(value, (int, float)):
        return _es6_number(value)
    if isinstance(value, list):
        return "[" + ",".join(canonicalize(v) for v in value) + "]"
    if isinstance(value, dict):
        items = sorted(value.items(), key=lambda kv: _utf16_key(kv[0]))
        return "{" + ",".join(_quote(k) + ":" + canonicalize(v) for k, v in items) + "}"
    raise TypeError(f"not a JSON value: {type(value).__name__}")


def _claim_subset(payload):
    if not isinstance(payload, dict):
        raise PayloadNotObject()
    subset = {}
    for field in CONTENT_HASH_FIELDS:
        if field not in payload:
            raise MissingPayloadField(field)
        subset[field] = payload[field]
    return subset


def _sha256_prefixed(canonical):
    return "sha256:" + hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def content_canonical(payload):
    """The exact bytes content_hash digests: the canonicalised claim subset."""
    return canonicalize(_claim_subset(payload))


def content_hash(payload):
    return _sha256_prefixed(content_canonical(payload))


def provenance_hash(provenance):
    return _sha256_prefixed(canonicalize(provenance))


if __name__ == "__main__":
    import sys
    payload = json.load(sys.stdin)
    print(content_hash(payload))

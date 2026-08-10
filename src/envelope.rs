//! RFC 8785 (JCS) canonicalization, the content/provenance hash pair, and the Envelope.
//! Contract: CONTRACTS.md §6, SNAG-SPEC §3.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::SnagError;

/// The exact payload fields covered by `content_hash` — the claim, nothing else.
/// classification, grounding, basis_note, and provenance are OUTSIDE by design
/// (SNAG-SPEC §3 hash-coverage table).
pub const CONTENT_HASH_FIELDS: [&str; 3] = ["label", "occurs_at", "participants"];

/// RFC 8785 (JCS) canonical form of a JSON value.
///
/// Infallible for `serde_json::Value`: map keys are always strings and numbers are always
/// finite (this workspace does not enable serde_json's `arbitrary_precision`), which
/// removes every error path in the serializer. Numbers serialize per ECMAScript
/// `Number::toString` (RFC 8785 §3.2.2.3): integers beyond 2^53 lose precision exactly as
/// IEEE-754 doubles do — the conformance vectors freeze that behavior explicitly.
pub fn canonicalize(value: &Value) -> String {
    match serde_json_canonicalizer::to_string(value) {
        Ok(s) => s,
        // Unreachable for `Value` inputs (see above); kept explicit rather than unwrap so
        // the invariant is stated where it is relied upon.
        Err(e) => unreachable!("JCS serialization of serde_json::Value cannot fail: {e}"),
    }
}

fn sha256_prefixed(canonical: &str) -> String {
    let digest = Sha256::digest(canonical.as_bytes());
    format!("sha256:{}", hex::encode(digest))
}

/// Extract the hash-covered subset of a payload: exactly `{label, occurs_at, participants}`.
/// A field that is present-but-null hashes as null (representable absence); a field that is
/// missing entirely is a typed error — a claim needs all three.
fn claim_subset(payload: &Value) -> Result<Value, SnagError> {
    let obj = payload.as_object().ok_or(SnagError::PayloadNotObject)?;
    let mut subset = serde_json::Map::new();
    for field in CONTENT_HASH_FIELDS {
        let v = obj
            .get(field)
            .ok_or(SnagError::MissingPayloadField(field))?;
        subset.insert(field.to_owned(), v.clone());
    }
    Ok(Value::Object(subset))
}

/// The canonical (JCS) form of the hash-covered payload subset — the exact bytes that
/// `content_hash` digests. Exposed so conformance tooling can assert on the bytes.
pub fn content_canonical(payload: &Value) -> Result<String, SnagError> {
    Ok(canonicalize(&claim_subset(payload)?))
}

/// `"sha256:" + hex` over the canonicalized `{label, occurs_at, participants}` — exactly
/// those three payload fields, nothing else.
pub fn content_hash(payload: &Value) -> Result<String, SnagError> {
    Ok(sha256_prefixed(&content_canonical(payload)?))
}

/// `"sha256:" + hex` over the canonicalized full provenance value.
pub fn provenance_hash(provenance: &Value) -> String {
    sha256_prefixed(&canonicalize(provenance))
}

/// One moment, exactly as stored (SNAG-SPEC §3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub id: String,
    pub provenance: Value,
    pub payload: Value,
    pub entity_ids: Vec<String>,
    pub content_hash: String,
    pub provenance_hash: String,
}

impl Envelope {
    /// Build an envelope, computing both hashes from the given payload and provenance.
    pub fn new(
        id: impl Into<String>,
        provenance: Value,
        payload: Value,
        entity_ids: Vec<String>,
    ) -> Result<Self, SnagError> {
        let content_hash = content_hash(&payload)?;
        let provenance_hash = provenance_hash(&provenance);
        Ok(Self {
            id: id.into(),
            provenance,
            payload,
            entity_ids,
            content_hash,
            provenance_hash,
        })
    }

    /// Recompute both hashes from the stored payload/provenance and compare against the
    /// stored values. A mismatch is a typed error naming which hash disagreed.
    pub fn verify(&self) -> Result<(), SnagError> {
        let recomputed_content = content_hash(&self.payload)?;
        if recomputed_content != self.content_hash {
            return Err(SnagError::HashMismatch {
                which: "content_hash",
                stored: self.content_hash.clone(),
                recomputed: recomputed_content,
            });
        }
        let recomputed_provenance = provenance_hash(&self.provenance);
        if recomputed_provenance != self.provenance_hash {
            return Err(SnagError::HashMismatch {
                which: "provenance_hash",
                stored: self.provenance_hash.clone(),
                recomputed: recomputed_provenance,
            });
        }
        Ok(())
    }
}

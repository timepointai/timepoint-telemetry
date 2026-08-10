//! tt-core — the TT envelope (RFC 8785/JCS canonicalization + the content/provenance
//! hash pair) and the ontology bundle loader with load-time validation.
//!
//! OWNED BY: tt-core agent (gate G1). Contract: CONTRACTS.md §6, TT-SPEC §2/§3,
//! genesis/snag/validate.py (all check families ported as loader-time assertions).
//!
//! The load-bearing rule (TT-SPEC §3 hash coverage): `content_hash` covers EXACTLY
//! `{label, occurs_at, participants}` — the claim. `classification`, `grounding`,
//! `basis_note` and the whole `provenance` object are OUTSIDE it, so a re-grounded or
//! re-classified moment keeps its identity. `provenance_hash` covers the full provenance
//! value. Same `content_hash` = same claim; same pair = same telling.
//!
//! Conformance vectors live in `vectors/*.json`; `tests/vectors.rs` recomputes every one
//! and asserts byte equality — the committed bytes are normative and must never drift.

mod bundle;
mod distance;
mod envelope;

pub use bundle::{Bridge, Bundle, LateralEdge, Lens, Lenses, Metric, Node, RELATIONS};
pub use distance::{DistanceIndex, distance};
pub use envelope::{
    CONTENT_HASH_FIELDS, Envelope, canonicalize, content_canonical, content_hash,
    provenance_hash,
};

/// Typed errors. Unknown/absent are representable states — a payload missing a claim
/// field is a `MissingPayloadField`, never a silently hashed hole.
#[derive(Debug, thiserror::Error)]
pub enum TtError {
    #[error("bundle io: {0}")]
    Io(#[from] std::io::Error),
    #[error("bundle parse: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("bundle invalid: {0}")]
    Invalid(String),
    #[error("payload is not a JSON object")]
    PayloadNotObject,
    #[error("payload missing required claim field `{0}`")]
    MissingPayloadField(&'static str),
    #[error("{which} mismatch: stored {stored}, recomputed {recomputed}")]
    HashMismatch {
        which: &'static str,
        stored: String,
        recomputed: String,
    },
}

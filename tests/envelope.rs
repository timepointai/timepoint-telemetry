//! Hash-coverage property tests (SNAG-SPEC §3): mutating classification/grounding/
//! basis_note/provenance never changes content_hash; mutating label/occurs_at/participants
//! always does; provenance mutation changes only provenance_hash.

use serde_json::{Value, json};
use snag_core::{Envelope, SnagError, content_hash, provenance_hash};

fn worked_payload() -> Value {
    json!({
        "label": "Novabloom signs with Harborline; the wire lands at an $18M cap",
        "occurs_at": "2026-11-14",
        "participants": [
            "/person/sofia-almeida",
            "/organization/novabloom",
            "/organization/harborline-capital"
        ],
        "classification": {
            "bundle": "snag-ontology/1.0 v1.1.0",
            "classifier": "cls-2026-07-a",
            "lens_b": { "negotiation-and-agreement": 0.55, "deciding-and-judging": 0.25 },
            "lens_a": { "corporate-founding-and-milestone": 0.6, "banking-and-exchange": 0.3 },
            "shadow": { "relation": "scales-up-to", "event": "enterprise-and-commerce" }
        },
        "grounding": "ACCUMULATED",
        "basis_note": "simulated outcome, branch 3 of 4; not an observed event"
    })
}

fn worked_provenance() -> Value {
    json!({
        "source": "pro.deep_sim",
        "run_id": "run_meridian_9f2",
        "branch": 3,
        "timepoint": 9,
        "producer_version": "pro-api 0.4.1",
        "created_at": "2026-07-28T17:04:11Z"
    })
}

fn envelope() -> Envelope {
    Envelope::new(
        "mnt_01j9x4qe7",
        worked_provenance(),
        worked_payload(),
        vec![
            "/person/sofia-almeida".into(),
            "/organization/novabloom".into(),
            "/organization/harborline-capital".into(),
        ],
    )
    .expect("worked-moment envelope builds")
}

#[test]
fn new_computes_both_hashes_and_verify_passes() {
    let env = envelope();
    assert_eq!(env.content_hash, content_hash(&worked_payload()).unwrap());
    assert_eq!(env.provenance_hash, provenance_hash(&worked_provenance()));
    assert!(env.content_hash.starts_with("sha256:"));
    assert!(env.provenance_hash.starts_with("sha256:"));
    env.verify().expect("untampered envelope verifies");
}

#[test]
fn fields_outside_content_hash_do_not_change_it() {
    let base = content_hash(&worked_payload()).unwrap();

    let mut p = worked_payload();
    p["classification"]["lens_b"] = json!({ "deciding-and-judging": 0.9 });
    p["classification"]["classifier"] = json!("cls-2099-01-z");
    assert_eq!(content_hash(&p).unwrap(), base, "classification is outside");

    let mut p = worked_payload();
    p["grounding"] = json!("GROUNDED");
    assert_eq!(content_hash(&p).unwrap(), base, "grounding is outside");

    let mut p = worked_payload();
    p["basis_note"] = json!("re-grounded after observation");
    assert_eq!(content_hash(&p).unwrap(), base, "basis_note is outside");

    let mut p = worked_payload();
    p["dialog"] = json!([{ "speaker": "/person/sofia-almeida", "line": "Send it." }]);
    assert_eq!(content_hash(&p).unwrap(), base, "extra payload fields are outside");

    let mut p = worked_payload();
    p.as_object_mut().unwrap().remove("classification");
    p.as_object_mut().unwrap().remove("grounding");
    p.as_object_mut().unwrap().remove("basis_note");
    assert_eq!(
        content_hash(&p).unwrap(),
        base,
        "stripping everything outside the claim keeps the hash"
    );
}

#[test]
fn claim_fields_always_change_content_hash() {
    let base = content_hash(&worked_payload()).unwrap();

    let mut p = worked_payload();
    p["label"] = json!("Novabloom walks away from the Harborline term sheet");
    assert_ne!(content_hash(&p).unwrap(), base, "label is the claim");

    let mut p = worked_payload();
    p["occurs_at"] = json!("2026-11-15");
    assert_ne!(content_hash(&p).unwrap(), base, "occurs_at is the claim");

    let mut p = worked_payload();
    p["participants"] = json!(["/person/sofia-almeida", "/organization/novabloom"]);
    assert_ne!(content_hash(&p).unwrap(), base, "participants are the claim");

    // Array order is claim-significant: JCS never reorders arrays.
    let mut p = worked_payload();
    p["participants"] = json!([
        "/organization/novabloom",
        "/person/sofia-almeida",
        "/organization/harborline-capital"
    ]);
    assert_ne!(
        content_hash(&p).unwrap(),
        base,
        "participant order is part of the claim"
    );
}

#[test]
fn provenance_mutation_changes_only_provenance_hash() {
    let env = envelope();
    let mut prov = worked_provenance();
    prov["branch"] = json!(4);
    let retold = Envelope::new(env.id.clone(), prov, worked_payload(), env.entity_ids.clone())
        .expect("retold envelope builds");
    assert_eq!(
        retold.content_hash, env.content_hash,
        "same claim, different telling"
    );
    assert_ne!(
        retold.provenance_hash, env.provenance_hash,
        "the telling changed"
    );
}

#[test]
fn missing_claim_field_is_a_typed_error_not_a_fabrication() {
    let mut p = worked_payload();
    p.as_object_mut().unwrap().remove("occurs_at");
    match content_hash(&p) {
        Err(SnagError::MissingPayloadField(f)) => assert_eq!(f, "occurs_at"),
        other => panic!("expected MissingPayloadField(occurs_at), got {other:?}"),
    }

    match content_hash(&json!(["not", "an", "object"])) {
        Err(SnagError::PayloadNotObject) => {}
        other => panic!("expected PayloadNotObject, got {other:?}"),
    }

    // Present-but-null is representable absence: it hashes (as null), it does not error.
    let mut p = worked_payload();
    p["label"] = Value::Null;
    let hashed = content_hash(&p).expect("null label is representable");
    assert_ne!(hashed, content_hash(&worked_payload()).unwrap());
}

#[test]
fn verify_detects_tampering_with_typed_mismatch() {
    let mut env = envelope();
    env.payload["label"] = json!("A different claim entirely");
    match env.verify() {
        Err(SnagError::HashMismatch { which, .. }) => assert_eq!(which, "content_hash"),
        other => panic!("expected content_hash mismatch, got {other:?}"),
    }

    let mut env = envelope();
    env.provenance["source"] = json!("someone.else");
    match env.verify() {
        Err(SnagError::HashMismatch { which, .. }) => assert_eq!(which, "provenance_hash"),
        other => panic!("expected provenance_hash mismatch, got {other:?}"),
    }
}

#[test]
fn envelope_round_trips_through_serde() {
    let env = envelope();
    let text = serde_json::to_string(&env).expect("envelope serializes");
    let back: Envelope = serde_json::from_str(&text).expect("envelope deserializes");
    assert_eq!(back, env);
    back.verify().expect("round-tripped envelope still verifies");
}

//! Source identity is independent of model grounding, probability, and event time.
//! These constructors are for trusted ingestion/authentication/producer boundaries,
//! never for accepting a model's self-description as evidence.
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SOURCE_CONTRACT: &str = "timepoint-source/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionMethod {
    ModelGenerated,
    UserProvided,
    SourceExtracted,
    SystemObserved,
    Derived,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimRole {
    Simulation,
    Assumption,
    Report,
    Observation,
    Inference,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    pub record_id: String,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAttribution {
    pub contract: String,
    pub method: ProductionMethod,
    pub role: ClaimRole,
    pub producer: String,
    pub evidence: Vec<EvidenceRef>,
}

impl SourceAttribution {
    pub fn generated(producer: impl Into<String>, role: ClaimRole) -> Self {
        Self {
            contract: SOURCE_CONTRACT.into(),
            method: ProductionMethod::ModelGenerated,
            role,
            producer: producer.into(),
            evidence: vec![],
        }
    }
    pub fn unknown() -> Self {
        Self {
            contract: SOURCE_CONTRACT.into(),
            method: ProductionMethod::Unknown,
            role: ClaimRole::Unknown,
            producer: String::new(),
            evidence: vec![],
        }
    }
    pub fn valid(&self) -> bool {
        self.contract == SOURCE_CONTRACT
            && (self.method == ProductionMethod::Unknown || !self.producer.trim().is_empty())
            && (!matches!(
                self.method,
                ProductionMethod::SourceExtracted | ProductionMethod::Derived
            ) || !self.evidence.is_empty())
            && self.evidence.iter().all(|e| {
                !e.record_id.trim().is_empty()
                    && matches!(
                        e.relation.as_str(),
                        "quotes" | "summarizes" | "derived_from" | "reports"
                    )
            })
            && match self.method {
                ProductionMethod::ModelGenerated => {
                    matches!(self.role, ClaimRole::Simulation | ClaimRole::Inference)
                }
                ProductionMethod::UserProvided => {
                    matches!(self.role, ClaimRole::Assumption | ClaimRole::Report)
                }
                ProductionMethod::SourceExtracted => self.role == ClaimRole::Report,
                ProductionMethod::SystemObserved => self.role == ClaimRole::Observation,
                ProductionMethod::Derived => self.role == ClaimRole::Inference,
                ProductionMethod::Unknown => self.role == ClaimRole::Unknown,
            }
    }
    pub fn label(&self) -> &'static str {
        if !self.valid() {
            return "SOURCE UNKNOWN";
        }
        match (&self.method, &self.role) {
            (ProductionMethod::ModelGenerated, ClaimRole::Simulation) => "SIMULATED",
            (ProductionMethod::ModelGenerated, _) => "MODEL INFERENCE",
            (ProductionMethod::UserProvided, ClaimRole::Assumption) => "USER-PROVIDED ASSUMPTION",
            (ProductionMethod::UserProvided, _) => "USER-ATTESTED",
            (ProductionMethod::SourceExtracted, _) => "SOURCE-REPORTED",
            (ProductionMethod::SystemObserved, _) => "SYSTEM-OBSERVED",
            (ProductionMethod::Derived, _) => "DERIVED",
            _ => "SOURCE UNKNOWN",
        }
    }
}

/// Recover only server-recorded legacy producer identities. A grounding flag or
/// model-supplied source URL is deliberately not an input to this decision.
pub fn source_from_provenance(provenance: &Value) -> SourceAttribution {
    let producer = provenance
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("");
    match producer {
        "pro.deep_sim" | "flash.quick_sim" | "flash.story_sim" => {
            SourceAttribution::generated(producer, ClaimRole::Simulation)
        }
        "pro.portal_sim" => SourceAttribution::generated(producer, ClaimRole::Inference),
        "user_attested" => {
            let actor = provenance
                .get("attested_by")
                .and_then(Value::as_str)
                .unwrap_or("");
            if actor.trim().is_empty() {
                return SourceAttribution::unknown();
            }
            SourceAttribution {
                contract: SOURCE_CONTRACT.into(),
                method: ProductionMethod::UserProvided,
                role: ClaimRole::Report,
                producer: actor.into(),
                evidence: vec![],
            }
        }
        _ => SourceAttribution::unknown(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn generated_grounded_claim_cannot_become_evidence() {
        for p in [0.0, 0.5, 1.0] {
            let s = source_from_provenance(
                &json!({"source":"pro.deep_sim", "grounding":"g", "confidence":p,
                "attribution":{"method":"source_extracted"}}),
            );
            assert_eq!(s.label(), "SIMULATED");
        }
        assert_eq!(
            source_from_provenance(&json!({"grounding":"g"})).label(),
            "SOURCE UNKNOWN"
        );
        assert_eq!(
            source_from_provenance(&json!({"source":"user_attested"})).label(),
            "SOURCE UNKNOWN"
        );
    }
    #[test]
    fn invalid_version_and_unreferenced_extraction_fail_closed() {
        let mut s = SourceAttribution::generated("test", ClaimRole::Simulation);
        s.contract = "future/2".into();
        assert_eq!(s.label(), "SOURCE UNKNOWN");
        s.contract = SOURCE_CONTRACT.into();
        s.method = ProductionMethod::SourceExtracted;
        s.role = ClaimRole::Report;
        assert_eq!(s.label(), "SOURCE UNKNOWN");
    }
}

//! The relations vocabulary, `tt-relations/1.0`: entity kinds, relation kinds
//! between entities, and the edge-statement contract.
//!
//! A separately versioned artifact (`bundle/relations-v1.0.json`), shipped next
//! to the taxonomy and never inside it: the taxonomy's bytes and version string
//! are what every stored classification cites, and adding a vocabulary to them
//! would make every published reading cite a release that is no longer loaded.
//! Proposal: docs/proposals/ENTITY-RELATIONS.md (§2.4 load rules, §2.5 contract).
//!
//! Two surfaces, both reject-never-repair:
//!
//! - **Load** ([`Vocabulary::load_from_file`] and friends). An invalid artifact
//!   is an error and MUST fail boot, exactly as an invalid taxonomy does. Every
//!   failure is collected, each under a stable rule code.
//! - **Edge statements** ([`Vocabulary::validate_edge`]). TT validates what a
//!   relation SAYS — relation, endpoint kinds, attributes — and nothing about
//!   who said it or how. `basis`, `score`, `sources` and `observed_at` belong to
//!   the consumer and are rejected as unknown keys, so provenance is stripped
//!   before TT is asked. TT never sees entity ids either: that the endpoints
//!   exist, that their stored kinds match, and that they are two different
//!   entities are the consumer's checks.
//!
//! Relation kinds connect entities in a consumer's registry. They are not edges
//! between taxonomy nodes and never enter the TT-SPEC §5 metric.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::TtError;

/// The schema id every relations artifact carries.
pub const RELATIONS_SCHEMA: &str = "tt-relations/1.0";

/// The attribute types this implementation can check. The artifact declares
/// its own `attribute_types`; one outside this set is refused at load, because
/// a type nobody can check would let any value through.
pub const ATTRIBUTE_TYPES: [&str; 2] = ["date", "text"];

/// The closed set of top-level keys in an edge statement (§2.5 rule 1).
pub const EDGE_KEYS: [&str; 5] = [
    "relation",
    "from_kind",
    "to_kind",
    "attributes",
    "vocabulary",
];

/// A `text` attribute is at most this many Unicode scalar values — not bytes,
/// not UTF-16 code units.
pub const TEXT_MAX_SCALARS: usize = 200;

const DIRECTIONS: [&str; 2] = ["directed", "symmetric"];
const NATURES: [&str; 2] = ["structural", "social"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityKind {
    pub id: String,
    pub label: String,
    pub definition: String,
    /// Retirement follows GOVERNANCE.md §3, as nodes do.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated_in: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecation_note: Option<String>,
}

impl EntityKind {
    pub fn is_retired(&self) -> bool {
        self.deprecated_in.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeSpec {
    #[serde(rename = "type")]
    pub ty: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationKind {
    pub id: String,
    pub label: String,
    /// A reading of the relation from its `to` end, for display. Present if and
    /// only if the relation is directed. It is never an id and never aliased.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse_label: Option<String>,
    /// `directed` or `symmetric`.
    pub direction: String,
    /// `structural` or `social`.
    pub nature: String,
    /// Allowed `[from_kind, to_kind]` pairs. For a symmetric relation either
    /// order of a pair is allowed.
    pub endpoints: Vec<[String; 2]>,
    /// The closed attribute schema.
    pub attributes: BTreeMap<String, AttributeSpec>,
    pub definition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated_in: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecation_note: Option<String>,
}

impl RelationKind {
    pub fn is_retired(&self) -> bool {
        self.deprecated_in.is_some()
    }

    pub fn is_symmetric(&self) -> bool {
        self.direction == "symmetric"
    }

    pub fn is_structural(&self) -> bool {
        self.nature == "structural"
    }

    /// Whether `from -> to` is an allowed pair. Order matters for a directed
    /// relation and not for a symmetric one; nothing is reordered either way.
    pub fn allows(&self, from: &str, to: &str) -> bool {
        self.endpoints
            .iter()
            .any(|[a, b]| (a == from && b == to) || (self.is_symmetric() && a == to && b == from))
    }
}

/// One load-rule failure. `rule` is stable and is what the conformance vectors
/// pin; `detail` is for people.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoadFailure {
    pub rule: &'static str,
    pub detail: String,
}

/// One edge-statement rejection. `code` is normative; `detail` is advisory
/// (vectors/relation-verdicts.json, `conformance`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EdgeRejection {
    pub code: &'static str,
    pub detail: String,
}

#[derive(Debug, Deserialize)]
pub struct Vocabulary {
    pub schema: String,
    pub version: String,
    /// One step back, `"<schema> v<version>"`; `None` for 1.0.0.
    #[serde(default)]
    pub supersedes: Option<String>,
    pub attribute_types: Vec<String>,
    pub entity_kinds: Vec<EntityKind>,
    pub relation_kinds: Vec<RelationKind>,
    /// The artifact as parsed, kept verbatim so a server can hand it out
    /// unchanged (`governance`, `respectful_modeling` are not modelled).
    #[serde(skip)]
    pub raw: Value,
}

impl Vocabulary {
    /// Load and validate a relations artifact from a file path. Invalid is an error.
    pub fn load_from_file(path: &str) -> Result<Self, TtError> {
        let raw = std::fs::read_to_string(path)?;
        Self::load_from_str(&raw)
    }

    /// Load and validate a relations artifact from JSON text. Invalid is an error.
    pub fn load_from_str(raw: &str) -> Result<Self, TtError> {
        let value: Value = serde_json::from_str(raw)?;
        Self::load_from_value(value)
    }

    /// Load and validate an already-parsed artifact. Invalid is an error whose
    /// message lists every failure as `rule: detail`.
    pub fn load_from_value(value: Value) -> Result<Self, TtError> {
        Self::try_from_value(value).map_err(|failures| {
            TtError::Invalid(
                failures
                    .iter()
                    .map(|f| format!("{}: {}", f.rule, f.detail))
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })
    }

    /// Load with the failures kept typed. A document of the wrong shape fails
    /// with the single rule `malformed`; otherwise every §2.4 rule is checked
    /// and every failure is returned.
    pub fn try_from_value(value: Value) -> Result<Self, Vec<LoadFailure>> {
        let mut vocab: Vocabulary = match serde_json::from_value(value.clone()) {
            Ok(v) => v,
            Err(e) => {
                return Err(vec![LoadFailure {
                    rule: "malformed",
                    detail: e.to_string(),
                }]);
            }
        };
        vocab.raw = value;
        let failures = vocab.check();
        if failures.is_empty() {
            Ok(vocab)
        } else {
            Err(failures)
        }
    }

    /// `"tt-relations/1.0 v<version>"`. An edge statement cites it; the
    /// validator stamps it.
    pub fn version_string(&self) -> String {
        format!("{} v{}", self.schema, self.version)
    }

    pub fn entity_kind(&self, id: &str) -> Option<&EntityKind> {
        self.entity_kinds.iter().find(|k| k.id == id)
    }

    pub fn relation_kind(&self, id: &str) -> Option<&RelationKind> {
        self.relation_kinds.iter().find(|r| r.id == id)
    }

    /// Every §2.4 load rule, all failures collected. Empty means valid.
    pub fn check(&self) -> Vec<LoadFailure> {
        let mut out: Vec<LoadFailure> = Vec::new();
        let mut fail = |rule: &'static str, detail: String| out.push(LoadFailure { rule, detail });

        // Rule 7: schema, version, one step back.
        if self.schema != RELATIONS_SCHEMA {
            fail(
                "bad-schema",
                format!("schema is `{}`, expected `{RELATIONS_SCHEMA}`", self.schema),
            );
        }
        if !is_semver(&self.version) {
            fail(
                "bad-version",
                format!("version `{}` is not MAJOR.MINOR.PATCH", self.version),
            );
        }
        match (self.version.as_str(), self.supersedes.as_deref()) {
            ("1.0.0", Some(s)) => fail(
                "bad-supersedes",
                format!("1.0.0 is the first release and supersedes nothing, not `{s}`"),
            ),
            ("1.0.0", None) => {}
            (_, None) => fail(
                "bad-supersedes",
                format!(
                    "version {} must name the release it supersedes",
                    self.version
                ),
            ),
            (_, Some(s)) if s == self.version_string() => {
                fail("bad-supersedes", format!("`{s}` supersedes itself"))
            }
            _ => {}
        }

        // Attribute types: only ones this implementation can check.
        for t in &self.attribute_types {
            if !ATTRIBUTE_TYPES.contains(&t.as_str()) {
                fail(
                    "unsupported-attribute-type",
                    format!("attribute type `{t}` is not one of date, text"),
                );
            }
        }

        // Rule 1: kebab-case, unique, disjoint.
        let kind_ids: HashSet<&str> = self.entity_kinds.iter().map(|k| k.id.as_str()).collect();
        let relation_ids: HashSet<&str> =
            self.relation_kinds.iter().map(|r| r.id.as_str()).collect();
        for (what, ids) in [
            (
                "entity kind",
                self.entity_kinds
                    .iter()
                    .map(|k| k.id.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "relation kind",
                self.relation_kinds
                    .iter()
                    .map(|r| r.id.as_str())
                    .collect::<Vec<_>>(),
            ),
        ] {
            let mut seen: HashSet<&str> = HashSet::new();
            for id in ids {
                if !is_kebab(id) {
                    fail(
                        "id-not-kebab",
                        format!("{what} id `{id}` is not kebab-case"),
                    );
                }
                if !seen.insert(id) {
                    fail("duplicate-id", format!("{what} id `{id}` appears twice"));
                }
            }
        }
        for r in &self.relation_kinds {
            if kind_ids.contains(r.id.as_str()) {
                fail(
                    "ids-not-disjoint",
                    format!("`{}` is both an entity kind and a relation kind", r.id),
                );
            }
        }

        // Rule 2: non-empty label and definition.
        for k in &self.entity_kinds {
            if k.label.trim().is_empty() {
                fail(
                    "empty-label",
                    format!("entity kind `{}` has no label", k.id),
                );
            }
            if k.definition.trim().is_empty() {
                fail(
                    "empty-definition",
                    format!("entity kind `{}` has no definition", k.id),
                );
            }
        }
        for r in &self.relation_kinds {
            if r.label.trim().is_empty() {
                fail(
                    "empty-label",
                    format!("relation kind `{}` has no label", r.id),
                );
            }
            if r.inverse_label
                .as_deref()
                .is_some_and(|l| l.trim().is_empty())
            {
                fail(
                    "empty-label",
                    format!("relation kind `{}` has an empty inverse_label", r.id),
                );
            }
            if r.definition.trim().is_empty() {
                fail(
                    "empty-definition",
                    format!("relation kind `{}` has no definition", r.id),
                );
            }
        }

        let attribute_types: HashSet<&str> =
            self.attribute_types.iter().map(String::as_str).collect();
        for r in &self.relation_kinds {
            // Rule 3: direction, nature, inverse_label iff directed.
            if !DIRECTIONS.contains(&r.direction.as_str()) {
                fail(
                    "bad-direction",
                    format!(
                        "relation kind `{}`: direction `{}` is not directed or symmetric",
                        r.id, r.direction
                    ),
                );
            } else if r.direction == "directed" && r.inverse_label.is_none() {
                fail(
                    "inverse-label-mismatch",
                    format!(
                        "relation kind `{}` is directed and has no inverse_label",
                        r.id
                    ),
                );
            } else if r.direction == "symmetric" && r.inverse_label.is_some() {
                fail(
                    "inverse-label-mismatch",
                    format!(
                        "relation kind `{}` is symmetric and has an inverse_label",
                        r.id
                    ),
                );
            }
            if !NATURES.contains(&r.nature.as_str()) {
                fail(
                    "bad-nature",
                    format!(
                        "relation kind `{}`: nature `{}` is not structural or social",
                        r.id, r.nature
                    ),
                );
            }

            // Rule 4: endpoints name existing kinds; a live relation names no
            // retired kind; no duplicate pair (unordered when symmetric).
            let mut pairs: BTreeSet<(&str, &str)> = BTreeSet::new();
            let mut retired_named: BTreeSet<&str> = BTreeSet::new();
            for [a, b] in &r.endpoints {
                for end in [a, b] {
                    match self.entity_kind(end) {
                        None => fail(
                            "unknown-endpoint-kind",
                            format!("relation kind `{}` names unknown entity kind `{end}`", r.id),
                        ),
                        Some(k) if k.is_retired() && !r.is_retired() => {
                            retired_named.insert(end.as_str());
                        }
                        Some(_) => {}
                    }
                }
                let key = if r.is_symmetric() && b < a {
                    (b.as_str(), a.as_str())
                } else {
                    (a.as_str(), b.as_str())
                };
                if !pairs.insert(key) {
                    fail(
                        "duplicate-endpoint-pair",
                        format!("relation kind `{}` lists {a} -> {b} twice", r.id),
                    );
                }
            }
            for k in retired_named {
                fail(
                    "retired-endpoint-kind",
                    format!(
                        "live relation kind `{}` names retired entity kind `{k}`",
                        r.id
                    ),
                );
            }

            // Rule 5: attribute names and types.
            for (name, spec) in &r.attributes {
                if !is_attribute_name(name) {
                    fail(
                        "bad-attribute-name",
                        format!(
                            "relation kind `{}`: attribute name `{name}` does not match ^[a-z][a-z0-9_]*$",
                            r.id
                        ),
                    );
                }
                if !attribute_types.contains(spec.ty.as_str()) {
                    fail(
                        "unknown-attribute-type",
                        format!(
                            "relation kind `{}`: attribute `{name}` has type `{}`, not in attribute_types",
                            r.id, spec.ty
                        ),
                    );
                }
            }
        }

        // Rule 6: retirement, GOVERNANCE §3, for both collections alike.
        let kinds: Vec<Retirable<'_>> = self
            .entity_kinds
            .iter()
            .map(|k| Retirable {
                id: &k.id,
                deprecated_in: k.deprecated_in.as_deref(),
                superseded_by: k.superseded_by.as_deref(),
                note: k.deprecation_note.as_deref(),
            })
            .collect();
        let relations: Vec<Retirable<'_>> = self
            .relation_kinds
            .iter()
            .map(|r| Retirable {
                id: &r.id,
                deprecated_in: r.deprecated_in.as_deref(),
                superseded_by: r.superseded_by.as_deref(),
                note: r.deprecation_note.as_deref(),
            })
            .collect();
        for (what, items, ids) in [
            ("entity kind", &kinds, &kind_ids),
            ("relation kind", &relations, &relation_ids),
        ] {
            for it in items.iter().filter(|it| it.deprecated_in.is_some()) {
                match it.superseded_by {
                    Some(s) if s == it.id => fail(
                        "self-supersession",
                        format!("{what} `{}` supersedes itself", it.id),
                    ),
                    Some(s) if !ids.contains(s) => fail(
                        "unknown-successor",
                        format!("{what} `{}` is superseded_by unknown `{s}`", it.id),
                    ),
                    None if it.note.is_none() => fail(
                        "retired-without-successor-or-note",
                        format!(
                            "{what} `{}` is retired with neither superseded_by nor deprecation_note",
                            it.id
                        ),
                    ),
                    _ => {}
                }
            }
            // A chain may pass through retired items; it may not close. Each
            // cycle is reported once, walked from its smallest id, so that the
            // failure list is the same in every implementation. A one-item loop
            // is `self-supersession`, already reported above.
            for it in items.iter().filter(|it| it.deprecated_in.is_some()) {
                let mut seen: Vec<&str> = Vec::new();
                let mut cur = Some(*it);
                while let Some(c) = cur {
                    if let Some(p) = seen.iter().position(|s| *s == c.id) {
                        let on_cycle = &seen[p..];
                        if p == 0 && on_cycle.len() > 1 && on_cycle.iter().all(|s| *s >= it.id) {
                            seen.push(c.id);
                            fail(
                                "supersession-cycle",
                                format!("{what} supersession cycle: {}", seen.join(" -> ")),
                            );
                        }
                        break;
                    }
                    seen.push(c.id);
                    cur = c
                        .superseded_by
                        .and_then(|s| items.iter().find(|x| x.id == s).copied());
                }
            }
        }

        out
    }

    /// Validate one edge statement (§2.5). `Ok` is the normalized statement —
    /// the input with `attributes` defaulted to `{}` and the `vocabulary` stamp
    /// set — and it validates again unchanged. `Err` is every failure, never
    /// only the first, and the statement is thrown back whole.
    pub fn validate_edge(&self, statement: &Value) -> Result<Value, Vec<EdgeRejection>> {
        let vs = self.version_string();
        let obj = match statement.as_object() {
            Some(o) => o,
            None => {
                return Err(vec![EdgeRejection {
                    code: "not-an-object",
                    detail: "edge statement must be a JSON object".to_owned(),
                }]);
            }
        };
        let mut errors: Vec<EdgeRejection> = Vec::new();
        let mut reject =
            |code: &'static str, detail: String| errors.push(EdgeRejection { code, detail });

        // Rule 1: the closed key set. Provenance is the consumer's.
        let mut unknown: Vec<&String> = obj
            .keys()
            .filter(|k| !EDGE_KEYS.contains(&k.as_str()))
            .collect();
        unknown.sort();
        for k in unknown {
            reject("unknown-key", format!("unknown top-level key `{k}`"));
        }

        // Rule 2: the relation exists and is live.
        let relation: Option<&RelationKind> = match string_field(obj, "relation") {
            Field::Absent => {
                reject("missing-field", "`relation` is required".to_owned());
                None
            }
            Field::NotString => {
                reject("field-not-string", "`relation` must be a string".to_owned());
                None
            }
            Field::Str(id) => match self.relation_kind(id) {
                None => {
                    reject(
                        "unknown-relation",
                        format!("no relation kind `{id}` in {vs}"),
                    );
                    None
                }
                Some(r) => {
                    if let Some(dep) = &r.deprecated_in {
                        reject(
                            "retired-relation",
                            format!(
                                "relation `{id}` retired in {dep}; {}",
                                successor_text(
                                    r.superseded_by.as_deref(),
                                    r.deprecation_note.as_deref()
                                )
                            ),
                        );
                    }
                    Some(r)
                }
            },
        };

        // Rule 3: both kinds exist and are live.
        let mut kinds: [Option<&str>; 2] = [None, None];
        for (slot, field) in ["from_kind", "to_kind"].into_iter().enumerate() {
            match string_field(obj, field) {
                Field::Absent => reject("missing-field", format!("`{field}` is required")),
                Field::NotString => {
                    reject("field-not-string", format!("`{field}` must be a string"))
                }
                Field::Str(id) => match self.entity_kind(id) {
                    None => reject(
                        "unknown-kind",
                        format!("`{field}`: no entity kind `{id}` in {vs}"),
                    ),
                    Some(k) => {
                        if let Some(dep) = &k.deprecated_in {
                            reject(
                                "retired-kind",
                                format!(
                                    "`{field}`: entity kind `{id}` retired in {dep}; {}",
                                    successor_text(
                                        k.superseded_by.as_deref(),
                                        k.deprecation_note.as_deref()
                                    )
                                ),
                            );
                        }
                        kinds[slot] = Some(id);
                    }
                },
            }
        }

        // Rule 4: the pair is allowed. Checked only when all three are known;
        // an unknown name is its own failure and a pair of it says nothing more.
        if let (Some(r), [Some(from), Some(to)]) = (relation, kinds)
            && !r.allows(from, to)
        {
            let (arrow, allowed) = pair_text(r);
            reject(
                "endpoint-pair-not-allowed",
                format!(
                    "`{}` does not allow {from} {arrow} {to}; allowed: {allowed}",
                    r.id
                ),
            );
        }

        // Rules 5 and 6: attributes against the relation's closed schema.
        let empty = Map::new();
        let attributes: Option<&Map<String, Value>> = match obj.get("attributes") {
            None => Some(&empty),
            Some(Value::Object(m)) => Some(m),
            Some(_) => {
                reject(
                    "attributes-not-object",
                    "`attributes` must be a JSON object".to_owned(),
                );
                None
            }
        };
        if let (Some(r), Some(attrs)) = (relation, attributes) {
            let mut dates: BTreeMap<&str, (DateBound, &str)> = BTreeMap::new();
            // Reported in sorted key order in every implementation. Sorted
            // explicitly: serde_json's `preserve_order` feature, if anything in
            // a consumer's build enables it, would otherwise change the order.
            let mut names: Vec<(&String, &Value)> = attrs.iter().collect();
            names.sort_by(|a, b| a.0.cmp(b.0));
            for (name, value) in names {
                match r.attributes.get(name) {
                    None => reject(
                        "unknown-attribute",
                        format!("`{}` has no attribute `{name}`", r.id),
                    ),
                    Some(spec) => match check_value(&spec.ty, value) {
                        Err(why) => reject("attribute-type", format!("`{name}`: {why}")),
                        Ok(Some(bound)) => {
                            dates.insert(name.as_str(), (bound, value.as_str().unwrap_or("")));
                        }
                        Ok(None) => {}
                    },
                }
            }
            for (name, spec) in &r.attributes {
                if spec.required && !attrs.contains_key(name) {
                    reject(
                        "missing-attribute",
                        format!("`{}` requires attribute `{name}`", r.id),
                    );
                }
            }
            if let (Some((from, from_raw)), Some((to, to_raw))) =
                (dates.get("valid_from"), dates.get("valid_to"))
                && from.earliest > to.latest
            {
                reject(
                    "tenure-order",
                    format!("`valid_from` {from_raw} begins after `valid_to` {to_raw} ends"),
                );
            }
        }

        // Rule 7: a cited vocabulary must be this one.
        match obj.get("vocabulary") {
            None => {}
            Some(Value::String(cited)) if *cited == vs => {}
            Some(Value::String(cited)) => reject(
                "vocabulary-mismatch",
                format!("edge cites `{cited}`, loaded vocabulary is `{vs}`"),
            ),
            Some(_) => reject(
                "vocabulary-mismatch",
                format!("`vocabulary` must be the string `{vs}`"),
            ),
        }

        if !errors.is_empty() {
            return Err(errors);
        }
        let mut normalized = obj.clone();
        normalized.insert(
            "attributes".to_owned(),
            Value::Object(attributes.cloned().unwrap_or_default()),
        );
        normalized.insert("vocabulary".to_owned(), Value::String(vs));
        Ok(Value::Object(normalized))
    }
}

#[derive(Clone, Copy)]
struct Retirable<'a> {
    id: &'a str,
    deprecated_in: Option<&'a str>,
    superseded_by: Option<&'a str>,
    note: Option<&'a str>,
}

enum Field<'a> {
    Absent,
    NotString,
    Str(&'a str),
}

fn string_field<'a>(obj: &'a Map<String, Value>, key: &str) -> Field<'a> {
    match obj.get(key) {
        None => Field::Absent,
        Some(Value::String(s)) => Field::Str(s),
        Some(_) => Field::NotString,
    }
}

/// What a caller holding a retired id should do next: the successor when there
/// is one, otherwise the note that says why nothing replaces it.
fn successor_text(superseded_by: Option<&str>, note: Option<&str>) -> String {
    match (superseded_by, note) {
        (Some(s), _) => format!("use `{s}`"),
        (None, Some(n)) => format!("no successor: {n}"),
        (None, None) => "no successor".to_owned(),
    }
}

fn pair_text(r: &RelationKind) -> (&'static str, String) {
    let arrow = if r.is_symmetric() { "--" } else { "->" };
    let allowed = r
        .endpoints
        .iter()
        .map(|[a, b]| format!("{a} {arrow} {b}"))
        .collect::<Vec<_>>()
        .join(", ");
    (arrow, allowed)
}

/// The earliest and latest calendar day a reduced-precision date covers, as
/// `(year, month, day)`, so that ordering is plain tuple ordering.
#[derive(Debug, Clone, Copy)]
struct DateBound {
    earliest: (u32, u32, u32),
    latest: (u32, u32, u32),
}

/// Check one attribute value against its type. `Ok(Some(_))` for a valid date.
fn check_value(ty: &str, value: &Value) -> Result<Option<DateBound>, String> {
    match ty {
        "date" => {
            let s = value
                .as_str()
                .ok_or_else(|| "a date must be a JSON string".to_owned())?;
            parse_date(s).map(Some).ok_or_else(|| {
                "not a date at year, month or day precision (YYYY, YYYY-MM or YYYY-MM-DD) \
                 naming a real calendar day with year >= 1"
                    .to_owned()
            })
        }
        "text" => {
            let s = value
                .as_str()
                .ok_or_else(|| "text must be a JSON string".to_owned())?;
            if s.chars().any(is_control) {
                return Err("text must not contain control characters".to_owned());
            }
            if s.trim().is_empty() {
                return Err("text must not be empty after trimming".to_owned());
            }
            let n = s.chars().count();
            if n > TEXT_MAX_SCALARS {
                return Err(format!(
                    "text is {n} Unicode scalar values; at most {TEXT_MAX_SCALARS}"
                ));
            }
            Ok(None)
        }
        // Unreachable for a loaded vocabulary: load refuses unknown types.
        other => Err(format!("attribute type `{other}` cannot be checked")),
    }
}

/// Unicode general category Cc.
fn is_control(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{1f}' | '\u{7f}'..='\u{9f}')
}

/// `YYYY`, `YYYY-MM` or `YYYY-MM-DD`, ASCII digits, proleptic Gregorian,
/// year >= 1, a real calendar day. Reduced precision is kept, never padded.
fn parse_date(s: &str) -> Option<DateBound> {
    let digits = |p: &str, n: usize| -> Option<u32> {
        (p.len() == n && p.bytes().all(|b| b.is_ascii_digit()))
            .then(|| p.parse().ok())
            .flatten()
    };
    let parts: Vec<&str> = s.split('-').collect();
    let year = digits(parts.first()?, 4)?;
    if year < 1 {
        return None;
    }
    match parts.len() {
        1 => Some(DateBound {
            earliest: (year, 1, 1),
            latest: (year, 12, 31),
        }),
        2 => {
            let month = digits(parts[1], 2)?;
            let last = days_in_month(year, month)?;
            Some(DateBound {
                earliest: (year, month, 1),
                latest: (year, month, last),
            })
        }
        3 => {
            let month = digits(parts[1], 2)?;
            let day = digits(parts[2], 2)?;
            let last = days_in_month(year, month)?;
            (1..=last).contains(&day).then_some(DateBound {
                earliest: (year, month, day),
                latest: (year, month, day),
            })
        }
        _ => None,
    }
}

fn days_in_month(year: u32, month: u32) -> Option<u32> {
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if leap { 29 } else { 28 }),
        _ => None,
    }
}

fn is_semver(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// `^[a-z][a-z0-9_]*$` without a regex dependency.
fn is_attribute_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// `^[a-z0-9]+(-[a-z0-9]+)*$`, the rule taxonomy node ids follow.
fn is_kebab(id: &str) -> bool {
    !id.is_empty()
        && id.split('-').all(|seg| {
            !seg.is_empty()
                && seg
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_keep_their_precision() {
        let b = parse_date("2019").unwrap();
        assert_eq!((b.earliest, b.latest), ((2019, 1, 1), (2019, 12, 31)));
        let b = parse_date("2024-02").unwrap();
        assert_eq!((b.earliest, b.latest), ((2024, 2, 1), (2024, 2, 29)));
        assert!(parse_date("2000-02-29").is_some());
        for bad in [
            "1900-02-29",
            "2023-02-29",
            "0000",
            "2022-13-01",
            "2022-7-01",
            "20220",
            "",
            "2022-",
            "2022-01-01-01",
            "+2022",
            "２０２２",
        ] {
            assert!(parse_date(bad).is_none(), "{bad:?} must not parse");
        }
    }

    /// python/tt_relations.py spells out the set `str::trim` uses (the
    /// Unicode White_Space property), because Python's own strip() differs.
    /// This pins the two to the same 25 code points.
    #[test]
    fn trim_is_the_white_space_property() {
        let spelled_out: Vec<u32> = [0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x20, 0x85, 0xa0, 0x1680]
            .into_iter()
            .chain(0x2000..=0x200a)
            .chain([0x2028, 0x2029, 0x202f, 0x205f, 0x3000])
            .collect();
        let rust: Vec<u32> = (0..=0x10ffff_u32)
            .filter_map(char::from_u32)
            .filter(|c| c.is_whitespace())
            .map(u32::from)
            .collect();
        assert_eq!(rust, spelled_out);
    }

    #[test]
    fn attribute_names_and_kebab_ids() {
        assert!(is_attribute_name("valid_from"));
        assert!(!is_attribute_name("ValidFrom"));
        assert!(!is_attribute_name("_x"));
        assert!(is_kebab("holds-office"));
        assert!(!is_kebab("holds_office"));
    }
}

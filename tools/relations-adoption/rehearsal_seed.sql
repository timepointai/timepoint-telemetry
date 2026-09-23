-- Synthetic rows for rehearsing relations_adoption.py on a DISPOSABLE database
-- built from timepoint-beta's migrations. Never run this against a real one.
-- Every name is synthetic. Every row exists to exercise one branch of the
-- report, and the figures it must produce are listed at the end.

BEGIN;

INSERT INTO entity.entities (id, kind, slug, display_name, content_hash, owner_id) VALUES
  ('00000000-0000-4000-8000-000000000001', 'person', 'person-a', 'Person A', 'sha256:synthetic-1', 'owner-1'),
  ('00000000-0000-4000-8000-000000000002', 'person', 'person-b', 'Person B', 'sha256:synthetic-2', 'owner-1'),
  ('00000000-0000-4000-8000-000000000003', 'person', 'person-c', 'Person C', 'sha256:synthetic-3', 'owner-2'),
  ('00000000-0000-4000-8000-000000000004', 'org',    'district-a', 'District A', 'sha256:synthetic-4', 'owner-1'),
  ('00000000-0000-4000-8000-000000000005', 'org',    'vendor-a', 'Vendor A', 'sha256:synthetic-5', 'owner-1'),
  ('00000000-0000-4000-8000-000000000006', 'market', 'btc-usd', 'BTC-USD', 'sha256:synthetic-6', 'owner-1');

INSERT INTO entity.assertions (entity_id, attribute, value_json, basis, source, observed_at, owner_id) VALUES
  ('00000000-0000-4000-8000-000000000001', 'current-role', '"Superintendent of District A"', 'GROUNDED', 'console_manual', '2026-01-01T00:00:00Z', 'owner-1'),
  ('00000000-0000-4000-8000-000000000002', 'current-role', '"Chair of the Board"', 'GROUNDED', 'https://example.org/minutes', '2026-01-02T00:00:00Z', 'owner-1'),
  ('00000000-0000-4000-8000-000000000003', 'current-role', '"Deputy"', 'GENERATED', 'model', '2026-01-03T00:00:00Z', 'owner-2'),
  ('00000000-0000-4000-8000-000000000001', 'works-at', '"District A"', 'ACCUMULATED', 'run', '2026-01-04T00:00:00Z', 'owner-1'),
  ('00000000-0000-4000-8000-000000000002', 'named-in-uploaded-document', '{"role": "Board member"}', 'GROUNDED', 'upload', '2026-01-05T00:00:00Z', 'owner-1'),
  ('00000000-0000-4000-8000-000000000002', 'employer-size', '"large"', 'GROUNDED', 'console_manual', '2026-01-06T00:00:00Z', 'owner-1');

-- cockpit_doc is the production kind: it is what beta writes today, with the
-- run's ties at the top level. frame is kept only for older rows: no code on
-- beta's cr/coord writes a `frame` artifact (frame is an LLM step there), so a
-- frame row, if one exists, predates that code.
INSERT INTO run.artifacts (run_id, kind, content, content_hash) VALUES
  ('00000000-0000-4000-8000-0000000000a1', 'frame',
   '{"ties": [{"a": "x", "b": "y"}, {"a": "x", "b": "z"}, {"a": "y", "b": "z"}]}', 'sha256:synthetic-a1f'),
  ('00000000-0000-4000-8000-0000000000a1', 'cockpit_doc',
   '{"ties": [{"a": "x", "b": "y"}], "appendix": {"moment_readings": {
       "m1": {"lens_a": {}, "lens_b": {}, "abstain": true, "bundle": "tt-ontology/1.0 v2.1.0"},
       "m2": {"lens_a": {}, "lens_b": {}, "abstain": true, "bundle": "tt-ontology/1.0 v2.0.0"},
       "m3": {"lens_a": {}}}}}', 'sha256:synthetic-a1c'),
  ('00000000-0000-4000-8000-0000000000b2', 'frame', '{"ties": []}', 'sha256:synthetic-b2f'),
  ('00000000-0000-4000-8000-0000000000b2', 'report', '{"answer": "no ties key"}', 'sha256:synthetic-b2r'),
  -- Hostile: a kind that rewrote the old tab-separated parser. Counted as (other).
  ('00000000-0000-4000-8000-0000000000b2', E'x\t1\n@@D1\nperson\t999999',
   '{"ties": [{"a": "x", "b": "y"}]}', 'sha256:synthetic-b2h');

INSERT INTO run.moments (id, run_id, t, label, occurs_at, participants, payload, provenance,
                         classification, grounding, content_hash, provenance_hash) VALUES
  ('m1', '00000000-0000-4000-8000-0000000000a1', 1, 'moment one', '2026-01-01',
   '["/person/person-a", "/org/district-a"]', '{}', '{}',
   '{"lens_a": {}, "lens_b": {}, "abstain": true, "bundle": "tt-ontology/1.0 v2.1.0"}', 'GROUNDED', 'sha256:m1c', 'sha256:m1p'),
  ('m2', '00000000-0000-4000-8000-0000000000a1', 2, 'moment two', '2026-01-02',
   '["/person/person-b", "/market/btc-usd", "/place/somewhere"]', '{}', '{}',
   '{"lens_a": {}, "lens_b": {}, "abstain": true, "bundle": "tt-ontology/1.0 v2.0.0"}', 'ACCUMULATED', 'sha256:m2c', 'sha256:m2p'),
  ('m3', '00000000-0000-4000-8000-0000000000a1', 3, 'moment three', '2026-01-03',
   '["not a path"]', '{}', '{}', '{"lens_a": {}}', 'INFERRED', 'sha256:m3c', 'sha256:m3p'),
  ('m4', '00000000-0000-4000-8000-0000000000b2', 1, 'moment four', '2026-01-04',
   '{}', '{}', '{}', NULL, 'DESIGNED_SILENCE', 'sha256:m4c', 'sha256:m4p'),
  -- Hostile: a participant path and a bundle stamp built to forge output.
  ('m5', '00000000-0000-4000-8000-0000000000b2', 2, 'moment five', '2026-01-05',
   jsonb_build_array(E'/person\t999999\n@@D1'), '{}', '{}',
   jsonb_build_object('bundle', E'\n'), 'INFERRED', 'sha256:m5c', 'sha256:m5p');

INSERT INTO tt.verdicts (moment_id, verdict, bundle_version, note, asserted_by) VALUES
  ('m1', 'supported', 'tt-ontology/1.0 v2.1.0', 'synthetic', 'rehearsal'),
  ('m2', 'unsupported', NULL, 'synthetic', 'rehearsal'),
  ('m3', 'contradicted', 'free text, not a version', 'synthetic', 'rehearsal'),
  ('m5', 'unsupported', E'tt-ontology/1.0 v2.1.0\n@@D1\nperson\t999999', 'synthetic', 'rehearsal'),
  -- Version-shaped but never published: counted as (other), never printed.
  ('m5', 'supported', 'personaldata/1.0 v1.2.3', 'synthetic', 'rehearsal');

COMMIT;

-- The report must say (the hostile rows only ever add to "(other)"):
--   D1 person=3 org=2 market=1 role=0 outside-TT-kinds=0
--   D2 current-role/GENERATED=1 current-role/GROUNDED=2 works-at/ACCUMULATED=1
--      named-in-uploaded-document/GROUNDED=1 (employer-size is not counted);
--      load-bearing 4
--   D3 cockpit_doc=1 (in 1 doc) frame=3 (in 2 docs) (other)=1 (in 1 doc);
--      the report artifact has no ties key
--   D4 /person=2 /org=1 /market=1 /role=0 /other=3; 1 moment whose participants are not an array
--   D5 moments: (other)=1 <unstamped>=1 v2.0.0=1 v2.1.0=1; readings: <unstamped>=1
--      v2.0.0=1 v2.1.0=1; verdicts: (other)=3 <unstamped>=1 v2.1.0=1;
--      citing another version: 6; unstamped: 3

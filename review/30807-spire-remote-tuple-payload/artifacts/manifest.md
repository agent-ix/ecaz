# Artifact Manifest: 30807 SPIRE Remote Tuple Payload

- head SHA: `cbdfe34820756a66a0f66defdf9e0058b82b5953`
- packet/topic: `30807-spire-remote-tuple-payload`
- lane: PG18 focused pgrx test
- fixture: `test_ec_spire_remote_search_tuple_payload_side_channel`
- storage format: default SPIRE test storage
- rerank mode: existing local heap candidate path
- isolated/shared surface: single local PG18 test relation; endpoint shape is
  the remote-node tuple-payload surface used by CustomScan fanout
- command:
  `script -q -c 'cargo test tuple_payload --lib' review/30807-spire-remote-tuple-payload/artifacts/cargo-test-tuple-payload-lib.log`
- timestamp: `2026-05-10T22:24:40-07:00`
- key result lines:
  - `test tests::pg_test_ec_spire_remote_search_tuple_payload_side_channel ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1605 filtered out`

- head SHA: `cbdfe34820756a66a0f66defdf9e0058b82b5953`
- packet/topic: `30807-spire-remote-tuple-payload`
- lane: static diff hygiene
- fixture: n/a
- storage format: n/a
- rerank mode: n/a
- isolated/shared surface: n/a
- command:
  `script -q -c 'git diff --check' review/30807-spire-remote-tuple-payload/artifacts/git-diff-check.log`
- timestamp: `2026-05-10T22:24:40-07:00`
- key result lines:
  - command exited successfully with no whitespace errors

---
packet: 30929
topic: spire-dml-pk-predicate-edge-fixture
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30929
head_sha: 2c2ca55f1c8f7554a870dca5ea0ab0d2358d37a0
---

# Artifact Manifest

## git-diff-check.log

- head SHA: `2c2ca55f1c8f7554a870dca5ea0ab0d2358d37a0`
- packet/topic: `30929-spire-dml-pk-predicate-edge-fixture`
- lane: Phase 12.6 negative DML classifier coverage
- fixture: DML PK predicate edge fixture
- storage format: N/A
- rerank mode: N/A
- isolated/shared surface: shared-table coordinator catalog fixture
- command: `git diff --check HEAD^ HEAD`
- timestamp: `2026-05-12T14:17:31-07:00`
- key result lines:
  - no whitespace errors reported

## cargo-fmt-check.log

- head SHA: `2c2ca55f1c8f7554a870dca5ea0ab0d2358d37a0`
- packet/topic: `30929-spire-dml-pk-predicate-edge-fixture`
- lane: Phase 12.6 negative DML classifier coverage
- fixture: DML PK predicate edge fixture
- storage format: N/A
- rerank mode: N/A
- isolated/shared surface: shared-table coordinator catalog fixture
- command: `cargo fmt --check`
- timestamp: `2026-05-12T14:17:31-07:00`
- key result lines:
  - command exited successfully
  - log contains the repo's existing stable-rustfmt warnings for unstable options

## cargo-pgrx-test-dml-pk-edge.log

- head SHA: `2c2ca55f1c8f7554a870dca5ea0ab0d2358d37a0`
- packet/topic: `30929-spire-dml-pk-predicate-edge-fixture`
- lane: Phase 12.6 negative DML classifier coverage
- fixture: `test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes`
- storage format: N/A
- rerank mode: N/A
- isolated/shared surface: shared-table coordinator catalog fixture
- command: `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes`
- timestamp: `2026-05-12T14:17:31-07:00`
- key result lines:
  - `test tests::pg_test_ec_spire_dml_frontdoor_rejects_pk_predicate_edge_shapes ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1684 filtered out`

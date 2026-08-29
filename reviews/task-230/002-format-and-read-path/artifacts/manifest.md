# Task 230 packet 002 artifact manifest

- Head SHA: `03a4015a2b6ae42ec3b6e92fcbfe4737d58db443`
- Task bucket: `reviews/task-230/002-format-and-read-path/`
- Packet: NULL projection and packet-002 closure checkpoint
- Timestamp: 2026-08-28 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: local PG18 callbacks plus pure
  format/read tests; descriptor V4, Graph V2, receipt V3, manifest V4, compact
  paired hot/cold heaps; production exact rerank; no corpus benchmark
- Isolation: focused pgrx tests create transaction-scoped one-index and
  three-owner source/index fixtures; no corpus or benchmark fixture was created

## Seq-10 NULL projection artifacts

All seq-10 artifacts below were produced at
`03a4015a2b6ae42ec3b6e92fcbfe4737d58db443` on 2026-08-28 PDT.

### `hot-cold-null-pg18-seq-10.log`

- Command: `cargo test --lib --no-default-features --features 'pg18 pg_test' test_distann_hot_cold_typed_materialization_and_visibility`
- Cited result: `1 passed; 0 failed`; the existing exact-vector/tier-laziness
  and fail-closed matrix still passes, and a canonical NULL handoff row proves
  byte-exact cold-only and mixed logical reconstruction.

### `format-check-seq-10.log`

- Command: `cargo fmt --all -- --check`
- Cited result: exit status 0 (stable-rustfmt nightly-option warnings are
  non-failures).

## Seq-09 production read-admission artifacts

All seq-09 artifacts below were produced at
`f4c8fcedfb3620e41105b33e364620941056ce0c` on 2026-08-28 PDT.

### `read-unit-tests-seq-09.log`

- Command: `cargo test --lib --no-default-features --features pg18 manifest_preserves_legacy_v2_and_round_trips_covered_v3_fingerprints` followed by the same command with `identified_tier_projection_echoes_requested_and_stored_identity`.
- Cited result: both focused tests pass; partial hot/cold manifest shape is not
  version-admitted, and ordinary/packed identified payload SQL binds and echoes
  TID plus `vec_id`.

### `compile-gates-seq-09.log`

- Command: three `cargo check --no-default-features` gates with features
  `pg18`, `pg18 pg_test`, and `pg18 distann-head-attribution-benchmark`.
- Cited result: all three gates exit 0.

### `hot-cold-read-pg18-seq-09.log`

- Command: `cargo test --lib --no-default-features --features 'pg18 pg_test' test_distann_hot_cold_typed_materialization_and_visibility`
- Cited result: `1 passed; 0 failed`; exact-vector physical ordinal, id/hot/cold/
  mixed typed reconstruction, tier-lazy counters, identity drift, half-pair
  failure, and both-missing behavior pass.

### `hot-cold-projection-pg18-seq-09.log`

- Command: `cargo test --lib --no-default-features --features 'pg18 pg_test' test_distann_hot_cold_projection_contract`
- Cited result: `1 passed; 0 failed`; the three-owner production CustomScan
  matrix passes with external cold-tier TOAST and local/remote, rescan,
  deepening, and forced-retry shapes.

### `sidecar-regression-pg18-seq-09.log`

- Command: `cargo test --lib --no-default-features --features 'pg18 pg_test' test_distann_payload_projection_contract`
- Cited result: `1 passed; 0 failed`; Task 229's three-owner payload-sidecar
  projection contract remains unchanged.

### `format-check-seq-09.log`

- Command: `cargo fmt --all -- --check`
- Cited result: exit status 0 (stable-rustfmt nightly-option warnings are
  non-failures).

### `clippy-seq-09.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Cited result: nonzero only for the same five pre-existing failures in
  `ambuild.rs:139`, `generation_descriptor.rs:872`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1195`, and `ec_distann_physical_lifecycle.rs:8661`; no
  failure is introduced by the seq-09 read implementation.

## Seq-08 receipt V3 / manifest V4 sealing artifacts

All seq-08 artifacts below were produced at
`5214b6d98a76340c9ceed38c95386c776da19286` on 2026-08-28 PDT.

### `receipt-manifest-tests-seq-08.log`

- Command: `cargo test --no-default-features --features pg18 manifest_v2::tests`
- Cited result: `8 passed; 0 failed; 1 ignored`; legacy V1/V2 and covered V2/V3
  bytes remain canonical, while receipt V3, manifest V4, fingerprint V4,
  mutual-exclusion, and digest-corruption checks pass.

### `on-disk-fixtures-seq-08.log`

- Command: `cargo test --no-default-features --features pg18 --test on_disk_fixtures distann_`
- Cited result: all 26 DistANN independent persisted-format fixtures pass,
  including field-by-field V3/V4 walkers, exact production re-encoding, and
  byte-swapped version rejection.

### `upgrade-matrix-seq-08.log`

- Command: `cargo test --no-default-features --features pg18 --test upgrade_matrix`
- Cited result: `2 passed; 0 failed` after adding exact 383-byte Ready V3 SQL
  admission.

### `hot-cold-seal-pg18-seq-08.log`

- Command: `cargo pgrx test pg18 test_distann_hot_cold_handoff_v2_locator`
- Cited result: focused callback test `1 passed; 0 failed`; Graph V2 locator
  integrity and paired row materialization still hold, then seal reconstructs
  the unchanged logical digest and emits Ready V3 with nonzero per-tier digests
  and exact positive heap bytes.

### `format-check-seq-08.log`

- Command: `cargo fmt --all -- --check`
- Cited result: exit status 0 (stable-rustfmt nightly-option warnings are
  non-failures).

### `clippy-seq-08.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Cited result: nonzero only for the same five pre-existing failures in
  `ambuild.rs:139`, `generation_descriptor.rs:872`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1069`, and `ec_distann_physical_lifecycle.rs:8314`; no
  failure is in a seq-08 touched line.

## Seq-07 hot/cold handoff artifacts

All seq-07 artifacts below were produced at
`885b86be0ac9d86ccd840fa42b40921108c1f4e3` on 2026-08-28 PDT.

### `hot-cold-handoff-pg18-seq-07.log`

- Command: `cargo pgrx test pg18 test_distann_hot_cold_handoff_v2_locator`
- Cited result: focused callback test `1 passed; 0 failed`; unchanged wire
  values are partitioned into compact hot/cold tuples, the graph hot TID joins
  the hot tuple, and the decoded Graph V2 trailer equals the inserted cold CTID.

### `legacy-handoff-pg18-seq-07.log`

- Command: `cargo pgrx test pg18 test_distann_stage_batch_atomic_replay_and_directory`
- Cited result: focused callback test `1 passed; 0 failed`; legacy full-row
  staging, exact replay, graph/directory insertion, and atomicity are preserved.

### `format-check-seq-07.log`

- Command: `cargo fmt --all -- --check`
- Produced after the seq-07 review verdict at `5547fca02`; reviewed code remains
  `885b86be0`.
- Cited result: exit status 0 (stable-rustfmt nightly-option warnings are
  non-failures).

### `clippy-seq-07.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Produced after the seq-07 review verdict at `5547fca02`; reviewed code remains
  `885b86be0`.
- Cited result: nonzero only for the same five pre-existing failures in
  `ambuild.rs:139`, `generation_descriptor.rs:872`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1069`, and `ec_distann_physical_lifecycle.rs:8289`; no
  failure is in a seq-07 touched line.

## Seq-06 relation-creation artifacts

All seq-06 artifacts below were produced at
`7751746599c0b100d137f92c2e13d3f65c194a1d` on 2026-08-28 PDT.

### `hot-cold-relation-pg18-seq-06.log`

- Command: `cargo pgrx test pg18 test_distann_hot_cold_relation_ddl_and_abort`
- Cited result: focused callback test `1 passed; 0 failed`; it creates paired
  hot/cold relations at the 1,536-dimension boundary, verifies catalog and DDL
  invariants plus exact formed-tuple sizing and replay, then proves abort drops
  all four generation relations.

### `row-layout-fixture-seq-06.log`

- Command: `cargo test --no-default-features --features pg18 --test on_disk_fixtures distann_row_tier_layout_v1_fixture`
- Cited result: `1 passed; 0 failed`; the independent decoder pins a Cold
  placement and the changed layout digest.

### `generation-descriptor-fixture-seq-06.log`

- Command: `cargo test --no-default-features --features pg18 --test on_disk_fixtures distann_generation_descriptor_v4_fixture`
- Cited result: `1 passed; 0 failed`; descriptor V4 binds the layout containing
  the Cold placement.

### `upgrade-matrix-seq-06.log`

- Command: `cargo test --no-default-features --features pg18 --test upgrade_matrix`
- Cited result: `2 passed; 0 failed`.

### `format-check-seq-06.log`

- Command: `cargo fmt --all -- --check`
- Cited result: exit status 0 (stable-rustfmt nightly-option warnings are
  non-failures).

### `clippy-seq-06.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Produced after the seq-06 review verdict at `0396b2069`; reviewed code remains
  `775174659`.
- Cited result: nonzero only for the same five pre-existing failures in
  `ambuild.rs:139`, `generation_descriptor.rs:872`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1069`, and `ec_distann_physical_lifecycle.rs:8202`; no
  failure is in a seq-06 touched line.

## Seq-05 descriptor V4/layout identity artifacts

All seq-05 artifacts below were produced at `1407d4504` on 2026-08-28 PDT.

### `row-layout-tests-seq-05.log`

- Command: `cargo test --no-default-features --features pg18 row_layout::tests`
- Cited result: `5 passed; 0 failed`

### `generation-descriptor-tests-seq-05.log`

- Command: `cargo test --no-default-features --features pg18 generation_descriptor_`
- Cited result: descriptor unit tests `3 passed; 0 failed`; independent fixture
  tests `3 passed; 0 failed`

### `row-layout-fixture-seq-05.log`

- Command: `cargo test --no-default-features --features pg18 --test on_disk_fixtures distann_row_tier_layout_v1_fixture`
- Cited result: `1 passed; 0 failed`

### `registration-digest-tests-seq-05.log`

- Command: `cargo test --no-default-features --features pg18 registration_digest_golden_binds_private_transport_fields`
- Cited result: `1 passed; 0 failed`

### `graph-v2-tests-seq-05.log`

- Command: `cargo test --no-default-features --features pg18 distann_physical_node`
- Cited result: `5 passed; 0 failed`

### `hot-cold-registration-pg18-seq-05.log`

- Command: `cargo pgrx test pg18 test_distann_begin_build_binds_hot_cold_row_layout`
- Cited result: focused callback test `1 passed; 0 failed`; it creates the
  hot/cold coordinator and participant, registers the participant, begins the
  build, and verifies exact replay.

### `format-check-seq-05.log`

- Command: `cargo fmt --all -- --check`
- Cited result: exit status 0 (stable-rustfmt nightly-option warnings are
  non-failures)

### `clippy-seq-05.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Cited result: nonzero only for the same five pre-existing failures in
  `ambuild.rs:139`, `generation_descriptor.rs:872`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1069`, and `ec_distann_physical_lifecycle.rs:8004`; no
  failure is in a seq-05 touched line.

## Seq-04 Graph V2 review-gap artifacts

### `graph-v2-tests-seq-04.log`

- Command: `cargo test --no-default-features --features pg18 distann_physical_node`
- Cited result: `5 passed; 0 failed`, including rejection of a valid cold
  locator by both V1 writers and V2-to-V1 pooled decode locator clearing.

### `graph-v2-fixtures-seq-04.log`

- Command: `cargo test --no-default-features --features pg18 --test on_disk_fixtures distann_physical_graph_record`
- Cited result: `2 passed; 0 failed`

### `format-check-seq-04.log`

- Command: `cargo fmt --all -- --check`
- Cited result: exit status 0 (stable-rustfmt nightly-option warnings are
  non-failures)

### `clippy-seq-04.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Cited result: nonzero only for the same five pre-existing failures in
  `ambuild.rs:139`, `generation_descriptor.rs:798`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1069`, and `ec_distann_physical_lifecycle.rs:7951`; no
  failure in `tuple.rs`.

## Seq-03 Graph V2 artifacts

### `graph-v2-tests-seq-03.log`

- Command: `cargo test --no-default-features --features pg18 distann_physical_node`
- Expected cited result: `3 passed; 0 failed`

### `graph-v2-fixtures-seq-03.log`

- Command: `cargo test --no-default-features --features pg18 --test on_disk_fixtures distann_physical_graph_record`
- Expected cited result: `2 passed; 0 failed`

### `format-check-seq-03.log`

- Command: `cargo fmt --all -- --check`
- Expected cited result: exit status 0 (stable-rustfmt nightly-option warnings
  are non-failures)

### `clippy-seq-03.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Expected cited result: nonzero only for the same five pre-existing failures
  recorded under seq-02; no failure in a Graph V2 touched line. The first local
  attempt found an MSRV error in `Option::is_none_or`; code `9b13d2aca` fixed it
  before these artifacts were regenerated.

## Seq-02 descriptor artifacts

### `row-layout-tests-seq-02.log`

- Command: `cargo test --no-default-features --features pg18 row_layout::tests`
- Expected cited result: `5 passed; 0 failed`

### `row-layout-reloption-test-seq-02.log`

- Command: `cargo test --no-default-features --features pg18 hot_payload_attnums_and_layout_are_canonical`
- Expected cited result: `1 passed; 0 failed`

### `format-check-seq-02.log`

- Command: `cargo fmt --all -- --check`
- Expected cited result: exit status 0 (the host's stable-rustfmt warnings about
  nightly-only import grouping are non-failures)

### `clippy-seq-02.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Expected cited result: nonzero because of five pre-existing failures outside
  the touched files: `ambuild.rs:139`, `generation_descriptor.rs:798`,
  `head_sample.rs:1818`, `remote_endpoint.rs:1069`, and
  `ec_distann_physical_lifecycle.rs:7951`.
- Checkpoint result: no clippy failure in `row_layout.rs`, `row_schema.rs`, or
  `options.rs`; reviewer seq-01's new `options.rs:1652` failure is absent.

All commands use the host's shared `CARGO_TARGET_DIR`; no runtime output is
written under the repository `target/` directory.

The seq-01 logs without a suffix remain the immutable validation artifacts for
code checkpoint `ef558a669`; seq-02 artifacts belong to `8faac4bad`.

# Task 230 packet 003 artifact manifest

- Head SHA: `deb245711ad1d2373b6f54903b918292e16e67d5`
- Task bucket: `reviews/task-230/003-lifecycle-and-dml/`
- Packet: payload-offset contract and CLI lint gate, seq-09
- Timestamp: 2026-08-29 America/Los_Angeles
- Lane / fixture / storage format / rerank mode: ecaz-cli unit validation;
  row-heap or descriptor V4 / Graph V2 hot/cold per-shape I/O attribution; no
  corpus benchmark or rerank measurement
- Isolation: pure CLI/suite harness validation; no dynamic fault fixture has
  run yet, because this runner extension is reviewed before the packet consumes it

## Seq-09 payload-offset contract and CLI lint gate artifacts

All seq-09 artifacts were produced at
`deb245711ad1d2373b6f54903b918292e16e67d5` on 2026-08-29 PDT.

### `cargo-fmt-seq-09.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `5736f20c03ab68c5dd6155bd6454835d337fde36c51da96539c7666067a9b2cd`

### `cargo-test-cli-seq-09.log`

- Command: `cargo test -p ecaz-cli task230_`
- Result: six passed, zero failed; 546 filtered out.
- SHA-256: `9917ebe23cbdd38daa5c50713b4b9725817e464a1bea9c7d01df8fbecb39ad5e`

### `cargo-clippy-seq-09.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing root-package findings;
  there is no finding at the seq-09 change.
- SHA-256: `21628b56ff2e29a2cd1fb2890aa2f644bd0eacb80064bdf141b1e1ae6dface76`

### `cargo-clippy-cli-seq-09.log`

- Command: `cargo clippy -p ecaz-cli --all-targets`
- Result: exit 0 with the pre-existing warning baseline; there is no finding
  at the seq-09 change.
- SHA-256: `b7564d6ab0c630c27f8107c4df82b7496922bbc1527cad1c163cfbcb2fbd00cc`

## Seq-08 lifecycle and fault-matrix harness artifacts

All seq-08 artifacts were produced at
`c4b96fe89e8cde09ed015ecccbe8cba33dd4c2d9` on 2026-08-29 PDT.

### `cargo-fmt-seq-08.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `a2edfecda929be428ef204afacc2a781da472a9665ce86474ac281e959706315`

### `cargo-clippy-seq-08.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings recorded in
  seq-01 through seq-07; no new finding in the seq-08 changes.
- SHA-256: `f17ef6e87ac4f9cd24f14c036328fec3f9cea5709c1accb65bee85135e3a3e7f`

### `cargo-test-cli-seq-08.log`

- Command: `cargo test -p ecaz-cli task230_`
- Result: six passed, zero failed; 546 filtered out. The new suite test covers
  typed read/write fault flags, packet-local expected artifacts, hot/cold
  expansion, and the four-node read-matrix minimum.
- SHA-256: `7f9b46c9ce2c7f0c9668718c4eb51b511d4ec9493963c7c32b8d58ed3b9c94cf`

## Seq-07 projection-mirror fix artifacts

All seq-07 artifacts were produced at
`95448fa7530f35346c03e76f11e43307a9e49e3c` on 2026-08-29 PDT.

### `cargo-fmt-seq-07.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `dd977680bc1bbbe23b5fcdc013a95b086e60b29962cd7335b69320adb9329c5c`

### `cargo-clippy-seq-07.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings recorded in
  seq-01 through seq-06; no new finding in the seq-07 changes.
- SHA-256: `193c5e74eb9222c74b47dd228d761053ff32334d3f139efd7e03aac80088e35a`

### `cargo-test-cli-seq-07.log`

- Command: `cargo test -p ecaz-cli task230_`
- Result: five passed, zero failed; 546 filtered out. The new table-driven test
  pins both control and candidate projections for all six shapes and proves
  cold-only has no hot projection.
- SHA-256: `47997d88d06dcb33f5da45070a1b3c0edf915bb7939eca5d43800ac42be6d47e`

## Seq-06 complete six-shape attribution artifacts

All seq-06 artifacts were produced at
`428ae9b943b34374be015e8f613b1e33923521da` on 2026-08-29 PDT.

### `cargo-fmt-seq-06.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `85352cf5dd94d4052c26da8eb265349f88fd9d6d8ee05659d30924c80d4704f8`

### `cargo-clippy-seq-06.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings recorded in
  seq-01 through seq-05; no new finding in the seq-06 changes.
- SHA-256: `46132229366aaa894a8cb462ae82e781baa998c147f5f1322cbae1900fcd5105`

### `cargo-test-cli-seq-06.log`

- Command: `cargo test -p ecaz-cli task230_`
- Result: four passed, zero failed; 546 filtered out. The suite expansion test
  additionally covers hot-scalar and exact-vector as hot-only shapes that do
  not require the Task 230 TOAST fixture.
- SHA-256: `6f61735d9ee16619a9374aef29629111937766b50890a7b145853aab303e7c42`

## Seq-05 row-tier I/O attribution artifacts

All seq-05 artifacts were produced at
`50701c2048550990e3e6ef6123743b94cd57c522` on 2026-08-29 PDT.

### `cargo-fmt-seq-05.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `43bba626ab655ef7ba80bc92c51810a973c574f5d943531090db239e1410c30c`

### `cargo-clippy-seq-05.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings recorded in
  seq-01 through seq-04; no new finding in the seq-05 changes.
- SHA-256: `14efc939b5661bb9ba917952e77131c52ba00f9e727ce7ba2a47d0d4e0afbcd0`

### `cargo-test-cli-seq-05.log`

- Command: `cargo test -p ecaz-cli task230_`
- Result: four passed, zero failed; 546 filtered out. The added counter test
  covers heap/TOAST/tidx read/hit subtraction and reset rejection.
- SHA-256: `83f84d5fed709edac56c37bc45f90152c0204878d37c5cb3c9689e4e7d8c2ad7`

## Seq-04 multinode and suite harness artifacts

All seq-04 artifacts were produced at
`41a01606063d2ffdee76d7d5a2ac3f4fb01ce3db` on 2026-08-29 PDT.

### `cargo-fmt-seq-04.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `b64aad102e7949f9f084c0f797c50a9afb82d7f2a35012597b4552674a6cd4a2`

### `cargo-clippy-seq-04.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings recorded in
  seq-01 through seq-03; no new finding in the seq-04 changes.
- SHA-256: `5dae0854b85758819a126e8185e6670021d6b20b6f202eb1d7fdb87cb7f3523d`

### `cargo-test-cli-seq-04.log`

- Command: `cargo test -p ecaz-cli task230_`
- Result: three passed, zero failed; 546 filtered out. The tests cover
  canonical hot attnums, complete hot/cold topology admission, and typed suite
  expansion.
- SHA-256: `e07de66c24c316a66338459501d4d6fc0892be962b2f5da24aecc580b2d07f94`

## Seq-03 retained-history and destructive-lifecycle artifacts

All seq-03 artifacts were produced at
`7ff55c0a3a9aab4d86d56d6c7d67e61f80801e7e` on 2026-08-29 PDT.

### `cargo-fmt-seq-03.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `5c6c9722c33ed227570ea8f0a5196d1a1e2e31f381ea62630eae55a5b303d8ea`

### `cargo-clippy-seq-03.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings recorded in
  seq-01 and seq-02; no new finding in the seq-03 changes.
- SHA-256: `0e9dd05f84b81f381b0d32f2c7c2b1f6f49332478d32d1baf52dade416a2f32b`

### `cargo-pgrx-test-lifecycle-seq-03.log`

- Command: `cargo pgrx test pg18 test_distann_hot_cold_d --no-default-features --features 'pg18 pg_test'`
- Result: two passed, zero failed; 2,642 filtered out. The callbacks prove the
  raw retained-tuple topology counts and hot/cold DROP, REINDEX, and rollback
  dependency behavior.
- SHA-256: `2ea12e029361139d62fd861f9235d60c839353b5dac8691e6498a59a29684cb1`

## Seq-02 topology artifacts

All seq-02 artifacts were produced at
`760ed15a75bb2f3ed499665b8dbc0b7b3cd92a3c` on 2026-08-29 PDT.

### `cargo-fmt-seq-02.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `bdcee12933694b90dbf4905fbea8a7d44c55e817841648513a37e48913c2a0d2`

### `cargo-clippy-seq-02.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings recorded in
  seq-01; no finding in `handoff.rs` or the new topology test.
- SHA-256: `8a8645b4026124507d16252044418fdf8b7dcc4c8050c74251e2b20e56d386af`

### `cargo-pgrx-test-topology-seq-02.log`

- Command: `cargo pgrx test pg18 test_distann_hot_cold_topology_reports_both_tiers --no-default-features --features 'pg18 pg_test'`
- Result: one passed, zero failed; 2,642 filtered out. The callback proves Graph
  V2 diagnostic admission, logical reconstruction/digest equality, separate
  hot/cold row counts and orphan counts, and both heap byte values.
- SHA-256: `04d98fa364adffad5dd31b9a097a8f75b39888ee23536a158c0bdb7ce4218d39`

## Seq-01 DML and reclaim artifacts

## `cargo-fmt-seq-01.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0. Stable-rustfmt nightly-option warnings are non-failures.
- SHA-256: `e5615dc4c940d0a399590f25e86421255bdff6b96b091fb02156f504cbc16851`

## `cargo-clippy-seq-01.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: exit 101 only for the same five pre-existing findings at
  `ambuild.rs:139`, `generation_descriptor.rs:872`, `head_sample.rs:1818`,
  `remote_endpoint.rs:1195`, and the existing sidecar assertion in
  `ec_distann_physical_lifecycle.rs:8835`. There is no new finding in
  `physical_dml.rs` or either new hot/cold test.
- SHA-256: `fb86e390ad91ef25796703fdc4c64b1af03a2bc01f394115e2f624f4d74adf8d`

## `cargo-pgrx-test-hot-cold-seq-01.log`

- Command: `cargo pgrx test pg18 test_distann_hot_cold_ --no-default-features --features 'pg18 pg_test'`
- Result: six passed, zero failed. The group includes relation DDL/abort,
  handoff V2 locator, projection contract, typed materialization/visibility,
  new DML atomicity, and new retire/reclaim rollback coverage.
- Key result: `6 passed; 0 failed; 2636 filtered out`.
- SHA-256: `7d2137f3ade4e686d78e21c81efb03057efa662e5d3d4233048ca6ca64d76336`

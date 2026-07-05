# Artifact manifest

- Head SHA: `ce378e208dfc151ac82acb3e7f8d3982ce3090cd`
- Task bucket: `reviews/task-39/026-rabitq-mutation`
- Lane: RaBitQ mutation campaign
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `rabitq-mutants-list.log`

- Command: `cargo mutants --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --list`
- Timestamp: 2026-05-19
- Key result: cargo-mutants listed 456 RaBitQ mutants.

### `rabitq-mutants-initial.log`

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/initial/rabitq.rs.mutants`
- Timestamp: 2026-05-19
- Key result: 456 mutants tested in 38m: 118 missed, 317 caught, 21 unviable.

### `rabitq-mutants-rerun.log`

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/rerun/rabitq.rs.mutants`
- Timestamp: 2026-05-19
- Key result: 456 mutants tested in 44m: 27 missed, 408 caught, 21 unviable.

### `rabitq-mutants-final.log`

- Command: `cargo mutants --in-place --package ecaz-careful-hardening --file hardening/careful/src/../../../src/quant/rabitq.rs --output reviews/task-39/026-rabitq-mutation/artifacts/final/rabitq.rs.mutants`
- Timestamp: 2026-05-19
- Key result: 455 mutants tested in 74m: 9 missed, 423 caught, 21 unviable, 2 timeouts.

### `initial/rabitq.rs.mutants/mutants.out/*`

- Produced by the initial mutation run.
- Key files: `missed.txt`, `caught.txt`, `unviable.txt`, `outcomes.json`.

### `rerun/rabitq.rs.mutants/mutants.out/*`

- Produced by the intermediate mutation run.
- Key files: `missed.txt`, `caught.txt`, `unviable.txt`, `outcomes.json`.

### `final/rabitq.rs.mutants/mutants.out/*`

- Produced by the final checkpoint mutation run.
- Key files: `missed.txt`, `timeout.txt`, `caught.txt`, `unviable.txt`,
  `outcomes.json`.

### `rabitq-focused-tests.log`

- Command: `cargo test --manifest-path hardening/careful/Cargo.toml --lib rabitq -- --nocapture`
- Timestamp: 2026-05-19
- Key result: 25 RaBitQ tests passed.

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Timestamp: 2026-05-19
- Key result: passed with pre-existing warnings.

### `diff-check.log`

- Command: `git diff --check`
- Timestamp: 2026-05-19
- Key result: passed.

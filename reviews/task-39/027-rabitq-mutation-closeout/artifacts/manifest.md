# Artifact manifest

- Head SHA: `2522248b6985d4edcb2f3ce81af766e4fe05014c`
- Task bucket: `reviews/task-39/027-rabitq-mutation-closeout`
- Lane: RaBitQ residual mutation closeout
- Fixture / storage format / rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `rabitq-focused-tests.log`

- Command: `cargo test --manifest-path hardening/careful/Cargo.toml
  --lib rabitq -- --nocapture`
- Timestamp: 2026-05-19
- Key result: 30 RaBitQ tests passed.

### `diff-check.log`

- Command: `git diff --check -- src/quant/rabitq.rs`
- Timestamp: 2026-05-19
- Key result: empty (passed).

### `targeted/rabitq-targeted-mutants.log`

- Command: pre-edit targeted run captured by the prior coder run on the
  former survivor lines (755 / 757 / 998 / 1132 pre-commit). Recorded
  here as the baseline that established the residual missed set.
- Timestamp: 2026-05-19
- Key result: 24 mutants tested in 10m: 4 missed, 20 caught (the 4
  missed entries are the closeout targets).

### `targeted/rabitq.rs.mutants/mutants.out/*`

- Produced by the pre-edit targeted run.
- Key files: `missed.txt`, `caught.txt`, `unviable.txt`,
  `outcomes.json`.

### `targeted-rerun/rabitq-targeted-rerun.log`

- Command: `cargo mutants --in-place --package ecaz-careful-hardening
  --file hardening/careful/src/../../../src/quant/rabitq.rs
  --re 'rabitq\.rs:(763|765|1006|1140):'
  --output reviews/task-39/027-rabitq-mutation-closeout/artifacts/targeted-rerun/rabitq.rs.mutants`
- Timestamp: 2026-05-19
- Key result: 14 mutants tested in 7m: 14 caught, 0 missed, 0 timeouts.
  Lines 763 / 765 / 1006 / 1140 are the post-commit positions of the
  former 755 / 757 / 998 / 1132 survivors after the
  `RABITQ_QUANT_RANGE` const and added doc comments shifted the source.

### `targeted-rerun/rabitq.rs.mutants/mutants.out/*`

- Produced by the post-edit targeted rerun.
- Key files: `missed.txt`, `caught.txt`, `unviable.txt`,
  `outcomes.json`.

### `full/rabitq-full-mutants.log`

- Command: `cargo mutants --package ecaz-careful-hardening
  --file hardening/careful/src/../../../src/quant/rabitq.rs
  --output reviews/task-39/027-rabitq-mutation-closeout/artifacts/full/rabitq.rs.mutants
  -j 4`
- Timestamp: 2026-05-19
- Key result: 447 mutants tested in 8m: 426 caught, 21 unviable,
  0 missed, 0 timeouts.

### `full/rabitq.rs.mutants/mutants.out/*`

- Produced by the full-file sweep.
- Key files: `missed.txt`, `timeout.txt`, `caught.txt`,
  `unviable.txt`, `outcomes.json`.

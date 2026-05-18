# Artifact manifest: Task 39 mutation lane careful target

- Head SHA: `19de557cf1f0c168f2df3c71cbd0408454ec4197`
- Task bucket: `reviews/task-39/006-mutation-lane-careful-target`
- Timestamp: `2026-05-18T21:42:13Z`
- Lane: Task 39 mutation runner wiring
- Fixture: pure-Rust careful harness package metadata
- Storage format: not applicable
- Rerank mode: not applicable
- Surface isolation: not applicable; no PostgreSQL benchmark surface was used

## Artifacts

### `bash-n-hardening.log`

- Command:
  `bash -n scripts/hardening.sh`
- Key result:
  command exited successfully.

### `make-n-mutants-simd.log`

- Command:
  `make -n mutants MUTANTS_MODULE=src/quant/simd.rs MUTANTS_OUTPUT_DIR=target/quality/mutants MUTANTS_JOBS=2`
- Key result:
  dry-run Make output still routes through `scripts/hardening.sh mutants --file src/quant/simd.rs`.

### `careful-simd-mutants-list.log`

- Command:
  `cargo mutants --package ecaz-careful-hardening --file 'hardening/careful/src/../../../src/quant/simd.rs' --list`
- Key result:
  lists 9 generated SIMD mutants through the careful package path.

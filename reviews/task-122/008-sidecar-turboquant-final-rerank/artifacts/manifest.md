# Task 122 Packet 008 Artifact Manifest

- head SHA: `d6ffc96c8f7613cc4e667cf9a4f600ddc12ee39c`
- task bucket: `reviews/task-122/008-sidecar-turboquant-final-rerank`
- timestamp: `2026-06-27T14:38:09Z`
- scope: benchmark harness code checkpoint
- task phases: Phase 3 (`TQ as candidate reducer before f32 rerank`) and Phase 7 (`Correct comparator matrix`)
- changed files:
  - `crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs`
  - `crates/ecaz-cli/src/commands/bench/suite.rs`

## Commands

Validation:

```sh
cargo test -p ecaz-cli sidecar_rerank > reviews/task-122/008-sidecar-turboquant-final-rerank/artifacts/cargo-test-ecaz-cli-sidecar-rerank.log 2>&1
```

## Artifacts

- `cargo-test-ecaz-cli-sidecar-rerank.log`: focused ecaz-cli unit test output.

No corpus TSVs, truth caches, operational logs, or generated benchmark data are
committed in this packet.

## Key Result Lines

```text
running 7 tests
test commands::bench::sidecar_rerank::tests::turboquant4_sidecar_variant_has_stable_label ... ok
test commands::bench::sidecar_rerank::tests::final_f32_rerank_off_preserves_sidecar_order ... ok
test commands::bench::sidecar_rerank::tests::final_f32_rerank_rescores_only_sidecar_prefix ... ok
test commands::bench::suite::tests::expands_sidecar_rerank_with_variants ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 405 filtered out; finished in 0.00s
```

The validation log also includes known PostgreSQL header warnings from
`pg18_pgstat_shim.c`; they did not fail the build.

## Measurement Status

This packet does not claim a TQ win or close Task 122. It adds the comparator
surface needed for the next `ecaz bench suite` matrix:

- IVF/RaBitQ `rerank=off` candidate frontier.
- Sidecar variants including `f32`, `rabitq8`, and `turboquant4`.
- Optional final exact f32 rerank over the sidecar top-M via
  `--final-rerank-k`.
- Required follow-up scales: 10k, 50k, and 100k with recall, latency, and
  storage/sidecar bytes evidence.

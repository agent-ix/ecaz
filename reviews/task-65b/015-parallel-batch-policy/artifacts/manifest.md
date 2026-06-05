# Task 65b Packet 015 Artifact Manifest

- head SHA: `76777656cd575182d0ea782b148c97250a8fa661`
- task bucket: `reviews/task-65b/015-parallel-batch-policy`
- timestamp: `2026-06-05T19:15:45Z`
- lane: local Rust validation, PG18 feature set
- storage format: not applicable; no corpus measurement run in this packet
- rerank mode: not applicable
- isolation: source-only policy/test slice

## Code Validation

| command | result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo check -p ecaz --lib --no-default-features --features pg18` | passed |
| `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b_` | passed, 6 tests |
| `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana::tests::task65b_` | passed, 5 tests |

## Key Source Assertions

- `BuildParallelStats` now records both requested and effective batch size.
- `effective_parallel_batch_size` caps small builds (`n <= 10000`) at batch
  `64`.
- `task65b_small_build_caps_effective_batch_size` verifies requested `96`
  becomes effective `64` for a small build.
- `ec_diskann_ambuild_timing` now emits requested batch, effective batch,
  alpha-growth-disabled, and stale-read ppm fields.
- `ec_diskann_parallel_build_policy` NOTICE exposes the policy before the
  build graph phase runs.

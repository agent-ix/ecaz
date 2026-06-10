# Task 94 Phase 4 Manifest

- head SHA: `f67107166e6dc1485b1a884250c52c79e1d9cc32`
- task bucket: `reviews/task-94/`
- packet path: `reviews/task-94/004-grouped-pq-sve-backend/`
- phase: Phase 4 SVE/SVE2 backend
- lane: LUT kernel family / grouped-PQ PqFastScan
- timestamp: `2026-06-09T17:13:24Z`
- code checkpoint: `f67107166 Add grouped-PQ SVE block backend`

## Changes

- Replaced the grouped-PQ SVE fallback stub with an aarch64 SVE/SVE2 backend.
- The backend keeps scalar LUT gathers, then uses a vector-length-agnostic SVE
  accumulation loop over the 32 candidate scores.
- Runtime dispatch returns `Isa::Sve2` on SVE2 hosts, `Isa::Sve` on base-SVE
  hosts, and `Isa::Scalar` on unsupported hosts.
- Added a test hook for SVE/SVE2 parity when the local host exposes SVE.
- Added a runtime SVE vector-lane helper backed by `cntw` for future Graviton-4
  evidence packets.

## Validation

Artifacts:

- `test-grouped-pq-block.log`
- `sve-asm-probe.log`
- `aarch64-cargo-check-blocked.log`

Commands:

```text
cargo test grouped_pq_block --lib
rustc --target aarch64-unknown-linux-gnu --emit=obj /tmp/grouped_pq_sve_backend_probe.rs -o /tmp/grouped_pq_sve_backend_probe.o
cargo check --target aarch64-unknown-linux-gnu --lib
```

Results:

- Focused local tests: 7 passed, 0 failed.
- Standalone representative SVE backend probe: compiled to an aarch64 object.
- Full aarch64 cargo check: blocked before this crate by missing
  `aarch64-linux-gnu-gcc` for `ring`.

## Runtime Vector Length

Not measured in this packet. This local host is not the approved Graviton-4
runtime lane, and no AWS run was performed. The code includes
`runtime_sve_vector_lanes_for_test()` for future Graviton-4 evidence, where the
manifest must record the actual runtime value verbatim.

No CI, AWS, or benchmark runs were performed.

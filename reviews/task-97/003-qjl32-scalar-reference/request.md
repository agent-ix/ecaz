# Task 97 Packet 003: qjl32 Scalar Reference

This packet implements the scalar qjl32 reference family for the clarified Task
97 path: current production QJL-active TurboQuant at canonical `bits=4` and a
non-tiled dimension such as 1024. It does not add a 2-bit TQ mode, a new storage
surface, or any AM call-site registration yet.

## Changes

- Adds `src/quant/qjl32/{mod,scalar,neon,sve,avx2}.rs`.
- Implements scalar qjl32 scoring for packed `[mse_packed][qjl_packed]` codes:
  3-bit MSE centroid indices plus one QJL residual sign bit per dimension.
- Adds fallback ISA stubs that delegate to scalar and return `Isa::Scalar`.
- Adds `score_turboquant_qjl_batch_for` in CandidateBatch with:
  - width gate at 32 candidates,
  - scalar tail attribution under `(surface, turboquant_qjl, scalar)`,
  - kernel-row attribution under the returned ISA,
  - shape rejection for the 1536d no-QJL lane before counters are recorded.
- Updates the Task 97 status pointer to this scalar-reference packet.

## Validation

- `cargo test qjl32 --lib -- --color never`
  - 4 passed; 0 failed
- `cargo test candidate_batch --lib -- --color never`
  - 17 passed; 0 failed

Logs are under `artifacts/`.

## Review Request

Please review the scalar qjl32 reference, the QJL-active 4-bit shape gates, and
the `turboquant_qjl` counter attribution. If accepted, the next checkpoint will
move to the first real ISA kernel slice.

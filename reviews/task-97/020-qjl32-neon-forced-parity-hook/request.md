# Task 97 Packet 020: qjl32 Forced NEON Parity Hook

This packet closes a local evidence gap before the deferred Graviton 4 run.
Packet 007's reviewer checklist carries forward a requirement that NEON parity
execute for real on the approved ARM lane. Before this slice, qjl32 had forced
AVX2 and SVE test hooks, but NEON could only be reached through normal runtime
dispatch and therefore fell back to scalar on the local Intel host.

## Changes

- Adds `score_block32_neon_for_test` in `src/quant/qjl32/neon.rs`.
- Adds `score_turboquant_qjl_block32_neon_for_test` in
  `src/quant/qjl32/mod.rs`.
- Adds
  `qjl32_neon_block32_matches_pre_slice_scorer_tolerance_when_available`.

The test hook is available only under `#[cfg(test)]`. It returns `None` on
non-AArch64 hosts, and on AArch64 it requires runtime NEON detection, calls the
NEON scorer directly, returns `Isa::Neon`, and compares against the pre-slice
scalar reference under the existing 4 ULP qjl32 tolerance contract.

## Validation

- `cargo fmt --check`: passed
- `cargo test qjl32 --lib -- --color never`: 12 passed, 0 failed
- `git diff --check`: passed

Logs are under `artifacts/`.

## Not Run

- No GitHub CI.
- No AWS tests.
- No AWS benchmarks.

The Graviton 4 evidence packet still needs explicit approval before running the
on-host NEON/SVE2 runtime checks and `[block-kernel-counters]` suite evidence.

## Review Request

Please review whether this forced NEON parity hook is the right local
preparation for the Task 97 Graviton 4 checklist, specifically the carry-forward
requirement that NEON parity execute for real on ARM.

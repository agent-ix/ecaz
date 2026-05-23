# Review Request: Hadamard AVX2 Unsafe Stage Consolidation

## Scope

This packet reviews commit `610172d0e4e8e7ca7d36edd8d49b50faa8623bea` (`Consolidate Hadamard AVX2 unsafe stages`).

The slice consolidates repeated AVX2 unsafe blocks in the Fast Walsh-Hadamard Transform implementation used by quantization/RaBitQ paths. It does not change dispatch, target-feature gating, transform stages, or lane ordering.

## Unsafe Burndown

- Replaces three separate exact-size AVX2 branches with one selected two-level tiling boundary.
- Consolidates bootstrap/stage target-feature calls that share the same AVX2 and tiling invariants.
- Consolidates AVX2 stage pointer arithmetic, loads, arithmetic, and stores into per-lane-region blocks.
- Consolidates recursive AVX2 block-transform calls under their existing per-width target-feature contracts.

Unsafe ledger movement:

- previous packet 179 ledger: `1841`
- packet 180 ledger: `1829`
- net reduction: `12`

High-signal file counts from `make unsafe-block-count`:

- `src/quant/hadamard.rs`: `43 -> 31`

## Validation

Packet-local artifacts are under `reviews/task-50/180-hadamard-avx2-unsafe-stage-consolidation/artifacts/`.

Passed:

- `cargo-check-pg18-bench.log`
- `cargo-check-pg18-pg-test.log`
- `cargo-test-hadamard-pg18-no-run.log`
- `git-diff-check.log`
- `rustfmt-hadamard-check.log`
- `unsafe-block-count.log`
- `unsafe-ledger-generate.log`
- `unsafe-ledger-check.log`

Blocked:

- `cargo-test-hadamard-pg18-run-blocked.log`: the filtered unit-test binary built, then failed before running tests with an unresolved PostgreSQL symbol (`LockBuffer`) when invoked directly through `cargo test`.

## Reviewer Focus

Please check that each consolidated AVX2 block still has a single coherent target-feature/tiling/pointer-bounds contract, and that no scalar or NEON behavior changed.

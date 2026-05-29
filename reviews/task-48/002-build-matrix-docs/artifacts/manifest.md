# Packet 002 — Task 48: build matrix docs

## Head

- Task bucket: `reviews/task-48/`
- Packet path: `reviews/task-48/002-build-matrix-docs/`
- Validation head SHA: `ddb51741e`
- Branch: `main`
- Surface under validation: new file `docs/build-matrix.md`.
  Documentation-only.

## Diff summary

- `docs/build-matrix.md`: +114 lines (new file).

No code change. The doc references existing surfaces:
- `make soak DURATION=…` — partial; the kickoff harness lives at
  `crates/ecaz-cli/src/commands/stress/soak_quant_cache.rs` from
  packet 001.
- `make resource-exhaustion` — named, implementation deferred to
  a follow-up slice.
- `make endian-qemu` — named, implementation deferred (depends on
  Task 42 fixtures already present under `fixtures/m5_*`).
- `make ci-matrix-local` — named, implementation deferred.

## Key result lines cited by request.md

- Matrix table at `docs/build-matrix.md` rows: 5 target triples
  with PG versions, toolchains, cadence, and notes columns.
- Resource-exhaustion table: 6 scenarios from Task 48 §Scope
  bullet 4.
- Cadence policy table: per-PR / nightly / weekly / pre-release
  drivers + failure policy.

## Task 48 progress after this slice

| # | §Exit Criterion | Status |
|---|---|---|
| 1 | CI matrix covers aarch64-darwin, x86_64-linux, aarch64-linux, pg17, pg18 | 0% |
| 2 | `make soak DURATION=24h` weekly | partial (kickoff harness only) |
| 3 | `make resource-exhaustion` nightly | 0% |
| 4 | `docs/build-matrix.md` documents matrix, cadence, policy | **✓ done (this)** |

Task 48 ≈ 25% complete (1 of 4 gates closed; one partial).

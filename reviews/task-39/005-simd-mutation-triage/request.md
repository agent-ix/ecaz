# Review request: SIMD coverage and mutation triage

## Scope

This packet closes the Task 39 `src/quant/simd.rs` coverage gap from packet `004` and provides the first real mutation triage packet.

Code checkpoints:

- `e1b8a77a`: adds focused SIMD backend override tests, makes `hardening/careful` a workspace test package, routes pure-Rust mutation targets to that package, and raises the `quant/simd.rs` coverage baseline.
- `f82589c1`: extracts the x86 AVX2/FMA boolean gate so ARM can kill the `&&` to `||` mutation.

## Key results

Coverage from `artifacts/coverage/summary.txt`:

- `quant/simd.rs`: `95.18%` line coverage, up from `48.00%` in packet `004`
- merged total: `4.58%`

Mutation from `artifacts/mutants-careful-inplace-run.log`:

- `9 mutants tested in 19s: 6 caught, 3 unviable`
- `0 missed`
- `0 timed out`

Validation artifacts:

- `artifacts/careful-simd-tests.log`: `6 passed; 0 failed`
- `artifacts/coverage-baseline-check.log`: baseline still complete for 40 critical paths
- `artifacts/coverage-delta-check.log`: raised baseline passes
- `triage.md`: one table for all SIMD mutation outcomes; no survivors

## Notes

The mutation command targets the `ecaz-careful-hardening` package and uses the path reported by cargo-mutants for the careful package:

`cargo mutants --in-place --package ecaz-careful-hardening --file 'hardening/careful/src/../../../src/quant/simd.rs' --output reviews/task-39/005-simd-mutation-triage/artifacts/simd-careful-inplace.mutants`

`--in-place` was used for the packet run because this checkout has large untracked benchmark artifacts; scratch-copy mode tried to copy those local files. CI can continue to use scratch mode because those local artifacts are not present there.

## Remaining Task 39 gaps

- Mutation triage has only been completed for `src/quant/simd.rs`; `src/quant/prod.rs` and the rest of the target list still need runs and survivor triage.
- `quant/mod.rs`, AM pages, SPIRE storage/coordinator paths, DiskANN build/routine/scan, planner cost callbacks, and storage guards still need coverage work.
- Flake-hunt still needs burn-in evidence.

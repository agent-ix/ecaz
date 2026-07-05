# Packet 001 — Task 46 kickoff: first structured-fuzz target

## Head

- Task bucket: `reviews/task-46/`
- Packet path: `reviews/task-46/001-structured-fuzz-kickoff/`
- Validation head SHA: `a079f1e8bc10d4137ce0e65adf3aef77a04542d1` (main)
- Branch: `main`
- Surface under validation: MSE index pack/unpack
  (`src/quant/prod.rs:1567` `pack_mse_indices`,
   `src/quant/prod.rs:1663` `unpack_mse_indices`).
- Storage format / fixture: N/A — fuzz target generates its own input
  via `arbitrary::Arbitrary` derive.
- Rerank mode / lane: N/A — coverage and round-trip property assertion,
  not a recall/latency benchmark.
- Surface isolation: single-process libFuzzer binary, no PostgreSQL.

## What changed

| Path | Kind | Purpose |
|---|---|---|
| `fuzz/Cargo.toml` | manifest | adds `arbitrary` dep and registers the new bin |
| `fuzz/src/lib.rs` | re-export | exposes `pack_mse_indices` via `bench_api` |
| `fuzz/fuzz_targets/unpack_mse_structured.rs` | target | new structured target |

## Artifacts

### fuzz-unpack-mse-structured-10s.log

- Command:
  `PATH=$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH
   RUSTUP_TOOLCHAIN=nightly cargo fuzz run fuzz_unpack_mse_structured
   -- -max_total_time=10 -print_final_stats=1`
- Timestamp: 2026-05-25
- Key result lines (cited by request.md):
  - `#2832801 DONE   cov: 213 ft: 681 corp: 85/11163b lim: 4096 exec/s: 257527`
  - `stat::number_of_executed_units: 2832801`
  - `stat::new_units_added:          57`
  - `Done 2832801 runs in 11 second(s)`
- Result: clean exit (0), no crashes, 57 new units discovered in the
  10s window.

### fuzz-unpack-mse-raw-baseline-10s.log

- Command:
  `PATH=$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH
   RUSTUP_TOOLCHAIN=nightly cargo fuzz run fuzz_unpack_mse
   -- -max_total_time=10 -print_final_stats=1`
- Timestamp: 2026-05-25 (immediately after the structured run)
- Key result lines (cited by request.md):
  - `#8981766 DONE   cov: 40 ft: 91 corp: 24/381b lim: 4096 exec/s: 816524`
  - `stat::number_of_executed_units: 8981766`
  - `stat::new_units_added:          0`
  - `Done 8981766 runs in 11 second(s)`
- Result: clean exit (0), no crashes, raw target is saturated against
  its 35-file seed corpus — zero new units discovered in the same 10s
  window.

## Comparison cited by request.md

| metric | raw | structured | ratio |
|---|---:|---:|---:|
| cov  | 40  | 213 | 5.33× |
| ft   | 91  | 681 | 7.48× |
| corp | 24  | 85  | 3.54× |
| new units in 10s | 0 | 57 | n/a (raw saturated) |

`cov` and `ft` ratios clear the Task 46 §Validation ≥ 5× threshold for
this surface.

## Notes

- The structured target's exec/sec (257k) is lower than the raw target's
  (816k) because each iteration pays for an Arbitrary draw plus a real
  encode → decode round-trip; this is expected and is the trade
  Task 46 §Why describes ("same code coverage in a fraction of the time
  *of mutations*", not of wall-clock executions).
- `fuzz/corpus/fuzz_unpack_mse_structured/` was populated to 125 files
  by the runs but `fuzz/corpus` is gitignored (`.gitignore:4`). Corpus
  minimization + committing is deferred to a follow-up packet (Task 46
  §Approach 5).

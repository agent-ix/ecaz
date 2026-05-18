# Review Request: C1 Task16 TurboQuant Int8 Exact-Score Experiment

Current head at execution: `e0ba7ee`

## Context

Packet `433` closed the "measure all remaining scorer options" step for task 16.
Its result was:

- lever 4 full LUT: no win
- lever 4 tiled LUT: worse
- lever 5 int8 approx: the only remaining scorer option with a real speed signal

This slice does not claim the task-16 decision is finished. It lands the first
real scan-path experiment for that remaining lever so it can be measured on the
actual TurboQuant runtime instead of only in the offline scorer study.

## Scope

Added an opt-in TurboQuant scan scoring experiment:

- env: `TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE`
- default: `exact`
- experimental override: `int8_approx`

The important scope constraint is that this is **opt-in** and **default-off**.
Current behavior stays unchanged unless the env is set.

## Implementation

### 1. Prepared scan state can now hold a TurboQuant int8 query

`src/am/scan.rs` now stores:

- the existing prepared exact query
- the existing prepared binary-sign query
- an optional prepared int8 query for the TurboQuant no-QJL `1536 @ 4-bit` lane

That state is allocated and freed with the rest of the scan-local prepared-query
state.

### 2. The experiment is gated to the current task-16 lane

When `TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE=int8_approx` is requested:

- TurboQuant scans prepare `Int8ApproxNoQjl4BitQuery`
- unsupported lanes error immediately

The validation gate is explicit:

- only the no-QJL 4-bit TurboQuant lane may use this override

### 3. The scan-local exact-score path can now dispatch to int8 approx

`score_scan_element_result(...)` now does:

- default path: current `score_ip_from_parts(...)`
- opt-in TurboQuant path: `score_ip_from_parts_int8_approx_no_qjl_4bit(...)`

This keeps the scan-local score cache working for the experiment, which was the
main reason to wire it here instead of adding a one-off uncached comparison
path.

### 4. Debug stage profiling now reports the effective TurboQuant score mode

`tests.tqhnsw_debug_turboquant_scan_stage_profile(...)` now reports the active
effective score mode through the existing columns:

- `turboquant_exact_score_mode`
- `turboquant_exact_score_uses_lut`
- `turboquant_exact_score_uses_qjl`

So the scan-path measurement surface can distinguish:

- default `mse_no_qjl_4bit`
- opt-in `int8_approx_no_qjl_4bit`

without adding a second TurboQuant-specific runtime helper first.

## Tests

Added coverage for:

1. the happy path

- `pg_test_turboquant_scan_stage_profile_int8_mode`
- verifies the stage-profile SQL surface reports
  `turboquant_exact_score_mode = int8_approx_no_qjl_4bit`
- verifies binary prefilter and deferred rerank work still show up

2. env validation

- `pg_test_turboquant_exact_score_mode_rejects_invalid_env`
- rejects anything other than `[exact, int8_approx]`

## Validation

Ran on this exact tree:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

All passed.

## Readout

### 1. This is the first real lever-5 scan-path seam

Packet `433` only proved that int8 approx looked promising in the offline
scorer study. This packet makes that option available on the real TurboQuant
scan path under an explicit gate.

### 2. Default behavior is unchanged

Without `TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE=int8_approx`, TurboQuant still
uses the existing exact scorer and still reports:

- `turboquant_exact_score_mode = mse_no_qjl_4bit`

### 3. The next justified slice is measurement, not more plumbing

The remaining task-16 question for lever 5 is now measurable on the real scan
path:

- does the opt-in int8 mode improve the task-16 TurboQuant lane materially?
- what does it do to the serious heap-rerank lane versus the quantized lane?
- does the runtime win survive the actual graph / prefilter / rerank mix?

Packet `434` intentionally stops at the validated implementation seam so the
next packet can be pure measurement on top of `e0ba7ee`.

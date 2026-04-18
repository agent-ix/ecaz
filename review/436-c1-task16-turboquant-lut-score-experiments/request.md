# Review Request: C1 Task16 TurboQuant LUT Score Experiments

Current head at execution: `572dd53`

## Context

Task 16's remaining open item is the direct lever-4 / lever-5 comparison on the
real TurboQuant scan path. Packet `433` already measured the scorer options
offline, and packet `434` added the first live opt-in seam for `int8_approx`.

To compare all remaining scorer options on the actual scan path, the live seam
needed the two lever-4 modes as well:

- `full_lut`
- `tiled_lut`

This slice lands those modes under the same existing
`TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE` gate.

It also standardizes the last missing scratch harness wrapper needed to answer
the V3 vacuum-concurrency feedback without env-prefixed script invocation.

## Code Changes

### 1. Extended TurboQuant live exact-score mode enum

`src/am/scan.rs` now supports four TurboQuant exact-score modes:

- `exact` (default)
- `full_lut`
- `tiled_lut`
- `int8_approx`

The env parser now accepts all four values and rejects anything else with an
explicit error message.

### 2. Added prepared-query state for live LUT experiments

Scan-local state now stores:

- `PreparedLutNoQjl4BitQuery`
- `PreparedTiledLutNoQjl4BitQuery`
- existing `Int8ApproxNoQjl4BitQuery`

These are prepared during `amrescan`, freed with the rest of scan-local query
state, and only enabled on the existing no-QJL `1536 @ 4-bit` TurboQuant lane.

### 3. Wired score dispatch through the new modes

`score_scan_element_result(...)` now dispatches to:

- current exact scorer
- `score_ip_from_parts_lut_no_qjl_4bit(...)`
- `score_ip_from_parts_tiled_lut_no_qjl_4bit(...)`
- `score_ip_from_parts_int8_approx_no_qjl_4bit(...)`

The default path remains unchanged unless the env override is set.

### 4. Extended stage-profile coverage

`src/lib.rs` now has pg tests covering:

- default TurboQuant stage profile
- `full_lut`
- `tiled_lut`
- `int8_approx`
- invalid env rejection with the expanded allowed-value list

The stage-profile surface continues to report:

- `turboquant_exact_score_mode`
- `turboquant_exact_score_uses_lut`
- `turboquant_exact_score_uses_qjl`

so the follow-on measurement packet can distinguish the modes without adding
another debug helper first.

### 5. Standardized vacuum concurrency harness targeting

`scripts/vacuum_concurrency_scratch.sh` now accepts:

- `--socket-dir DIR`
- `--port PORT`

This keeps the V3 vacuum-concurrency rerun on the same args-only script surface
used elsewhere, instead of relying on env-prefixed invocation.

## Validation

Ran on this exact tree:

```bash
bash -n scripts/vacuum_concurrency_scratch.sh
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

All passed.

## Next Step

Run the real-corpus task-16 matrix on the pushed tree and compare:

- default `exact`
- `full_lut`
- `tiled_lut`
- `int8_approx`

on the selected TurboQuant lanes, then packet the results together with the V3
vacuum-concurrency rerun outcome requested in feedback.

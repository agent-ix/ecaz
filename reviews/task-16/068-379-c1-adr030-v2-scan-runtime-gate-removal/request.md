# Review Request: C1 ADR-030 V2 Scan Runtime Gate Removal

## Context

Packet 378 moved grouped build selection onto the SQL reloption path:

- old build selection: `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD`
- new build selection: `WITH (storage_format = 'pq_fastscan')`

That left one remaining mismatch in the build+scan flow:

- build was now selected per index through reloptions and persisted metadata
- grouped ordered scan still depended on the process-wide
  `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN` env var

So a `pq_fastscan` index could be built and persisted correctly, but ordered
scan availability still depended on ambient process configuration instead of the
index format itself.

## Problem

The grouped scan runtime gate had three concrete issues:

1. scan availability was still controlled by process state instead of index
   metadata
2. grouped ordered-scan tests and scratch flows still depended on an env var
   that no longer matched the reloption-based build path
3. the runtime settings/debug surface still described grouped scan selection as
   gated, even though ADR-032/task-15 now want format choice to be first-class

With packet 378 in place, keeping the scan gate no longer made architectural
sense.

## Planned Slice

One narrow checkpoint:

1. remove `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`
2. allow grouped ordered scans whenever persisted metadata decodes as grouped
3. keep the remaining grouped scan tuning env vars in place
   - window size
   - grouped score mode
   - rerank mode / source column
   - exact traversal controls
4. update tests and scratch docs to reflect metadata-based grouped scan
   selection

Insert/vacuum parity is explicitly out of scope for this packet.

## Implementation

Updated:

- `src/am/scan.rs`
- `src/lib.rs`
- `scripts/restart_adr030_scratch.sh`

### 1. Grouped scan format validation now trusts metadata

`src/am/scan.rs` now makes `validate_runtime_scan_format(...)` a straight call
to `GraphStorageDescriptor::from_metadata(...)`.

The old grouped runtime-gate branch and helper were deleted:

- removed `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`
- removed the `experimental_grouped_v2_scan_enabled()` helper
- removed the old "scan runtime does not support ADR-030 grouped-v2 indexes
  yet" error for grouped metadata selection

So ordered scan selection now follows persisted format metadata directly:

- scalar metadata => scalar scan path
- grouped metadata => grouped scan path

### 2. Exact grouped scoring now errors for the real unsupported condition

The grouped scan gate used to double as the error surfaced when exact grouped
payloads were unavailable.

That message is now replaced with the actual condition:

- `tqhnsw grouped exact scoring requires the grouped cold rerank payload path`

This keeps the remaining exact-score limitation explicit without conflating it
with top-level grouped scan selection.

### 3. Tests and debug settings now reflect always-available grouped scan selection

`src/lib.rs` now:

- removes all `ScopedEnvVar::set("TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN", "1")`
  setup from grouped scan tests
- converts the old runtime-rejection test into a grouped ordered-scan smoke test
  that now succeeds
- keeps the grouped plan smoke test, but without any scan gate setup
- updates the runtime settings probe so `grouped_scan_enabled` reports `true`
  as a fixed capability rather than reflecting an env var

This means grouped scan tests now exercise the reloption+metadata selection
mechanism directly instead of bootstrapping through a process-global switch.

### 4. Scratch helper no longer claims a grouped scan gate

`scripts/restart_adr030_scratch.sh` no longer exports
`TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN`.

Its help text now explains the current contract:

- `PqFastScan` selection comes from
  `WITH (storage_format = 'pq_fastscan')`
- the script still wires grouped scan tuning env vars for runtime experiments

## Measurements

No new benchmark or recall measurements in this slice. This is a control-plane
and runtime-selection checkpoint.

## Validation

Passed:

- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`, and
  `errstart`

## Outcome

This checkpoint completes the reloption-driven grouped build+scan cutover:

1. grouped build is selected by `storage_format = 'pq_fastscan'`
2. grouped ordered scan is selected by persisted grouped metadata
3. grouped scan no longer depends on a process-global runtime gate
4. grouped scan tuning env vars remain available for window/rerank/exact
   experiments

What it still does **not** do:

- grouped insert still has no success path
- grouped vacuum still has no success path
- grouped exact scoring still requires the grouped cold rerank payload path
- grouped runtime naming cleanup (`GroupedV2` -> `PqFastScan`) is still pending

## Next Slice

The next practical slices are:

1. remove the grouped insert/vacuum hard-reject architecture by giving
   `PqFastScan` real payload append/repair hooks
2. continue renaming the runtime surface from experimental `grouped_v2`
   terminology to `PqFastScan` / `TurboQuant` terminology

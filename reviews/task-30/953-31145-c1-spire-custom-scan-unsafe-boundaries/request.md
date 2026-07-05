# Review Request: SPiRE Custom Scan Unsafe Boundaries

## Summary

This slice hardens the SPiRE custom scan explain and registration boundaries.
It keeps the unavoidable PostgreSQL callback/raw-pointer work local while
removing unsafe requirements from normal registration callers.

Code checkpoint: `9b4b66104ba0e9df617466bd5c470ca34502b104`

## Safety Handling

- Added `OpenIndexRelation`, an owning guard around `index_open` /
  `index_close`, so `custom_scan_explain_context()` cannot return without
  closing the index relation it opened.
- Made `custom_scan_explain_context()` safe; it now returns the zero/default
  explain context when no index OID or relation is available.
- Narrowed `ec_spire_explain_custom_scan()` from one large unsafe body to
  local unsafe blocks around the PostgreSQL callback pointer reads and explain
  property calls.
- Made `register_custom_scan()` safe at both the SPiRE module boundary and
  `am` module boundary. The process-global PostgreSQL hook/method writes stay
  inside the registration function with a SAFETY contract.
- Documented the process-local custom scan status flag read.

## Baseline Delta

- Before: 4,787 unsafe baseline entries across 110 files.
- After: 4,782 unsafe baseline entries across 108 files.
- Net: 5 entries removed, 2 files removed from the unsafe baseline.

Removed baseline entries:

- `src/am/ec_spire/custom_scan/explain.rs:14`
- `src/am/ec_spire/custom_scan/explain.rs:47`
- `src/am/ec_spire/custom_scan/mod.rs:149`
- `src/am/ec_spire/custom_scan/mod.rs:165`
- `src/am/mod.rs:199`

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`
- `git diff --check HEAD^ HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passes with the existing PostgreSQL header warnings and existing
unused SPIRE re-export warning.

## Artifacts

- `artifacts/unsafe-baseline-before.log`
- `artifacts/unsafe-baseline-after.log`
- `artifacts/audit-unsafe.log`
- `artifacts/fmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18.log`

## Review Focus

- Is `OpenIndexRelation` the right local owner for the `index_open` /
  `index_close` pairing in the explain callback?
- Are the remaining unsafe blocks in `ec_spire_explain_custom_scan()` narrow
  enough and accurately documented?
- Is it acceptable for `register_custom_scan()` to be safe now that hook
  mutation invariants are contained inside the function?

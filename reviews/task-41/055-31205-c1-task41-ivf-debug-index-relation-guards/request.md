# Review Request: Task 41 IVF Debug Index Relation Guards

## Summary

This slice migrates the simple IVF pg-test debug helpers from manual
`index_open` / `index_close` pairs to the shared `IndexRelationGuard`.

Code commit: `aee30476e23b28063b31c96b27ece01e8147430e`

## Changes

Updated these helpers in `src/am/ec_ivf/scan.rs`:

- `debug_ec_ivf_rescan_query_prep`
- `debug_ec_ivf_pq_fastscan_model_cache_reused`
- `debug_ec_ivf_metadata`
- `debug_ec_ivf_quantizer_cache_ptr`
- `debug_ec_ivf_rerank_mode`
- `debug_ec_ivf_build_metadata`
- `debug_ec_ivf_directory_summary`
- `debug_ec_ivf_directory_entry`

The helpers now hold an `IndexRelationGuard` while deriving raw relation
pointers for the existing scan and metadata routines. Early error and return
paths in directory inspection now drop through the guard instead of manually
closing the index relation.

The heap-backed debug scan struct is intentionally left for a later slice
because it owns an index scan descriptor plus heap/snapshot state.

## Baseline

- Before: `4165`
- After: `4146`

## Review Focus

- Confirm each removed manual `index_close` is covered by
  `IndexRelationGuard` drop, including early directory-summary error paths.
- Confirm scan helpers still call `ec_ivf_amendscan` and `IndexScanEnd`
  before the relation guard drops.
- Confirm the remaining local `index_open` / `index_close` sites in this
  file are only the intentionally deferred heap-backed debug scan struct.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.md`.

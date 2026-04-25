# PG18 Candidate ID Diagnostics

## Summary

This packet covers commit `12849cde18eac9122c1663251f6022db0bf6cef3`.

`ecaz dev test pg18-parallel-scan` now prints ID-level divergence diagnostics whenever serial and candidate ID streams are collected:

- `candidate_duplicate_ids`
- `candidate_missing_serial_ids`
- `candidate_extra_ids`

The diagnostics are multiset-aware, so duplicate candidates are separated from truly extra IDs and IDs missing from the serial reference stream. This makes the diagnostic multi-emitter failure easier to triage without manually comparing the full ID vectors.

## Result

Default elected-emitter PG18 still passes and reports no ID-set drift:

```text
[pg18-parallel] candidate_duplicate_ids=[]
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Diagnostic multi-emitter mode still fails as expected, now with direct ID drift detail:

```text
[pg18-parallel] candidate_duplicate_ids=[93x2, 258x2]
[pg18-parallel] candidate_missing_serial_ids=[71, 459, 284]
[pg18-parallel] candidate_extra_ids=[93, 258, 86]
[pg18-parallel] validation failed
```

## Artifacts

- `artifacts/pg18-parallel-id-diagnostic-default.log`
- `artifacts/pg18-parallel-id-diagnostic-multi-emitter.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `git diff --check`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-id-diagnostic-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-id-diagnostic-multi-emitter.log` (expected validation failure)

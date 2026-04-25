# PG18 Multi-Emitter Flag Hardening

## Summary

This packet covers commit `a30e0b960c4eb29f1f09c5e16987289aaf0d2ccd`.

The diagnostic multi-emitter path now requires:

```text
TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1
```

Before this slice, any present value, including `0`, enabled the diagnostic bypass. The flag is intentionally unsafe for production equivalence, so the parser now accepts only the literal value `1`. The scan path delegates to the same helper used by the planner snapshot, keeping the runtime gate and `next_runtime_blocker` text aligned.

## Result

With `TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=0`, the PG18 fixture stays on the elected-emitter path and passes:

```text
next_runtime_blocker=PG18 planner-visible Parallel Index Scan is enabled with one elected tuple emitter; rank-compatible multi-emitter Gather Merge output remains the next runtime step
[pg18-parallel] candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

With `TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1`, the diagnostic path still activates and fails as expected:

```text
next_runtime_blocker=PG18 diagnostic multi-emitter env is enabled; direct multi-emitter output remains rank-incompatible with Gather Merge
[pg18-parallel] candidate_ids=[379, 177, 472, 473, 378, 165, 172, 93, 280, 57, 366, 258, 176, 82, 71, 377]
[pg18-parallel] validation failed
```

## Artifacts

- `artifacts/pg18-parallel-multi-emitter-env0.log`
- `artifacts/pg18-parallel-multi-emitter-env1.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test parallel_scan_backend_may_emit_tuples --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=0 --log-output target/pg18-parallel-multi-emitter-env0.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-multi-emitter-env1.log` (expected validation failure)

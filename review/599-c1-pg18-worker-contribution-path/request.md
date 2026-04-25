# PG18 Worker Contribution Path

## Summary

This packet covers commit `019c56d3e8fdab94ca773ed97991098c4dd8681a`.

The PG18 runtime blocker text and Task 18/ADR-040 docs now point at the next production path:

- keep one elected backend as the only visible tuple emitter for `Gather Merge`;
- make non-elected workers contribute through the shared coordinator behind that single output stream;
- keep direct multi-emitter output diagnostic-only because it remains rank-incompatible with `Gather Merge`.

This follows packet 598, which showed that projected query scores match exact recomputation and that the current serial HNSW stream itself has adjacent exact-score inversions.

## Result

The default PG18 fixture still passes and reports the updated blocker:

```text
next_runtime_blocker=PG18 planner-visible Parallel Index Scan is enabled with one elected visible tuple emitter; next runtime step is shared worker contribution behind that single output stream
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The diagnostic direct multi-emitter lane still fails as expected and now says it is not the production path:

```text
next_runtime_blocker=PG18 diagnostic multi-emitter env is enabled; direct multi-emitter output remains rank-incompatible with Gather Merge and is not the production path
[pg18-parallel] validation failed
```

## Artifacts

- `artifacts/pg18-parallel-worker-contribution-blocker-default.log`
- `artifacts/pg18-parallel-worker-contribution-blocker-multi-emitter.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `git diff --check`
- `cargo test test_ech_planner --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-worker-contribution-blocker-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-worker-contribution-blocker-multi-emitter.log` (expected validation failure)

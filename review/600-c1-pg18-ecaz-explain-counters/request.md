# PG18 Ecaz EXPLAIN Counters

## Summary

This packet covers commit `000d3b9ae515fab51a20c375b6d6d42e0d92c239`.

The `ecaz-cli` PG18 parallel-scan live harness now runs the ordered candidate query with:

```text
EXPLAIN (ANALYZE, COSTS OFF, SUMMARY OFF, ecaz)
```

That keeps the existing serial-equivalence checks and adds the extension's per-node counter block to the same raw logs used for Task 18 diagnostics.

## Result

The default elected-emitter lane still passes and now exposes the relevant counter surface:

```text
Index Searches: 0
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The diagnostic direct multi-emitter lane still fails as expected and now records the same counters:

```text
Index Searches: 0
Bootstrap Expansions: 67
Elements Scored: 67
Heap TIDs Returned: 2
[pg18-parallel] candidate_duplicate_ids=[82x2]
[pg18-parallel] candidate_missing_serial_ids=[71, 459, 284]
[pg18-parallel] candidate_extra_ids=[387, 82, 165]
[pg18-parallel] validation failed
```

The important measurement signal is that the counter block is now present in both live lanes, so the next worker-contribution protocol can be measured without adding a separate SQL or shell harness.

## Artifacts

- `artifacts/pg18-parallel-ecaz-explain-default.log`
- `artifacts/pg18-parallel-ecaz-explain-multi-emitter.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `git diff --check`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-ecaz-explain-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-ecaz-explain-multi-emitter.log` (expected validation failure)

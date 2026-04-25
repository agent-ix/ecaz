# PG18 Projected Score Diagnostics

## Summary

This packet covers commit `09683f38b3410b2c4adc403991987d6cc60b95cb`.

`ecaz dev test pg18-parallel-scan --diagnose-planner` now collects ordered query rows as `(id, projected ORDER BY score)` instead of collecting only IDs, derives the ID list from those rows, and prints:

- serial and candidate projected ORDER BY scores
- serial and candidate projected-score adjacent inversions
- candidate exact-score adjacent inversions, matching the existing serial exact-score inversion output

This is a diagnostic-only CLI change. It keeps the CLI checkpoint separate from extension runtime changes so it can be pushed independently if needed.

## Result

The default PG18 elected-emitter lane still passes. The projected query scores match the exact recomputed scores, and the serial HNSW order already has adjacent score inversions:

```text
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] serial_projected_orderby_scores=[(177, -1.646769881), (379, -1.742236257), ...]
[pg18-parallel] candidate_projected_orderby_scores=[(177, -1.646769881), (379, -1.742236257), ...]
[pg18-parallel] serial_projected_orderby_score_adjacent_inversions=[177(-1.646769881) before 379(-1.742236257), ...]
[pg18-parallel] candidate_projected_orderby_score_adjacent_inversions=[177(-1.646769881) before 379(-1.742236257), ...]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The diagnostic direct multi-emitter lane still fails as expected. The new score lines show the candidate row set drift and score-order drift together:

```text
[pg18-parallel] candidate_missing_serial_ids=[459]
[pg18-parallel] candidate_extra_ids=[438]
[pg18-parallel] candidate_projected_orderby_scores=[(379, -1.742236257), (177, -1.646769881), ...]
[pg18-parallel] candidate_projected_orderby_score_adjacent_inversions=[472(-1.641386509) before 473(-1.769334435), ...]
[pg18-parallel] candidate_exact_score_adjacent_inversions=[472(-1.641386509) before 473(-1.769334435), ...]
[pg18-parallel] validation failed
```

The important takeaway is that the current serial HNSW contract is not exact-score-sorted on this fixture. A future `Gather Merge`-compatible path needs an explicit contract decision: either preserve serial HNSW rank with a sequencer, or intentionally switch the parallel path to an exact-key-sorted result contract.

## Artifacts

- `artifacts/pg18-parallel-projected-score-default.log`
- `artifacts/pg18-parallel-projected-score-multi-emitter.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `git diff --check`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-projected-score-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-projected-score-multi-emitter-rerun.log` (expected validation failure)
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

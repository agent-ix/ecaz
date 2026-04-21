# Review Request: stop reloption `top_k` from capping SQL DiskANN results

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/ec_diskann/routine.rs`

## What this packet is

This is a DiskANN runtime-correctness slice in the ordered SQL path.

The pure `scan::vamana_scan_with(...)` shell still supports a caller-supplied
`top_k`, but the SQL AM path in `ec_diskann_amrescan` does **not** know the
executor's final `LIMIT`. Until now it still passed the reloption `top_k`
through as the eager result cap, which meant a default `ec_diskann` index
could silently stop an ordered query early before the executor ever applied
`LIMIT`.

That contradicted the scan design note in `plan/design/diskann-scan-pgrx.md`,
which already called out the first-land rule: if `LIMIT` is not visible,
materialize the full rerank window and let the executor truncate externally.

## What changed

### SQL scan path now uses the full rerank window

`src/am/ec_diskann/routine.rs` now derives a SQL-path result cap with:

```rust
let sql_result_cap = sql_scan_result_cap(opaque.top_k, opaque.rerank_budget);
```

and then uses that cap for both:

```rust
ScanParams {
    ...
    rerank_budget: opaque.rerank_budget,
    top_k: sql_result_cap,
}
```

and duplicate expansion:

```rust
expand_scan_results_with_bound_heap_tids(..., sql_result_cap)
```

The helper is intentionally simple:

```rust
fn sql_scan_result_cap(reloption_top_k: usize, rerank_budget: usize) -> usize {
    let _ = reloption_top_k;
    rerank_budget
}
```

That is the narrow fix: the SQL AM path now materializes the full rerank
window and leaves truncation to the executor instead of treating the reloption
`top_k` as a hard SQL result ceiling.

### Tests pin the behavior

- Pure test: `sql_scan_result_cap_defaults_to_rerank_budget`
  - proves the SQL path resolves to `rerank_budget`, not the reloption
    `top_k`, when `LIMIT` is not visible
- pg18 regression:
  `test_ec_diskann_sql_limit_can_exceed_reloption_top_k`
  - creates a default `ec_diskann` index
  - inserts 12 duplicate live rows through the supported insert path
  - forces ordered index execution
  - proves `ORDER BY ... LIMIT 12` now returns all 12 rows even though the
    default reloption `top_k` is still `10`

## Why this slice

- DiskANN-only, AM-local, and directly about SQL correctness.
- Matches the existing scan design instead of inventing a new runtime policy.
- Keeps scope tight: one file, one helper, one runtime call-site switch, one
  pure test, one pg test.
- Avoids CLI/tool churn entirely.

## Test evidence

```text
$ cargo test -p ecaz-cli 2>&1 | tail -3

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Also passed on `pg18` for this checkpoint:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Plumbing executor-visible `LIMIT` into `ec_diskann_amrescan`. This slice
  takes the already-documented first-land behavior instead of adding planner /
  executor coupling.
- Reworking the pure scan shell's `top_k` contract. The shell still supports a
  generic caller-supplied truncation bound; this packet only fixes the SQL AM
  path where `LIMIT` lives outside the AM.
- Any change to `rerank_budget` semantics. This slice removes the incorrect
  `top_k` cap without broadening the rerank window itself.

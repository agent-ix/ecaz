# Task 111e: Coarse Rerank SQL Contract Validation

## Summary

This packet adds a PG test proving the explicit `coarse_rerank` contract works
through SQL index creation and admin diagnostics.

The new test creates an IVF index with:

```text
storage_format = 'coarse_rerank'
coarse_format = 'rabitq'
coarse_bits = 1
rerank = 'heap_f32'
rerank_placement = 'heap'
rerank_format = 'heap_f32'
rerank_width = 3
```

It then verifies `ec_ivf_index_admin_snapshot` reports the normalized contract:

```text
coarse_rerank/rabitq/1/heap_f32/table/f32/3
```

and confirms a simple IVF debug scan can still return the expected nearest
candidate.

## Code Under Review

- `src/tests/ec_ivf.rs`
- `src/am/ec_ivf/page.rs` warning-only import cleanup for `pg_test` builds

## Validation

Artifacts are under `reviews/task-111e/004-coarse-rerank-sql-contract/artifacts/`.

```text
cargo test -q coarse_rerank --lib --no-default-features --features pg18
6 passed; 0 failed; 2131 filtered out

cargo test -q metadata_roundtrip --lib --no-default-features --features pg18
8 passed; 0 failed; 2129 filtered out
```

The `coarse_rerank` filtered run invokes the pgrx PG18 harness and includes the
new SQL test `test_ec_ivf_coarse_rerank_contract_admin_snapshot`.

## Review Ask

Please review whether this covers the heap-f32 baseline gate sufficiently for
the current contract slice, before moving on to compact rerank representation
or index-side placement work.

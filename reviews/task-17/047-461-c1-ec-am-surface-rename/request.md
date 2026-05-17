# Review Request: C1 EC Access Method Surface Rename

Current head at execution: `72fa6e2`

## Context

The previous ADR-041 split kept the historical `tqhnsw` / `tqdiskann` names.
That no longer matches the product direction: the HNSW AM is not
TurboQuant-specific, and DiskANN is also moving under the `ec_` naming family.

This checkpoint renames the live access-method surface to:

- `ec_hnsw`
- `ec_diskann`

The rename is applied consistently across Rust module paths, SQL/bootstrap,
upgrade SQL, specs, docs, helper scripts, and script-side unit tests. Review
history under `review/` was intentionally left unchanged.

## What changed

### 1. Renamed the HNSW AM module tree

- moved `src/am/tqhnsw/` to `src/am/ec_hnsw/`
- updated `src/am/mod.rs` and callers to use the new module name
- kept the previously landed `src/am/common/` split intact

This preserves the ADR-041 structure while removing the TurboQuant-specific
label from the native HNSW implementation.

### 2. Renamed the public SQL/catalog surface

Updated the live extension surface from `tqhnsw` to `ec_hnsw`, including:

- access method handler naming in `sql/bootstrap.sql`
- `CREATE ACCESS METHOD ... USING ec_hnsw`
- helper SQL names and guardrails that reference the AM name
- upgrade SQL in `tqvector--0.1.0--0.1.1.sql`

I also updated the related `ec_diskann` naming references so the sibling AM
uses the same convention.

### 3. Renamed operational docs and scripts

Updated README/docs/plan text plus helper scripts and their tests so current
operator workflows now refer to `ec_hnsw` fixture prefixes, helper functions,
GUC names, and benchmark expectations instead of `tqhnsw`.

Because the helper-script surface changed, I added a script-side validation
pass in addition to the required Rust checks.

### 4. Shortened internal pg-test symbol names where needed

The public SQL surface stays `ec_hnsw`, but some internal `#[pg_test]`
function identifiers in `src/lib.rs` exceeded the Rust symbol length limit
after the rename. Those internal test function names were shortened to the
`pg_test_ech_*` pattern without changing the public AM/API naming.

## Validation

Green validation for this checkpoint:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
python3 -m unittest \
  scripts.tests.test_load_real_corpus_storage_format \
  scripts.tests.test_bench_sql_latency_verified \
  scripts.tests.test_bench_tqvector_sql_overhead_breakdown \
  scripts.tests.test_manifest_portability
```

## Review focus

1. Does the rename cover the complete live surface for `ec_hnsw` /
   `ec_diskann`, especially `sql/bootstrap.sql`, `tqvector--0.1.0--0.1.1.sql`,
   and the helper SQL/script paths?
2. Are there any compatibility or migration concerns in the upgrade/bootstrap
   story that should be addressed before this lands on `main`?
3. Is the `src/am/ec_hnsw/` naming and the internal `pg_test_ech_*`
   shortening a reasonable stopping point for this checkpoint?

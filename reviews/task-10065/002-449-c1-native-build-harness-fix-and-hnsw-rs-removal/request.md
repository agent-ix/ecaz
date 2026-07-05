# Review Request: C1 Native Build Harness Fix And hnsw_rs Removal

Current head at execution: `8ac9bda`

## Context

This checkpoint processes the reviewer feedback that landed after packets `446`
and `448`.

Two things were intentionally kept together because they were already coupled in
the working tree:

- fold in the external-summary exact-baseline fix instead of leaving it as WIP
- finish removing `hnsw_rs` from the crate/test surface now that native BUILD is
  the production path

This is still a narrow cleanup slice. It does not change persisted tuple/page
layout, and it does not claim any new recall measurements beyond what packets
`446` and `448` already recorded.

## What changed

### 1. The external summary exact baseline now ignores sibling index families

`src/lib.rs` now forces the "exact quantized" baseline query path to avoid index
scans when the external recall summary helper runs against a shared corpus table
with multiple tqhnsw index families present.

That prevents the helper from drifting into a sibling grouped / heap-f32 lane
when the requested index is the TurboQuant index.

The new regression coverage is:

- `test_tqhnsw_external_summary_exact_baseline_multiidx`

This creates scalar + TurboQuant + pq_fastscan indexes on the same synthetic
corpus and asserts that the explicit TurboQuant summary path remains valid.

### 2. `hnsw_rs` is no longer part of the crate dependency/test surface

This checkpoint removes:

- the `hnsw_rs` entry from `Cargo.toml`
- the `src/lib.rs` import of `hnsw_rs`
- the old `probe_hnsw_rs_*` helper functions
- the ignored `test_hnsw_rs_*` comparison tests
- the HNSW-specific distance helper structs that only existed for those tests

The vendored `vendor/hnsw_rs/` directory is intentionally left on disk because
earlier task constraints said not to remove it in this branch, but the tqvector
crate no longer depends on it or references it.

## Reviewer feedback addressed

From packet `448`:

- the summary-helper drift fix is now committed instead of left WIP

From packet `446`:

- the crate/test surface no longer carries `hnsw_rs`

I did not address the remaining native-build shape questions from `446`
(`ef_construction` upper-layer walk, redundant descent note, page-size cap note)
in this cleanup slice because they are separate from the harness/dependency
cleanup and do not affect the now-green validation state here.

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Additional quick check:

```bash
rg -n "hnsw_rs|probe_hnsw_rs|test_hnsw_rs" src/lib.rs Cargo.toml
```

This returns no matches.

## Review focus

1. Is the external-summary exact-baseline fix the right scope for the mixed
   index-family harness drift, or do you want stricter helper-level coverage?
2. Is leaving `vendor/hnsw_rs/` in-tree but fully disconnected from the crate
   the right stopping point for this branch, given the earlier "do not remove
   vendored directory" constraint?

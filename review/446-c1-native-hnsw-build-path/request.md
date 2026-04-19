# Review Request: C1 Native HNSW Build Path

Current head at execution: `948d3c4`

## Context

This checkpoint starts the post-task16 native-build replacement tracked in
plan task `10065` / ADR-042, but it is filed as a coder1 packet because the
work is on the core BUILD/INSERT path and follows the task16 + ecvector row
model cleanup on `main`.

Before this slice:

- production BUILD in `src/am/build.rs` still delegated graph construction to
  vendored `hnsw_rs`
- BUILD leaked `hnsw_rs::hnsw::Neighbour` through internal helper signatures
- INSERT and BUILD used different level sampling / neighbor-selection stacks
- `hnsw_rs` still sat in normal `[dependencies]`

This checkpoint replaces the production BUILD graph construction path with a
native serial builder that reuses tqvector-owned HNSW decisions from INSERT.

## What changed

### 1. Production BUILD no longer uses `hnsw_rs`

`src/am/build.rs` now:

- removes all `hnsw_rs` imports
- removes the old `BuildCodeDistance` / `BuildVectorDistance` adapters
- builds the in-memory HNSW graph natively using tqvector-owned code
- keeps the existing `HnswBuildNode -> staged page tuple` flush shape unchanged

`grep -R "hnsw_rs" src/am/` is now empty.

### 2. Native BUILD now reuses INSERT-side policy

This slice does not fork a second heuristic stack. Instead it reuses the same
decision rules BUILD-side by sharing INSERT helpers for:

- deterministic level sampling (`choose_insert_level`)
- fixed layer slot bounds / forward-slot selection
- backlink rewrite ordering and tie-breaking

The new builder uses tqvector’s own beam-search helpers against an in-memory
node list, then applies the same backlink replacement rule that live INSERT
already uses:

- insert into a free slot when available
- otherwise score existing slice members plus the new node
- keep the best `M` / `2M` candidates with deterministic tie order

### 3. Both BUILD variants now run through the native path

The native builder supports:

- code-graph BUILD
- source-graph BUILD

The scoring mode is selected from the build tuples:

- quantized-code inner-product for the normal path
- raw source-vector inner-product for `build_source_column`

### 4. `hnsw_rs` is test-only now

`Cargo.toml` moves `hnsw_rs` out of `[dependencies]` and into
`[dev-dependencies]`.

`src/lib.rs` still retains the ignored oracle probes and can continue using the
vendored crate for comparison work.

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Oracle / recall probe commands run on this tree:

```bash
cargo test test_hnsw_rs_code_graph_recall_uniform_10k --no-default-features --features pg17 -- --ignored --nocapture
cargo test test_hnsw_rs_source_graph_recall_uniform_10k --no-default-features --features pg17 -- --ignored --nocapture
cargo test test_hnsw_rs_source_graph_recall_clustered_10k --no-default-features --features pg17 -- --ignored --nocapture
```

Packet-local artifacts and the cited result lines live under
`review/446-c1-native-hnsw-build-path/artifacts/`.

## Results cited from artifacts

- code-graph 10k oracle lane:
  - `hnsw_rs` recall@10 = `0.2900`
  - tqvector build-code baseline = `0.8050`
  - exact quantized baseline = `0.8400`
- source-graph 10k uniform oracle lane:
  - `hnsw_rs` recall@10 = `0.3000`
- source-graph 10k clustered oracle lane:
  - `hnsw_rs` recall@10 = `0.2850`
- source-graph 10k uniform `m=16 ef_search=200` oracle lane:
  - `hnsw_rs` recall@10 = `0.6550`

## Review focus

1. Is the new native BUILD loop the right serial shape for ADR-042 while still
   leaving room for FR-021 parallel feed later?
2. Did I preserve the INSERT-side backlink rewrite semantics closely enough, or
   is there any missed tie/order edge in the in-memory adaptation?
3. Are there any remaining source-graph parity gaps before I package the next
   checkpoint with heavier real-corpus recall evidence?

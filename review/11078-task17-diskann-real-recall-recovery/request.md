# Review Request: recover real-corpus DiskANN recall on pg18

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `src/am/mod.rs`
- `src/am/ec_diskann/mod.rs`
- `src/am/ec_diskann/options.rs`
- `src/am/ec_diskann/scan_state.rs`
- `src/am/ec_diskann/cost.rs`
- `src/am/ec_diskann/routine.rs`
- `src/am/ec_diskann/ambuild.rs`
- `src/am/ec_diskann/insert.rs`
- `review/11078-task17-diskann-real-recall-recovery/artifacts/rebuild.log`
- `review/11078-task17-diskann-real-recall-recovery/artifacts/pre-distance-sweep.log`
- `review/11078-task17-diskann-real-recall-recovery/artifacts/post-distance-sweep.log`
- `review/11078-task17-diskann-real-recall-recovery/artifacts/manifest.md`

## What this packet is

This is the next DiskANN-specific recovery slice after the first real-corpus
pg18 measurement came back catastrophically wrong.

Two independent AM bugs were hiding inside that result:

1. `ec_diskann.list_size` existed only as a CLI convention. The extension never
   registered or consumed it, so `ecaz bench recall --profile ec_diskann
   --sweep ...` was sweeping a placeholder session variable and not changing the
   scan width at all.
2. Vamana build / insert neighbor selection used `max(0, -ip)` as its
   nonnegative “distance”. On the real OpenAI fixture, the source vectors are
   unit-normalized, so that collapsed every positive-inner-product pair to
   `0.0` and destroyed the graph geometry.

This packet fixes both issues and captures the rebuilt real-10k result.

## What changed

### Runtime tuning is now real

`src/am/ec_diskann/options.rs` now registers a real userset GUC:

```rust
GucRegistry::define_int_guc(
    c"ec_diskann.list_size",
    ...,
)
```

and resolves effective scan tuning from:

- the relation reloption `list_size`, or
- a session override via `SET ec_diskann.list_size = ...`

`scan_state.rs` now loads the resolved width into `DiskannScanOpaque`, and
`cost.rs` uses the same effective width for planner costing so runtime and
planner tuning no longer drift.

`routine.rs` adds a pg test proving the session override changes the resolved
scan width.

### Build and insert distance no longer flatten positive-IP pairs

`ambuild.rs` and `insert.rs` now use:

```rust
distance = max(0, 1.0 - ip)
```

instead of:

```rust
distance = max(0, -ip)
```

for graph build / insert-time exact neighbor selection on unit-normalized
vectors.

That preserves the `<#>` ordering while keeping Vamana’s distance nonnegative,
instead of turning most genuinely similar pairs into the same zero-distance
bucket.

## Operator outcome

The intermediate sweep after the GUC fix but before the graph-distance rebuild
was still broken (`artifacts/pre-distance-sweep.log`):

```text
│ 200       ┆ 0.0095   ┆ 0.4935 ┆ 39.29 ms    │
```

After rebuilding the same real-10k DiskANN index with the corrected build
distance (`artifacts/rebuild.log`) and rerunning the same recall sweep
(`artifacts/post-distance-sweep.log`), DiskANN recovered to the expected range:

```text
│ 64        ┆ 0.9280   ┆ 0.9959 ┆ 43.69 ms    │
│ 128       ┆ 0.9310   ┆ 0.9966 ┆ 55.46 ms    │
│ 200       ┆ 0.9315   ┆ 0.9966 ┆ 69.62 ms    │
│ 400       ┆ 0.9315   ┆ 0.9966 ┆ 122.57 ms   │
│ 800       ┆ 0.9315   ┆ 0.9966 ┆ 299.29 ms   │
```

That is the same canonical pg18 `ecaz` surface as packet `11073`, on the same:

- prefix `ec_hnsw_real_10k`
- profile `ec_diskann`
- reloptions `graph_degree=32`, `build_list_size=100`, `alpha=1.2`

## Why this slice

- The bug was in DiskANN itself, not in generic tooling.
- The runtime fix is AM-local and directly tied to the DiskANN sweep contract.
- The distance fix is the smallest credible change that explains the enormous
  real-corpus quality gap and the rebuild-time behavior.
- No `ecaz-cli` API changes were needed in this slice.

## Test evidence

```text
$ cargo test -p ecaz-cli --quiet

test result: ok. 218 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

Also passed on `pg18` for this checkpoint:

- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

## Follow-ups intentionally not in this packet

- Further Pareto tuning of `graph_degree`, `build_list_size`, or `alpha`. This
  packet restores sane DiskANN quality on the intended baseline configuration.
- Any attempt to reduce the post-fix latency slope at large `list_size`. The
  immediate task-17 blocker was broken quality, not latency optimization.

# Review Request: Task 29 local DiskANN real-10k baseline blocker

Branch: `task29-diskann-initial-tuning`
Author: coder-1
Target:

- `src/am/ec_diskann/mod.rs`
- `crates/ecaz-cli/src/commands/dev/test.rs`
- `crates/ecaz-cli/src/commands/bench/overhead.rs`
- `crates/ecaz-cli/src/commands/compare/pgvector.rs`
- `review/675-c1-task29-diskann-real10k-baseline/artifacts/`

## What This Packet Is

This is the first Task 29 local real-10k baseline attempt after the DiskANN
rebase verification packet.

It did not produce a trustworthy full DiskANN sweep. The baseline run exposed a
more basic blocker: the prepared `ecaz-cli bench recall` query shape is falling
back to a disabled seqscan/top-N-sort path instead of using an ANN index.

Until that is fixed, DiskANN recall/latency numbers from `ecaz-cli bench recall`
are not valid tuning inputs.

## Code Fixes Landed First

Commit `9291ec00` fixes small rebase/benchmark-harness drift before the packet:

- `src/am/ec_diskann/mod.rs`
  - makes `maybe_check_for_interrupts()` a no-op under Rust `cfg(test)` so
    `cargo pgrx test pg18 pg_test_ec_diskann_` can start without resolving the
    backend-only `InterruptPending` symbol in the Rust test binary.
- `crates/ecaz-cli/src/commands/dev/test.rs`
  - updates stale `ConnectParams/connect_with` references to the current
    `ConnectionOptions/connect` helper.
- `crates/ecaz-cli/src/commands/bench/overhead.rs`
- `crates/ecaz-cli/src/commands/compare/pgvector.rs`
  - passes the selected profile into the current `build_knn_sql(profile, ...)`
    helper signature.

## Validation

Passed:

```text
cargo pgrx test pg18 pg_test_ec_diskann_
cargo check -p ecaz-cli
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
git diff --check
```

The focused PG18 DiskANN pass ran 19 `pg_test_ec_diskann_*` tests covering
build, scan, insert, planner, and vacuum paths.

## Measurement Context

Fixture:

- DBPedia/OpenAI3 real 10k fixture generated through `ecaz-cli corpus fetch`
  and `ecaz-cli corpus prepare`
- corpus rows: `10000`
- query rows: `200`
- dimension: `1536`
- corpus SHA: `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
- query SHA: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`

DiskANN reloptions:

```text
graph_degree=32
build_list_size=100
alpha=1.2
```

Local machine:

- WSL2 Linux `6.6.87.2-microsoft-standard-WSL2`
- Intel Core i9-10900K, 20 logical CPUs
- 62 GiB RAM
- warm local page cache; no cache drop attempted

## Results

### DiskANN single-query probe

`bench recall --profile ec_diskann --k 10 --sweep 64 --queries-limit 1`:

```text
│ list_size ┆ recall@k ┆ ndcg@k ┆ mean q-time │
│ 64        ┆ 1.0000   ┆ 1.0000 ┆ 3993.69 ms  │
```

This is far too slow to treat as a DiskANN baseline. A full
`64,128,200,400,800` sweep was started and then cancelled after one active KNN
query continued far beyond the expected real-10k latency envelope.

### HNSW reference probe

`bench recall --profile ec_hnsw --k 10 --sweep 64 --queries-limit 1` on the
same corpus:

```text
│ ef_search ┆ recall@k ┆ ndcg@k ┆ mean q-time │
│ 64        ┆ 1.0000   ┆ 1.0000 ┆ 4044.26 ms  │
```

The matching 4s shape on HNSW points away from DiskANN-specific graph traversal
and toward the benchmark query path / planner selection.

### EXPLAIN blocker

The prepared-query shape used by the CLI was reproduced through
`ecaz-cli dev sql`:

```text
Limit (actual time=4002.280..4002.283 rows=10.00 loops=1)
  ->  Sort (actual time=4002.278..4002.279 rows=10.00 loops=1)
        Disabled: true
        ->  Seq Scan on ec_hnsw_real_10k_corpus (actual time=0.773..3994.989 rows=10000.00 loops=1)
Execution Time: 4002.321 ms
```

So the measurement is not timing an ANN index scan. It is timing a disabled
sequential scan and top-N sort over 10k `ecvector` rows.

### Storage

The DiskANN index was built and is present:

```text
ec_hnsw_real_10k_idx  ec_diskann  {graph_degree=32,build_list_size=100,alpha=1.2}  4.7 MiB  494.0 B/row
```

## Recommendation

Do not optimize DiskANN graph/search code from these numbers.

The first Task 29 blocker is to repair the `ecaz-cli bench recall/latency`
query path so profile-specific ANN indexes are selected for prepared benchmark
queries. Once EXPLAIN shows `Index Scan` on `ec_diskann`, rerun the requested
real-10k sweep:

```text
list_size=64,128,200,400,800
```

Then compare the resulting DiskANN rows against the HNSW reference on the same
fixture. Only after that should we tune `graph_degree`, `build_list_size`,
`alpha`, or scan internals.

## Artifact Notes

Raw artifacts are under `artifacts/`, with metadata in
`artifacts/manifest.md`.

No build-time/load-time claim is made in this packet. The current loader uses
ordinary stdout/stderr in places that `--log-file` does not mirror, and this
packet avoided shell redirection/wrappers after the Task 29 constraint was
clarified.

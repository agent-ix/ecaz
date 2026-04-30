# Artifact Manifest: Task 29 DiskANN Graph Diagnostics

Head SHA: `c09ab1674de347216f46aafb4d08df294c3a9285`
Packet: `679-c1-task29-diskann-graph-diagnostics`
Timestamp: `2026-04-29T17:35:08-07:00`

Lane: Task 29 DiskANN initial tuning
Fixture: local real-10k corpus from
`target/real-corpus/ec_hnsw_real_10k/`
Storage format: default `ecvector` / `ec_diskann`
Rerank mode: DiskANN V0 heap rerank path
Surface: isolated one-index-per-table prefixes in database
`task29_diskann_baseline`
Cache state: no cache flush; graph diagnostics run after earlier recall and
latency sweeps on the same local cluster.

## `check-function-before.log`

Command:

```text
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "SELECT count(*) FROM pg_proc WHERE proname = 'ec_diskann_index_graph_summary';" --log-output review/679-c1-task29-diskann-graph-diagnostics/artifacts/check-function-before.log
```

Key result:

```text
count = 0
```

The loaded benchmark database predates the new SQL wrapper, so the packet-local
install SQL below was applied without dropping or recreating the extension.

## `install-diskann-graph-summary.sql`

Packet-local SQL used to expose the new C-backed wrapper in the existing
benchmark database without disturbing loaded corpus tables.

## `install-diskann-graph-summary.log`

Command:

```text
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --file review/679-c1-task29-diskann-graph-diagnostics/artifacts/install-diskann-graph-summary.sql --log-output review/679-c1-task29-diskann-graph-diagnostics/artifacts/install-diskann-graph-summary.log
```

Key result:

```text
CREATE FUNCTION
```

## `graph-diskann-baseline.log`

Command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline bench diskann-graph --prefix task29_diskann_real10k --log-output review/679-c1-task29-diskann-graph-diagnostics/artifacts/graph-diskann-baseline.log
```

Key result lines:

```text
node_count = 10000
live_node_count = 10000
reachable_live_node_count = 9999
unreachable_live_node_count = 1
reachable_live_fraction = 0.999900
neighbor_ref_count = 227822
dead_neighbor_ref_count = 0
invalid_neighbor_ref_count = 0
self_neighbor_ref_count = 0
duplicate_neighbor_ref_count = 0
unresolvable_neighbor_ref_count = 0
out degree: zero=0 min=1 avg=22.782200 p50=22 p95=32 p99=32 max=32
in degree: zero=1 min=0 avg=22.782200 p50=21 p95=41 p99=59 max=3800
```

## `graph-diskann-prior.log`

Command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline bench diskann-graph --prefix task29_diskann_prior_real10k --log-output review/679-c1-task29-diskann-graph-diagnostics/artifacts/graph-diskann-prior.log
```

Key result lines:

```text
node_count = 10000
live_node_count = 10000
reachable_live_node_count = 10000
unreachable_live_node_count = 0
reachable_live_fraction = 1.000000
neighbor_ref_count = 245035
dead_neighbor_ref_count = 0
invalid_neighbor_ref_count = 0
self_neighbor_ref_count = 0
duplicate_neighbor_ref_count = 0
unresolvable_neighbor_ref_count = 0
out degree: zero=0 min=1 avg=24.503500 p50=25 p95=32 p99=32 max=32
in degree: zero=0 min=1 avg=24.503500 p50=22 p95=43 p99=61 max=3250
```

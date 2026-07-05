# Task 65b Review Request: Serial Neighbor Cache

## Scope

This packet covers Task 65b Slice B: introduce a single-threaded
`BuilderNeighborCache` abstraction in `src/am/ec_diskann/vamana.rs`.

The change keeps the serial Vamana algorithm deterministic. It does not start
parallel execution and does not change the on-disk format, scan path, insert
path, or Postgres build callback. The build loop no longer reads and mutates
`graph.neighbors` directly; it routes greedy-search reads, candidate-pool reads,
out-edge replacement, backlink insertion, and reprune replacement through the
cache object. The cache currently owns the same `Vec<Vec<u32>>` adjacency shape
as the old builder so this slice isolates the abstraction cost before any
locking or worker model is introduced.

## Result Summary

Code checkpoint under review:

- `37052b81693d3b5693d95fcaa0f29f7ffc748063`

Validation passed:

| check | result |
| --- | --- |
| `cargo fmt --check` | passed with existing stable-rustfmt warnings |
| `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana` | passed, `12 passed` |
| `cargo check -p ecaz --lib --no-default-features --features pg18` | passed |
| `ecaz bench suite audit` | passed, 4 steps |
| real10k cache-backed suite | passed |

Real10k comparison against packet 001 measurement floor:

| metric | packet 001 floor | packet 002 cache |
| --- | ---: | ---: |
| SQL index build | `6.72s` | `6.86s` |
| total load | `9.50s` | `10.55s` |
| build-probe seconds | `62.045s` | `62.168s` |
| recall@10 L64/L128/L200 | `0.9965 / 0.9970 / 0.9975` | `0.9965 / 0.9970 / 0.9975` |
| DiskANN index size | `4.7 MiB` | `4.7 MiB` |
| in-degree p95/p99/max | `52 / 79 / 2881` | `52 / 79 / 2881` |

The abstraction is therefore behavior-neutral on the local real10k gate. The
small timing movement is within local run-to-run noise and does not indicate a
meaningful regression.

## Evidence

Packet-local artifacts under `artifacts/`:

- `manifest.md`
- `install-ecaz-pg-test-release.log`
- `cargo-fmt-check.log`
- `cargo-test-vamana.log`
- `cargo-check-pg18-lib.log`
- `suite-audit.log`
- `suite-manifest.json`
- `results.jsonl`
- `load-real10k-r32-l100.log`
- `build-probe-real10k-r32-l100.log`
- `recall-real10k-r32-l100.log`
- `storage-real10k-r32-l100.log`
- `truth-real10k-k10.json`

The real10k suite mirrors packet 001's R32/L100 `pq_fastscan` row so recall and
build timing can be compared directly against the measurement floor.

## Review Focus

- Whether the cache abstraction is narrow enough for Slice B.
- Whether all build-loop adjacency reads and writes now route through
  `BuilderNeighborCache`.
- Whether the real10k validation is sufficient to proceed to the Slice C
  locking-design packet.

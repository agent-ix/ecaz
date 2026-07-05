# HNSW Task 35 Production Coverage

This table summarizes the Task 35 packets that cleared `src/am/ec_hnsw` production-source unsafe-comment baseline entries.

| Packet | Surface | File(s) | Baseline Movement |
|---|---|---|---|
| 012 | scan setup/cache | `src/am/ec_hnsw/scan.rs` | 258 -> 193 (-65) |
| 013 | graph cache scoring | `src/am/ec_hnsw/scan.rs` | 193 -> 141 (-52) |
| 014 | successor prefetch | `src/am/ec_hnsw/scan.rs` | 141 -> 109 (-32) |
| 015 | traversal state | `src/am/ec_hnsw/scan.rs` | 109 -> 83 (-26) |
| 016 | frontier set | `src/am/ec_hnsw/scan.rs` | 83 -> 60 (-23) |
| 017 | frontier refill | `src/am/ec_hnsw/scan.rs` | 60 -> 44 (-16) |
| 018 | linear output | `src/am/ec_hnsw/scan.rs` | 44 -> 25 (-19) |
| 019 | scan tail | `src/am/ec_hnsw/scan.rs` | 25 -> 0 (-25) |
| 020 | graph tuple loader | `src/am/ec_hnsw/graph.rs` | 56 -> 35 (-21) |
| 065 | module wrappers | `src/am/ec_hnsw/mod.rs` | 4 -> 0 (-4) |
| 084 | options | `src/am/ec_hnsw/options.rs` | 8 -> 0 (-8) |
| 085 | graph | `src/am/ec_hnsw/graph.rs` | 35 -> 0 (-35) |
| 086 | build | `src/am/ec_hnsw/build.rs` | 33 -> 0 (-33) |
| 091 | scan debug lifecycle | `src/am/ec_hnsw/scan_debug.rs` | 354 -> 287 (-67) |
| 092 | scan debug oracle | `src/am/ec_hnsw/scan_debug.rs` | 287 -> 181 (-106) |
| 093 | scan debug result state | `src/am/ec_hnsw/scan_debug.rs` | 181 -> 0 (-181) |
| 094 | parallel DSM layout | `src/am/ec_hnsw/build_parallel.rs` | 203 -> 184 (-19) |
| 095 | parallel DSM insert/search | `src/am/ec_hnsw/build_parallel.rs` | 184 -> 141 (-43) |
| 096 | parallel worker lifecycle | `src/am/ec_hnsw/build_parallel.rs` | 141 -> 39 (-102) |
| 097 | parallel test helper | `src/am/ec_hnsw/build_parallel.rs` | 39 -> 0 (-39) |
| 098 | insert entry/source | `src/am/ec_hnsw/insert.rs` | 133 -> 0 (-133) |
| 099 | vacuum | `src/am/ec_hnsw/vacuum.rs` | 99 -> 0 (-99) |
| 101 | source vectors | `src/am/ec_hnsw/source.rs` | 78 -> 0 (-78) |
| 102 | shared page/metadata | `src/am/ec_hnsw/shared.rs` | 73 -> 24 (-49) |
| 103 | shared snapshot/debug | `src/am/ec_hnsw/shared.rs` | 24 -> 0 (-24) |

## Totals

- HNSW production-source entries cleared in Task 35: `1299`.
- Current `src/am/ec_hnsw` residual in `scripts/unsafe_comment_baseline.txt`: `0`.
- Remaining HNSW-named baseline entries are under `src/tests/` and belong to the separate test-only sweep.

# Task 29d Final Readiness

Status: ready for review
Branch: `task29-diskann-initial-tuning`
Head before packet: `bc44adc5`

## Scope

This packet closes Task 29d after the three requested pre-landing items:

- `11106`: build heap-frontier release A/B, do not reland.
- `11107`: L=64 scan profile, no safe rerank-budget default change.
- `11108`: build source-distance SIMD, landed.

All measurements here use local PG18 with release-installed `ecaz`.

## Build And Storage

| Engine | Build row | Build time | Index size |
|---|---|---:|---:|
| `ec_diskann` | final restored index, `graph_degree=32, build_list_size=100, alpha=1.2` | 14.59 s | 4,939,776 B |
| `pgvectorscale` | `diskann`, `num_neighbors=32, search_list_size=100, max_alpha=1.2` | 5.72 s | 5,136,384 B |
| `ec_hnsw` | `m=32, ef_construction=100, build_source_column=source` | 5.77 s | 15,130,624 B |

The Task 29d build stop condition is met: `ec_diskann` is now below the
17.5 s target (3x the 5.82 s pgvectorscale reference from the review prompt,
and also 2.55x the refreshed 5.72 s pgvectorscale row).

## Final Sweep

| Tuning | ec_diskann recall / mean / p99 | pgvectorscale recall / mean / p99 | ec_hnsw recall / mean / p99 |
|---:|---:|---:|---:|
| 64 | 0.9965 / 7.80 ms / 10.3 ms | 0.9955 / 3.48 ms / 4.49 ms | 0.9695 / 2.91 ms / 4.78 ms |
| 128 | 0.9965 / 7.79 ms / 10.2 ms | 0.9990 / 5.81 ms / 6.74 ms | 0.9710 / 4.75 ms / 6.83 ms |
| 200 | 0.9970 / 7.98 ms / 10.3 ms | 1.0000 / 8.50 ms / 10.2 ms | 0.9710 / 6.75 ms / 8.58 ms |
| 400 | 0.9970 / 8.49 ms / 10.8 ms | 1.0000 / 17.3 ms / 22.2 ms | 0.9715 / 13.0 ms / 18.0 ms |
| 800 | 0.9975 / 9.34 ms / 12.9 ms | 1.0000 / 30.1 ms / 33.7 ms | 0.9715 / 25.5 ms / 41.1 ms |

Interpretation:

- `ec_diskann` is now landable for the initial tuning lane. It is smaller than
  both local references, meets the build stop condition, keeps recall near
  exact, and beats pgvectorscale latency from L=200 upward.
- pgvectorscale remains the low-L latency reference and reaches exact recall by
  L=200. That is good merge-discussion context, not a blocker.
- HNSW is faster at low tuning values but materially lower recall on this
  surface. At higher tuning values, `ec_diskann` is both higher recall and
  faster.

## Caveats

- The shared corpus table can have both `ec_diskann` and `ec_hnsw` indexes.
  Direct `ecaz bench` runs are only cited after isolating the intended index.
  The non-isolated ec_diskann rows in `compare-vectorscale-final.log` and the
  first `recall-diskann-final-*` logs are retained as audit artifacts but are
  not cited.
- The pgvectorscale rows come from the checked-in `ecaz compare vectorscale`
  helper on its sidecar table, which avoids the shared-table planner ambiguity.

## Recommendation

Mark Task 29d complete and send Task 29 / 29a / 29b / 29c / 29d back for
review. No AWS or production benchmark is needed for this landing decision.

## Artifacts

See `artifacts/manifest.md`.

# Functional Requirement Layout

Functional requirement files keep their immutable `FR-###` IDs, but are grouped
by bounded context so new work has an obvious home.

| Directory | Contents |
| --- | --- |
| `common/` | SQL bootstrap, row types, PG18 callbacks, planner/statistics, WAL, vacuum, and shared access-method contracts. |
| `quant/` | TurboQuant payloads, prepared scoring, `QuantCodec`, candidate batching, SIMD/block-kernel requirements. |
| `index/hnsw/` | `ec_hnsw` page, build, scan, insert, vacuum, and current AM surface. |
| `index/ivf/` | `ec_ivf` build/storage, scan/rerank/cost, insert/vacuum/admin surface. |
| `index/diskann/` | `ec_diskann` build/storage, scan/prefilter/rerank, insert/vacuum/diagnostic surface. |
| `operator/` | `ecaz` CLI, configured benchmark suites, and cloud operator surfaces. |
| `spire/` | SPIRE bounded context, storage, local execution, distributed execution, operations, and archived superseded SPIRE IDs. |

When moving a requirement, preserve its frontmatter `id` and update active spec,
docs, ADR, and test-matrix links. Historical review packet prose can remain as
the original record unless it is being republished as current evidence.

# Task 50 Top-15 Coverage Map

Task 50's exit criterion requires the densest residual modules to be processed
at least once and, where structurally possible, reduced by at least 30% from
the post-Task-35 direct unsafe-block count. Product priority puts
RaBitQ-enabling shared kernels first, then IVF/RaBitQ, then SPIRE as the
ultimate production target; HNSW and DiskANN are not optional and need named
later slices or explicit ceiling explanations.

The projections below are planning targets. Each implementation packet must
measure actual before/after counts with the direct block-count tool.

| Rank | File | Start | Target reduction | Planned slice(s) | Notes |
| ---: | --- | ---: | ---: | --- | --- |
| 1 | `src/am/ec_hnsw/scan_debug.rs` | 356 | >=107 | HNSW debug helper rollout after Slice 1; page tuple visitor consumption | High-density but lower product priority. Needs direct HNSW pass, not only shared helper drift. |
| 2 | `src/am/ec_hnsw/scan.rs` | 226 | >=68 | callback helper consumption; heap scorer; page/graph visitor follow-up | Shared helpers reduce part; likely needs a direct scan-state pointer-lift slice. |
| 3 | `src/am/ec_hnsw/build_parallel.rs` | 203 | >=61 | DSM atomic field wrapper; callback helper | Defer until SPIRE/IVF/RaBitQ slices land unless Task 39/47 exposes it as a blocker. |
| 4 | `src/am/ec_spire/dml_frontdoor/mod.rs` | 160 | >=48 | callback helper where applicable; NodeTag decoder helper; relation guard consumption | Not first SPIRE target because production read path is higher priority, but still in exit scope. |
| 5 | `src/am/ec_ivf/page.rs` | 134 | >=41 | Slice 2 IVF page tuple visitor; later WAL/exclusive buffer pair | First direct top-15 reduction target. |
| 6 | `src/am/ec_hnsw/insert.rs` | 133 | >=40 | heap source scorer; page tuple visitor consumption | Should follow Slice 4 once scorer allocation behavior is proven on IVF/SPIRE. |
| 7 | `src/am/ec_ivf/scan.rs` | 102 | >=31 | Slice 1b callback rollout; Slice 2 visitor callers; Slice 4 scorer | Priority target because IVF/RaBitQ profiling depends on it. |
| 8 | `src/am/ec_hnsw/vacuum.rs` | 99 | >=30 | heap source scorer; WAL/exclusive buffer pair | Later direct HNSW maintenance packet. |
| 9 | `src/am/ec_diskann/routine.rs` | 92 | >=28 | vector datum wrapper; WAL/page visitor; heap source scorer | Lower priority but should consume shared wrappers after IVF/SPIRE proof. |
| 10 | `src/am/ec_hnsw/source.rs` | 78 | >=24 | heap source scorer | Expected to reduce substantially once scorer is shared. |
| 11 | `src/am/ec_hnsw/shared.rs` | 73 | >=22 | page tuple visitor; graph/page view helper | Direct HNSW shared-page packet likely required. |
| 12 | `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 71 | >=22 | Slice 3b ActiveEpochAnchor snapshot rollout | Priority SPIRE target. |
| 13 | `src/am/common/parallel.rs` | 63 | >=19 | callback helper; parallel layout typed view; possibly Task 40 coordination | Must not duplicate Task 40 concurrency lifts. |
| 14 | `src/quant/hadamard.rs` | 62 | >=19 | SIMD load/store newtypes | Requires local x86_64 plus cloud Graviton measurement. |
| 15 | `src/am/ec_spire/coordinator/snapshots.rs` | 62 | >=19 | Slice 3b ActiveEpochAnchor snapshot rollout | Priority SPIRE target, coordinate with Task 30 phase 13d. |

## Exit Tracking Rule

Each request after Packet 002 should include a small table:

```text
file | before | after | delta | percent | top-15 target status
```

If a file cannot reach 30% without an architectural change owned by Task 40,
41, or 43, the packet should name that dependency and record the lower
structural ceiling.

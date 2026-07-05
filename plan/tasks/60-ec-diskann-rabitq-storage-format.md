# Task 60: ec_diskann RaBitQ Storage Format

Status: **complete** (2026-06-16) — `storage_format = 'rabitq'` for
`ec_diskann` landed on `main` (merge `a8b2aca41`; benchmark artifacts
`0be251132`; packets `reviews/task-60/001`–`014`). DiskANN/Vamana now
rides RaBitQ-1bit codes alongside the existing `pq_fastscan` payload.
Goal is to give operators a smaller-index DiskANN option without
giving up the graph-traversal recall/latency curve.

## Why

The Task 55 packet 005 scan-materialization win
(`cbf037334ce0a9f499507d206049574b8278282e`) restored `ec_diskann` as
a real latency contender on AWS Graviton (1.72 ms mean at 100k,
`list_size=64`, recall@10 unchanged, storage byte-identical).

`ec_diskann` is currently hardcoded to a single storage format at
`src/am/ec_diskann/options.rs:52`:

```
pub(super) enum StorageFormat {
    PqFastScan,
}
```

That leaves no quantizer choice for DiskANN. By comparison:

- `ec_ivf` supports `auto | turboquant | pq_fastscan | rabitq`
  (`src/am/ec_ivf/options.rs:54`).
- `ec_spire` supports `auto | turboquant | pq_fastscan | rabitq`.
- `ec_hnsw` supports `turboquant | pq_fastscan` (no rabitq either).

Size extrapolation (from landed measurements):

| index                            | 100k     | 1M (extrapolated) | per row     |
|----------------------------------|---------:|------------------:|------------:|
| `ec_diskann` + `pq_fastscan`     | 46.1 MiB | ~461 MiB          | 483 B       |
| `ec_ivf` + `rabitq`              | —        | 298 MiB (measured)| ~313 B      |
| `ec_diskann` + `rabitq` (target) | —        | ~150–250 MiB      | ~160–260 B  |

The DiskANN+RaBitQ target sits below `ec_ivf`+RaBitQ because RaBitQ's
1-bit code replaces the PQ payload while the per-node graph
neighbours (~32 × 4 B = 128 B) stay roughly constant. If recall
parity holds, this becomes the smallest competitive ANN index in the
workspace.

Storage win is the headline. The open question — and the gate — is
whether RaBitQ-1bit codes are precise enough to drive DiskANN greedy
descent without an unacceptable recall loss at fixed `list_size`.

## Baseline Evidence

- code path being extended:
  `src/am/ec_diskann/options.rs`,
  `src/am/ec_diskann/build*.rs`,
  `src/am/ec_diskann/scan*.rs`,
  `src/am/ec_diskann/page.rs`.
- packet that established the recent DiskANN scan baseline:
  `reviews/task-55/005-aws-diskann-scan-optimization/`,
  `benchmarks/task55-aws-diskann-lowcost-optimized/`.
- RaBitQ encoder already consumed by `ec_ivf` and `ec_spire`:
  `src/quant/` (binary sidecar / rabitq kernels) plus
  `src/am/ec_ivf/quantizer.rs` and
  `src/am/ec_spire/quantizer/` as integration references.
- prior reviewer feedback that prompted this task:
  `reviews/task-55/005-aws-diskann-scan-optimization/feedback/2026-05-24-02-reviewer.md`.

## Scope

1. **Enum + parse.** Extend
   `src/am/ec_diskann/options.rs::StorageFormat` with a `RaBitQ`
   variant; wire `parse_reloption`, `as_str`, `DEFAULT` (default
   stays `PqFastScan`). Reject ambiguous combinations explicitly
   rather than silently coercing.
2. **On-disk discriminator.** Tag the storage format in the
   `ec_diskann` metadata-page header so existing `pq_fastscan`
   indexes keep loading unchanged and new `rabitq` indexes are
   identified at `ambeginscan` without guessing. See
   `src/am/ec_diskann/page.rs` (`VamanaMetadataPage`,
   `INDEX_FORMAT_V3_DISKANN`).
3. **Build-side encode.** Emit RaBitQ payloads in the DiskANN build
   path. Reuse the existing shared RaBitQ encoder consumed by
   `ec_ivf` and `ec_spire`; do not duplicate the kernel. Decide
   per-node tuple layout: either replace the existing PQ payload
   slot in `VamanaNodeTuple` based on storage_format, or add a
   format-specific tuple variant — whichever keeps the
   binary-sidecar prefilter unchanged.
4. **Scan-side prefilter.** Implement a RaBitQ binary-sidecar
   prefilter for DiskANN greedy descent that consumes the new
   payload. Critically, the new path MUST preserve the
   packet-005 materialization-avoidance win: scan-time reads must
   stay on-demand via the `GraphReader` trait
   (`src/am/ec_diskann/reader.rs`), not regress to chain
   materialization.
5. **Backwards compatibility.** Existing `pq_fastscan` indexes
   must load and scan unchanged. No in-place migration to RaBitQ
   — operators rebuild if they want to switch formats.
6. **Tests.**
   - Unit / pgrx tests for build + scan against both formats.
   - Recall parity check at 100k showing RaBitQ-on-DiskANN
     recall@10 is within an explicit, measured delta of
     PqFastScan-on-DiskANN at matched `list_size`. The acceptable
     delta is a deliverable of the task, not pre-set here.
   - Storage assertion showing rabitq index size is meaningfully
     smaller than pq_fastscan at the same corpus.
7. **Bench gate** under `benchmarks/<topic>/` measuring
   `pq_fastscan` vs `rabitq` on `ec_diskann` at 100k AND 1M, per
   `StR-007`. Each row carries `storage_format`, `cache_state`,
   and host-parity fields per the systemic reporting fixes from
   `reviews/task-51/027-sidecar-sweep-pressure-reduction/feedback/2026-05-24-01-reviewer.md`.

## Non-Goals

- DiskANN `turboquant` storage format (separate decision; not on
  this task's critical path).
- Mixed-format indexes (one DiskANN index with mixed PQ + RaBitQ
  nodes). Pick one format per index at build time.
- In-place migration of existing `pq_fastscan` indexes to
  `rabitq`. Rebuild is the migration path.
- Heap-rerank mode changes. Whatever rerank mode operates on
  `pq_fastscan` today should operate the same way on `rabitq` —
  the rerank path reads from the heap, not from the index
  payload.

## Carry-Forward Reviewer Gates

From
`reviews/task-55/005-aws-diskann-scan-optimization/feedback/2026-05-24-01-reviewer.md`:

- bench rows MUST carry `storage_format` (in addition to the
  existing EXPLAIN-only carrier). Use this task to enforce it.
- bench rows MUST carry `cache_state` and host-parity fields per
  the systemic harness asks from task-51/027/01.
- pg_test for `RelationGraphReader` parity with
  `PersistedGraphReader` should already have landed via task 59;
  if it has not, this task must not assume that gate is closed.

## Acceptance Criteria

Minimum to land:

1. `CREATE INDEX ... USING ec_diskann WITH (storage_format = 'rabitq')`
   builds cleanly at 100k+ corpus.
2. Scans return correct results — recall@10 on `rabitq` is no
   worse than the explicit, measured delta below
   `pq_fastscan` at matched `list_size` (delta documented in
   the bench packet, justified by the recall-vs-storage trade).
3. `pq_fastscan` indexes built before this task continue to load,
   scan, and return identical results — no on-disk format
   regression for existing data.
4. Index size at 1M is ≥ 30% smaller than the corresponding
   `pq_fastscan` index. (Qualitative bar — if the measured win
   is smaller, the task closes with a recorded "not worth
   shipping" decision rather than landing the format.)
5. Bench packet under `benchmarks/<topic>/` records the
   comparison at 100k AND 1M with `storage_format`,
   `cache_state`, and host-parity fields per the harness gates
   above.

Desirable but not strictly required:

- `pq_fastscan` vs `rabitq` Pareto chart at 100k AND 1M (recall
  vs latency vs size) so operators can pick the format from the
  three axes.
- An ADR capturing the decision criteria for when a DiskANN
  operator should choose `rabitq` over `pq_fastscan` and vice
  versa.

## References

- `plan/tasks/55-diskann-unsafe-burndown.md`
- `plan/tasks/59-diskann-aws-graviton-tuning-1m-suite.md`
- `reviews/task-55/005-aws-diskann-scan-optimization/feedback/2026-05-24-01-reviewer.md`
- `reviews/task-55/005-aws-diskann-scan-optimization/feedback/2026-05-24-02-reviewer.md`
- `reviews/task-51/027-sidecar-sweep-pressure-reduction/feedback/2026-05-24-01-reviewer.md`
- `spec/stakeholder/StR-007-cloud-scale-benchmarking.md`
- `spec/non-functional/NFR-015-benchmark-reporting-standard.md`

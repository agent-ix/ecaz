# Task 51: IVF RaBitQ Second Optimization Round

Status: **proposed** — follow-on to the AWS RaBitQ/IVF Graviton v4 round.
This task owns the next latency-reduction cycle after the bits=1 NEON,
byte-LUT, `quant_bits=1`, and `rerank_width=50` work. The goal is to find
another material latency reduction without assuming that more index-side
payload is acceptable.

## Why

The current AWS evidence shows `ec_ivf` RaBitQ is much closer to the target,
but still needs a cleaner path to lower high-recall latency. The most recent
1M evidence is promising (`~67 ms @ recall 0.9936` at nprobe=256), but the
remaining gap cannot be responsibly attacked by guessing at the next kernel
micro-optimization. We need to determine whether latency is dominated by:

- candidate volume from IVF geometry,
- posting tuple decode / scan layout,
- RaBitQ scoring arithmetic,
- heap exact-rerank fetches and detoast,
- or benchmark methodology noise.

This task is deliberately staged from low-risk experiments to invasive storage
work. A Posting Layout v2 is in scope only after cheaper experiments show that
posting scan/decode is a real bottleneck.

## Non-Goals

- Do not make inline source-vector sidecars the default product direction.
  A sidecar/full-source experiment is allowed only as an upper-bound Pareto
  measurement.
- Do not change the on-disk posting format until the batch-decode / scratch
  SoA prototype shows a clear local and AWS signal.
- Do not add new ad hoc shell sweepers. If benchmark-suite coverage is
  missing, extend `ecaz bench suite` first.

## Baseline and Required Cleanup

Before starting new optimization slices:

1. Create or update the benchmark packet manifest for the previous AWS round.
   It must name authoritative artifacts and mark failed/incomplete artifacts
   explicitly.
2. Fix the AWS workflow rule that currently treats `nlists` as a no-rebuild
   knob. `nlists` is an index-build geometry parameter and must be recorded
   in snapshot metadata.
3. Add 1M scan counters before changing behavior:
   postings visited, postings scored, posting pages read, candidates emitted,
   duplicates filtered, heap rerank rows, heap blocks fetched, and time spent
   in approximate scan vs exact rerank.

## Experiment Plan

### 1. Counter Baseline Gate

Run a same-host 1M `ec_ivf` measurement:

- `ec_ivf` bits=1 + `heap_f32` + width=50 at nprobe values around the
  current frontier.
- Counters from Baseline #3 captured under the same suite config.

Acceptance criteria:

- Same host class and same PostgreSQL config.
- Same query IDs for latency and recall.
- q-count at least 500 unless cost forces a documented smaller run.
- Packet-local `manifest.md`, suite config, and structured results.

Competitor numbers come from the pinned `94c02c682` paired-comparator
baseline. Do not re-run vchord or pgvectorscale here.

### 2. `nlists` / nprobe Geometry Sweep

Rebuild 1M indexes with multiple list counts and sweep nprobe at matched
recall. Start with the current automatic geometry as baseline, then test
larger `nlists` values intended to reduce candidates per probed list.

Measure:

- recall@10 and NDCG@10,
- p50/p95/p99 latency,
- postings visited/scored per query,
- posting pages read,
- candidates reaching rerank,
- build time and index size.

Promotion criteria:

- If a geometry reaches the same recall with at least 25% lower p50 and no
  unacceptable p95 regression, it becomes the new AWS candidate.
- If larger `nlists` only shifts cost from scoring to centroid routing or
  hurts recall stability, document and stop this branch.

### 3. Local Batch-Decode / Scratch SoA Prototype

Before changing the disk format, keep the existing posting layout and batch
decode posting tuples into temporary scan-local structure-of-arrays buffers:

- contiguous heap TIDs,
- contiguous gammas / scalar metadata,
- contiguous bits=1 code bytes,
- optional precomputed scale/bound values if derivable without format change.

Then score the batch through a chunked bits=1 path.

Smoke criteria:

- Local candidates/sec improves by at least 20% on a fixed-candidate scan
  benchmark, or AWS p50 improves by at least 10% at a high-nprobe cell.
- Recall is byte-for-byte equivalent to the existing scan for the same index
  and query set.
- Temporary buffer allocation is bounded and reused across queries; no
  per-candidate heap allocation.

If this fails to move candidates/sec materially, do not pursue Posting Layout
v2 in this round.

### 4. Heap Rerank Locality

If counters show exact rerank or heap fetch dominates, preserve the compact
index format and improve locality:

- collect the rerank frontier,
- sort by heap block / TID for exact-vector fetch,
- score exact vectors in locality order,
- restore final result ranking by score.

Measure:

- heap blocks fetched per query,
- exact-rerank time,
- p50/p95 latency,
- recall parity,
- memory overhead of the reordered frontier.

Promotion criteria:

- At least 15% p50 improvement on high-recall 1M cells or a clear p95/tail
  reduction, with no recall change.

### 5. Adaptive nprobe and Adaptive `rerank_width`

Use centroid score margins, candidate score margins, or observed bound progress
to reduce work for easy queries.

Candidate policies:

- adaptive nprobe: stop probing once centroid margin or candidate frontier
  confidence exceeds a calibrated threshold;
- adaptive width: choose exact-rerank width from `k`, approximate score gap,
  and query difficulty instead of a fixed relation default.

Acceptance criteria:

- Report p50, p95, p99, recall@10, recall p10/p50/p90, and worst-query recall
  on the same q-set.
- Any adaptive policy must have a conservative mode suitable for production.
- No average-only recall claims; tail recall must be visible.

### 6. Posting Layout v2 Decision Gate

Only start a real on-disk Posting Layout v2 after the scratch SoA prototype
or counters prove posting decode/scan is a primary bottleneck.

Candidate design:

```text
ListChunkV2 {
  header: list_id, count, next
  heap_tids[count]
  gammas[count]
  scale_or_bound_metadata[count]   // only if proven useful
  code_bytes[count][code_len]      // contiguous and aligned
}
```

Expected advantages:

- fewer tuple headers and line-pointer walks,
- sequential code streaming,
- easier software prefetch,
- chunked/batched bits=1 scoring,
- cheaper candidate frontier construction.

Required design work:

- metadata version bump and rebuild rule,
- build path and insert path semantics,
- delete/vacuum/churn behavior,
- WAL and crash-recovery safety,
- page-space fragmentation analysis,
- compatibility with non-bits=1 formats or an explicit narrow scope.

Promotion criteria:

- At least 25-35% p50 improvement over the existing layout on 1M high-recall
  cells, or at least 30% candidates/sec improvement in scan counters.
- No recall drift.
- Index-size increase is reported and justified.
- Insert/delete/vacuum semantics have focused tests before the format is
  considered landable.

### 7. Upper-Bound Sidecar / Inline-Source Measurement

Because more index-side data is not the preferred product direction, keep this
as a measurement-only branch:

- full inline f32 source,
- f16 source,
- PQ/residual exact-rerank approximation.

Exit criteria:

- Produce a storage-vs-latency Pareto table.
- Use it only to quantify the maximum speed available from eliminating heap
  fetches. Do not default to sidecar storage without a separate product
  decision.

## Validation and Evidence Rules

Every reviewed slice must include a packet under `reviews/task-51/` or a
benchmark packet under `benchmarks/<topic>/` with:

- checked-in `ecaz bench suite` config when the run is a matrix/sweep,
- `manifest.md` naming head SHA, instance type, snapshot ID, reloptions,
  table names, q-count, warmup policy, and authoritative artifacts,
- structured results (`results.jsonl` or suite output) when available,
- recall@10, NDCG@10, latency p50/p95/p99, and relevant counters,
- explicit note whether the run uses isolated one-index-per-table surfaces.

Focused local smoke tests are allowed before AWS, but local results only
promote an experiment to AWS; they do not close the task.

## Exit Criteria

Task 51 closes when:

- the 1M counter baseline packet (Experiment 1) exists,
- at least two low-risk experiments have been measured (`nlists` sweep,
  heap-rerank locality, adaptive work reduction, or scratch SoA),
- any Posting Layout v2 work has either met its decision gate or been
  explicitly rejected with evidence,
- the final packet identifies the best latency/recall/storage Pareto points,
- and the remaining recommended work is split into follow-up tasks rather
  than left as chat-only notes.

## Dependencies and Coordination

- Builds on Task 28 / Task 31 IVF work and the AWS RaBitQ/IVF benchmark packet.
- Coordinate with Task 42 if a Posting Layout v2 metadata/version change
  proceeds.
- Coordinate with Task 47 for recall/cost-model gates and exact-KNN
  differential expectations.
- Coordinate with benchmark workflow owners before any new AWS snapshot or
  corpus rebuild; snapshot metadata must include `nlists`, `quant_bits`,
  `rerank`, `rerank_width`, and storage format.

---

## Amended Strategy (post-baseline-data, 2026-05-22)

The 1m baseline at head `94c02c682` shifts two priors:

1. The IVF + RaBitQ + `rerank='heap_f32'` recall curve is correct.
2. The remaining latency at high recall is dominated by a constant
   per-query cost — estimated ~18 ms of heap fetch + toast detoast on
   the `real[]` source column at 1m nprobe=128.

### Score of the experiment plan against the new data

| # | Expected gain | Confidence |
| --- | --- | --- |
| 2 | `nlists` / nprobe geometry sweep | 25-35% | med-high |
| 3 | Scratch SoA batch-decode | 20-30% candidates/sec | medium |
| 4 | Block-sort heap rerank | 5-10% | med-high |
| 5 | Adaptive nprobe / rerank_width | tail-latency only | medium |
| 6 | Posting Layout v2 | 25-35% | low (gated) |
| 7 | Sidecar measurement (f32, f16, bits=8) | ~3× | high |

Compounded best-case for Exp 2 + 3 + 4 + 6: **~2.0-2.2× on p50** —
non-architectural work alone is bounded.

### Cost decomposition (estimated, 1m nprobe=128)

| Component | ec_ivf |
| --- | --- |
| Centroid + RaBitQ scoring | ~10 ms |
| Posting iter + dedup | ~5 ms |
| Heap fetch (50 candidates) | ~15 ms |
| Toast detoast (`real[]`) | ~3 ms |
| **Total** | **~33 ms** |

The ~18 ms of heap+toast is **structurally invisible to Exp 2-6**.

### Pareto reframing — the operating points the data unlocks

| Configuration | 1m p50 @ 0.987 | 1m index size |
| --- | --- | --- |
| ec_ivf today (bits=1 + heap_f32 + w=50) | 33.8 ms | 1.5 GB |
| + Exp 2+3+4+6 (optimistic, no sidecar) | ~16-18 ms | 1.5 GB |
| + f16 sidecar (~3 GB extra) | ~12-15 ms (est.) | ~4.5 GB |
| + bits=8 RaBitQ sidecar (~1.5 GB extra) | ~15-18 ms (est.) | ~3 GB |
| + full f32 inline sidecar | ~9 ms (est. parity) | ~8 GB |

The interesting cells are the **middle Pareto points** (f16 sidecar,
bits=8 sidecar) — the smallest sidecar that closes most of the
latency gap.

### Amended sequencing

**Promote Experiment 7 (sidecar bounds) from "after everything else"
to "in parallel with Experiments 2 and 3."** Reasoning: Exp 2-6
collectively top out at ~2× on p50, while the heap+toast component
is invisible to them. Exp 7's upper-bound measurement sets the
ceiling so the rest of the round can target a known target.

Exp 7 stays measurement-only — it sets the upper bound. It does not
commit to a sidecar product direction.

Bounds to prove under Exp 7:

1. **Upper bound:** full inline f32 sidecar (same layout, our kernel
   work compounds on top).
2. **Middle bound:** f16 sidecar. Tells us how much of the latency
   win is bandwidth-bound vs format-bound. f16 IP via NEON `bfdot`
   is already-tested infrastructure.
3. **Smallest sidecar:** bits=8 RaBitQ sidecar — reuses the existing
   RaBitQ encoder, smaller index growth than f16 or f32.

### Methodology gaps to close on our own AWS measurement packets

From reviewer 2026-05-22-04, our own AWS measurement methodology had
known gaps to address before Task 51 closes:

- q=100 instead of q≥500 (cost reason logged in `paired-comparison.md`)
- Ad-hoc shell sweepers instead of an `ecaz bench suite` config
- Missing `suite-manifest.json` / `results.jsonl` aggregation

These are not blockers for starting Exp 2 / Exp 7, but they must close
before the round's final claim packet.

### Recommended start order (revised)

1. **Cleanup pass:** add 1m EXPLAIN counters (Task 51 Baseline #3),
   write packet `manifest.md` + `suite-manifest.json`, drive the next
   1m sweep via `ecaz bench suite`.
2. **In parallel:** Exp 2 (`nlists` sweep) and Exp 7 (sidecar upper
   bounds).
3. **After Exp 7 lands:** decide whether the round ships modest Exp 2-6
   wins + recommends sidecar as a future feature flag, *or* commits to
   a sized sidecar product decision now.
4. Exp 3 (SoA), Exp 4 (block-sort heap), Exp 6 (Layout v2) get
   prioritized only against what Exp 7 reveals about non-sidecar
   headroom.


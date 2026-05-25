# Task 61: HNSW AWS Graviton First-Pass Tuning

Status: **proposed**

Follow-on to the completed low-cost Graviton DiskANN Task 59 suite. This task
owns the first AWS Graviton optimization pass for `ec_hnsw`, using the same
review-packet and `ecaz bench suite` discipline. Task 33 remains the local M5
HNSW design/measurement lane; this task is specifically cloud Graviton evidence
and first-pass tuning.

## Goal

Establish a clean Graviton baseline for `ec_hnsw`, identify the first dominant
build/scan/config bottleneck, land only narrow optimizations or configuration
changes that are justified by the baseline, and prove the result with a
repeatable `ecaz bench suite` at 10k, 50k, 100k, and, when feasible for the
chosen low-cost profile, 1M rows.

## Why

HNSW is still the reference AM and fallback comparator, but it has not had the
same AWS Graviton treatment as IVF/RaBitQ and DiskANN. The existing HNSW task
is M5-focused and design-oriented. We need a cloud-native first pass that
answers:

- how current `ec_hnsw` behaves on low-cost Graviton,
- whether build time, scan latency, recall, memory, or storage dominates,
- which Graviton instance/storage profile is the right first candidate,
- which optimization slice is worth doing first, and
- what comparator rows HNSW should provide for future DiskANN/IVF decisions.

## Scope

- Access method: `ec_hnsw`.
- Hardware lane: AWS Graviton only.
- Initial profile: reuse a currently running low-cost Graviton profile when
  safe, preferably `10k-medium` while it remains available; do not tear it down
  during active cycles unless the operator explicitly asks.
- Candidate profiles before any Intel work:
  - `m8g.large` or equivalent cost-floor profile for small/medium cells;
  - `m8g.xlarge` / `m8g.2xlarge` for memory headroom and 1M feasibility;
  - `c8g.xlarge` or `c8g.2xlarge` only if CPU traversal/scoring is clearly the
    limiter;
  - `gd` variants only if EBS wait or local-storage sensitivity is measured.
- Required datasets for first pass: DBpedia/OpenAI3 `ec_real_10k`,
  `ec_real_50k`, and `ec_real_100k`.
- Target dataset for scale proof: `ec_real_1m`, unless the first-pass profile
  evidence shows HNSW 1M is not cost-valid on the low-cost Graviton lane; in
  that case, record the exact blocker and the next profile needed.
- Suite runner: all matrices, sweeps, retries, resumes, status, and reporting
  must use `ecaz bench suite` with checked-in JSON configs.

## Non-Goals

- Do not benchmark Intel in this task.
- Do not make broad HNSW design changes such as a native/offline builder
  replacement unless this task first produces a design packet and the operator
  accepts that scope expansion. That larger direction remains aligned with
  Task 33 / coder2 native-build tasks.
- Do not write ad hoc shell sweepers or one-off benchmark scripts.
- Do not use direct SSM as the normal interaction path. Prefer `ecaz cloud`
  commands and packet-local artifacts.
- Do not claim a speedup from one noisy run. Repeat same-head cells when the
  measured delta is near the run-to-run variance.

## Required First Step: Baseline Packet

Create a benchmark packet under `benchmarks/task61-aws-hnsw-graviton-baseline/`
with a checked-in `suite.json`. The baseline must include:

- host profile, instance id, region, EBS/local-storage shape, PostgreSQL
  settings, kernel/CPU/memory precheck, extension SHA, and suite config hash;
- load/build timing split where the existing loader exposes it;
- recall@10, latency mean/p50/p95/p99, storage, and explain rows;
- cache-state labels for latency rows using the `cache_state` suite field;
- 10k, 50k, and 100k cells at minimum;
- 1M attempt or an explicit, artifact-backed reason it was deferred.

Minimum sweep:

- HNSW tuning axis values that cover low, default, and high recall behavior for
  each scale. Use the existing `ec_hnsw` profile defaults as the starting
  point, then widen only if recall/latency curves are under-sampled.
- `concurrency=1` first for clean scan latency; concurrent QPS is a later cell
  once the single-query curve is understood.

## First-Pass Tuning Questions

Answer these before editing production HNSW code:

1. **Build bottleneck.** Is 1M blocked by graph build wall time, PostgreSQL
   worker headroom, memory pressure, disk, or a loader/staging issue?
2. **Scan bottleneck.** At fixed recall, is latency dominated by graph tuple
   reads, scoring, candidate heap churn, duplicate expansion, planner/path
   setup, or memory allocation?
3. **Profile shape.** Does `m8g` memory headroom matter more than `c8g` CPU
   shape for HNSW? Is EBS visible enough to justify a `gd` variant?
4. **Comparator value.** At 100k and any successful 1M cell, where does HNSW
   sit relative to the Task 59 DiskANN rows on recall, latency, build time, and
   storage?

## Candidate Optimization Slices

Only promote a slice after the baseline points at it.

### 1. Graviton Profile and PostgreSQL Config

Tune instance size, memory settings, and storage before invasive code changes
if the evidence shows host shape is masking the implementation signal.

Promotion criteria:

- lower p50 or p95 at a recall-equivalent cell without unacceptable cost
  increase, or
- a clear 1M feasibility improvement such as removing swap/disk pressure or
  cutting build variance.

### 2. Build-Time Hot Path

If build dominates, profile HNSW build phases before changing code. Candidate
areas include worker fanout, DSM/shm_mq ingestion, graph construction, tuple
packing, and WAL/page flush behavior.

Promotion criteria:

- build wall time or graph phase time improves by at least 15% on 100k, or
  a 1M run becomes feasible on the selected Graviton profile;
- recall and storage remain within documented tolerance.

### 3. Scan Hot Path

If scan dominates at useful recall, inspect the live scan path for redundant
tuple decode, candidate heap churn, scoring, and allocation.

Promotion criteria:

- at least 10% p50 or p95 improvement at a recall-equivalent 100k cell, or
  a measured allocation reduction that justifies a follow-up latency target;
- recall@10 does not regress outside the agreed tolerance.

### 4. Comparator Suite Hardening

If the main blocker is measurement quality, extend `ecaz bench suite` rather
than working around it. Missing pieces may include HNSW-specific build timing
rows, cache-state enforcement, host diagnostics, or repeat-run reporting.

## Deliverables

- Task-local benchmark packet under `benchmarks/task61-aws-hnsw-graviton-*`.
- Any code or config changes as narrow commits with review packets under
  `reviews/task-61/`.
- Final summary comparing HNSW Graviton rows against the Task 59 DiskANN
  Graviton rows at 100k and any completed 1M cell.
- Clear statement of whether the next HNSW step is Graviton config tuning,
  build-path optimization, scan-path optimization, or a larger design task.

## Stop Conditions

- Stop after the baseline packet if HNSW 1M is dominated by build time and no
  first-pass config change can make it cost-valid; hand off to a design task
  for native/offline build work.
- Stop if same-head repeat variance is larger than the proposed optimization
  delta; add measurement instrumentation instead of claiming a win.
- Stop before Intel. Intel comparator work gets its own task after this
  Graviton first pass.

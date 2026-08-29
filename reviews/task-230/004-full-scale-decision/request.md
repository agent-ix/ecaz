---
task: 230
packet: 004-full-scale-decision
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 02
---

# Task 230 full-scale decision preregistration

Review the two seq-01 policy corrections below against the unchanged checked-in
suite at `5837f4bec` before any full-scale result is produced. The accepted
20-step shape and entry gates are not reopened. Packet 003 is review-closed
DONE. This packet alone decides **PROMOTE** or **STOP**.

Seq-02 changes only the two findings in
`feedback/2026-08-29-01-reviewer.md`: storage ratios now use stated and measured
denominators, and every preregistered directional prediction receives an
explicit supported/falsified disposition.

## Frozen matrix

The standard suite is
`crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json`:

- 12 primary steps: fresh row-heap versus fresh hot/cold at 10k, 50k, and
  100k, with pair A running row-heap first and pair B running hot/cold first;
- two independent counterbalanced primary pairs per scale;
- recall, warm latency/tails, storage, build, DML, stage counters, and isolated
  id-only I/O in every primary step;
- eight additional fresh 100k steps: matched row-heap/hot-cold attribution for
  exact-vector, cold-only, mixed, and select-all projections; and
- id-only and hot-scalar are deliberately one identical measurement under two
  labels, because both project only `id` in the frozen fixture.

The 10k/50k/100k primary format matrix supplies the mandatory per-scale
recall/latency/storage A/B. The secondary projection gates run at the largest
decision scale because they are cost guardrails, never alternative promotion
wins. Cold-only, mixed, and select-all use the same external uncompressed TOAST
fixture in both matched arms. Every shape has a fresh fixture; reuse is absent.

All 20 run directories are distinct children of `~/.ecaz/clusters` and are
removed after durable capture. Task 229 covering sidecars and Task 231 blocks
are absent. No step sets `allow_debug_extension`.

## Release and lint entry gates

Before the real run:

1. reinstall the extension **after** Packet 003's `pg_test` install with
   `cargo pgrx install --release --pg-config /home/peter/.ecaz/toolchains/pg18-ssl/bin/pg_config --no-default-features --features 'pg18 distann-head-attribution-benchmark'`;
2. rebuild `ecaz-cli` from the same head;
3. run `cargo clippy -p ecaz-cli --all-targets`; the recorded 77-warning binary
   and 78-warning test baselines may shrink but must not grow; and
4. require every multinode release preflight to report the preregistered SHA,
   `extension_build_profile=release`, and `debug_override=false`.

Any failed entry gate invalidates the run; it is fixed and rerun before results
are interpreted.

## Frozen decision math

For every matched pair, control is the row-heap step and candidate is the
hot/cold step. Percentage improvement is
`100 * (1 - candidate/control)`; absolute improvement is
`control - candidate`. No averaging can rescue a failed pair.

PROMOTE requires every gate below. Otherwise the disposition is STOP.

### 1. Semantic and quality gates

- All selected steps, suite thresholds, release preflights, topology checks,
  stage/calibration checks, and NFR-021 rows pass.
- Corpus/query slice digests match within every pair; physical prediction
  files are byte-identical within every primary pair.
- Candidate `distinct_recall` and membership recall may not be more than
  0.001 absolute below its matched control in any primary pair.
- The suite's absolute recall floors also pass: 0.995 at 10k, 0.980 at 50k,
  and 0.900 at 100k for every primary physical arm.
- No non-owned graph row, corruption signal, or invalid hot/cold pair is
  admitted.

### 2. Primary id-only/hot-scalar latency gate

- In **both** independent 100k pairs, candidate warm-concurrency-1 mean latency
  must improve by at least **5.0% and 0.50 ms**.
- At 10k and 50k, and for 100k p95/p99, a candidate value fails the guardrail
  only when it regresses by both more than **5.0% and 0.50 ms**. This combined
  rule prevents sub-millisecond noise from deciding the task while rejecting a
  material tail regression.
- The two 100k mean wins are conjunctive. There is no mean-of-pairs tie-break.

### 3. Storage, build, and DML guardrails

- The measured candidate hot main-heap bytes — the sum of published
  `physical_topology.row_bytes` across the three owners — must be at most
  **1.35×** `physical_benchmark_storage.raw_vector_bytes`. Both sides refer to
  the indexed exact vector: one 8 KiB hot page per 6,144-byte logical vector is
  about 1.333× before small relation overhead, directly testing the predicted
  PLAIN page amplification.
- Separately, candidate `physical_generation_bytes` must be at most **1.15×**
  its matched row-heap control in every primary pair at every scale. At 100k,
  the preregistered arithmetic is about +190 MB (819 MB PLAIN hot pages minus
  roughly 633 MB control TOAST storage, plus about 4 MB of tuple/locator
  overhead) on the Task 229 control's 2,498 MB generation, or about 1.08×.
  The 1.15× ceiling leaves real margin while still catching an unexplained
  whole-generation expansion.
- Candidate physical build and publish time must each be at most **1.25×** the
  matched control in every primary pair.
- For routed replacement and delete, candidate p95 must be at most **1.50×**
  matched control; the insert arm must preserve exact row distributions and
  achieve at least **0.67×** control throughput. A gate is not attributed if
  the two arms do not have the same fixture schema and payload.

### 4. Mechanism and secondary projection guardrails

- Hot/cold id-only and exact-vector attribution must show zero cold-tier
  heap/TOAST/tidx accesses; cold-only must show zero hot-tier accesses. Any
  violation is STOP because it falsifies tier laziness.
- At 100k exact-vector projection fails only if hot/cold elapsed time per query
  is both more than **5.0%** and **0.25 ms** worse than row-heap.
- At 100k each of cold-only, mixed, and select-all fails its secondary cost gate
  if hot/cold elapsed time per query is both more than **50%** and **1.0 ms**
  worse than the matched row-heap arm. Cold/mixed regressions inside this bound
  are disclosed costs, not promotion wins.
- Report per relation and tier all six `pg_statio_all_tables` deltas (heap,
  TOAST heap, TOAST index reads/hits), total accesses/hits, and shared-buffer
  hit ratio for every shape. Shared-buffer hit ratio is explicit attribution,
  not a post-hoc alternative decision metric.

### 5. Directional prediction accounting

The final PROMOTE/STOP request must classify every packet-001 §6 timing
prediction as **supported** or **falsified**, independent of whether its numeric
guardrail passes:

- id-only/hot-scalar: improve or remain neutral;
- exact-vector: improve;
- cold-only: regress; and
- mixed and select-all: regress.

For an `elapsed_ms / iterations` secondary comparison, equality is neutral;
strictly lower is improvement and strictly higher is regression. A flat or
worse exact-vector result is therefore explicitly a falsified prediction even
when it stays inside the 5%/0.25 ms guardrail. Conversely, an unexpectedly
neutral or faster cold/mixed/all result is also reported as falsified rather
than silently rewritten as success. Prediction classification is part of the
decision record; the numeric gates above still determine PROMOTE versus STOP.

## Review request

Please review only the corrected storage denominators/ceilings and the explicit
directional-prediction disposition. If DONE, the real suite is authorized at
the otherwise unchanged frozen config and policy.

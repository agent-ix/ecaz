---
task: 230
packet: 004-full-scale-decision
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 01
---

# Task 230 full-scale decision preregistration

Review the checked-in suite at `5837f4bec` and the numeric decision policy
below before any full-scale result is produced. Packet 003 is review-closed
DONE. This packet alone decides **PROMOTE** or **STOP**.

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

- Candidate `physical_generation_bytes` must be at most **1.35×** its matched
  row-heap control in every primary pair at every scale. This admits the
  preregistered roughly 30% PLAIN hot-page cost but not an unbounded second
  storage copy.
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

## Review request

Please review the 20-step suite shape, the release/lint entry gates, and every
numeric threshold. In particular, confirm that the largest-scale secondary
shape matrix and the conjunctive cold/mixed failure rule satisfy the carried
Packet 003 requirements. If DONE, the real suite is authorized at this frozen
config and decision policy.

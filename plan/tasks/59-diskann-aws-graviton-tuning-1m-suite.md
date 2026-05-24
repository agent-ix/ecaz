# Task 59: DiskANN AWS Graviton Tuning and 1M Benchmark Suite

Status: **proposed**

Follow-on to the 2026-05-24 low-cost Graviton DiskANN optimization that removed
per-scan full-index materialization from the normal scan path.

Task 55 remains the unsafe-burndown task. This task owns the next AWS-backed
`ec_diskann` performance cycle: tune on AWS Graviton, profile the remaining
latency after `cbf037334ce0a9f499507d206049574b8278282e`, land targeted
optimizations/config changes one at a time, and prove the result with a full
`ecaz bench suite` through 1M rows.

## Why

The first AWS optimization changed the 100k low-cost Graviton latency shape
from a flat fixed-cost band around `61.9-64.8 ms` to a scan-width-sensitive
curve of `1.72-10.6 ms` across `list_size` 64 through 800, with no recall or
storage regression.

That fixed-cost bug is closed, but the remaining curve still has real work:

- `list_size=400` is `5.88 ms` mean on 100k.
- `list_size=800` is `10.6 ms` mean on 100k.
- 100k recall is healthy and unchanged, so we should optimize execution cost,
  not graph quality first.
- We do not yet have a complete post-optimization AWS Graviton suite through
  1M, so the next task must both tune and prove the tuned path at 10k, 100k,
  and 1M.
- The live path now reads graph nodes on demand from the relation, so the next
  bottleneck is likely buffer/page reads, tuple decode, allocation churn,
  duplicate expansion, scoring, EBS/local-storage behavior, CPU saturation, or
  some combination of those.

## Baseline Evidence

Authoritative packets:

- before/config audit:
  `benchmarks/task55-aws-diskann-lowcost-config-audit/`
- first optimized run:
  `benchmarks/task55-aws-diskann-lowcost-optimized/`
- review packet:
  `reviews/task-55/005-aws-diskann-scan-optimization/`

The baseline commit for this task is:

- `cbf037334ce0a9f499507d206049574b8278282e`
  (`Optimize DiskANN scan materialization`)

The benchmark evidence commit is:

- `5499ebbff16b60930aee489500b9e06ff4a71117`
  (`Record DiskANN AWS optimization benchmarks`)

## Scope

- Access method: `ec_diskann`.
- Hardware lane: AWS Graviton only.
- Initial cloud profile: keep using `10k` (`m8g.large`) for low-cost iteration
  while it is already up, but this task must include Graviton profile tuning
  when 1M requires it.
- Required Graviton candidates before Intel:
  - `m8g.large` as the cost-floor reference when it can run the cell without
    swapping/noise dominating;
  - `m8g.xlarge` or `m8gd.xlarge` to separate memory headroom and EBS/local
    storage behavior;
  - `c8g.xlarge` or `c8gd.xlarge` if counters show CPU traversal/scoring is
    the likely limit.
- Required datasets: DBpedia/OpenAI3 `ec_real_10k`, `ec_real_100k`, and
  `ec_real_1m`.
- Suite runner: `ecaz bench suite` only. If the suite runner lacks a needed
  profiling step or metric, extend `ecaz-cli` first.

## Non-Goals

- Do not benchmark Intel in this task. Add a later comparator task once the
  Graviton path has a tuned 1M result.
- Do not tear down the current AWS `10k` profile during active cycles unless
  the operator explicitly asks for teardown.
- Do not use direct SSM as the routine interaction path. Prefer `ecaz cloud`
  commands, S3 artifact sync, and packet-local artifacts.
- Do not change DiskANN on-disk tuple format in this round unless counters
  prove tuple layout is the dominant bottleneck and a separate design packet is
  accepted.
- Do not write ad hoc benchmark sweep scripts. Extend `ecaz bench suite` for
  any missing matrix, profiling, resume, or reporting behavior.

## Required First Step: Graviton Counter/Profile Baseline

Before changing behavior again, collect a counter baseline on the optimized
commit. The profiler/counter surface must separate at least:

- graph nodes requested,
- relation blocks touched,
- graph/data tuples decoded,
- overflow heap-TID chains inspected,
- duplicate candidates filtered,
- candidate heap pushes/pops,
- visited-set lookups/inserts,
- distance/PQ scores computed,
- total approximate scan time,
- result materialization time,
- memory allocations if available through an existing allocator/profiler.

Minimum counter cells:

- 100k `list_size` 200, 400, 800.
- 1M `list_size` 200, 400, 800 on the smallest Graviton profile that can run
  without memory pressure invalidating the measurement.
- 10k `list_size` 200 and 800 as a small-corpus sanity check.

The packet must make a clear call on both:

- the next bottleneck class, and
- the right Graviton profile/storage shape for the full 1M suite.

## Graviton Tuning Work

Tune AWS Graviton configuration before or alongside code work when counters
show the host shape is masking the implementation signal.

Required tuning dimensions:

- instance family and size among the low-cost Graviton candidates listed in
  Scope;
- EBS vs local NVMe where `gd` variants are available;
- PostgreSQL memory settings that materially affect this workload
  (`shared_buffers`, `effective_cache_size`, `work_mem`,
  `maintenance_work_mem`);
- EBS volume class/throughput only if wait counters or latency traces show EBS
  pressure;
- benchmark warm/cold cache protocol, recorded explicitly in the suite
  manifest.

Promotion criteria:

- Pick one Graviton profile/config as the tuned candidate for the full 1M
  suite.
- Keep the cost-floor result as a reference row when it can run validly.
- Record why rejected Graviton profiles were rejected: memory pressure, EBS
  wait, CPU saturation, noisy tail latency, cost/benefit, or no improvement.

## Code Optimization Candidates

### 1. Per-Scan Page or Node Cache

The relation-backed reader avoids full materialization, but it may reread or
redecode the same page/node during greedy expansion and duplicate expansion.

Candidate implementation:

- bounded scan-local cache keyed by block number plus tuple offset or node TID,
- page-local tuple decode cache if page rereads dominate,
- no cross-scan stale pointer retention,
- no heap TID overflow expansion unless `has_overflow_heaptids` is true.

Promotion criteria:

- At least 15% mean or p50 improvement at 100k `list_size=400` or
  `list_size=800`, with no p95 regression larger than noise.
- Recall@10 exactly unchanged for the same query set.
- Cache memory is bounded and recorded.

### 2. Allocation and Scratch Reuse

After the fixed-cost bug, allocation churn in candidate maps, heaps, visited
sets, and result buffers may be visible.

Candidate implementation:

- pre-size scan-local maps/sets from effective `list_size`,
- reuse scratch buffers across `amrescan` calls where PostgreSQL lifetime rules
  allow it,
- replace high-churn structures only when counters prove they are hot.

Promotion criteria:

- At least 10% mean or p50 improvement at one high-recall cell, or a measured
  allocation reduction that explains a follow-up latency target.
- No recall drift and no memory-context lifetime risk.

### 3. Batched Graph Reads or Prefetch

If relation/page reads dominate, test whether batch-oriented reads or prefetch
can hide EBS/buffer latency.

Candidate implementation:

- batch candidate neighbor reads where the traversal frontier already exposes
  upcoming nodes,
- use an existing PostgreSQL-safe prefetch/read-stream surface if available,
- keep ordering and visited semantics identical.

Promotion criteria:

- At least 15% p95 improvement on the high-`list_size` cells, or clear
  reduction in block-read wait time with no mean regression.

### 4. PQ/Distance Fast-Path Audit

If scoring dominates after read/decode fixes, audit the `pq_fastscan` hot path
on Graviton.

Candidate implementation:

- verify the active storage format and scoring kernel in every benchmark row,
- compare scalar vs NEON-dispatched paths where applicable,
- remove avoidable query-side recomputation or branchy inner-loop work.

Promotion criteria:

- At least 10% scoring-time reduction in counters or at least 10% latency
  improvement in a scoring-dominated cell.
- Scalar/SIMD differential tests or equivalent exactness evidence for any
  scoring change.

### 5. Build-Time Follow-Up

Query latency is first priority. Build-time work is allowed only after scan
profiling shows query latency is no longer the best next optimization target.

Known current reference:

- 100k wide-alpha build probe stayed around `407-417 s` with
  `reachable_fraction=1.000000` and `recall@10=0.8630`.

## Full Benchmark Matrix Through 1M

Every optimization slice must use a checked-in `SuiteConfig` under the owning
benchmark packet. The closing suite must include 10k, 100k, and 1M rows.

Required rows for each corpus size:

- load/build timing for `ec_diskann`;
- recall@10 sweep at `list_size` 64, 128, 200, 400, 800 unless the 1M suite
  needs a documented narrower warmup plus final frontier sweep;
- latency sweep at the same `list_size` values;
- storage;
- explain/config diagnostics proving planner scan selection is live;
- graph/build-probe rows sufficient to rule out graph-quality regressions.

Required hardware/config rows:

- cost-floor Graviton reference where valid;
- tuned Graviton candidate;
- any intermediate Graviton profile used to justify the tuning choice.

The 1M row is mandatory for closeout. If a specific low-cost profile cannot run
1M validly, the task must move to the next Graviton candidate and record the
failed profile as a tuning result, not close at 100k.

## Acceptance Criteria

This task can close when:

- A Graviton counter/profile baseline identifies the dominant remaining
  bottleneck after `cbf037334`.
- AWS Graviton tuning selects a cost-aware profile/config for the full suite.
- At least one new targeted code optimization lands.
- The AWS benchmark packet includes the full 10k / 100k / 1M suite and all
  steps succeed or any skipped cells are explicitly justified as invalid
  hardware-profile tuning attempts.
- 100k and 1M recall@10 do not regress at any measured `list_size`.
- 100k and 1M storage do not increase unless the request explicitly justifies
  the tradeoff.
- The strongest optimized high-recall cell improves by at least 15% mean or
  p50 over the Task 55 optimized packet at 100k and establishes the tuned 1M
  Graviton number, or the task records evidence that the next material gain
  requires a larger design task.
- A review packet under `reviews/task-59/` points at all packet-local benchmark
  artifacts and states whether AWS was left running or intentionally shut down
  by operator request.

## References

- `reviews/task-55/005-aws-diskann-scan-optimization/request.md`
- `benchmarks/task55-aws-diskann-lowcost-config-audit/manifest.md`
- `benchmarks/task55-aws-diskann-lowcost-optimized/manifest.md`
- `plan/tasks/32-diskann-m5-optimization.md`
- `plan/tasks/51-ivf-rabitq-second-optimization-round.md`

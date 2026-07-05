# Task 74 Intel Profiler Baseline

Reviewer: please review this Task 74 profiler packet.

## Scope

The operator clarified that this host is the local Intel desktop and asked to
capture the Intel baselines. This packet therefore records an Intel-local
server-side `perf`/`flamegraph` profile for the Task 73 high-recall SPIRE point
and the matched IVF control.

This is not an M5 `samply` packet. The task remains pending reviewer approval
on whether the Intel-local profile satisfies the profiler gate or whether an
M5-specific profiler rerun is still required.

## Suite Baseline

The run used `ecaz bench suite` with checked-in config:
`reviews/task-74/005-intel-profiler-baseline/artifacts/suite.json`.

Host precheck:

- PostgreSQL: `18.3`, `x86_64-pc-linux-gnu`
- Socket: `/home/peter/.pgrx`, port `28818`
- Extension: `ecaz 0.1.1`
- Corpus: local `ec_real_100k` staged files under
  `target/real-corpus/staged-task50/`

Results at nprobe `96`:

| Surface | p50 | p95 | p99 | Mean | Count |
| --- | ---: | ---: | ---: | ---: | ---: |
| SPIRE `tg128/b0`, `rerank_width=25` | 137.9 ms | 151.4 ms | 161.8 ms | 138.8 ms | 200 |
| IVF `pq_fastscan`, `rerank_width=500` | 37.8 ms | 44.9 ms | 49.8 ms | 38.6 ms | 200 |

The Intel-local SPIRE/IVF p50 ratio is `3.65x` at this matched nprobe control.
The longer profiler driver runs measured SPIRE p50 `133.7 ms` and IVF p50
`36.9 ms`, a `3.62x` p50 ratio.

## Profiler Evidence

`perf record` system-wide mode was blocked by the host's
`perf_event_paranoid=2`, so the profile attached directly to each PostgreSQL
backend PID while `ecaz bench latency` drove the query workload.

Profiler samples:

- SPIRE: 2,947 samples, no lost samples
- IVF: 2,894 samples, no lost samples

Key self-time symbols:

| Phase / symbol group | SPIRE self-time | IVF self-time |
| --- | ---: | ---: |
| Actual quantized scoring | `75.06%` in `ProdQuantizer::score_ip_from_split_parts` | `42.92%` in `IvfQuantizer::score_ip_from_parts_with_min_bound` |
| IVF allocator/randomization path | `0.75%` in `randomize_mem` | `30.96%` in `randomize_mem` |
| SPIRE leaf/object decode + validation | `3.57%` combined across `decode_leaf_v2_vec_id`, `SpireLeafPartitionObjectV2Segment::validate_against_meta`, `SpireRoutingPartitionObject::decode` | not applicable |
| SPIRE candidate collection/budget surface | `1.36%` combined across `collect_validated_quantized_leaf_route_candidates` and `SpireScoredCandidateAccumulator::append` | not applicable |
| IVF scan/rerank visible overhead | not applicable | `7.47%` combined across `ec_ivf_amrescan` closure and `rerank_probe_candidates` |

Interpretation:

- The visible SPIRE profile is dominated by the actual quantized scoring path,
  not by recursive routing or candidate-budget bookkeeping.
- The identifiable SPIRE-specific orchestration above scoring is about `4.9%`
  of samples in this run (`3.57%` storage/routing-object work plus `1.36%`
  candidate collection/accumulation).
- The reviewer-requested distinction between `pg_am_callback!`-bounded work and
  routing draft consumption is only partially visible in this Linux perf sample:
  Rust symbols for storage/routing/candidate surfaces are present, but a large
  tail of callback/PG executor frames is not symbolized cleanly enough to claim
  a precise callback-boundary split.
- Because visible orchestration is below Task 74's `~10%` stop-condition floor,
  this packet does not propose Phase 2 SPIRE orchestration code slices. The
  measured latency gap appears to come primarily from doing substantially more
  scoring work at the quality-preserving SPIRE point, not from a single obvious
  routing/candidate bookkeeping hotspot.

## Artifacts

- Suite config: `artifacts/suite.json`
- Suite run log: `artifacts/suite-run.log`
- Suite manifest: `artifacts/suite-manifest.json`
- Suite report: `artifacts/suite-report.md`
- Parsed results: `artifacts/results.jsonl`
- SPIRE latency log: `artifacts/latency-100k-spire-highrecall-tg128-b0.log`
- IVF latency log: `artifacts/latency-100k-ivf-control.log`
- SPIRE profiler driver: `artifacts/profile-driver-spire-nprobe96.log`
- IVF profiler driver: `artifacts/profile-driver-ivf-nprobe96.log`
- SPIRE perf data: `artifacts/intel-spire-nprobe96.perf.data`
- IVF perf data: `artifacts/intel-ivf-nprobe96.perf.data`
- SPIRE flamegraph: `artifacts/intel-spire-nprobe96-flamegraph.svg`
- IVF flamegraph: `artifacts/intel-ivf-nprobe96-flamegraph.svg`
- SPIRE symbol report: `artifacts/intel-spire-nprobe96-perf-report-symbols.txt`
- IVF symbol report: `artifacts/intel-ivf-nprobe96-perf-report-symbols.txt`
- Manifest: `artifacts/manifest.md`

## Requested Review Decision

Please confirm one of:

- accept this Intel-local packet as satisfying Task 74's profiler gate and
  approve the no-Phase-2-slice closeout direction; or
- require an M5-local `samply`/`cargo flamegraph` rerun before Task 74 can move
  to complete.

# Task 31 M5 IVF Optimization Plan

Reviewer: please review this planning packet before the first real M5 IVF
baseline pass.

## Scope

This packet turns the Task 28 IVF evidence trail plus the Task 31 M5 setup and
smoke packets into a staged optimization loop. It does not make new measurement
claims, does not run benchmarks, and does not change code.

Task 31 stays local-development scoped: M5 measurements are evidence for
choosing and validating local implementation checkpoints, not product-class
cloud claims.

## Evidence Inventory

### Task 28 baseline and tuning packets

The strongest landed local IVF line is explicit PQ-FastScan group size 8 with
heap-f32 rerank.

- `30088` and `30089` established the first 25k PQ-FastScan g8 frontier.
  `nlists=64,nprobe=48,rerank_width=750` measured recall@10 `0.9940` with
  latency p50 `140.0 ms` before later A7 pruning.
- `30090` moved PQ-FastScan g8 to 100k at `nlists=64,rerank_width=750`.
  It reached recall@10 `0.9930` at `nprobe=32` and `1.0000` at `nprobe=48`,
  but latency was high enough to motivate more list-count tuning.
- `30092`, `30093`, `30094`, and `30095` found the current 100k shape.
  `nlists=128,nprobe=48,rerank_width=500` is the low-latency high-recall@10
  point; `nlists=256,nprobe=96` is a quality-biased alternative with longer
  build time.
- `30111`, `30113`, `30118`, and `30119` refreshed the selected 100k point at
  current head. The fresh rebuild packet recorded build `216788.531 ms`, index
  size `19,791,872` bytes, recall@10 `0.9920`, recall@100 `0.9552`, latency
  p50/p95/p99 `173.4/225.4/242.9 ms`, and HWM `157108 kB`.
- `30130`, `30132`, `30133`, `30135`, `30136`, `30149`, and `30150` are the
  990k local scale lane. The selected 990k surface built in `33:53.835`, had
  index size `177 MB`, and showed that `nprobe=40,rerank_width=500` is the
  recall-backed balanced local point while fresh 990k exact-recall fills are
  too costly for routine desktop gating.
- `30135` is the key bottleneck clue: on 990k, `nprobe=32/40/48` visited
  `253879/315958/372944` postings and read `5795/7212/8511` posting pages,
  while postings scored stayed nearly flat at `3228/3232/3235` because
  PQ-FastScan bound pruning rejected most later postings.
- `30137` closed the 10k/25k A7 PQ-FastScan bound-prune gap. At
  `nlists=64,nprobe=48,rerank_width=750`, recall stayed `0.9910` on 10k and
  `0.9940` on 25k, while p50 improved to `77.3 ms` and `116.8 ms`; counters
  showed active pruning.
- `30068`, `30069`, and `30070` landed smaller scan-path cleanups: pre-rerank
  top-k and borrowed posting scans. They were useful but not the main lever.
- `30072` and `30078` preserved negative trials. Post-score frontier pruning
  and TurboQuant suffix-bound variants did not win because they attacked work
  after, or around, the wrong hot path.
- `30073` proved score-kernel cost can matter: TurboQuant LUT scoring cut p50
  by roughly 35-40% on the old TurboQuant surface while preserving recall.
- `30142` proved churn needs an explicit build-time tradeoff:
  `posting_slack_percent=50` kept a 100k 10-cycle rotating-window workload flat,
  while default slack `0` had grown materially in packet `30141`.
- `30145` and `30151` summarize the local recommendation: keep `auto`
  unchanged, but recommend explicit `pq_fastscan,pq_group_size=8` for larger
  high-dimensional IVF surfaces where speed and index size dominate.

### Task 31 packets

- `30162` proved the M5 environment can build the repo, install PG18, run
  `ecaz dev sql`, and use the operator CLI. It also recorded no real benchmark
  corpus was loaded yet.
- `30163` proved an end-to-end synthetic 10k IVF smoke on the M5:
  `profile=ec_ivf`, `storage_format=pq_fastscan`, `pq_group_size=8`,
  `nlists=128`, `nprobe=8`, `rerank=heap_f32`, `rerank_width=500`. It copied
  10k synthetic rows, built in `4.76s`, ran a 3-query recall smoke, and recorded
  storage.
- `30164` documented the operator rule: use `/Users/peter/.cargo/bin/ecaz` for
  local PG18/pgrx, SQL, corpus, benchmark, storage, and setup operations.

### Evidence classes

- Local synthetic: Task 31 packet `30163`; useful only for plumbing and command
  validation.
- Local real corpus: Task 28 DBPedia-derived 10k, 25k, 100k, and 990k packets;
  useful for M5 baseline shape, but originally measured on a different local
  machine unless rerun under Task 31.
- Cloud/product-grade: none in this plan. Product-class claims remain deferred.

### Coverage gaps

- 10k: Task 28 has useful PQ-FastScan g8 evidence; Task 31 M5 has only
  synthetic smoke. Need real M5 10k recall, latency, storage, build-time,
  memory, and EXPLAIN/counter capture.
- 25k: Task 28 has good local evidence; Task 31 M5 has no real 25k baseline.
- 100k: Task 28 has the best selected-point evidence; Task 31 M5 needs a fresh
  real 100k rebuild and current-head M5 baseline.
- 990k: Task 28 has directional local evidence and clear exact-recall harness
  cost. Task 31 should make 990k optional and only run narrow latency/counter
  checks unless a specific hypothesis needs it.

## Baseline Matrix

The first pass should be small enough to finish and still classify the first
bottleneck.

Recommended surfaces:

| surface | status | reloptions | initial sweep |
|---|---|---|---|
| real 10k | required | `storage_format=pq_fastscan,pq_group_size=8,nlists=64,nprobe=48,rerank=heap_f32,rerank_width=750` | fixed point |
| real 25k | required | same as 10k | fixed point |
| real 100k | required | `storage_format=pq_fastscan,pq_group_size=8,nlists=128,nprobe=48,rerank=heap_f32,rerank_width=500` | fixed point, optional `nprobe=40,56` |
| real 990k | optional | `storage_format=pq_fastscan,pq_group_size=8,nlists=128,nprobe=40,rerank=heap_f32,rerank_width=500` | latency/counters first, recall only with cache |

Starting points come directly from Task 28:

- 10k/25k: `nlists=64,nprobe=48,rerank_width=750` because packet `30137`
  confirmed the smaller-corpus PQ-FastScan g8 A7 frontier.
- 100k: `nlists=128,nprobe=48,rerank_width=500` because packets `30094` and
  `30119` made it the selected current point.
- 990k: `nlists=128,nprobe=40,rerank_width=500` because packets `30132` and
  `30133` identified it as the balanced local point.

Required captures per required surface:

- Build-time: capture `corpus load` or explicit build SQL output in the packet.
- Recall: `k=10`, `queries-limit=100`; add `k=100` for 100k and any baseline
  point being considered as quality-sensitive.
- Latency: `k=10`, `iterations=100`, `--force-index`,
  `--sample-backend-memory --memory-sample-interval-ms 25`.
- Storage: `ecaz bench storage`.
- EXPLAIN/counters: one representative query per point with `ecaz dev sql` and
  packet-local SQL, including selected lists, posting pages read, postings
  visited/scored/pruned, candidates inserted, duplicate filters, rerank rows,
  execution time, and buffer blocks when emitted.
- Cache state: first pass is warm local development unless the packet explicitly
  defines a restart/drop policy.

Do not run all sweeps up front. The first pass should establish fixed-point
10k, fixed-point 25k, fixed-point 100k, and one 100k adjacent `nprobe` check
only if the fixed point looks noisy or surprising.

## Measurement Protocol

Packet structure:

```text
review/{NN}-{topic}/
  request.md
  artifacts/
    manifest.md
    load_<surface>.log
    recall10_<surface>.log
    recall100_<surface>.log
    latency_<surface>.log
    storage_<surface>.log
    explain_<surface>.sql
    explain_<surface>.log
```

Artifact names should encode rows, storage, list/probe/width, and purpose, for
example `latency_real100k_pqg8_n128_p48_w500.log`.

Command patterns:

```sh
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  --log-file review/30166-task31-m5-real-corpus-preflight/artifacts/corpus-list.log \
  corpus list

/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  --log-file review/<packet>/artifacts/load_real100k_pqg8_n128_w500.log \
  corpus load --prefix task31_m5_real100k_pqg8_n128 \
  --profile ec_ivf \
  --corpus-file data/<staged>/task31_m5_real100k_corpus.tsv \
  --queries-file data/<staged>/task31_m5_real100k_queries.tsv \
  --reloption storage_format=pq_fastscan \
  --reloption pq_group_size=8 \
  --reloption nlists=128 \
  --reloption nprobe=48 \
  --reloption rerank=heap_f32 \
  --reloption rerank_width=500

/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  bench recall --prefix task31_m5_real100k_pqg8_n128 \
  --profile ec_ivf --k 10 --queries-limit 100 --sweep 48 \
  --rerank-width 500 --force-index \
  --truth-cache-file review/<packet>/artifacts/truth_real100k_k10.json \
  --log-output review/<packet>/artifacts/recall10_real100k_pqg8_n128_p48_w500.log

/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  bench latency --prefix task31_m5_real100k_pqg8_n128 \
  --profile ec_ivf --k 10 --iterations 100 --sweep 48 \
  --rerank-width 500 --force-index \
  --sample-backend-memory --memory-sample-interval-ms 25 \
  --log-output review/<packet>/artifacts/latency_real100k_pqg8_n128_p48_w500.log

/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  --log-file review/<packet>/artifacts/storage_real100k_pqg8_n128.log \
  bench storage --prefix task31_m5_real100k_pqg8_n128

/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres \
  --socket-dir /Users/peter/.pgrx --port 28818 --raw \
  --file review/<packet>/artifacts/explain_real100k_pqg8_n128_p48_w500.sql \
  --log-output review/<packet>/artifacts/explain_real100k_pqg8_n128_p48_w500.log
```

Cache/warmup policy:

- Baseline pass: warm local development cache, stated explicitly.
- Latency repeat: run one unclaimed warmup latency pass only if the harness does
  not already prime; only cite the measured artifact.
- Cold-cache or restart claims need their own packet and explicit PG restart or
  OS-cache policy; do not mix them into the first baseline.

Surface rule:

- Use one-index-per-table prefixes for baseline and optimization comparisons.
- Use shared-table surfaces only when the packet explicitly measures planner or
  shared-table behavior.

Manifest requirements:

- Head SHA, packet/topic, timestamp, machine/OS/CPU/memory, PostgreSQL version,
  extension version, CLI path, command, corpus source/row count/query count/dim,
  table prefix, profile, storage format, PQ group size, `nlists`, `nprobe`,
  rerank mode, explicit rerank width, cache state, surface isolation
  classification, and key result lines cited by `request.md`.

Repeatability rule:

- A latency change is repeatable enough for an optimization checkpoint if the
  same fixed point is measured at least twice, p50 and p95 move in the same
  direction, and the improvement is at least 5% with unchanged recall.
- Counter-only classification is repeatable enough if the same representative
  query preserves the same dominant counter shape across two runs or across two
  adjacent surfaces.
- If p99 is the only metric that moves, treat it as noise until repeated.

## Bottleneck Classification

Posting-list I/O / scan volume:

- Confirmed by posting pages read and postings visited rising with latency while
  postings scored and rerank rows stay nearly flat.
- Needed evidence: EXPLAIN/counters, relation/index size, selected list ranges,
  buffer blocks, representative query at adjacent `nprobe` values, and optional
  profile showing time in page iteration/read-stream setup.

PQ-FastScan scoring throughput:

- Confirmed by postings scored scaling with latency, high CPU samples in
  `src/am/ec_ivf/quantizer.rs`, and little change in page count or rerank rows.
- Needed evidence: postings scored, pruned-by-bound ratio, scorer micro/profile
  output, and recall-stable latency repeats.

Exact heap rerank cost:

- Confirmed by latency scaling with `rerank_width`, `Rerank Rows`, heap fetch
  work, or source vector detoast/slot costs.
- Needed evidence: width sweep at fixed `nprobe`, rerank-row counters, heap
  block prefetch/fetch profile, and recall@10/100 deltas.

Candidate allocation/dedup/top-k overhead:

- Confirmed by high allocator/hash/top-k samples or candidate counters scaling
  despite stable posting read/scoring/rerank volume.
- Needed evidence: candidates inserted, duplicate filters, heap/top-k lengths,
  allocation profile, and a fixed-surface latency repeat.

Centroid routing / `nprobe` choice:

- Confirmed by recall-latency curves showing a better operating point without
  code changes, or centroid scoring/top-n selection appearing in profiles.
- Needed evidence: small `nprobe` sweep, selected-list counts, centroid timing
  if available, recall@10 and recall@100, and stable latency.

Live insert/vacuum churn:

- Confirmed by index growth, cross-list posting pages, mixed blocks, deleted
  posting tombstones, vacuum wall time, or insert latency under delete/refill.
- Needed evidence: `ecaz stress ivf-vacuum-scale`, page ownership snapshots,
  `posting_slack_percent` comparison, index bytes by cycle, and vacuum memory.

## Known Or Likely Optimizations

Merged block-order posting-list scan:

- Target metric: reduce posting pages read, repeated block visits, and scan
  setup overhead; especially 100k/990k p50/p95.
- Risk: wrong list filtering, changed result order before rerank, or worse
  locality on small surfaces.
- Likely files: `src/am/ec_ivf/scan.rs`, `src/am/ec_ivf/page.rs`, PG18
  ReadStream code in the same scan/page boundary, EXPLAIN counters in
  `src/am/common/explain.rs` if new counters are needed.
- Minimal validation: unit tests for selected-probe plan ordering/filtering,
  focused PG18 IVF scan test, 100k fixed-point recall/latency/counters.
- Stop condition: revert or stop iterating if posting pages/latency do not move
  at least 5% on 100k, or if recall changes.

Score-as-you-read / bounded candidate materialization:

- Target metric: lower allocations, candidate memory, and full candidate
  materialization cost.
- Risk: dedup semantics, heap-TID ordering, pre-rerank truncation correctness,
  and exact rerank visibility.
- Likely files: `src/am/ec_ivf/scan.rs`, candidate top-k state, `page.rs`
  posting visitors.
- Minimal validation: scan unit tests for dedup/top-k invariants, PG18
  heap-f32 rerank tests, fixed 25k and 100k recall/latency.
- Stop condition: stop if allocation counters/profile improve but latency does
  not, unless memory HWM drops materially.

PQ-FastScan scoring hot-path cleanup:

- Target metric: reduce CPU time per posting scored and widen pruning benefit.
- Risk: scorer divergence, architecture-specific assumptions on Apple Silicon,
  and prepared-query cache regressions.
- Likely files: `src/am/ec_ivf/quantizer.rs`, shared quantizer scoring code,
  `src/am/ec_ivf/scan.rs`, pure Rust benchmark modules if present.
- Minimal validation: scorer equivalence tests, microbench/profile evidence,
  10k/25k/100k recall parity and latency repeats.
- Stop condition: stop if profiles show the path is not scoring-bound or if
  pure scorer gains do not survive SQL latency.

Heap rerank budget/policy changes:

- Target metric: reduce p50/p95 while preserving recall@10 and recall@100.
- Risk: silent quality loss, confusing reloption/session-GUC interactions, and
  under-reranking small-corpus surfaces.
- Likely files: `src/am/ec_ivf/options.rs`, `src/am/ec_ivf/scan.rs`,
  `crates/ecaz-cli/src/profiles.rs`, docs if guidance changes.
- Minimal validation: width sweep with truth cache, rerank-row counters,
  fixed-point latency repeats.
- Stop condition: do not change default/guidance unless recall floors hold on
  10k, 25k, and 100k.

Candidate dedup/top-k allocation reduction:

- Target metric: reduce allocator and hash overhead; reduce HWM if candidate
  sets are broad.
- Risk: duplicate heap-TID handling and candidate score tie order.
- Likely files: `src/am/ec_ivf/scan.rs`, scan debug tests, candidate heap/top-k
  helper tests.
- Minimal validation: unit tests for duplicate and tie behavior, 25k/100k
  latency and recall, allocation/profile proof if available.
- Stop condition: stop if EXPLAIN shows candidate counts are too small to
  matter or if prior `30072` negative pattern repeats.

Centroid routing / `nprobe` heuristics:

- Target metric: better default operating points and lower scan volume without
  code-heavy changes.
- Risk: overfitting local DBPedia and damaging recall@100.
- Likely files: `src/am/ec_ivf/options.rs`, `src/am/ec_ivf/cost.rs`,
  `src/am/ec_ivf/admin.rs`, docs, and possibly `crates/ecaz-cli/src/profiles.rs`.
- Minimal validation: small sweeps around current points with recall@10/100,
  latency, and counters.
- Stop condition: keep as guidance only unless multiple surfaces agree.

Storage/layout adjustments:

- Target metric: reduce posting page count, improve block locality, reduce
  index bytes or read volume.
- Risk: page-layout compatibility, vacuum/insert complexity, and WAL churn.
- Likely files: `src/am/ec_ivf/page.rs`, `build.rs`, `insert.rs`, `vacuum.rs`,
  `admin.rs`.
- Minimal validation: page codec tests, build/insert/vacuum PG18 tests, storage
  and EXPLAIN counter packets.
- Stop condition: do not start until counters prove page count/locality is the
  dominant bottleneck and a narrower scan-level fix is insufficient.

Churn-related posting slack or vacuum changes:

- Target metric: stable index bytes and acceptable vacuum/refill wall time
  under delete/refill workloads.
- Risk: larger build-time index, wasted slack on append-only workloads, and
  page ownership bugs.
- Likely files: `src/am/ec_ivf/build.rs`, `insert.rs`, `vacuum.rs`,
  `admin.rs`, `crates/ecaz-cli/src/commands/stress/ivf_vacuum_scale.rs`.
- Minimal validation: stress harness with and without slack, page ownership
  diagnostics, storage by cycle.
- Stop condition: leave default unchanged unless a real workload needs churn
  headroom and storage growth is proven unacceptable.

## Staged Execution Plan

Phase A: baseline completion.

- `30166-task31-m5-real-corpus-preflight`: list/inspect local corpora, verify
  PG18/extension/CLI, and decide whether DBPedia fetch/prepare/load is needed.
- `30167-task31-m5-pqg8-10k-baseline`: real 10k load/build if needed, recall,
  latency/HWM, storage, and EXPLAIN/counters at `n64/p48/w750`.
- `30168-task31-m5-pqg8-25k-baseline`: same for real 25k at `n64/p48/w750`.
- `30169-task31-m5-pqg8-100k-baseline`: real 100k fresh build plus recall@10,
  recall@100, latency/HWM, storage at `n128/p48/w500`.
- `30170-task31-m5-pqg8-counter-matrix`: representative EXPLAIN/counter
  captures for 10k, 25k, 100k, and optional 100k adjacent probes.
- Optional `30171-task31-m5-pqg8-990k-latency-counter-check`: 990k latency and
  counter check only if the corpus already exists or setup cost is acceptable.

Phase B: bottleneck choice.

- `30172-task31-m5-ivf-bottleneck-selection`: classify the first bottleneck
  using Phase A. This packet should choose exactly one implementation target
  and explicitly reject the others for the first checkpoint.

Phase C: first optimization checkpoint.

- Preferred if counters match Task 28 990k: `30173-task31-m5-merged-block-order-posting-scan`.
- If profiles instead show candidate overhead:
  `30174-task31-m5-score-as-you-read-candidate-bound`.
- If scoring is unexpectedly dominant:
  `30175-task31-m5-pqfastscan-scoring-hotpath`.
- If rerank dominates:
  `30176-task31-m5-rerank-policy-checkpoint`.

Only one of these should be active first.

Phase D: re-measure and decide.

- `30177-task31-m5-first-optimization-remeasure`: rerun the same fixed points
  and counters from Phase A. Decision rule: keep if recall is unchanged and
  p50/p95 improve at least 5% repeatably; revert or stop if not.

Phase E: broader validation.

- `30178-task31-m5-ivf-broader-validation`: repeat the kept checkpoint on the
  remaining required surfaces, run the narrow PG18 tests covering touched
  behavior, add optional 990k latency/counters, and update plan/status docs.

## Immediate Next Action

Create packet `30166-task31-m5-real-corpus-preflight` and run only the local
inventory commands:

```sh
mkdir -p review/30166-task31-m5-real-corpus-preflight/artifacts

/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres \
  --socket-dir /Users/peter/.pgrx --port 28818 --raw \
  --sql "select version(); select extname, extversion from pg_extension where extname = 'ecaz';" \
  --log-output review/30166-task31-m5-real-corpus-preflight/artifacts/pg18-ecaz-status.log

/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  --log-file review/30166-task31-m5-real-corpus-preflight/artifacts/corpus-list.log \
  corpus list
```

This is the best next step because Task 31 currently has only synthetic smoke
on the M5, and packet `30162` said no real corpora were loaded at setup time.
If the real 10k/25k/100k prefixes already exist, Phase A can start with
`corpus inspect` and measurement. If they do not, `30166` should expand to
fetch/prepare/load planning before any recall or latency claim.

## Validation

No tests or benchmarks were run for this packet. It is a docs-only planning
checkpoint.

## Artifacts

No `artifacts/manifest.md` is included because this packet captured no new
measurements or command outputs.

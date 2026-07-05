# AWS Round Prep — RabitQ + IVF + SPIRE Optimization Punch List

Read-and-extract packet. No new code, no new measurements. Consolidates
the data that already exists into one place so the AWS round starts with
explicit optimization targets, named per access method × storage_format ×
corpus scale, with cited evidence.

## Head and host

| Field | Value |
| --- | --- |
| Branch | `aws-optimization-ivf-rabitq-spire` |
| Base | `origin/main` HEAD `24e7ea814` |
| Captured | `2026-05-22` (America/Los_Angeles) |
| Purpose | Prework for AWS-funded RabitQ/IVF tuning + SPIRE initial bench |

## TL;DR — what we already know, before spending another dollar on AWS

1. **RaBitQ-on-IVF is 1.4–1.65× slower than TurboQuant-on-IVF on
   Graviton 4.** Root cause already audited: `src/quant/rabitq/`
   contains **zero aarch64-specific code** while TurboQuant has explicit
   NEON paths in `src/quant/prod.rs` (`score_ip_from_split_parts_neon`,
   `score_ip_mse_codes_neon`) and `src/quant/hadamard.rs`
   (`fwht_in_place_neon`). Evidence:
   `benchmarks/cloud-scaling-multi-am/manifest.md` headline finding #2
   (lines 29–32) and `benchmarks/cloud-10k-graviton-preopt-baselines/manifest.md`
   source-audit lines 20–27.
2. **PQ_FASTSCAN is the SIMD champion on Graviton.** 4.79 ms p50 @ 1M
   corpus, nprobe=8 — `cloud-scaling-multi-am/manifest.md` lines 167–168
   and headline #1 (lines 23–28). 5–10× faster than TurboQuant at every
   size. This is the reference RaBitQ should aim to close on.
3. **Runtime feature dispatch only checks NEON.**
   `src/quant/simd.rs:35` only invokes
   `is_aarch64_feature_detected!("neon")`. The Graviton 4 host exposes
   SVE2, i8mm, bf16, dotprod, fp16 (cpuinfo captured at
   `cloud-10k-graviton-preopt-baselines/artifacts/kernels/env/cpuinfo.txt`)
   but the code can't reach any of those today.
4. **IVF scan-attribution counters already exist.** 11 counters
   (`stats_postings_scored`, `stats_postings_pruned_by_bound`,
   `stats_filtered_duplicates`, `stats_rerank_rows`, …) emit through
   `IvfExplainCounters` in `src/am/common/explain.rs:218-261` whenever a
   suite step runs `EXPLAIN (FORMAT JSON, ecaz, ANALYZE)`. No new
   instrumentation needed. Raw EXPLAIN JSON is in suite artifacts; a
   one-hour `suite.rs:parse_explain_rows()` extension can also surface
   them in `results.jsonl` (optional, Phase 0 step 3).
5. **SPIRE-on-RaBitQ builds fail at 50k+** with
   `ec_spire object tuple payload 11270 exceeds page size 8192`.
   Evidence: `benchmarks/task-50-local-baseline/manifest.md` lines
   105–115 + 119–124. Blocking the SPIRE phase entirely.

## Current AWS state (audited 2026-05-22)

### Live resources (action needed)

| Resource | State | Notes |
| --- | --- | --- |
| `i-00c9ecb965cbef319` (c8g.medium, `ecaz-cloud-10k-loader`) | **running since 2026-05-16** | Idle bench loader still up from prior cycle. **Recommend `ecaz cloud down --profile 10k` before provisioning Phase A** to avoid name/state collision and stop the meter. |

### Preserved snapshots (assets — restore instead of re-loading)

| Snapshot | Size | Description | Use |
| --- | --- | --- | --- |
| `snap-054feaffc50ecf1c9` | 50 GB | post-bench real DBpedia `ec_hnsw_real_{10k,50k}` + `ec_ivf` indexes, m8g.large pg18 | **Phase A 10k+50k restore** — skips ~15 min build/load |
| `snap-0f0806f9096f95fb7` | 20 GB | synth10k + synth50k, m8g.large pg18 | Lower priority; synth-uniform vectors don't reflect real cluster structure |
| `snap-09d29cccd558a4a47` | 250 GB | pgvector + pgvectorscale + vchord isolated comparator tables, post sweep cycle 2 | **Phase A comparison rows** — drives `compare-pgvector` / `compare-vectorscale` suite steps without rebuilding the competitor stack |

### Lost resources

- `vol-0d09cac38fcfb94c9` (150 GB, full multi-AM scaling curve) — **deleted**. The instance `i-05af7ea8e92f65b30` that held it is also gone. The numbers in `benchmarks/cloud-scaling-multi-am/manifest.md` are still authoritative as historical baselines but can't be incrementally re-run without rebuilding from `snap-054feaffc50ecf1c9` + re-loading 100k/1M corpora.

## Baseline numbers (already captured — do not re-bench)

All cells below were measured on AWS Graviton 4. Cite these packets directly
in Phase A and Phase B comparisons.

### `ec_ivf` × storage_format on real DBpedia (m8g.large or m8g.2xlarge, k=10, c=1, 200 iters)

| Corpus | storage_format | nprobe=8 mean ms | nprobe=64 mean ms | recall@10 @ nprobe=8 | Source |
| --- | --- | --- | --- | --- | --- |
| 10k | turboquant | 4.05 (4.18 in scaling pkt) | 9.15 | 0.9690 | `cloud-10k-real-baselines/manifest.md` lines 68–89; scaling pkt lines 133 |
| 10k | rabitq | 2.52 | 15.9 | 0.9730 | `cloud-scaling-multi-am/manifest.md` lines 143–149 |
| 10k | pq_fastscan | 0.63 | 1.55 | TODO (recorded in `artifacts/sweep/10k/ivf/pq_fastscan/recall.log`) | `cloud-scaling-multi-am/manifest.md` lines 167–173 |
| 50k | turboquant | 3.36 | 19.7 | 0.8290 | `cloud-10k-real-baselines/manifest.md` lines 99–117 |
| 100k | turboquant | — | — | — | scaling-pkt `artifacts/sweep/100k/ivf/turboquant/latency.log` (extract pending) |
| 1m | turboquant | 35.8 | 112.2 | OOM (10k queries × 990k corpus) | `cloud-scaling-multi-am/manifest.md` lines 131–138 |
| 1m | rabitq | 28.9 | 185.5 | OOM | `cloud-scaling-multi-am/manifest.md` lines 143–149 |
| 1m | pq_fastscan | 4.79 | 15.6 | OOM | `cloud-scaling-multi-am/manifest.md` lines 167–173 |

Recall @ 1M is OOM-bound at the brute-force ground-truth stage on m8g.2xlarge
(32 GB). `--queries-limit 1000` is the documented workaround
(`cloud-scaling-multi-am/manifest.md` lines 199–207).

### `ec_hnsw` for context (default config)

| Corpus | ef_search | mean ms | Source |
| --- | --- | --- | --- |
| 10k | 128 | 1.56 | `cloud-scaling-multi-am/manifest.md` lines 120–127 |
| 1M | 160 | 19.8 | same |

### `ec_diskann`

Out of scope for this round. Already known broken at default config
(914 ms @ 1M, list_size=64). Deferred to task-29 per CLAUDE.md.

### `ec_spire`

No production AWS numbers captured. Phase 13 AWS readiness packets
(`reviews/task-30/*768*`, `reviews/task-30/*767*`, `reviews/task-30/*765*`)
landed the topology and runbook but did not produce performance numbers.
The local Phase 1 gate (`benchmarks/30530-spire-phase1-recall-latency-gate/`)
captured only 10k TurboQuant: nprobe=8 p50 62.1 ms recall@10 0.9985,
nprobe=24 p50 140.7 ms recall@10 1.0000. **Blocked at 50k+** per local
baseline manifest (see TL;DR #5).

## Optimization punch list (ranked by expected impact)

### P0 — RaBitQ aarch64 SIMD kernel
- **Target**: Match TurboQuant (or PQ_FASTSCAN) at 10k/50k/100k/1M on Graviton.
- **Current gap**: 1.4–1.65× slower than TQ; 6× slower than PQ_FASTSCAN at 1M.
- **Where**: `src/quant/rabitq/` — add NEON `RaBitQQuantizer::estimate_ip`. The
  template is `src/quant/prod.rs:score_ip_from_parts_tiled_lut_no_qjl_4bit`
  (cited as "the reference for how RaBitQ should look post-optimization" in
  `cloud-scaling-multi-am/manifest.md` lines 263–265).
- **Phase A success criterion**: RaBitQ-on-IVF p50 @ 1M ≤ 1.1× TQ p50 across
  nprobe ∈ {8,16,32,64}.
- **Stretch**: also wire SVE2 dispatch in `src/quant/simd.rs:35`.

### P1 — IVF posting-list scan attribution (no code; data extraction)
- **Target**: classify the IVF latency-vs-nprobe slope on real corpora into
  {postings scored, dedup hits, rerank width}. Counters exist (see TL;DR #4).
- **Phase A action**: extract the `IvfExplainCounters` JSON from existing
  EXPLAIN artifacts in `benchmarks/30133-task28-ivf-990k-balanced-recall100/artifacts/`,
  `benchmarks/30113-task28-ivf-a9-100k-latency-memory/artifacts/`,
  `benchmarks/30081-task28-ivf-rabitq-profile/artifacts/`, and the new
  Phase A AWS sweep. Land an attribution table in
  `benchmarks/aws-round-rabitq-ivf/attribution.md`.
- **Decision gate**: only do scan/dedup/rerank code work if attribution shows
  >20% of latency in one of those buckets. Otherwise stay on P0.

### P2 — SPIRE >8K tuple payload (unblocks SPIRE Phase C)
- **Target**: SPIRE-on-RaBitQ builds clean at 25k/50k/100k locally.
- **Where**: `src/am/ec_spire/object/*` and `src/am/ec_spire/build/*`. The
  error path is the `tuple payload 11270 exceeds page size 8192` check in
  the partition-object header writer. Two candidate fixes:
  1. **Split partition object header across multiple pages** (the "right"
     fix; matches how btree splits internal nodes).
  2. **Compact the per-tuple payload** (cheaper if the fields are
     redundant; needs a payload-field audit).
- **Acceptance**: `ecaz corpus load` + `CREATE INDEX … USING ec_spire`
  succeeds at 25k/50k/100k locally; existing Phase 1 recall gate
  (`benchmarks/30530-…`) still passes at 10k.

### P3 — IVF rerank-vs-recompute on Graviton (RaBitQ-specific)
- **Target**: confirm the RaBitQ-on-IVF rerank pipeline gets the benefit
  from `rerank='heap_f32'` vs `rerank='source_column'`. We have local
  numbers in `benchmarks/task-50-local-baseline/`; we do not have a
  Graviton sweep that isolates rerank mode.
- **Phase A action**: add a rerank-mode dimension to the Phase A 10k/50k
  sweep matrix. No code; just suite-config rows.

### P4 — Optional: surface IVF EXPLAIN counters in `results.jsonl`
- **Target**: `suite.rs:parse_explain_rows()` extracts the 11
  `IvfExplainCounters` fields alongside `modeled_total_cost`.
- **Cost**: under one hour. Do this only if it doesn't compete with P0.

### Out of scope this round
- DiskANN reloption sweep (`graph_degree`, `alpha`) — defer to task-29.
- `ec_hnsw` tuning beyond context contrast.
- PQ_FASTSCAN grouped-PQ metadata persistence (deferred per Task 30 Phase 1).
- 100m / 1b corpora.
- Energy / power profiling.

## Decision: do we need a fresh AWS baseline before tuning?

**No.** The cells below already cover the matrix at both ends of the scale
range. Phase A's first AWS work is the **local↔AWS parity gate** at 10k
(rerun the same `suite-baseline-10k.json` locally and on AWS, restored
from `snap-054feaffc50ecf1c9`, and confirm parity within tolerance), then
the **P0 RaBitQ NEON kernel** lands locally with focused `cargo bench
--features bench --bench iai_quant_score` evidence before any AWS sweep
is repeated.

## Re-run / extraction recipe

This packet has no measurement steps of its own. The cells above were
extracted by reading:

```sh
benchmarks/task-50-local-baseline/manifest.md
benchmarks/cloud-10k-real-baselines/manifest.md
benchmarks/cloud-10k-graviton-preopt-baselines/manifest.md
benchmarks/cloud-scaling-multi-am/manifest.md
benchmarks/30530-spire-phase1-recall-latency-gate/manifest.md
src/quant/simd.rs:35
src/am/common/explain.rs:218-261
```

The AWS state snapshot (live instances, preserved snapshots, lost volumes)
was captured by:

```sh
aws ec2 describe-instances --region us-west-2 \
    --filters Name=instance-state-name,Values=running,pending,stopping,stopped \
    --query 'Reservations[].Instances[].{Id:InstanceId,Type:InstanceType,State:State.Name,Name:Tags[?Key==`Name`]|[0].Value}'
aws ec2 describe-snapshots --owner-ids self --region us-west-2 \
    --query 'Snapshots[].{Id:SnapshotId,Size:VolumeSize,Started:StartTime,Desc:Description}'
aws ec2 describe-volumes --volume-ids vol-0d09cac38fcfb94c9 --region us-west-2
```

Raw outputs are not preserved here (the AWS state is mutable; live re-query
gives current truth).

## Cross-references

- Plan: `/home/peter/.claude/plans/ok-we-re-starting-aws-glistening-sloth.md`
- Task index: `plan/tasks/README.md`
- Suite runner spec: `spec/functional/operator/FR-038-configured-benchmark-suite-runner.md`
- Per-AM isolation rule: `spec/adr/ADR-050-configured-benchmark-suite-runner.md`
- Bench provenance NFR: `spec/non-functional/NFR-007-benchmark-provenance.md`

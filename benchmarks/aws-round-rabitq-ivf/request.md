# aws-round-rabitq-ivf — Phase A: IVF storage_format sweep on real DBpedia 10k+50k

Phase A of the AWS-funded RabitQ + IVF optimization round (plan:
`/home/peter/.claude/plans/ok-we-re-starting-aws-glistening-sloth.md`).
Establishes the m8g.large Graviton baseline for `ec_ivf` across
`storage_format = {turboquant, rabitq, pq_fastscan}` at 10k and 50k
on real DBpedia data, restored from snapshot rather than rebuilt.

## What this packet measures

For every combination of:

- corpus scale ∈ {10k, 50k}
- access method = `ec_ivf`
- storage_format ∈ {turboquant, rabitq, pq_fastscan}
- nprobe sweep = {8, 16, 24, 32, 48, 64}

we capture:

- `recall@10` + `ndcg@10` (`ecaz bench recall`)
- mean / p50 / p95 / p99 query latency (`ecaz bench latency`, 200 iterations × concurrency 1)
- index size on disk (`ecaz bench storage`)

The point of the run is to confirm the **P0 RaBitQ NEON gap** identified
in `benchmarks/aws-round-prep/manifest.md` is still ~1.4–1.65× on the
current branch, and to act as the matching pre-optimization baseline
for the NEON kernel work that lands next.

## Environment

| Property | Value |
| --- | --- |
| Region | us-west-2 (AZ us-west-2a) |
| Profile | `10k` (`infra/cloud/terraform/profiles/10k.tfvars`) |
| DB instance | m8g.large (Graviton 4 Neoverse-V2, 2 vCPU / 8 GB) |
| EBS | gp3, 50 GB, restored from `snap-054feaffc50ecf1c9` |
| OS | Amazon Linux 2023 (kernel 6.1, aarch64) |
| PostgreSQL | 18.3 |
| ecaz branch | `aws-optimization-ivf-rabitq-spire` |
| ecaz base SHA | `24e7ea814` (origin/main) |

Cost guardrail: m8g.large + 50 GB gp3 ≈ $0.16/hr + $0.13/day. Stop after
each full Phase A cycle (`ecaz cloud snapshot` then
`ecaz cloud down`) per the snapshot-and-destroy memory rule.

## Snapshot reuse

The DB volume is restored from `snap-054feaffc50ecf1c9`, which already
contains:

- `ec_hnsw_real_10k_corpus` + `ec_hnsw_real_10k_queries` (10000 + 200 rows; real DBpedia)
- `ec_hnsw_real_50k_corpus` + `ec_hnsw_real_50k_queries` (50000 + 1000 rows; real DBpedia)
- `ec_hnsw_real_10k_idx` + `ec_hnsw_real_50k_idx` (`ec_ivf` default = TurboQuant)

This is the canonical real-DBpedia bench dataset preserved from
`benchmarks/cloud-10k-real-baselines/`. Per the
`feedback_no_recreate_corpus` memory rule, we do **not** re-fetch parquet
or re-load TSVs — the snapshot is the source of truth.

## Per-variant table fixtures

The snapshot only has the `ec_ivf` default (TurboQuant) index. To hold
the ADR-050 "one corpus table per AM × storage_format" invariant, we
materialize six per-variant corpus tables on top of the existing data:

| Prefix | AM | storage_format | Source table |
| --- | --- | --- | --- |
| `real_10k_ivf_tq` | ec_ivf | turboquant | `ec_hnsw_real_10k_corpus` |
| `real_10k_ivf_rabitq` | ec_ivf | rabitq | `ec_hnsw_real_10k_corpus` |
| `real_10k_ivf_pqfs` | ec_ivf | pq_fastscan | `ec_hnsw_real_10k_corpus` |
| `real_50k_ivf_tq` | ec_ivf | turboquant | `ec_hnsw_real_50k_corpus` |
| `real_50k_ivf_rabitq` | ec_ivf | rabitq | `ec_hnsw_real_50k_corpus` |
| `real_50k_ivf_pqfs` | ec_ivf | pq_fastscan | `ec_hnsw_real_50k_corpus` |

Each variant gets its own corpus + queries table + index. Driver SQL is
`setup-per-variant-tables.sql` in this packet; it is idempotent.

## Re-run

On the operator workstation, with terraform state already showing
`profile=10k` up and ecaz installed:

```bash
# 0. (One-time) restore + install — already done for the live run.
ecaz cloud up --profile 10k \
              --from-snapshot snap-054feaffc50ecf1c9 \
              --git-ref aws-optimization-ivf-rabitq-spire \
              --log-file benchmarks/aws-round-prep/artifacts/cloud-up.log
ecaz cloud install --profile 10k \
              --git-ref aws-optimization-ivf-rabitq-spire \
              --timeout 2400 \
              --log-file benchmarks/aws-round-prep/artifacts/cloud-install.log

# 1. Materialize per-variant tables (sends SQL to the DB host via SSM).
ecaz cloud sql --profile 10k \
               --file benchmarks/aws-round-rabitq-ivf/setup-per-variant-tables.sql \
               --log-output benchmarks/aws-round-rabitq-ivf/artifacts/setup-fixtures.log

# 2. Run the suite. The suite runner connects to the DB via the standard
#    ecaz dev sql path (SSM exec on the m8g.large) and drives bench commands.
ecaz bench suite run \
    --config benchmarks/aws-round-rabitq-ivf/suite-10k-50k.json \
    --log-file benchmarks/aws-round-rabitq-ivf/artifacts/suite.driver.log

# 3. Snapshot + teardown.
ecaz cloud snapshot --profile 10k --description "phase-A post-sweep 10k+50k"
ecaz cloud down --profile 10k --yes
```

## Expected headline numbers (from `aws-round-prep` punch list)

These cells are the **expectations to confirm or contradict**, not new
measurements. Numbers come from prior cycles on the same hardware.

| Cell | Expected p50 (mean) ms | Expected recall@10 |
| --- | --- | --- |
| 10k TQ nprobe=8 | ~4.0 | ~0.97 |
| 10k RaBitQ nprobe=8 | ~2.5 | ~0.97 |
| 10k PQ_FASTSCAN nprobe=8 | ~0.6 | TBD (recall not measured in prior cycle) |
| 50k TQ nprobe=8 | ~3.5 | ~0.83 |
| 50k RaBitQ nprobe=8 | TBD (not measured in prior cycle at 50k) | TBD |
| 50k PQ_FASTSCAN nprobe=8 | TBD | TBD |

Recall headline target: `recall@10 ≥ 0.95` at nprobe=24 for all three
formats at 10k. If RaBitQ recall diverges from TQ by more than 1% at
matched nprobe, that is itself a finding worth tagging.

## Out of scope for this packet

- HNSW context contrast rows — defer to a follow-up if needed; the
  punch list already cites prior HNSW numbers from `cloud-scaling-multi-am`.
- Rerank-mode sweep (`rerank ∈ {off, heap_f32, source_column}`) — P3
  in the punch list; lands as `suite-10k-50k-rerank.json` once Phase A
  baseline numbers confirm the formats behave consistently.
- 100k / 1m — Phase B closure work, separate packet.
- SPIRE — Phase C, blocked on the >8K payload fix.

## Cross-references

- Plan: `/home/peter/.claude/plans/ok-we-re-starting-aws-glistening-sloth.md`
- Prep packet: `benchmarks/aws-round-prep/manifest.md`
- Prior real-DBpedia baselines: `benchmarks/cloud-10k-real-baselines/manifest.md`
- Prior full scaling curve: `benchmarks/cloud-scaling-multi-am/manifest.md`
- Per-AM isolation rule: `spec/adr/ADR-050-configured-benchmark-suite-runner.md`
- Suite runner spec: `spec/functional/operator/FR-038-configured-benchmark-suite-runner.md`

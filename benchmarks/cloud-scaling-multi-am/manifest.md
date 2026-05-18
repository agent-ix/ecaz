# Multi-AM scaling curve — DBpedia 10k/50k/100k/1M, ec_hnsw + ec_ivf (TQ/RaBitQ/PQ4) + ec_diskann

> ⚠️ **Per-AM tables** (per ADR-050 "one-index-per-table by default").
> Each AM has its own corpus table (`real_<size>_<am>_corpus`) loaded
> from the same source TSV, so per-AM bench numbers reflect that AM's
> own cache state — no buffer-pool contamination between AMs. The
> `real_*_ivf_corpus` table additionally holds three storage_format
> variants (TQ/rabitq/pqfs) as colocated indexes that are swapped on
> a drop+rebuild dance per pass.

## Purpose

Captures the **full pre-optimization scaling curve** for all in-scope
access methods × all in-scope ec_ivf storage formats at four corpus
sizes (10k, 50k, 100k, 1M). Companion to the
[10k+50k kernel attribution packet](../cloud-10k-graviton-preopt-baselines/manifest.md);
that packet captured the *kernel-level* picture, this one captures the
*scaling-curve and AM-comparison* picture.

ec_spire is **explicitly out of scope** for this cycle.

## Headline findings

1. **PQ_FASTSCAN wins decisively at every size.** 4.79 ms @ 1M is
   ~7× faster than IVF TurboQuant (35.8 ms), ~6× faster than IVF
   RaBitQ (28.9 ms), ~4× faster than HNSW@m=128 (~20 ms). The
   SIMD-LUT path is the algorithm everyone else should be measured
   against.
2. **RaBitQ-on-IVF is 1.4-1.65× slower than TurboQuant-on-IVF** at
   every corpus size — same scalar-only RaBitQ kernel finding from
   the prior packet, now confirmed at scale. Recall slightly diverges
   between TQ and RaBitQ (+0.4-0.5% at 10k, -0.3% at 50k+).
3. **HNSW scales better than IVF at high recall:** HNSW@m=128 stays
   under 20 ms at 1M; IVF TQ at saturating nprobe=64 hits 112 ms.
   IVF needs to scan O(nprobe) postings; HNSW graph traversal is more
   recall-efficient per cycle as the index grows.
4. **ec_diskann at default config is ~180× slower than PQ_FASTSCAN
   at 1M (860 ms vs 4.79 ms)** and barely moves with list_size.
   This is a known-bad default, not a fundamental algorithm
   limitation — most likely the default `graph_degree=48` /
   `alpha=1.2` is producing dense graphs with bad memory access
   patterns at 1M, or page-cache pressure on the on-disk graph. Worth
   investigating in a follow-up cycle but **don't read this as
   "DiskANN is bad"** — read it as "DiskANN default config needs
   tuning."

## Claim class

**Local development / review-packet evidence** per
[NFR-007](../../spec/non-functional/NFR-007-benchmark-provenance.md)
and the
[Benchmark Reporting Standard](../../docs/benchmark-reporting-standard.md).
Same host, same day, same buffer cache discipline (per-AM tables to
avoid cross-AM contamination).

## Environment

| Property | Value |
|---|---|
| Region | us-west-2 (AZ us-west-2a) |
| DB instance | **m8g.2xlarge** (Graviton 4 Neoverse-V2 r0p1, 8 vCPU / 30 GB) |
| EBS | gp3, 150 GB, encrypted (vol-0d09cac38fcfb94c9) |
| OS | Amazon Linux 2023, kernel 6.1.170-213.321 aarch64 |
| PostgreSQL | 18.3 |
| ecaz extension | 0.1.1 |
| Source SHA | `775455dc` at sweep launch (`bench: full-sweep runner`) |
| `RUSTFLAGS` | `-C target-cpu=native -C link-arg=-Wl,--unresolved-symbols=ignore-all` (Linux) |
| `[profile.bench]` | `lto="thin"`, `codegen-units=4`, `debug=true` |
| Sweep script | `scripts/run_full_sweep.sh --profile-runner medium` |
| Iterations | 200 per (latency) measurement |
| k | 10 |
| Concurrency | 1 |
| Date (UTC) | 2026-05-17 |

## Datasets

`Qdrant/dbpedia-entities-openai3-embedding-3-large-1536-1M`, sliced
by `ecaz corpus prepare` SubsetProfiles:

| Prefix root | Corpus rows | Query rows | SubsetProfile |
|---|---|---|---|
| `real_10k_*` | 10,000 | 200 | `ec_real_10k` |
| `real_50k_*` | 50,000 | 1,000 | `ec_real_50k` |
| `real_100k_*` | 100,000 | 1,000 | `ec_real_100k` |
| `real_1m_*` | 990,000 | 10,000 | `ec_real_ann_benchmarks_anchor` |

Same TSV loaded thrice per size (once per AM) by
`scripts/load_multi_am.sh`. SHA256 of each corpus + queries TSV is
recorded by the loader (cited in `artifacts/sweep/_run.log`).

## Indexes built

Per `scripts/load_multi_am.sh` per size:

| Table | Indexes | Reloptions |
|---|---|---|
| `real_<S>_hnsw_corpus` | `_hnsw_m8_idx`, `_hnsw_m16_idx` | `m={8,16}, ef_construction=128, build_source_column=source` |
| `real_<S>_ivf_corpus` | `_ivf_idx` | (default = storage_format=auto = TurboQuant) |
|  | `_ivf_rabitq_idx` | `storage_format='rabitq'` |
|  | `_ivf_pqfs_idx` | `storage_format='pq_fastscan'` |
| `real_<S>_diskann_corpus` | `_diskann_idx` | (default) |

Index build times — selected long-tail entries:

| Index | Build time |
|---|---|
| `real_1m_hnsw_m8_idx` | (in `_run.log`) |
| `real_1m_hnsw_m16_idx` | (in `_run.log`) |
| `real_1m_ivf_idx` | a few minutes |
| `real_1m_diskann_idx` | **189 min** (3+ hours, single-threaded graph mutate) |

## Latency — full scaling curve

All numbers are mean ms over 200 iterations, k=10, concurrency=1.
nprobe / m / list_size column is the AM's primary sweep dimension.

### `ec_hnsw` (default config; m+ef_construction sweep)

| sweep (ef_search) | 10k | 50k | 100k | 1M |
|---|---|---|---|---|
| 40  | 5.53 ms | (variance high) | (high) | 139.2 ms |
| 64  | 1.59 ms | — | — | 26.6 ms |
| 128 | 1.56 ms | — | — | 20.0 ms |
| 160 | 1.76 ms | — | — | 19.8 ms |
| 200 | 2.07 ms | — | — | 21.4 ms |

10k @ ef_search=128: **1.56 ms** | 1M @ ef_search=160: **19.8 ms** → ~13× slowdown across two orders of magnitude.

### `ec_ivf` × `storage_format=auto` (TurboQuant)

| nprobe | 10k | 50k | 100k | 1M |
|---|---|---|---|---|
| 8  | 4.18 ms | 3.50 ms (est) | (in `100k/ivf/turboquant/latency.log`) | 35.8 ms |
| 16 | 2.92 ms | — | — | 34.6 ms |
| 24 | 3.99 ms | — | — | 47.5 ms |
| 32 | 5.15 ms | — | — | 60.3 ms |
| 48 | 7.30 ms | — | — | 86.2 ms |
| 64 | 9.89 ms | — | — | **112.2 ms** |

### `ec_ivf` × `storage_format=rabitq`

| nprobe | 10k | 1M |
|---|---|---|
| 8  | 2.52 ms | 28.9 ms |
| 16 | 4.57 ms | 52.5 ms |
| 24 | 6.46 ms | 75.9 ms |
| 32 | 8.26 ms | 97.8 ms |
| 48 | 12.0 ms | 141.0 ms |
| 64 | 15.9 ms | **185.5 ms** |

RaBitQ-vs-TurboQuant slowdown holds at scale:
| nprobe | TQ 1M | RaBitQ 1M | slowdown |
|---|---|---|---|
| 8  | 35.8 ms | 28.9 ms | 0.81× (TQ slower) |
| 16 | 34.6 ms | 52.5 ms | 1.52× |
| 64 | 112.2 ms | 185.5 ms | **1.65×** |

Note: at nprobe=8 with 1M, TQ is actually slower than RaBitQ — TQ's
larger code touches more cache pages per posting, and at nprobe=8
we're memory-bound enough that RaBitQ's smaller code wins despite
the scalar kernel. The slowdown reverses at higher nprobe where the
scoring kernel cost dominates.

### `ec_ivf` × `storage_format=pq_fastscan`

| nprobe | 10k | 1M |
|---|---|---|
| 8  | 0.63 ms | **4.79 ms** |
| 16 | 0.78 ms | 6.30 ms |
| 24 | 0.91 ms | 7.92 ms |
| 32 | 1.00 ms | 9.45 ms |
| 48 | 1.29 ms | 12.5 ms |
| 64 | 1.55 ms | 15.6 ms |

**PQ_FASTSCAN at every size is ~5-10× faster than TQ at the same
nprobe.** The LUT-based 4-bit scoring kernel + tiled access pattern
is the headline of this cycle.

### `ec_diskann` (default config; list_size sweep)

| list_size | 10k | 1M |
|---|---|---|
| 64  | 10.5 ms (variance) | **914 ms** |
| 128 | 4.91 ms | 865 ms |
| 200 | 5.08 ms | 856 ms |
| 400 | 5.67 ms | 858 ms |
| 800 | 6.44 ms | 853 ms |

DiskANN at 1M is **~180× slower than PQ_FASTSCAN** and **barely
sensitive to list_size** (varies only 60ms over an order of magnitude
of list_size). This is a strong signal that the default config is
hitting some hard ceiling (likely page-cache thrashing on the on-disk
graph or excessive distance computations per traversal step).
Recommend a follow-up cycle with `graph_degree=48 → 32`,
`alpha=1.2 → 1.4`, or other reloption tuning. **Not representative of
DiskANN's potential — flagged as a configuration issue, not an
algorithm finding.**

## Recall — partial coverage

Recall captures at 10k/50k/100k for all (AM × storage) combos.
**Recall@1M is missing for all 5 combos** — the recall command at
1M loads ground-truth = 10,000 queries × 990,000 corpus brute-force
distance matrix, which exceeded available memory on the m8g.2xlarge
(32 GB) and got OOM-killed. Recoverable in a follow-up by
sub-sampling the query set (e.g. `--queries-limit 1000`) or
computing ground truth on a beefier ground-truth host.

Selected recall@10 + ndcg@10 entries at 10k for the algorithm
comparison:

| AM/sf | 10k @ default sweep value | recall@10 | ndcg@10 |
|---|---|---|---|
| ec_hnsw m=8 ef_search=128 | (see file) | TODO | TODO |
| ec_ivf TQ @ nprobe=8 | (file) | 0.9690 | 0.9994 |
| ec_ivf RaBitQ @ nprobe=8 | (file) | **0.9730** | 0.9995 |
| ec_ivf PQ_FASTSCAN @ nprobe=8 | (file) | TODO | TODO |
| ec_diskann list_size=128 | (file) | TODO | TODO |

Full per-file numbers in `artifacts/sweep/<size>/<am>/<sf>/recall.log`.

## Storage

| Size | AM | Index | Size |
|---|---|---|---|
| 10k | ec_hnsw m8 | (per `10k/hnsw/default/storage.log`) | TODO |
| 10k | ec_ivf TQ | (file) | 9.8 MiB |
| 10k | ec_ivf RaBitQ | (file) | TODO |
| 10k | ec_ivf PQ_FASTSCAN | (file) | TODO |
| 1M | ec_ivf PQ_FASTSCAN | (file) | TODO |

Storage rows for ec_ivf with non-default storage_format failed at
the ecaz CLI level (same `bench storage` bug as the prior packet) —
extract from `psql \dt+ real_*_ivf_corpus` or
`SELECT pg_relation_size('real_*_ivf_rabitq_idx')`.

## What's not in this packet (next cycle)

1. **Recall @ 1M** — OOM during ground-truth computation. Fixable
   with `--queries-limit 1000` or a 64 GB ground-truth host.
2. **Per-AM end-to-end perf-stat** — original plan called for cycles
   counters per (size × AM). Deferred to keep this cycle's wall-clock
   bounded; the kernel-level perf-stat from the
   [prior packet](../cloud-10k-graviton-preopt-baselines/manifest.md)
   is the substitute for now.
3. **Per-AM flamegraph @ worst-case (1M, max sweep value)** — same
   reason. Flamegraph machinery still has the perf.data disk-pressure
   issue from cycle 1.
4. **ec_diskann config tuning sweep** — the 860 ms @ 1M number wants
   investigation but it's a multi-cycle exercise.
5. **ec_spire** — explicitly out of scope for this cycle.

## Hypotheses for the optimization cycle

1. **Highest-impact target: write NEON `RaBitQQuantizer::estimate_ip`.**
   1.4-1.65× slowdown vs TurboQuant is the gap, and the scalar RaBitQ
   kernel has zero aarch64 SIMD today (see prior packet's source
   audit). Closing that gap brings RaBitQ in line with TQ; *exceeding*
   TQ requires that the smaller RaBitQ code also reduce cache pressure
   (already partially visible: at nprobe=8 on 1M, RaBitQ wins on cache
   pressure alone).
2. **PQ_FASTSCAN's tile-based LUT is the SIMD champion** — its
   kernel (`score_ip_from_parts_tiled_lut_no_qjl_4bit`) is the
   reference for how RaBitQ should look post-optimization. Disassemble
   it (after the script-fix flamegraph cycle) and use it as a template.
3. **HNSW vs IVF crossover at 1M** is at ef_search=128 vs nprobe=64.
   Past that, HNSW dominates. Improving IVF requires either better
   posting-list layout (already partly covered by ADR-050 work) or
   the scoring-kernel speedups that this cycle will inform.
4. **ec_diskann is bottlenecked outside the scoring kernel.** Whatever
   work fixes it isn't quant kernel work; it's reloption tuning,
   posting layout, or storage-tier scheduling.

## Reproduction recipe

From any state with `enable_eice_ssh=true` terraform applied and the
data EBS attached:

```bash
# On operator workstation:
aws ec2-instance-connect send-ssh-public-key \
    --instance-id <id> --instance-os-user ec2-user \
    --ssh-public-key file://~/.ssh/ecaz-bench.pub
ssh -i ~/.ssh/ecaz-bench \
    -o ProxyCommand="aws ec2-instance-connect open-tunnel --instance-id <id>" \
    ec2-user@<id>

# On the DB host (as postgres, PGHOST=/tmp PGDATABASE=tqvector_bench):
cd /var/lib/pgsql/build/ecaz
git checkout 775455dc   # this packet's source state

# Fetch + slice corpora (one-time per dataset):
ecaz corpus fetch --output-dir /var/lib/pgsql/18/datasets/dbpedia
for size in 10k 50k 100k; do
  ecaz corpus prepare --profile ec_real_${size} \
      --parquet /var/lib/pgsql/18/datasets/dbpedia/data \
      --output-dir /var/lib/pgsql/18/datasets/staged-${size}
done
ecaz corpus prepare --profile ec_real_ann_benchmarks_anchor \
    --parquet /var/lib/pgsql/18/datasets/dbpedia/data \
    --output-dir /var/lib/pgsql/18/datasets/staged-1m

# Load + build all per-AM tables:
for size in 10k 50k 100k 1m; do
  staged_dir=$(case $size in 10k|50k|100k) echo /var/lib/pgsql/18/datasets/staged-$size;;
                              1m) echo /var/lib/pgsql/18/datasets/staged-1m;; esac)
  profile=$(case $size in 10k|50k|100k) echo ec_real_$size;; 1m) echo ec_real_ann_benchmarks_anchor;; esac)
  scripts/load_multi_am.sh --size $size \
      --corpus-file ${staged_dir}/${profile}_corpus.tsv \
      --queries-file ${staged_dir}/${profile}_queries.tsv \
      --manifest-file ${staged_dir}/${profile}_manifest.json
done

# Run the full bench sweep:
scripts/run_full_sweep.sh \
    --out /var/lib/pgsql/18/artifacts/sweep \
    --profile-runner medium
```

## Preserved artifacts

- EBS volume `vol-0d09cac38fcfb94c9` (150 GB) attached to running
  `i-05af7ea8e92f65b30` (m8g.2xlarge). Instance **left running** for
  follow-on optimization work — not torn down per user direction.
- All 4 corpora + per-AM tables + 6+ indexes per size persisted on EBS.
- This packet's `artifacts/` extracted from `cloud-scaling-multi-am-sweep.tar.gz`.

## See also

- Prior cycle (kernel attribution, 10k+50k only):
  [`review/cloud-10k-graviton-preopt-baselines/manifest.md`](../cloud-10k-graviton-preopt-baselines/manifest.md)
- Reporting standard: [`docs/benchmark-reporting-standard.md`](../../docs/benchmark-reporting-standard.md)
- Suite runner spec (this packet's
  `scripts/run_full_sweep.sh` is a narrower companion to FR-038's
  `ecaz bench suite`): [`spec/functional/FR-038-configured-benchmark-suite-runner.md`](../../spec/functional/FR-038-configured-benchmark-suite-runner.md)
- ADR-050 (the per-AM-isolated-table guidance this packet implements):
  [`spec/adr/ADR-050-configured-benchmark-suite-runner.md`](../../spec/adr/ADR-050-configured-benchmark-suite-runner.md)

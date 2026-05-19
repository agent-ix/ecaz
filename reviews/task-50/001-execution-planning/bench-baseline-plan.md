# Task 50 Bench Baseline Plan

Task 50 packets need before/after evidence whenever a structural change touches
hot scoring, traversal, cache, build, or distributed-read paths. The default
fast-iteration gate is local before/after benchmarking on the developer host.
AWS Graviton 4 `m8g.2xlarge` measurements are closeout confirmation, not the
normal per-slice iteration loop. Earlier `m8g.large` / `m8g.xlarge` artifacts
remain useful historical context, but the closeout-sized AWS lane should use
the 2xlarge-class host because smaller instances did not complete the full
benchmark set reliably.

The initial plan proposed a new generic `benchmarks/task-50-baseline/` packet.
That remains useful, but it should be local-first: capture the local smoke
baseline once, compare each slice against same-host local before/after numbers,
then use the existing AWS baselines and any missing SPIRE AWS capture for
closeout confirmation.

## Local Fast-Iteration Baseline

Capture a local baseline before Slice 1a lands:

```text
benchmarks/task-50-local-baseline/
  manifest.md
  artifacts/
    unsafe-block-count-baseline.log
    corpus-prepare-<profile>.log
    corpus-load-<profile>-<surface>.log
    recall-<profile>-<surface>.log
    latency-<profile>-<surface>.log
    storage-<profile>.log
    iai-quant-score.log
    iai-hadamard.log
    iai-bitpack.log
    criterion-quant-score.log
    criterion-hadamard.log
    criterion-page-codec.log
    recall-gate-small.log
```

The manifest must record HEAD SHA, host CPU, target features, kernel/OS,
PostgreSQL version, corpus/fixture, storage format, rerank mode, and whether
the surface is isolated one-index-per-table or shared.

The baseline packet is also responsible for making the local fixture and index
set complete. If any prepared TSV/manifest or loaded index is missing, Packet
003 should generate, load, and benchmark it before recording baseline numbers.

Full local profile spread:

| Profile | Corpus rows | Query rows |
| --- | ---: | ---: |
| `ec_real_10k` | 10,000 | 200 |
| `ec_real_25k` | 25,000 | 500 |
| `ec_real_50k` | 50,000 | 1,000 |
| `ec_real_100k` | 100,000 | 1,000 |
| `ec_real_ann_benchmarks_anchor` | 990,000 | 10,000 |

The full-spread local baseline should include every profile above for the
priority AM/storage surfaces below wherever they are locally runnable. If a row
is unsupported or operationally blocked, the packet should record it as missing
or deferred in `manifest.md`; it should not disappear silently.

| Surface label | Corpus load args | Why it is required |
| --- | --- | --- |
| `ec_ivf_rabitq` | `--profile ec_ivf --storage-format rabitq` | Priority IVF/RaBitQ optimization target. |
| `ec_spire_rabitq` | `--profile ec_spire --storage-format rabitq` | Priority SPIRE production target when locally runnable. |
| `ec_hnsw` | `--profile ec_hnsw` | Top-15 unsafe-density follow-through. |
| `ec_diskann` | `--profile ec_diskann` | Top-15 unsafe-density follow-through. |

Additional IVF storage formats such as `turboquant` or `pq_fastscan` can be
included when a slice touches shared storage-format code, but they are not a
substitute for the RaBitQ rows.

If the 990k anchor is too slow for routine per-slice iteration, it still belongs
in the local baseline packet as a closeout-scale local row or as an explicitly
deferred/missing row with the reason recorded.

`local-bench-plan.md` is the detailed local inventory and per-slice smoke table.

## Existing AWS Closeout Baselines

Use these as closeout comparators and historical context, not as the normal
per-slice iteration gate:

| Directory | Role |
| --- | --- |
| `benchmarks/cloud-10k-baselines/` | Historical m8g.large synth 10k + 50k baseline; useful for small-lane context, not full closeout sizing. |
| `benchmarks/cloud-10k-graviton-preopt-baselines/` | DBpedia 10k + 50k, including `ec_ivf` TurboQuant/RaBitQ kernel attribution; kernel battery used m8g.2xlarge. |
| `benchmarks/cloud-10k-real-baselines/` | Historical DBpedia 10k + 50k real-data baseline on m8g.large; useful context, not full closeout sizing. |
| `benchmarks/cloud-scaling-multi-am/` | Canonical full pre-optimization scaling curve on m8g.2xlarge: 10k/50k/100k/1M across `ec_hnsw`, `ec_ivf` TurboQuant/RaBitQ/PqFastScan, and `ec_diskann`. |
| `benchmarks/comparators-50k-100k-1m/` | m8g.2xlarge pgvector / pgvectorscale / vchord competitor latency context. |

## Slice Mapping

| Slice | Existing comparator | New capture needed? |
| --- | --- | --- |
| 1a callback helper seed | Local compile/block-count; no bench unless first user is hot | AWS closeout only if helper later rolls into hot callbacks. |
| 1b IVF callback rollout | Local IVF/RaBitQ smoke if hot scan/build callbacks change | AWS closeout compares against existing IVF cloud baselines. |
| 1c SPIRE callback rollout | Local SPIRE smoke if read-efficiency path changes | AWS closeout needs SPIRE evidence if no durable Task 30 phase 13d baseline exists. |
| 2 IVF page visitor | Local `page_codec`, recall gate, and IVF/RaBitQ smoke | AWS closeout compares against existing IVF cloud baselines. |
| 3a ActiveEpochAnchor seed | Local compile/block-count; targeted tests | No AWS until read-efficiency path is touched. |
| 3b SPIRE snapshots | Local SPIRE diagnostic/read-profile smoke if available | AWS only if snapshot path feeds production read profile. |
| 3c SPIRE read-efficiency rollout | Local SPIRE read-efficiency smoke for iteration | AWS closeout required: Task 30 phase 13d durable evidence or new Task 50 SPIRE capture. |
| 4 heap source scorer, IVF side | Local `quant_score`, `dhat score`, recall gate | AWS closeout compares against existing IVF/RaBitQ baselines. |
| 4 heap source scorer, SPIRE side | Local SPIRE read-efficiency smoke | AWS closeout uses SPIRE baseline from Slice 3c. |
| 5 reloptions or vector datum | Local compile/tests; `quant_encode`/`dhat encode` if vector datum | AWS closeout only if hot build/scan path changed. |
| SIMD load/store newtypes | Local x86_64 AVX2/FMA before/after is required | AWS Graviton NEON before/after confirms closeout. |

Per-slice runs can use a smaller profile for fast iteration, but the baseline
packet should establish the full spread once so later slices can choose the
smallest local profile that still covers their risk.

## SPIRE Baseline Gap

SPIRE was explicitly out of scope for the multi-AM cloud scaling cycle, so Task
50 cannot close out SPIRE no-regression from the existing AM baselines.

Before closing Slice 3c or any SPIRE hot-path structural change, do one of:

- cite Task 30 phase 13d read-efficiency evidence if it is durable under
  NFR-007 and records head SHA, command, fixture, storage format, rerank mode,
  and isolated/shared surface; or
- capture a new `benchmarks/task-50-spire-baseline/` packet with raw logs under
  `artifacts/` and `manifest.md` at the packet root.

The SPIRE baseline should include `ec_spire_remote_search_production_read_profile`
where available, candidate counts, heap session reuse, remote/local split, and
final latency. Regression tolerance follows Task 30 phase 13d / M5 policy. No
new remote candidate loss, identity mismatch, or degraded-mode behavior is
acceptable.

## Local Vs. AWS Policy

Local evidence is the fast-iteration gate:

- same-host before/after local measurements are required for slices that touch
  hot scoring, traversal, page, heap rerank, vector datum, or SIMD paths;
- callback and reloption-only slices may use compile/lint/block-count evidence
  unless the callback/option path is in a hot loop;
- local numbers can be cited in request packets as iteration evidence, but
  closeout claims still need AWS confirmation.

AWS evidence is the closeout gate:

- use existing AWS baselines where they cover the touched AM and storage
  format, preferring m8g.2xlarge artifacts for closeout-sized claims;
- capture only missing AWS lanes, notably SPIRE production read-efficiency;
- SIMD closeout requires both local x86_64 and AWS Graviton because each
  covers a different architecture path.

`local-bench-plan.md` records the local bench inventory, smoke-gate table, and
SIMD cross-architecture policy.

## Per-Packet Artifact Rule

Each implementation packet should store:

- `artifacts/block-count-before.log`;
- `artifacts/block-count-after.log`;
- relevant local before/after logs for touched hot paths;
- AWS confirmation logs when closing out a hot path or an AM surface;
- `artifacts/manifest.md` with HEAD SHA, command, timestamp, lane, fixture,
  storage format, rerank mode, and isolated-vs-shared table choice.

For callback-only packets that do not touch hot path behavior, the request may
explicitly skip runtime benches and cite this plan plus block-count evidence.

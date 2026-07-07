# Benchmarks

These results are engineering measurements on the DBpedia OpenAI embeddings
corpus (1536-dimensional `text-embedding-3-large` vectors). They are useful for
engineering decisions and review packets, but they are not product benchmark
claims: product claims need dedicated hardware with controlled cache state,
memory, storage, and repeatability.

The current production evidence is the **Task 105 full-scale matrix** — the
standard sweep (all four access methods × quantizations × 10K/50K/100K/1M) run on
both AWS production lanes at head `main=1345ca603`:

- **Graviton 4** — AWS `m8g.2xlarge` (Neoverse V2, ARM NEON dispatch)
- **Intel Sapphire Rapids** — AWS `m7i.2xlarge` (AVX2 dispatch)

Source packet: `reviews/task-105/006-full-scale-matrix/` (synthesized matrix +
per-lane raw evidence in `004-g4-lane/` and `005-intel-lane/`). Distributed
(multi-node) SPIRE evidence is `reviews/task-107/`. New benchmark rows should
follow the [Benchmark Reporting Standard](benchmark-reporting-standard.md), which
defines the common fields for comparing access methods, quantizers, storage
formats, trained formats, and option sets.

## Targets

NFR-003 recall targets (`ec_hnsw`):

| Configuration | Target |
| --- | ---: |
| `m = 8`, `ef_search = 128` | >= 89% |
| `m = 8`, `ef_search = 200` | >= 93% |
| `m = 16`, `ef_search = 200` | >= 97% |

NFR-001 latency targets, top-10 query on 50K vectors:

| Percentile | Target |
| --- | ---: |
| p50 | < 5 ms |
| p99 | < 15 ms |

## Production Benchmark Matrix

Each family below is shown at its headline production quantization across scales
on both AWS lanes. Cells are recall@10 and p50 latency at a representative
high-recall operating point (HNSW `ef_search=160`, IVF/SPIRE `nprobe=64`, DiskANN
`list_size=128`), kernel-on, `k=10`. Index sizes are from the G4 lane (Intel
matches within build noise). The full grid — every quant mode, both sweep points,
kernel on/off A/B, and ISA attribution — is in the Task 105 matrix.

**100K note:** by Phase 2 design the 100K on/off A/B was delegated to Task 99
(`reviews/task-99/008-g4-lane`, `009-intel-lane`); Task 105 collected only the G4
100K dispatch-confirm column. The 10K/50K/1M cells below are the cleanly measured,
both-lane scales.

### `ec_hnsw` (turboquant)

| Scale | Graviton 4 recall@10 / p50 | Intel recall@10 / p50 | Index size |
| --- | --- | --- | ---: |
| 10K | 0.968 / 2.6 ms | 0.968 / 2.3 ms | 13.0 MiB |
| 50K | 0.926 / 3.3 ms | 0.926 / 3.0 ms | 65.1 MiB |
| 1M | 0.930 / 13.7 ms | 0.930 / 15.0 ms | 1.3 GiB |

Also benched: `rabitq` (1M G4 0.917 / 14.7 ms). HNSW recall tops out lower than
the other families at 1M in this sweep.

### `ec_ivf` (rabitq, 1-bit)

| Scale | Graviton 4 recall@10 / p50 | Intel recall@10 / p50 | Index size |
| --- | --- | --- | ---: |
| 10K | 1.000 / 2.3 ms | 1.000 / 2.0 ms | 3.7 MiB |
| 50K | 0.998 / 10.3 ms | 0.998 / 7.1 ms | 15.3 MiB |
| 1M | 0.980 / 56.8 ms | 0.968 / 61.5 ms | 290 MiB |

Also benched: `turboquant` (1M G4 0.986 / 237 ms, 862 MiB) and `pq_fastscan`
(fast but low recall — 1M G4 0.648 / 70.7 ms, 179 MiB). RaBitQ 1-bit is the
recall/size sweet spot.

### `ec_diskann` (rabitq)

| Scale | Graviton 4 recall@10 / p50 | Intel recall@10 / p50 | Index size |
| --- | --- | --- | ---: |
| 10K | 0.999 / 1.6 ms | 0.999 / 1.4 ms | 4.1 MiB |
| 50K | 0.991 / 2.6 ms | 0.991 / 1.8 ms | 20.6 MiB |
| 1M | 0.981 / 5.0 ms | 0.978 / 5.1 ms | 407 MiB |

Also benched: `turboquant` (1M G4 0.982 / 6.7 ms, 967 MiB) and `pq_fastscan`
binary (1M G4 0.964 / 4.8 ms, 455 MiB). DiskANN has the best recall-per-ms at
scale; it requires unit-normalized source vectors.

**Task 168 (2026-07)**: `rabitq` is now the code default (`storage_format`
reloption no longer needed), scans use width-4 batched-beam expansion
(`ec_diskann.beam_width` GUC, `SET ... = 1` restores the one-pop loop), and
the beam loop decodes allocation-free. Local-Intel-desktop A/B on the staged
real corpus: −6 to −23% mean latency at 50k/100k mid-to-high `list_size`
(best 100k L=200 7.31 → 5.61 ms cumulative) with recall@10 equal or better
at every sweep point (100k L=64 improves 0.9275 → 0.9360). The AWS-lane
cells above predate this and refresh on their next canonical run. Evidence:
`reviews/task-168/00{1..5}-*`.

### `ec_spire` (rabitq, single-node)

| Scale | Graviton 4 recall@10 / p50 | Intel recall@10 / p50 | Index size |
| --- | --- | --- | ---: |
| 10K | 1.000 / 8.8 ms | 1.000 / 8.0 ms | 8.2 MiB |
| 50K | 0.998 / 22.5 ms | 0.998 / 21.0 ms | 40.6 MiB |
| 1M | 0.986 / 136.6 ms | 0.983 / 125.0 ms | 779 MiB |

Also benched: `turboquant` (1M G4 0.986 / 165.8 ms). The rows above are
single-node; SPIRE's value-prop is multi-node (below).

#### SPIRE distributed (multi-node)

A real **3-node** SPIRE deployment (1 coordinator + 2 remotes), 1M corpus sharded
across the remotes (~505K + ~485K rows), genuine remote-heap reads, `nprobe=64`
(Task 107):

| Topology | Quant | Recall@10 | p50 | p95 |
| --- | --- | ---: | ---: | ---: |
| 3-node distributed | rabitq | 0.951 | 117 ms | 135 ms |
| 3-node distributed | turboquant | 0.949 | 140 ms | 164 ms |

Distributing across 3 nodes is ~5x faster at 1M than the same index single-node
(121 ms vs 620 ms at matched `nprobe=32`): SPIRE trades latency for scale-out
partitioning. It is currently a research / scale-out surface with latency
optimization ongoing. Source: `reviews/task-107/`
(`005-product-decision/`, `004-distributed-completion/`).

## Competitor Comparison

Pinned competitor latencies on the same Graviton 4 corpus, k=10
(`benchmarks/comparators-50k-100k-1m/`, vchord probe sweep in
`comparators-vchord-warm-g4/`). vchord RaBitQ-on-IVF is the only tuned-competitive
comparator; pgvector / pgvectorscale defaults below were untuned upper bounds.

1M Pareto (p50 / recall@10):

| System | p50 | recall@10 |
| --- | ---: | ---: |
| ecaz `ec_diskann` rabitq | 5.0 ms | 0.981 |
| ecaz `ec_ivf` rabitq1 (nprobe64) | 56.8 ms | 0.980 |
| vchord RaBitQ-on-IVF (default) | 90.3 ms | 0.9995 |
| pgvectorscale DiskANN sl40 | 6.5 ms | 0.980 |
| pgvector HNSW ef40 | 2.9 ms | 0.932 |
| pgvector IVFFlat p100 | 265 ms | 0.987 |

At 1M, `ec_ivf` rabitq1 serves 0.980 recall faster than the tuned vchord bar
(vchord reaches ~1.0 recall at higher latency), and `ec_diskann` matches
pgvectorscale DiskANN's latency at comparable recall with no tuning required.
Competitor numbers are not re-run unless the competitor version or hardware
changes.

## Development Lanes

The M5-local (Apple Silicon, NEON) and Intel-local lanes have canonical standard
sweep configs (`crates/ecaz-cli/suites/current/{m5-local,intel-local}.json`) but
no promoted full-scale result yet; treat them as pending until a dev-lane sweep is
packetized. The IVF rerank-format work has its own recent Intel-local evidence
under `reviews/task-111h/` and `benchmarks/ivf-111g-115-attribution/`.

## Storage

Encoded payload size per vector, 1536 dimensions (quantized code bytes; the
on-disk index adds posting-list/graph structure — see index sizes above):

| Representation | Bytes per vector | Relative size |
| --- | ---: | ---: |
| Raw fp32 | 6,144 B | 1.00x |
| PQ-FastScan g8 | 96 B | 64.0x smaller |
| RaBitQ 1-bit | 204 B | 30.1x smaller |
| RaBitQ 2-bit | 396 B | 15.5x smaller |
| TurboQuant 2-bit | 399 B | 15.4x smaller |
| RaBitQ 4-bit | 780 B | 7.88x smaller |
| TurboQuant 4-bit | 783 B | 7.85x smaller |
| RaBitQ 8-bit | 1,548 B | 3.97x smaller |
| TurboQuant 8-bit | 1,551 B | 3.96x smaller |

Storage-format comparisons should use
[Benchmark Reporting Standard](benchmark-reporting-standard.md) candidate
identity fields, including payload format, model metadata, sidecar bytes, and
serving readiness.

## Running Benchmarks

### Criterion microbenchmarks

```bash
make bench
make bench-quant_score
```

### Instruction-count benchmarks

Requires valgrind:

```bash
make bench-iai
```

### SQL and corpus benchmarks

Requires PostgreSQL with the extension installed. The supported repeatable
operator surface is the `ecaz` CLI. All benchmark matrices, sweeps, and
multi-step runs go through `ecaz bench suite` with a checked-in `SuiteConfig`;
the canonical per-lane configs are
`crates/ecaz-cli/suites/current/{m5-local,intel-local,aws-intel,aws-graviton}.json`
(the standard ecaz sweep: 4 profiles × 10K/50K/100K/1M × load/recall/latency/storage).

```bash
PACKET=benchmarks/<topic>
ecaz bench suite run \
  --config crates/ecaz-cli/suites/current/aws-graviton.json \
  --artifact-dir "$PACKET/artifacts"
```

Single-step commands are also available for ad hoc checks:

```bash
ecaz corpus load --prefix ec_real_10k --profile ec_hnsw --log-file "$PACKET/artifacts/load.log"
ecaz bench recall --prefix ec_real_10k --profile ec_hnsw --log-file "$PACKET/artifacts/recall.log"
ecaz bench latency --prefix ec_real_10k --profile ec_hnsw --log-file "$PACKET/artifacts/latency.log"
```

See the [Operator CLI README](../crates/ecaz-cli/README.md) for all command
groups and profile behavior.

### AWS benchmark cycles

For benches that run against AWS (`ecaz cloud up/install/bench/snapshot/down`),
the [AWS Benchmark Workflow](aws-bench-workflow.md) is **required reading**.
It encodes the snapshot-before-destroy invariant and the
reuse-before-rebuild policy. `ecaz cloud down` will refuse to tear down a
stack whose data volume has no EBS snapshot, to prevent the kind of data
loss the prior `cloud-scaling-multi-am` cycle hit.

The workflow doc also maintains the **snapshot inventory** — the
ground-truth list of which snapshots cover which corpora and access
methods — that future cycles consult before paying to rebuild.

## Methodology

See [Benchmark Reporting Standard](benchmark-reporting-standard.md),
[Recall Methodology](recall-methodology.md), and
[Real Corpus Recall](RECALL_REAL_CORPUS.md) for the reporting schema, dataset
contracts, corpus selection rules, and reproduction instructions.

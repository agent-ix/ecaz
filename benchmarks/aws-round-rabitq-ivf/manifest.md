# Manifest: aws-round-rabitq-ivf

- packet: `benchmarks/aws-round-rabitq-ivf`
- manifest update head SHA: `b61a794d7357ffd77b659522dbfb798afb167880`
- manifest update timestamp: `2026-05-23T05:08:17Z`
- lane: AWS Graviton IVF + RaBitQ optimization round
- primary access method under optimization: `ec_ivf`
- primary storage format under optimization: `rabitq`
- non-authoritative historical comparators: vchord rows in
  `paired-comparison.md`; no new comparator run is introduced by this
  manifest update

This manifest is the packet-root provenance index required by the benchmark
packet rules. It consolidates the older artifact-local
`artifacts/MANIFEST.md` and records which artifacts are authoritative versus
historical or incomplete.

## Environment And Snapshots

AWS artifacts in this packet were produced in `us-west-2` on Graviton hosts:

- initial 10k/50k baseline: `m8g.large`, 50 GB gp3, restored from
  `snap-054feaffc50ecf1c9`
- later 10k/50k/100k experiments: `m8g.xlarge` where noted in `results.md`
- 1M recall and paired comparison: `m8g.2xlarge`, 100 GB gp3, because the
  exhaustive ground-truth corpus did not fit on the smaller host

Relevant snapshot inventory entries live in
`docs/aws-bench-workflow.md#snapshot-inventory`:

- `snap-0bb07e0b82150a062`: post-NEON 10k/50k storage-format variants
- `snap-01838d965fa09c433`: post bits=1 + rerank variants
- `snap-0975811a1da6ea302`: post 100k/1M IVF RaBitQ closure state
- `snap-0e9c7743263e61d70`: post 1M recall measurement state
- `snap-091251b06d2da2df4`: post paired vchord sweep state

The AWS workflow doc now treats `nlists` as a rebuild-required IVF geometry
parameter, not as a scan-time knob.

## Benchmark Surfaces

### Initial Suite-Driven Baseline

- config: `suite-10k-50k.json`
- fixture: isolated one-index-per-table surfaces for 10k and 50k
- storage formats: `turboquant`, `rabitq`, `pq_fastscan`
- nprobe sweep: `8,16,24,32,48,64`
- q-count: default `200`, with 50k recall steps overriding to `1000`
- runner: `ecaz bench suite run`
- status: baseline suite config is checked in; the final checked-in packet
  does not include the suite runner's structured `suite-manifest.json` or
  `results.jsonl`

### IVF RaBitQ Optimization Measurements

- fixture: DBpedia real corpora at 50k, 100k, and 1M
- storage format: `rabitq`
- quantization: `quant_bits=1` for the final operating points
- rerank: `heap_f32`
- rerank width: `50` for final operating points
- warmup: mixed across the historical round; authoritative logs are named
  below and failed prewarm attempts are listed as failed/incomplete

### Paired Comparator Measurement

- summary: `paired-comparison.md`
- host: `m8g.2xlarge`
- q-count: paired comparison text says `q=100`; logs contain later `q=300`
  sections for ec_ivf and exhaustive ground truth. Treat this as useful
  exploratory paired evidence, not final Task 51 closure evidence.
- runner status: this paired pass was not driven by a checked-in
  `ecaz bench suite` config and has no `suite-manifest.json` / `results.jsonl`
  aggregation.
- current Task 51 scope note: no new vchord or pgvectorscale run is scheduled
  from this manifest update; current local optimization work is IVF/RaBitQ-only.

## Authoritative Artifacts

The artifact-local `artifacts/MANIFEST.md` gives per-file detail. The
authoritative files for the historical AWS round are:

| File | Role |
| --- | --- |
| `artifacts/latency-truly-prewarmed.log` | 10k/50k bits=4 latency after prewarm quoting fix |
| `artifacts/explain-counters.log` | 50k EXPLAIN counter attribution |
| `artifacts/latency-bits1-v3.log` | 50k bits=1 first-cut scalar-select kernel |
| `artifacts/latency-bits1-bytelut-fixed.log` | 50k bits=1 byte-LUT after per-query hoist |
| `artifacts/latency-bits1-width-sweep.log` | 50k rerank-width sweep |
| `artifacts/latency-bits1-width50-nprobe-sweep.log` | 50k bits=1 width=50 nprobe curve |
| `artifacts/latency-100k-bits1.log` | 100k bits=1 + width=50 latency and recall |
| `artifacts/latency-1m-bits1-v3.log` | 1M bits=1 latency; first cell is marked warmup-noisy in `results.md` |
| `artifacts/recall-1m-bits1-q500.log` | 1M bits=1 recall, q=500, exhaustive f32 ground truth |
| `artifacts/closure-prep.log` | 100k/1M corpus prepare and 100k load reference |
| `artifacts/closure-1m-load.log` | 1M load and index build reference |
| `artifacts/paired-all.log` | paired 50k/100k and partial 1M exploratory comparison log |
| `artifacts/paired-1m-full.log` | paired 1M exploratory comparison log |
| `artifacts/recall-final.log` | vchord recall rows for the paired exploratory comparison |
| `paired-comparison.md` | human-readable paired comparison summary and caveats |
| `results.md` | historical round summary, including corrected paired-comparison caveat |

## Failed Or Incomplete Artifacts

These files remain in the packet for traceability but are not authoritative:

| File | Status |
| --- | --- |
| `artifacts/latency-bits1.log` | failed bits=1 build-path threading; superseded by `latency-bits1-v3.log` |
| `artifacts/latency-bits1-bytelut.log` | failed SSM shell quoting around parens; superseded by `latency-bits1-bytelut-fixed.log` |
| `artifacts/latency-bits1-v2.log` | failed SSM dollar-quoting around prewarm SQL; superseded by `latency-bits1-v3.log` |
| `artifacts/latency-1m-bits1.log` | failed/truncated first 1M run; latency superseded by `latency-1m-bits1-v3.log`, recall by `recall-1m-bits1-q500.log` |
| `artifacts/latency-1m-bits1-v2.log` | failed SSM dollar-quoting; no benchmark rows |
| `artifacts/recall-1m-bits1.log` | OOM-killed first recall pass on `m8g.xlarge`; superseded by `recall-1m-bits1-q500.log` |
| `artifacts/cloud-snapshot-1m.log` | zero-byte stdout capture; snapshot id is recorded in `docs/aws-bench-workflow.md` |

## Key Historical Results

The current historical operating point for compact IVF RaBitQ is:

- 50k: `3.81 ms p50 @ recall@10 0.986`, bits=1 + `heap_f32` + width=50,
  nprobe=64
- 100k: `9.48 ms p50 @ recall@10 0.9896`, bits=1 + `heap_f32` + width=50,
  nprobe=128
- 1M: `67.3 ms p50 @ recall@10 0.9936`, bits=1 + `heap_f32` + width=50,
  nprobe=256, q=500 recall

The paired exploratory comparison later corrected the earlier vchord claim:
at matched recall, compact `ec_ivf` RaBitQ was 3-4x slower than vchord on the
same `m8g.2xlarge`, while using substantially less index storage.

## Known Gaps Before Task 51 Closeout

This manifest closes the packet-root provenance gap for the historical AWS
round, but it does not close Task 51. Remaining gaps include:

- the final Task 51 matrix still needs suite-driven structured output
  (`suite-manifest.json` and `results.jsonl`);
- 1M EXPLAIN counters are still needed for the current compact IVF RaBitQ
  frontier;
- at least two Task 51 experiments still need local measurement under the
  current IVF/RaBitQ-only scope;
- AWS should remain the final gate after local code and benchmark evidence are
  ready.

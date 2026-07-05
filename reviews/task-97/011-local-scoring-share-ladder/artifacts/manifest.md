# Task 97 Packet 011 Artifact Manifest

- head SHA: `c07590302f2467cc2b52f84fb856acd3c612688c`
- task bucket: `reviews/task-97/011-local-scoring-share-ladder`
- lane: Task 97 TurboQuant QJL block kernel
- fixture: local PG18, deterministic synthetic corpus, `dim=1024`, `bits=4`, `seed=42`, `queries_seed=43`
- storage format: `turboquant`
- rerank / exact mode: production QJL (`MseLutQjl`), not no-QJL / LUT32
- host ISA: local x86_64 AVX2 where block width reaches 32; scalar tails otherwise
- suite config: `reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json`
- AWS / CI: not run

## Commands

- Kernel-on local slice:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/011-local-scoring-share-ladder/artifacts/suite-kernel-on-cli.log bench suite run --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json --artifact-dir reviews/task-97/011-local-scoring-share-ladder/artifacts --only-tag kernel_on --manifest-output reviews/task-97/011-local-scoring-share-ladder/artifacts/suite-kernel-on-manifest.json --results-output reviews/task-97/011-local-scoring-share-ladder/artifacts/results-kernel-on.jsonl`
- Kernel-off local slice:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/011-local-scoring-share-ladder/artifacts/suite-kernel-off-cli.log bench suite run --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json --artifact-dir reviews/task-97/011-local-scoring-share-ladder/artifacts --only-tag kernel_off --manifest-output reviews/task-97/011-local-scoring-share-ladder/artifacts/suite-kernel-off-manifest.json --results-output reviews/task-97/011-local-scoring-share-ladder/artifacts/results-kernel-off.jsonl`

## Primary Artifacts

- `suite-kernel-on-cli.log`, `suite-kernel-on-manifest.json`, `results-kernel-on.jsonl`
- `suite-kernel-off-cli.log`, `suite-kernel-off-manifest.json`, `results-kernel-off.jsonl`
- `recall-ivf-turboquant-qjl32-batch-on.log`
- `recall-spire-turboquant-qjl32-batch-on.log`
- `recall-hnsw-turboquant-qjl32-batch-on.log`
- `latency-ivf-turboquant-qjl32-batch-on.log`
- `latency-ivf-turboquant-qjl32-batch-off.log`
- `latency-spire-turboquant-qjl32-batch-on.log`
- `latency-spire-turboquant-qjl32-batch-off.log`
- `latency-hnsw-turboquant-qjl32-batch-on.log`
- `latency-hnsw-turboquant-qjl32-batch-off.log`

## Local End-To-End Latency Ratios

Speedup is `kernel_off_mean / kernel_on_mean`; values below `1.0x` are slower with kernel-on.

| Surface | Parameter | Kernel off mean | Kernel on mean | Speedup |
| --- | ---: | ---: | ---: | ---: |
| IVF | `nprobe=8` | `1.19 ms` | `1.23 ms` | `0.97x` |
| IVF | `nprobe=16` | `1.50 ms` | `1.55 ms` | `0.97x` |
| SPIRE | `nprobe=8` | `9.10 ms` | `9.04 ms` | `1.01x` |
| SPIRE | `nprobe=16` | `17.5 ms` | `17.4 ms` | `1.01x` |
| HNSW | `ef_search=32` | `1.71 ms` | `1.84 ms` | `0.93x` |

These local end-to-end ratios are far below the Task 97 scoring-share ladder (`1.5x` stop-condition floor, `1.8x` acceptable, `2.0x` target).

## Direct Counter Rows

IVF kernel-on direct rows:

- `nprobe=8`: `surface=ivf quant=turboquant_qjl isa=avx2 kernel_candidates=24096 kernel_elapsed_ms=20.721745`; scalar tails `scalar_candidates=1263 scalar_elapsed_ms=1.343043`.
- `nprobe=16`: `surface=ivf quant=turboquant_qjl isa=avx2 kernel_candidates=51200 kernel_elapsed_ms=44.323746`.

IVF kernel-off emitted no direct `[block-kernel-counters]` rows in this local run, so this packet does not claim an IVF scoring-share ratio from direct counters.

SPIRE direct counter comparison:

| Parameter | Kernel-off scalar elapsed | Kernel-on AVX2 elapsed | Kernel-on scalar-tail elapsed | Kernel-on total scoring elapsed | Direct scoring speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| `nprobe=8` | `22.801836 ms` | `11.755793 ms` | `12.685565 ms` | `24.441358 ms` | `0.93x` |
| `nprobe=16` | `45.574628 ms` | `25.250680 ms` | `24.405899 ms` | `49.656579 ms` | `0.92x` |

HNSW kernel-on direct row:

- `ef_search=32`: `surface=hnsw quant=turboquant_qjl isa=scalar scalar_candidates=29763 scalar_elapsed_ms=32.545251`.

The HNSW local `m=8` fixture still does not reach block width 32 during graph expansion, so it is scalar-tail evidence only.

## Interpretation

This packet is evidence-only. It does not include or request approval for a kernel optimization change.

The corrected local QJL fixture confirms routing/counter attribution, but the current AVX2 QJL block path does not satisfy the Task 97 performance ladder on local evidence. Before Task 97 closeout, the project needs either:

- a reviewed optimization slice for the qjl32 AVX2 block path, followed by refreshed local and approved Graviton 4 evidence; or
- a reviewed stop-condition disposition accepting the current Task 97 QJL performance state.

## Validation

- Local PG18 suite slices only.
- No code changed.
- No GitHub CI or AWS runs were used.

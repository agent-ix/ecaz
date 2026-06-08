# Task 89 Validation Matrix

Date: 2026-06-08

Head inspected: `3aef930a1`

## Purpose

Task 89 completion requires evidence for all index types:

- IVF
- SPIRE
- HNSW
- DiskANN

The validation must compare standard TurboQuant against TQ+ using the ADR-076
production surface:

```text
storage_format=turboquant
turboquant_profile=standard | tqplus
```

This matrix records the required suite cells before the ports land. It is a
post-port execution plan, not evidence that the current branch already supports
the reloption.

## Required DBPedia Per-AM Matrix

All runs use:

- PG: 18
- bits: 4
- seed: 42
- k: 10
- profile pair: `standard`, `tqplus`
- metrics: load, storage, recall, latency
- isolation: one index per table
- corpus: `data/task31_m5_dbpedia_staged/`

| AM | Scale | Build shape | Probe/search sweep | Rerank | Required artifacts |
| --- | --- | --- | --- | --- | --- |
| IVF | 10k | `nlists=32` | `nprobe=[8,24,32]` | `heap_f32`, width 25 | load, storage, recall, latency for standard and TQ+ |
| IVF | 50k | `nlists=64` | `nprobe=[16,48,64]` | `heap_f32`, width 25 | load, storage, recall, latency for standard and TQ+ |
| IVF | 100k | `nlists=128` | `nprobe=[32,96,128]` | `heap_f32`, width 25 | load, storage, recall, latency for standard and TQ+ |
| SPIRE | 10k | `nlists=32` | `nprobe=[8,24,32]` | width 25 | load, storage, recall, latency, pipeline for standard and TQ+ |
| SPIRE | 50k | `nlists=64` | `nprobe=[16,48,64]` | width 25 | load, storage, recall, latency, pipeline for standard and TQ+ |
| SPIRE | 100k | `nlists=128` | `nprobe=[32,96,128]` | width 25 | load, storage, recall, latency, pipeline for standard and TQ+ |
| HNSW | 10k | `m=16`, `ef_construction=128` | `ef_search=[64,128,200,400]` | current HNSW rerank defaults | load, storage, recall, latency for standard and TQ+ |
| HNSW | 50k | `m=16`, `ef_construction=128` | `ef_search=[64,128,200,400]` | current HNSW rerank defaults | load, storage, recall, latency for standard and TQ+ |
| HNSW | 100k | `m=16`, `ef_construction=128` | `ef_search=[64,128,200,400]` | current HNSW rerank defaults | load, storage, recall, latency for standard and TQ+ |
| DiskANN | 10k | `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `list_size=[64,128,200,400,800]` | current DiskANN rerank defaults | load, storage, recall, latency for standard and TQ+ |
| DiskANN | 50k | `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `list_size=[64,128,200,400,800]` | current DiskANN rerank defaults | load, storage, recall, latency for standard and TQ+ |
| DiskANN | 100k | `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | `list_size=[64,128,200,400,800]` | current DiskANN rerank defaults | load, storage, recall, latency for standard and TQ+ |

## Truth Cache Inputs

Reuse or regenerate exact ground truth per scale:

- 10k: `reviews/task-89/<packet>/artifacts/truth-real10k-k10.json`
- 50k: `reviews/task-89/<packet>/artifacts/truth-real50k-k10.json`
- 100k: `reviews/task-89/<packet>/artifacts/truth-real100k-k10.json`

If reusing Task 86 truth caches, the packet manifest must record the source
path and prove the corpus/query files match the Task 89 suite inputs.

## Cross-Corpus Matrix

Task 89 requires one non-DBPedia embedding distribution. The cross-corpus
packet should run all four AMs at one scale, preferably 10k first:

| AM | Scale | Corpus | Profile pair | Required metrics |
| --- | --- | --- | --- | --- |
| IVF | 10k or 50k | non-DBPedia | standard vs. TQ+ | storage, recall, latency |
| SPIRE | 10k or 50k | non-DBPedia | standard vs. TQ+ | storage, recall, latency, pipeline |
| HNSW | 10k or 50k | non-DBPedia | standard vs. TQ+ | storage, recall, latency |
| DiskANN | 10k or 50k | non-DBPedia | standard vs. TQ+ | storage, recall, latency |

Acceptable corpus choices are whichever is locally available and not DBPedia:
text-embedding-3-large, Cohere, multilingual-e5, or image embeddings. The
packet must record manifest/source identity and dimensions.

## Streaming-Insert Drift Matrix

Per ADR-076:

- build TQ+ at 10k;
- insert 10%, 25%, and 50% more rows after build;
- compare recall@10 against a full-rebuild TQ+ baseline at each post-insert
  size;
- acceptance thresholds:
  - 25% inserted rows: recall delta <= 0.5 percentage points;
  - 50% inserted rows: recall delta <= 1.0 percentage point.

The drift matrix should run all AMs if inserts are supported. If an AM lacks a
live insert path for the required storage shape, that is a Task 89 finding and
must be escalated to redesign/defer rather than omitted.

## Closeout Evidence Table

The final closeout packet must include this table filled with packet paths:

| Requirement | Evidence packet |
| --- | --- |
| Phase 1 ADR approved | `reviews/task-89/001-format-design-adr/feedback/...` |
| IVF DBPedia matrix | TBD |
| SPIRE DBPedia matrix | TBD |
| HNSW DBPedia matrix | TBD |
| DiskANN DBPedia matrix | TBD |
| Cross-corpus all-AM matrix | TBD |
| Streaming-insert drift | TBD |
| Deterministic rebuild/golden page tests | TBD |
| Final promote/redesign/defer decision | TBD |

## Notes From Current Code Inventory

- IVF and SPIRE can likely adopt the suite template first because their suite
  load steps already accept arbitrary reloptions.
- HNSW load steps also accept reloptions through the CLI load path, but the
  suite template must include `storage_format=turboquant` explicitly after the
  profile reloption lands.
- DiskANN must first add baseline `storage_format=turboquant`; current suite
  configs only exercise `pq_fastscan`/default and `rabitq`.

# Task 80 AWS 1M Block16 Pruning Manifest

Status: pending AWS run

- Head SHA: `b236adf231bf384beff51682a20ec666af416842`
- Task bucket: `reviews/task-80/001-aws-1m-block16-pruning/`
- Suite config: `reviews/task-80/001-aws-1m-block16-pruning/suite-aws-1m-block16-pruning.json`
- Artifact directory: `reviews/task-80/001-aws-1m-block16-pruning/artifacts/aws-1m-block16-pruning/`
- Lane: AWS 1M, PG18, SPIRE, RaBitQ
- Fixture: retained `task67_1m_hnsw_m7g2xlarge` corpus and queries in AWS profile `1m`
- Surface: shared retained 1M table with one active SPIRE index after the build step
- Index shape: `nlists=128`, `recursive_fanout=8`, `storage_format=rabitq`,
  `boundary_replica_count=0`, `top_graph_search_list_size=256`,
  `ec_spire.leaf_block_rows=16`
- Query shape: q500, `rerank_width=25`, nprobe sweep `96,128,256`, global
  block caps `1152,2048,4096,8192`, production read profile enabled
- Truth cache: `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`

## Commands

Pending.

## Artifacts

Pending.

## Key Results

Pending.

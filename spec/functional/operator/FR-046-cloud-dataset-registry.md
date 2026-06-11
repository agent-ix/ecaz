---
id: FR-046
title: Cloud Dataset Registry
type: functional-requirement
artifact_type: FR
status: PROPOSED
object_type: data-source
relationships:
  - target: "ix://agent-ix/ecaz/US-021"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-044"
    type: "supports"
    cardinality: "1:1"
---
# FR-046: Cloud Dataset Registry

## Requirement

The cloud harness SHALL ship a dataset registry that maps short
names to source locations, dimensions, distance metrics, and the
third-party benchmarks each dataset is comparable against.

## Behavior

1. The registry SHALL be the single source of truth for
   `ecaz cloud corpus stage --dataset <name>`.
2. The registry SHALL include at least the following entries:

   | Name | Source | Rows | Dim | Comparable to |
   |---|---|---|---|---|
   | `dbpedia-1m` | HF `Qdrant/dbpedia-entities-openai3-embedding-3-large-1536-1M` | 1M | 1536 | Qdrant blog benches |
   | `dbpedia-ada-1m` | HF `KShivendu/dbpedia-entities-openai-1M` | 1M | 1536 | pgvector posts |
   | `cohere-wiki-10m` | HF `Cohere/wikipedia-22-12-en-embeddings` (subset) | 10M | 768 | Qdrant, Weaviate |
   | `cohere-wiki-35m` | HF `Cohere/wikipedia-22-12-en-embeddings` | 35M | 768 | Qdrant, Weaviate |
   | `laion-100m` | HF `laion/laion2B-en-vit-l-14-embeddings` (subset) | 100M | 768 | LAION research |
   | `bigann-1b` | `big-ann-benchmarks.com` mirror | 1B | 128 | NeurIPS Big-ANN, ann-benchmarks.com |

3. Parquet-native sources SHALL flow through the existing
   `ecaz corpus fetch` + `ecaz corpus prepare` pipeline unchanged.
4. Non-parquet binary formats (`.fbin`, `.u8bin`) used by BIGANN /
   DEEP1B SHALL be converted to parquet during the
   `corpus stage` step via a new adapter; downstream load logic is
   unchanged.
5. Each registry entry SHALL declare `dim`, `distance`, `row_count`,
   and an `expected_sha256` for the staged parquet manifest so
   `corpus stage` is verifiable and re-runnable.
6. `ecaz cloud corpus list-datasets` SHALL print the registry as a
   human-readable table and as JSON with `--json`.

## Acceptance Criteria

### FR-046-AC-1

Every registered dataset has a non-empty `source`, `dim`,
`row_count`, and `comparable_to` field.

### FR-046-AC-2

`corpus stage --dataset bigann-1b --dry-run` reports the planned
S3 keys and total bytes without downloading.

### FR-046-AC-3

A staged dataset's manifest SHA matches the registry's
`expected_sha256` after a successful `corpus stage` run.

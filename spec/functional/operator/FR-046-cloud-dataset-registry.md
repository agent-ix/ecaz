---
id: FR-046
title: Cloud Dataset Registry
type: FR
status: PROPOSED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/US-021"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-044"
    type: "supports"
    cardinality: "1:1"
---
# FR-046: Cloud Dataset Registry

## Description

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
5. Each registry entry SHALL declare `dim`, `distance`, `rows`, and
   `format` so `corpus stage` is deterministic and re-runnable.
   Staging verification SHALL rely on the staged `_manifest.json`
   written into S3 (`{name, format, rows, dim}`); the registry does
   not store a per-entry `expected_sha256`.
6. `ecaz cloud corpus list-datasets` SHALL print the registry as a
   human-readable table and as JSON with `--json`.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-046-AC-1 | Every registered dataset has a non-empty `source`, `dim`, `row_count`, and `comparable_to` field | Test |
| FR-046-AC-2 | `corpus stage --dataset bigann-1b --dry-run` reports the planned S3 keys and total bytes without downloading | Test |
| FR-046-AC-3 | A staged dataset's `_manifest.json` records `{name, format, rows, dim}` matching the registry entry after a successful `corpus stage` run | Demonstration |

### FR-046-AC-1

Every registered dataset has a non-empty `source`, `dim`,
`row_count`, and `comparable_to` field.

### FR-046-AC-2

`corpus stage --dataset bigann-1b --dry-run` reports the planned
S3 keys and total bytes without downloading.

### FR-046-AC-3

A staged dataset's `_manifest.json` (written to S3 by `corpus stage`)
records `{name, format, rows, dim}` matching the registry entry after a
successful `corpus stage` run.

## Schema

The registry is a static array (`datasets::REGISTRY`) of `Dataset` records.
The schema below reflects the implemented `Dataset` struct (the
`distance` and `format` fields are enums; `comparable_to` is an array). The
struct currently has no `expected_sha256` field — staging verification relies on
the staged `_manifest.json` written into S3 (`{name, format, rows, dim}`).

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Dataset",
  "type": "object",
  "additionalProperties": false,
  "required": ["name", "source", "source_path", "rows", "dim", "distance", "format", "comparable_to"],
  "properties": {
    "name": { "type": "string", "description": "short registry key, e.g. dbpedia-1m" },
    "source": { "type": "string", "description": "HF repo path or canonical mirror URL" },
    "source_path": { "type": "string", "description": "glob or path under the source pointing at the data, e.g. *.parquet" },
    "rows": { "type": "integer", "minimum": 0, "description": "declared row count (u64)" },
    "dim": { "type": "integer", "minimum": 1, "description": "embedding dimensionality (u32)" },
    "distance": { "type": "string", "enum": ["Cosine", "InnerProduct", "L2"] },
    "format": {
      "type": "string",
      "enum": ["Parquet", "BigAnnFbin"],
      "description": "Parquet flows through corpus prepare/load unchanged; BigAnnFbin (.u8bin/.fbin) is converted to parquet during corpus stage"
    },
    "comparable_to": {
      "type": "array",
      "items": { "type": "string" },
      "minItems": 1,
      "description": "third-party benchmarks this dataset is comparable against"
    }
  }
}
```

## Dependencies

- **Upstream**: US-021 (implements), FR-044 (supports)
- **Downstream**: FR-047 (loader fan-out consumes registry `row_count` declarations)

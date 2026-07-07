---
type: index
title: "Functional"
description: "Index of functional requirements, grouped by bounded context."
---
# Functional

Functional requirement files keep their immutable `FR-###` IDs, but are grouped
by bounded context so new work has an obvious home.

## Contents

* [`common/`](./common/) — SQL bootstrap, row types, PG18 callbacks, planner/statistics, WAL, vacuum, and shared access-method contracts.
* [`quant/`](./quant/) — TurboQuant payloads, prepared scoring, `QuantCodec`, candidate batching, SIMD/block-kernel requirements.
* [`index/hnsw/`](./index/hnsw/) — `ec_hnsw` page, build, scan, insert, vacuum, and current AM surface.
* [`index/ivf/`](./index/ivf/) — `ec_ivf` build/storage, scan/rerank/cost, insert/vacuum/admin surface.
* [`index/diskann/`](./index/diskann/) — `ec_diskann` build/storage, scan/prefilter/rerank, insert/vacuum/diagnostic surface.
* [`index/distann/`](./index/distann/) — `ec_distann` single global Vamana graph: record format, sharded build + stitch, hash placement, remote expansion, head index, hop-round orchestration, epoch lifecycle, DML.
* [`operator/`](./operator/) — `ecaz` CLI, configured benchmark suites, and cloud operator surfaces.
* [`spire/`](./spire/) — SPIRE bounded context: storage, local execution, distributed execution, operations, and archived superseded SPIRE IDs.

## Authoring Notes

When moving a requirement, preserve its frontmatter `id` and update active spec,
docs, ADR, and test-matrix links. Historical review packet prose can remain as
the original record unless it is being republished as current evidence.

New requirements are authored as domain-typed objects: frontmatter `type: FR`
plus an `object:` kind (`entity`, `process`, `configuration`, `interface`,
`data_schema`, `binary_format`, ...) with that kind's required anchor section in
the body (`## Properties` for entity, `## Workflow` mermaid for process,
`## Configuration` for configuration, `## Contract` yaml for interface).
`FR-061`..`FR-074` are the reference examples. One object per FR.

---
id: StR-003
title: Per-Agent Isolation via Partitioned HNSW
type: stakeholder-requirement
status: APPROVED
derived_usecases:
  - US-003
---
# StR-003: Per-Agent Isolation via Partitioned HNSW

## Need

The system serves 100K+ agents, each with isolated memory. Queries within one agent must not scan other agents' data, and HNSW indexes must support per-partition operation.

## Expectation

The HNSW index access method SHALL operate correctly on partitioned tables. A query scoped to a single partition SHALL traverse only that partition's index pages. The extension SHALL NOT require global coordination across partitions.

## Rationale

- The agent memory table is partitioned by `HASH(agent_id)` into 16 partitions
- Each within-agent query hits exactly one partition — cross-partition scan is unacceptable for latency
- Cross-agent queries fan out to all shards via the query router (not the extension's concern)

## Success Criteria

- HNSW index per partition operates independently
- INSERT/SCAN/VACUUM on one partition does not touch other partitions' index pages

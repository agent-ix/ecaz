# Benchmark Reporting Standard

This document defines how Ecaz reports measurements across access methods,
quantizers, storage formats, and option sets. It complements
[NFR-007](../spec/non-functional/NFR-007-benchmark-provenance.md), which covers
artifact provenance, and implements
[NFR-015](../spec/non-functional/NFR-015-benchmark-reporting-standard.md), which
defines the common reporting schema.

The standard applies to `ec_hnsw`, `ec_ivf`, `ec_diskann`, `ec_spire`, future
access methods, `turboquant`, `pq_fastscan`, `rabitq`, trained quantizers, and
future storage formats.

## Claim Classes

| Class | Meaning | Required evidence |
| --- | --- | --- |
| Local development evidence | Useful engineering result from a developer workstation or local PG cluster. | Packet-local artifacts or clearly identified historical/local source. |
| Review-packet evidence | Measurement used to justify a landed task or docs/spec claim. | `review/<topic>/artifacts/manifest.md` plus raw logs under the packet. |
| Product benchmark claim | User-facing performance claim for product comparison or scale planning. | Controlled hardware, cache state, PostgreSQL settings, commands, raw logs, and repeatability summary. |

Local and review-packet evidence must not be promoted to product claims without
a new product benchmark packet.

## Required Run Fields

Every benchmark row should preserve these fields, either in the table itself or
in the cited packet manifest:

| Field group | Required fields |
| --- | --- |
| Provenance | head SHA, packet, artifact path, command, timestamp, claim class |
| Environment | platform, CPU/architecture, OS, PostgreSQL version, feature flags, build profile, storage/cache state |
| Dataset | corpus name, source manifest, row count, query count, dimension, distance metric, normalization |
| Candidate identity | access method, opclass, storage format or quantizer, payload format, model metadata, reloptions, GUC overrides, rerank mode |
| Surface | isolated one-index-per-table or shared table, local or multicluster, forced or natural planner path |
| Quality | recall@10, recall@100, nDCG when emitted, exact-truth source |
| Latency | iteration count, p50, p95, p99, mean, cache state |
| Storage | index size, table size when relevant, per-row or per-vector bytes, codebook/model bytes, sidecar bytes |
| Memory | RSS, high-water mark, build memory, query memory when available |
| Mutation/maintenance | ingest rate, update/delete rate, vacuum time, cleanup debt, WAL or page churn when relevant |
| Distributed transport | topology, selected remotes, payload bytes, rows returned, connection/read/decode/merge timing |

If a field is not emitted by the current harness, the report should either mark
it as `not captured` or leave the row out of a standards-complete comparison.

## Candidate Comparison Matrix

Use this shape when comparing quantizers, storage formats, AMs, or option sets:

| AM | Candidate | Options | Dataset | Recall | Latency | Storage | Memory | Status | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `ec_ivf` | `pq_fastscan` g8 | `nlists`, `nprobe`, `rerank_width`, `pq_group_size` | corpus/query shape | recall@10/100 | p50/p95/p99 | index size | HWM/RSS | measured local/profile/product | packet |
| `ec_spire` | `rabitq` | topology, `nprobe`, local/remote settings | local or multicluster fixture | recall/quality if measured | pipeline/read timings | object/index bytes | HWM/RSS | serving-supported | packet |

Candidate `Status` values should be plain and narrow:

| Status | Meaning |
| --- | --- |
| `measured` | Current packet includes the metric family being cited. |
| `serving-supported` | The implementation can serve this candidate on the named path, but the row is not necessarily a latency or quality claim. |
| `recognized-deferred` | The implementation recognizes the candidate but cannot serve or measure it on this path yet. |
| `planned` | Candidate is a design target without packet-backed implementation evidence. |

## Current Storage Candidate Framing

RaBitQ is the first SPIRE remote-serving storage profile. Current SPIRE
endpoint readiness treats RaBitQ as the supported serving path for remote heap
candidate delivery; PQ-FastScan is recognized but still needs grouped-PQ model
metadata persistence and scorer readiness before it can be reported as a SPIRE
serving candidate.

For IVF, current packeted evidence shows:

- `turboquant`: conservative baseline with strong recall.
- `pq_fastscan` g8: best measured local speed/index-size candidate on the
  100K high-dimensional IVF lane.
- `rabitq`: high recall and lower memory high-water mark in the 10K/25K local
  checks, but current IVF scan latency is not competitive.

Future trained quantizers and storage formats should enter the docs only when
their candidate identity, model metadata, scorer status, and metric rows can be
reported through this standard.

## Updating Benchmark Docs

When adding a benchmark packet:

1. Store raw logs under `review/<topic>/artifacts/`.
2. Add or update `artifacts/manifest.md` with the required provenance,
   environment, candidate identity, command, and key result lines.
3. Update [Benchmark Index](benchmark-index.md) with the packet lane.
4. Update [Benchmarks](benchmarks.md) only for selected current rows.
5. Label gaps as gaps instead of inventing empty results.

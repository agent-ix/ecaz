---
artifact_type: adr-index
name: ecaz
status: IMPLEMENTED
---
# ADR Index

This index is the canonical navigation surface for Ecaz architecture decisions. Some historical ADR files share numeric IDs because they were added before the repository enforced unique ADR numbering. The files are preserved to avoid link churn; this index records the current interpretation.

## Current Implemented Decisions

| Canonical topic | File | Current status | Notes |
| --- | --- | --- | --- |
| Own quantizer | `ADR-006-own-quantizer.md` | DECIDED | In-tree TurboQuant-family implementation. |
| Query scoring and payload | `ADR-007-query-scoring-and-payload.md` | DECIDED | Gamma-aware raw-query path plus code-to-code path. |
| `ef_search` control surface | `ADR-016-ef-search-control-surface.md` | DECIDED | Session override over relation default. |
| PG18 primary target | `ADR-016-pg18-primary-target.md` | DECIDED | Historical duplicate ID; current PG18 target decision. |
| HNSW graph quality | `ADR-018-hnsw-quantized-graph-quality.md` | DECIDED | HNSW remains default graph AM. |
| Live insert lock ordering | `ADR-026-live-insert-backlink-lock-ordering.md` | ACCEPTED | HNSW insert lock ordering. |
| Vacuum graph repair lock ordering | `ADR-027-vacuum-graph-repair-lock-ordering.md` | ACCEPTED | HNSW vacuum repair ordering. |
| Module structure | `ADR-041-module-structure-for-multi-am-multi-quantizer-growth.md` | IMPLEMENTED | Current `src/am/{common,ec_hnsw,ec_ivf,ec_diskann,ec_spire}` layout. |
| Native HNSW build | `ADR-042-native-hnsw-build-path.md` | DECIDED | Production build path no longer depends on `hnsw_rs`. |
| Canonical `ecvector` row type | `ADR-043-native-ecvector-raw-f32-column-type.md` | IMPLEMENTED | `ecvector(dim)` is the canonical user row type. |
| Extension identity rename | `ADR-047-rename-extension-identity-to-ecaz.md` | DECIDED | Supersedes old single `tqvector` extension identity direction. |
| IVF access method | `ADR-048-ivf-access-method.md` | IMPLEMENTED | Historical duplicate ID; current IVF AM decision. |
| Parallel HNSW build graph assembly | `ADR-048-parallel-hnsw-build-graph-assembly.md` | DECIDED | Historical duplicate ID; current HNSW parallel build decision. |

## Current Optional or Deferred Decisions

| Topic | File | Current status | Notes |
| --- | --- | --- | --- |
| DiskANN second AM | `ADR-034-diskann-second-access-method.md` | IMPLEMENTED | Local v1 has landed; larger product-scale claims are deferred. |
| `ecvector` storage policy | `ADR-044-ecvector-rerank-source-location-and-storage-policy.md` | PROPOSED | Deferred measurement decision. |
| Graph page-layout discipline | `ADR-045-page-layout-discipline-for-graph-access-methods.md` | PROPOSED | Still useful design guidance. |
| SymphonyQG | `ADR-045-symphonyqg-quantized-graph-access-method.md` | SHELVED | Historical duplicate ID; RaBitQ survived independently, but the Symphony AM is not active roadmap work. |
| GPU offline trainer | `ADR-046-gpu-accelerated-offline-build-trainer.md` | PROPOSED | Future/offline optimization lane. |
| Vamana insert lock ordering | `ADR-046-vamana-insert-lock-ordering.md` | ACCEPTED | Historical duplicate ID; applies to `ec_diskann`. |
| Vamana vacuum lock ordering | `ADR-047-vamana-vacuum-lock-ordering.md` | ACCEPTED | Historical duplicate ID; applies to `ec_diskann`. |
| SPIRE partition-object IVF | `ADR-049-spire-on-single-level-ivf-foundation.md` | PROPOSED | Phase 0 chooses relation-backed PID objects, epoch manifests, index-local `vec_id`s, and Phase 1 `ec_spire`; preserve boundary replication, local multi-NVMe stores, future node placement, and epoch publication. |
| Configured benchmark suite runner | `ADR-050-configured-benchmark-suite-runner.md` | PROPOSED | Declarative long-running `ecaz bench suite` orchestration for index and architecture onboarding. |
| SPIRE multi-probe centroid scoring | `ADR-051-multi-probe-centroid-scoring-deferred.md` | DEFERRED | Deferred from Task 30 Phase 9 because anisotropic centroid scoring is expected to subsume the main benefit. |
| SPIRE learned NN-routing classifier | `ADR-052-learned-nn-routing-classifier-deferred.md` | DEFERRED | Deferred research track until drift, retraining, artifact, and eval-harness questions are answered. |
| SPIRE learned routing reranker | `ADR-053-routing-reranker-deferred.md` | DEFERRED | Deferred research track until deterministic Phase 9 routing-quality baselines and false-negative risk measurements exist. |
| SPIRE top-graph frontier contract | `ADR-054-spire-top-graph-frontier-contract.md` | ACCEPTED | Top graph nodes are the active root/top routing object's child frontier; root fanout, graph node count, and leaf count must be diagnosed separately. |
| SPIRE vector identity contract | `ADR-055-spire-vector-identity-contract.md` | ACCEPTED | Global `0x02` vec IDs dedupe across nodes; existing local `0x01` vec IDs are node-scoped during remote merge. |
| SPIRE eager bounded scan contract | `ADR-056-spire-eager-bounded-scan-contract.md` | ACCEPTED | Current AM scans materialize a bounded candidate cursor in `amrescan`; `amgettuple` remains forward-only cursor drain until a separate streaming ADR is accepted. |
| SPIRE local-store read scheduling contract | `ADR-057-spire-local-store-read-scheduling-contract.md` | ACCEPTED | Local store reads are grouped by `(node_id, local_store_id)` and prefetched before sequential scoring inside one backend; true parallel store execution requires a later ADR and benchmark packet. |
| SPIRE remote libpq executor boundary | `ADR-058-spire-remote-libpq-executor-boundary.md` | ACCEPTED | Current SQL-visible libpq executor remains diagnostic/operator-only; production remote AM execution still needs concurrent pipeline/async dispatch, cancellation, timeouts, identity validation, fail-closed behavior, and final row delivery. |
| SPIRE remote heap resolution contract | `ADR-059-spire-remote-heap-resolution-contract.md` | ACCEPTED | Remote heap resolution is origin-node owned; coordinator row locators stay opaque and production remote final rows remain blocked until origin-node heap visibility plus global vec-id allocation land. |
| Parallel index scan | `ADR-040-parallel-index-scan.md` | SHELVED | Not current scaling frontier; reopen only by new accepted ADR. |
| SPANN | `ADR-035-spann-billion-scale.md` | DROPPED | Dropped from active roadmap. |

## Superseded Historical Decisions

| File | Superseded by |
| --- | --- |
| `ADR-001-code-to-code-scoring.md` | ADR-006 |
| `ADR-002-hnsw-rs-no-delete.md` | ADR-042 and native page-owned AMs |
| `ADR-003-hnsw-rs-serialization.md` | ADR-042 |
| `ADR-005-turbocode-serialization.md` | ADR-006 and ADR-007 |
| `ADR-011-planner-cost-override-until-ordered-scan.md` | Live cost model and planner selection |
| `ADR-017-pg18-module-identity-and-upgrade-direction.md` | ADR-047 extension identity rename |
| `ADR-031-rabitq-binary-prefilter.md` | Landed first-class RaBitQ quantizer plus IVF `rabitq` storage/profile support |

## Numbering Policy

New ADRs SHALL use the next unused numeric identifier and SHALL NOT reuse an existing number. Historical duplicate IDs remain in place as legacy filenames. If a duplicate must become a frequent reference target, prefer linking this index row plus the filename rather than relying on the number alone.

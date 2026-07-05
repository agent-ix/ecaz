# Review Migration Manifest

Generated for the task-scoped review reorg. Moved packets use task-local sortable prefixes so directory order matches migration order (`00`, `01`, `02`, ...; widened when needed).

## Moved Buckets

- `task-05`: 181 entries
- `task-06`: 17 entries
- `task-07`: 13 entries
- `task-08`: 33 entries
- `task-10065`: 1 entries
- `task-11`: 52 entries
- `task-15`: 1 entries
- `task-16`: 130 entries
- `task-17`: 32 entries
- `task-25`: 18 entries
- `task-26`: 34 entries
- `task-28`: 142 entries
- `task-29`: 22 entries
- `task-29a`: 1 entries
- `task-29b`: 1 entries
- `task-29d`: 3 entries
- `task-29e`: 1 entries
- `task-30`: 955 entries
- `task-31`: 38 entries
- `task-32`: 1 entries
- `task-34`: 1 entries
- `task-36`: 1 entries
- `task-42`: 18 entries
- `task-49`: 1 entries
- `task-archive-early-hnsw`: 122 entries

## Skipped Legacy Entries

These were left under `review/` by the first pass for focused follow-up.

- `doc`: 2 entries
- `skip-benchmark`: 115 entries
- `skip-task41`: 77 entries

## Follow-up Benchmark Packet Migration

The remaining non-Task-41 flat packets have now been moved from `review/` to
top-level `benchmarks/` packet directories, preserving their original packet
slugs. Legacy `review/` is now reserved for deferred Task 41 packets only.

## Mapping

| Old path | New path | Bucket | Rule |
|---|---|---|---|
| `review/06-scan-descriptor-scaffolding` | `reviews/task-05/001-06-scan-descriptor-scaffolding` | `task-05` | `scan` |
| `review/07-rescan-query-validation` | `reviews/task-05/002-07-rescan-query-validation` | `task-05` | `scan` |
| `review/08-amgettuple-state-gating` | `reviews/task-05/003-08-amgettuple-state-gating` | `task-05` | `scan` |
| `review/09-rescan-defensive-cases` | `reviews/task-05/004-09-rescan-defensive-cases` | `task-05` | `scan` |
| `review/11-scan-lifecycle-idempotency` | `reviews/task-05/005-11-scan-lifecycle-idempotency` | `task-05` | `scan` |
| `review/13-amgettuple-empty-index-noop` | `reviews/task-05/006-13-amgettuple-empty-index-noop` | `task-05` | `scan` |
| `review/14-rescan-query-payload-state` | `reviews/task-05/007-14-rescan-query-payload-state` | `task-05` | `scan` |
| `review/15-amgettuple-linear-forward-scan` | `reviews/task-05/008-15-amgettuple-linear-forward-scan` | `task-05` | `scan` |
| `review/17-linear-scan-exhaustion-and-direction-guards` | `reviews/task-05/009-17-linear-scan-exhaustion-and-direction-guards` | `task-05` | `scan` |
| `review/18-linear-scan-rescan-after-exhaustion` | `reviews/task-05/010-18-linear-scan-rescan-after-exhaustion` | `task-05` | `scan` |
| `review/23-scan-metadata-cache` | `reviews/task-05/011-23-scan-metadata-cache` | `task-05` | `scan` |
| `review/26-scan-prepared-query-cache` | `reviews/task-05/012-26-scan-prepared-query-cache` | `task-05` | `scan` |
| `review/27-scan-current-result-state` | `reviews/task-05/013-27-scan-current-result-state` | `task-05` | `scan` |
| `review/28-scan-current-result-lifecycle` | `reviews/task-05/014-28-scan-current-result-lifecycle` | `task-05` | `scan` |
| `review/35-am-build-tuple-and-source-scan-split` | `reviews/task-05/015-35-am-build-tuple-and-source-scan-split` | `task-05` | `scan` |
| `review/36-am-build-graph-and-page-staging-split` | `reviews/task-05/016-36-am-build-graph-and-page-staging-split` | `task-05` | `scan` |
| `review/38-scan-current-result-scoring` | `reviews/task-05/017-38-scan-current-result-scoring` | `task-05` | `scan` |
| `review/39-am-scan-module-boundary` | `reviews/task-05/018-39-am-scan-module-boundary` | `task-05` | `scan` |
| `review/41-scan-page-level-neighbor-access` | `reviews/task-05/019-41-scan-page-level-neighbor-access` | `task-05` | `scan` |
| `review/42-graph-read-surface-and-explicit-scan-result` | `reviews/task-05/020-42-graph-read-surface-and-explicit-scan-result` | `task-05` | `scan` |
| `review/43-scan-entry-candidate-state` | `reviews/task-05/021-43-scan-entry-candidate-state` | `task-05` | `scan` |
| `review/44-entry-candidate-lifecycle` | `reviews/task-05/022-44-entry-candidate-lifecycle` | `task-05` | `scan` |
| `review/45-successor-candidate-seeding` | `reviews/task-05/023-45-successor-candidate-seeding` | `task-05` | `scan` |
| `review/46-two-slot-candidate-frontier` | `reviews/task-05/024-46-two-slot-candidate-frontier` | `task-05` | `scan` |
| `review/47-two-slot-frontier-head-ordering` | `reviews/task-05/025-47-two-slot-frontier-head-ordering` | `task-05` | `scan` |
| `review/48-frontier-head-lifecycle` | `reviews/task-05/026-48-frontier-head-lifecycle` | `task-05` | `scan` |
| `review/49-frontier-head-consumption` | `reviews/task-05/027-49-frontier-head-consumption` | `task-05` | `scan` |
| `review/50-skip-invalid-successor-neighbor-refs` | `reviews/task-05/028-50-skip-invalid-successor-neighbor-refs` | `task-05` | `scan` |
| `review/53-scan-visited-seed-state` | `reviews/task-05/029-53-scan-visited-seed-state` | `task-05` | `scan` |
| `review/54-vector-backed-candidate-frontier` | `reviews/task-05/030-54-vector-backed-candidate-frontier` | `task-05` | `scan` |
| `review/55-frontier-head-option-index` | `reviews/task-05/031-55-frontier-head-option-index` | `task-05` | `scan` |
| `review/56-wider-bootstrap-frontier-seeding` | `reviews/task-05/032-56-wider-bootstrap-frontier-seeding` | `task-05` | `scan` |
| `review/57-bootstrap-frontier-refill-after-consume` | `reviews/task-05/033-57-bootstrap-frontier-refill-after-consume` | `task-05` | `scan` |
| `review/60-scan-candidate-provenance` | `reviews/task-05/034-60-scan-candidate-provenance` | `task-05` | `scan` |
| `review/61-bootstrap-frontier-multihop-fill` | `reviews/task-05/035-61-bootstrap-frontier-multihop-fill` | `task-05` | `scan` |
| `review/62-bootstrap-expansion-policy-seam` | `reviews/task-05/036-62-bootstrap-expansion-policy-seam` | `task-05` | `scan` |
| `review/64-bootstrap-expanded-state-groundwork` | `reviews/task-05/037-64-bootstrap-expanded-state-groundwork` | `task-05` | `scan` |
| `review/65-scan-owned-bootstrap-expanded-state` | `reviews/task-05/038-65-scan-owned-bootstrap-expanded-state` | `task-05` | `scan` |
| `review/66-bootstrap-candidate-consumption-state` | `reviews/task-05/039-66-bootstrap-candidate-consumption-state` | `task-05` | `scan` |
| `review/67-active-candidate-pending-drain-bridge` | `reviews/task-05/040-67-active-candidate-pending-drain-bridge` | `task-05` | `scan` |
| `review/68-visible-active-candidate-scan-bridge` | `reviews/task-05/041-68-visible-active-candidate-scan-bridge` | `task-05` | `scan` |
| `review/69-active-candidate-first-visible-results` | `reviews/task-05/042-69-active-candidate-first-visible-results` | `task-05` | `scan` |
| `review/70-bootstrap-frontier-top-up-after-consume` | `reviews/task-05/043-70-bootstrap-frontier-top-up-after-consume` | `task-05` | `scan` |
| `review/71-skip-reexpanding-consumed-bootstrap-sources` | `reviews/task-05/044-71-skip-reexpanding-consumed-bootstrap-sources` | `task-05` | `scan` |
| `review/72-direct-frontier-result-materialization` | `reviews/task-05/045-72-direct-frontier-result-materialization` | `task-05` | `scan` |
| `review/73-search-module-boundary` | `reviews/task-05/046-73-search-module-boundary` | `task-05` | `scan` |
| `review/76-scan-debug-module-boundary` | `reviews/task-05/047-76-scan-debug-module-boundary` | `task-05` | `scan` |
| `review/77-ef-search-and-search-api-groundwork` | `reviews/task-05/048-77-ef-search-and-search-api-groundwork` | `task-05` | `scan` |
| `review/78-incremental-bootstrap-top-up-scheduler` | `reviews/task-05/049-78-incremental-bootstrap-top-up-scheduler` | `task-05` | `scan` |
| `review/79-scan-owned-bootstrap-expansion-scheduler` | `reviews/task-05/050-79-scan-owned-bootstrap-expansion-scheduler` | `task-05` | `scan` |
| `review/80-forget-consumed-frontier-nodes` | `reviews/task-05/051-80-forget-consumed-frontier-nodes` | `task-05` | `scan` |
| `review/81-beam-owned-frontier-head-selection` | `reviews/task-05/052-81-beam-owned-frontier-head-selection` | `task-05` | `scan` |
| `review/82-scheduler-first-frontier-consume` | `reviews/task-05/053-82-scheduler-first-frontier-consume` | `task-05` | `scan` |
| `review/83-direct-discovered-candidate-beam-seeding` | `reviews/task-05/054-83-direct-discovered-candidate-beam-seeding` | `task-05` | `scan` |
| `review/84-unified-initial-frontier-seeding` | `reviews/task-05/055-84-unified-initial-frontier-seeding` | `task-05` | `scan` |
| `review/87-derived-frontier-head-state` | `reviews/task-05/056-87-derived-frontier-head-state` | `task-05` | `scan` |
| `review/89-frontier-head-tid-surface` | `reviews/task-05/057-89-frontier-head-tid-surface` | `task-05` | `scan` |
| `review/90-localized-frontier-node-lookup` | `reviews/task-05/058-90-localized-frontier-node-lookup` | `task-05` | `scan` |
| `review/91-visible-frontier-container-seam` | `reviews/task-05/059-91-visible-frontier-container-seam` | `task-05` | `scan` |
| `review/92-visible-frontier-read-seam` | `reviews/task-05/060-92-visible-frontier-read-seam` | `task-05` | `scan` |
| `review/93-visible-frontier-iteration-seam` | `reviews/task-05/061-93-visible-frontier-iteration-seam` | `task-05` | `scan` |
| `review/94-private-frontier-slice-boundary` | `reviews/task-05/062-94-private-frontier-slice-boundary` | `task-05` | `scan` |
| `review/95-owned-visible-frontier-state` | `reviews/task-05/063-95-owned-visible-frontier-state` | `task-05` | `scan` |
| `review/96-frontier-behavior-on-owned-state` | `reviews/task-05/064-96-frontier-behavior-on-owned-state` | `task-05` | `scan` |
| `review/97-search-owned-stale-frontier-cleanup` | `reviews/task-05/065-97-search-owned-stale-frontier-cleanup` | `task-05` | `scan` |
| `review/98-search-owned-scheduler-consume` | `reviews/task-05/066-98-search-owned-scheduler-consume` | `task-05` | `scan` |
| `review/99-visible-frontier-beam-candidate-storage` | `reviews/task-05/067-99-visible-frontier-beam-candidate-storage` | `task-05` | `scan` |
| `review/100-beam-candidates-through-runtime-seeding` | `reviews/task-05/068-100-beam-candidates-through-runtime-seeding` | `task-05` | `scan` |
| `review/101-beam-candidates-through-runtime-consume` | `reviews/task-05/069-101-beam-candidates-through-runtime-consume` | `task-05` | `scan` |
| `review/102-queued-beam-candidate-lookup` | `reviews/task-05/070-102-queued-beam-candidate-lookup` | `task-05` | `scan` |
| `review/103-visible-frontier-beam-iteration` | `reviews/task-05/071-103-visible-frontier-beam-iteration` | `task-05` | `scan` |
| `review/104-active-candidate-beam-state` | `reviews/task-05/072-104-active-candidate-beam-state` | `task-05` | `scan` |
| `review/105-frontier-remove-node-beam-return` | `reviews/task-05/073-105-frontier-remove-node-beam-return` | `task-05` | `scan` |
| `review/106-beam-native-frontier-head-selection` | `reviews/task-05/074-106-beam-native-frontier-head-selection` | `task-05` | `scan` |
| `review/107-remove-redundant-scan-beam-adapters` | `reviews/task-05/075-107-remove-redundant-scan-beam-adapters` | `task-05` | `scan` |
| `review/108-remove-test-only-frontier-alias` | `reviews/task-05/076-108-remove-test-only-frontier-alias` | `task-05` | `scan` |
| `review/109-beam-native-frontier-debug-boundaries` | `reviews/task-05/077-109-beam-native-frontier-debug-boundaries` | `task-05` | `scan` |
| `review/110-beam-native-scan-test-fixtures` | `reviews/task-05/078-110-beam-native-scan-test-fixtures` | `task-05` | `scan` |
| `review/111-remove-dead-scan-candidate-boundary` | `reviews/task-05/079-111-remove-dead-scan-candidate-boundary` | `task-05` | `scan` |
| `review/112-remove-dead-queued-beam-lookup` | `reviews/task-05/080-112-remove-dead-queued-beam-lookup` | `task-05` | `scan` |
| `review/113-remove-stale-bootstrap-candidate-staging` | `reviews/task-05/081-113-remove-stale-bootstrap-candidate-staging` | `task-05` | `scan` |
| `review/114-remove-dead-bootstrap-expand-helper` | `reviews/task-05/082-114-remove-dead-bootstrap-expand-helper` | `task-05` | `scan` |
| `review/115-gate-debug-only-frontier-helpers` | `reviews/task-05/083-115-gate-debug-only-frontier-helpers` | `task-05` | `scan` |
| `review/118-remove-stale-debug-frontier-alias` | `reviews/task-05/084-118-remove-stale-debug-frontier-alias` | `task-05` | `scan` |
| `review/119-scan-unsafe-audit-cleanup` | `reviews/task-05/085-119-scan-unsafe-audit-cleanup` | `task-05` | `scan` |
| `review/122-bootstrap-frontier-ef-search-limit` | `reviews/task-05/086-122-bootstrap-frontier-ef-search-limit` | `task-05` | `scan` |
| `review/123-skip-stale-bootstrap-candidates` | `reviews/task-05/087-123-skip-stale-bootstrap-candidates` | `task-05` | `scan` |
| `review/124-frontier-fallback-after-scheduler-drain` | `reviews/task-05/088-124-frontier-fallback-after-scheduler-drain` | `task-05` | `scan` |
| `review/125-defer-bootstrap-refill-until-adjudication` | `reviews/task-05/089-125-defer-bootstrap-refill-until-adjudication` | `task-05` | `scan` |
| `review/126-align-debug-bootstrap-materialization-with-runtime` | `reviews/task-05/090-126-align-debug-bootstrap-materialization-with-runtime` | `task-05` | `scan` |
| `review/127-explicit-bootstrap-to-linear-transition` | `reviews/task-05/091-127-explicit-bootstrap-to-linear-transition` | `task-05` | `scan` |
| `review/128-bootstrap-phase-debug-transition` | `reviews/task-05/092-128-bootstrap-phase-debug-transition` | `task-05` | `scan` |
| `review/129-pending-result-drain-before-fallback` | `reviews/task-05/093-129-pending-result-drain-before-fallback` | `task-05` | `scan` |
| `review/133-shared-scan-result-materialization-state` | `reviews/task-05/094-133-shared-scan-result-materialization-state` | `task-05` | `scan` |
| `review/134-explicit-scan-execution-phase` | `reviews/task-05/095-134-explicit-scan-execution-phase` | `task-05` | `scan` |
| `review/135-extract-staged-scan-tuple-production-helper` | `reviews/task-05/096-135-extract-staged-scan-tuple-production-helper` | `task-05` | `scan` |
| `review/136-unified-scan-result-state-clearing` | `reviews/task-05/097-136-unified-scan-result-state-clearing` | `task-05` | `scan` |
| `review/137-scan-result-state-container` | `reviews/task-05/098-137-scan-result-state-container` | `task-05` | `scan` |
| `review/139-bootstrap-selection-before-materialization` | `reviews/task-05/099-139-bootstrap-selection-before-materialization` | `task-05` | `scan` |
| `review/143-trim-legacy-bootstrap-materialization-surface` | `reviews/task-05/100-143-trim-legacy-bootstrap-materialization-surface` | `task-05` | `scan` |
| `review/145-explicit-bootstrap-completion-at-phase-dispatch` | `reviews/task-05/101-145-explicit-bootstrap-completion-at-phase-dispatch` | `task-05` | `scan` |
| `review/146-gate-test-only-bootstrap-selection-helper` | `reviews/task-05/102-146-gate-test-only-bootstrap-selection-helper` | `task-05` | `scan` |
| `review/148-gate-frontier-head-candidate-accessor` | `reviews/task-05/103-148-gate-frontier-head-candidate-accessor` | `task-05` | `scan` |
| `review/154-overall-scan-graph-architecture` | `reviews/task-05/104-154-overall-scan-graph-architecture` | `task-05` | `scan` |
| `review/155-move-scan-query-debug-readout-into-scan-debug` | `reviews/task-05/105-155-move-scan-query-debug-readout-into-scan-debug` | `task-05` | `scan` |
| `review/156-remove-redundant-frontier-head-tid-helper` | `reviews/task-05/106-156-remove-redundant-frontier-head-tid-helper` | `task-05` | `scan` |
| `review/158-graph-owned-layer0-neighbor-loader` | `reviews/task-05/107-158-graph-owned-layer0-neighbor-loader` | `task-05` | `scan` |
| `review/159-graph-owned-layer0-successor-candidates` | `reviews/task-05/108-159-graph-owned-layer0-successor-candidates` | `task-05` | `scan` |
| `review/160-graph-owned-layer0-beam-search-runner` | `reviews/task-05/109-160-graph-owned-layer0-beam-search-runner` | `task-05` | `scan` |
| `review/161-seed-bootstrap-entry-frontier-from-layer0-beam-trace` | `reviews/task-05/110-161-seed-bootstrap-entry-frontier-from-layer0-beam-trace` | `task-05` | `scan` |
| `review/162-refill-bootstrap-frontier-from-layer0-beam-trace` | `reviews/task-05/111-162-refill-bootstrap-frontier-from-layer0-beam-trace` | `task-05` | `scan` |
| `review/163-seed-bootstrap-frontier-directly-from-layer0-beam-trace` | `reviews/task-05/112-163-seed-bootstrap-frontier-directly-from-layer0-beam-trace` | `task-05` | `scan` |
| `review/164-unify-entry-seeding-on-layer0-beam-trace` | `reviews/task-05/113-164-unify-entry-seeding-on-layer0-beam-trace` | `task-05` | `scan` |
| `review/165-beam-driven-bootstrap-top-up-from-visible-frontier` | `reviews/task-05/114-165-beam-driven-bootstrap-top-up-from-visible-frontier` | `task-05` | `scan` |
| `review/166-graph-beam-search-batch-review` | `reviews/task-05/115-166-graph-beam-search-batch-review` | `task-05` | `scan` |
| `review/169-resolved-ef-search-runtime-bootstrap` | `reviews/task-05/116-169-resolved-ef-search-runtime-bootstrap` | `task-05` | `scan` |
| `review/170-search-seam-batch-review` | `reviews/task-05/117-170-search-seam-batch-review` | `task-05` | `scan` |
| `review/171-amrescan-prefill-graph-result` | `reviews/task-05/118-171-amrescan-prefill-graph-result` | `task-05` | `scan` |
| `review/172-graph-prefetch-after-duplicate-drain` | `reviews/task-05/119-172-graph-prefetch-after-duplicate-drain` | `task-05` | `scan` |
| `review/173-graph-phase-prefetched-cursor` | `reviews/task-05/120-173-graph-phase-prefetched-cursor` | `task-05` | `scan` |
| `review/174-phase-local-fallback-output` | `reviews/task-05/121-174-phase-local-fallback-output` | `task-05` | `scan` |
| `review/175-phase-local-fallback-materialize-emit` | `reviews/task-05/122-175-phase-local-fallback-materialize-emit` | `task-05` | `scan` |
| `review/176-graph-phase-on-demand-prefill` | `reviews/task-05/123-176-graph-phase-on-demand-prefill` | `task-05` | `scan` |
| `review/177-graph-state-driven-fallback-gate` | `reviews/task-05/124-177-graph-state-driven-fallback-gate` | `task-05` | `scan` |
| `review/178-materialized-graph-fallback-gate` | `reviews/task-05/125-178-materialized-graph-fallback-gate` | `task-05` | `scan` |
| `review/179-graph-phase-emit-boundary` | `reviews/task-05/126-179-graph-phase-emit-boundary` | `task-05` | `scan` |
| `review/180-graph-prefetched-hot-path` | `reviews/task-05/127-180-graph-prefetched-hot-path` | `task-05` | `scan` |
| `review/181-graph-prefetch-refresh-boundary` | `reviews/task-05/128-181-graph-prefetch-refresh-boundary` | `task-05` | `scan` |
| `review/182-graph-cursor-state-seam` | `reviews/task-05/129-182-graph-cursor-state-seam` | `task-05` | `scan` |
| `review/183-separate-fallback-result-state` | `reviews/task-05/130-183-separate-fallback-result-state` | `task-05` | `scan` |
| `review/185-linear-fallback-cursor-state-seam` | `reviews/task-05/131-185-linear-fallback-cursor-state-seam` | `task-05` | `scan` |
| `review/186-cursor-owned-output-emission` | `reviews/task-05/132-186-cursor-owned-output-emission` | `task-05` | `scan` |
| `review/187-graph-cursor-prefetch-readiness` | `reviews/task-05/133-187-graph-cursor-prefetch-readiness` | `task-05` | `scan` |
| `review/188-graph-prefetch-materialization-boundary` | `reviews/task-05/134-188-graph-prefetch-materialization-boundary` | `task-05` | `scan` |
| `review/189-remove-bootstrap-prefetch-wrapper` | `reviews/task-05/135-189-remove-bootstrap-prefetch-wrapper` | `task-05` | `scan` |
| `review/190-direct-graph-prefetch-materialization` | `reviews/task-05/136-190-direct-graph-prefetch-materialization` | `task-05` | `scan` |
| `review/191-graph-prefetch-context-packaging` | `reviews/task-05/137-191-graph-prefetch-context-packaging` | `task-05` | `scan` |
| `review/192-cursor-owned-graph-prefetch` | `reviews/task-05/138-192-cursor-owned-graph-prefetch` | `task-05` | `scan` |
| `review/193-gate-bootstrap-debug-helpers` | `reviews/task-05/139-193-gate-bootstrap-debug-helpers` | `task-05` | `scan` |
| `review/196-a4-neighbor-slot-packing-audit` | `reviews/task-05/140-196-a4-neighbor-slot-packing-audit` | `task-05` | `scan` |
| `review/214-a4-hnsw-rs-code-graph-baseline` | `reviews/task-05/141-214-a4-hnsw-rs-code-graph-baseline` | `task-05` | `scan` |
| `review/215-a4-source-graph-reference-baseline` | `reviews/task-05/142-215-a4-source-graph-reference-baseline` | `task-05` | `scan` |
| `review/248-c1-ordered-scan-runtime-fix` | `reviews/task-05/143-248-c1-ordered-scan-runtime-fix` | `task-05` | `scan` |
| `review/251-c1-scan-runtime-profiling` | `reviews/task-05/144-251-c1-scan-runtime-profiling` | `task-05` | `scan` |
| `review/252-c1-scan-graph-read-cache` | `reviews/task-05/145-252-c1-scan-graph-read-cache` | `task-05` | `scan` |
| `review/253-c1-scan-cpu-hot-path-breakdown` | `reviews/task-05/146-253-c1-scan-cpu-hot-path-breakdown` | `task-05` | `scan` |
| `review/255-c1-scan-fast-hash-state` | `reviews/task-05/147-255-c1-scan-fast-hash-state` | `task-05` | `scan` |
| `review/258-c1-layer0-search-bookkeeping-breakdown` | `reviews/task-05/148-258-c1-layer0-search-bookkeeping-breakdown` | `task-05` | `scan` |
| `review/262-c1-graph-tuple-copy-boundary` | `reviews/task-05/149-262-c1-graph-tuple-copy-boundary` | `task-05` | `scan` |
| `review/263-c1-graph-direct-decode` | `reviews/task-05/150-263-c1-graph-direct-decode` | `task-05` | `scan` |
| `review/269-c1-beam-search-lazy-queued-removal` | `reviews/task-05/151-269-c1-beam-search-lazy-queued-removal` | `task-05` | `scan` |
| `review/270-c1-layer-neighbor-iteration-no-temp-vec` | `reviews/task-05/152-270-c1-layer-neighbor-iteration-no-temp-vec` | `task-05` | `scan` |
| `review/271-c1-inline-successor-candidate-buffers` | `reviews/task-05/153-271-c1-inline-successor-candidate-buffers` | `task-05` | `scan` |
| `review/278-c1-scan-cache-code-payload-elision` | `reviews/task-05/154-278-c1-scan-cache-code-payload-elision` | `task-05` | `scan` |
| `review/287-c1-adr031-tier1-inline-scan-cache` | `reviews/task-05/155-287-c1-adr031-tier1-inline-scan-cache` | `task-05` | `scan` |
| `review/288-c1-adr031-tier1-high-ef-frontier` | `reviews/task-05/156-288-c1-adr031-tier1-high-ef-frontier` | `task-05` | `scan` |
| `review/10026-visible-frontier-search-seam` | `reviews/task-05/157-10026-visible-frontier-search-seam` | `task-05` | `scan` |
| `review/10027-visible-frontier-progression-search-seam` | `reviews/task-05/158-10027-visible-frontier-progression-search-seam` | `task-05` | `scan` |
| `review/10028-layer0-top-up-graph-seam` | `reviews/task-05/159-10028-layer0-top-up-graph-seam` | `task-05` | `scan` |
| `review/10029-bootstrap-selection-search-seam` | `reviews/task-05/160-10029-bootstrap-selection-search-seam` | `task-05` | `scan` |
| `review/10030-discovered-candidate-search-seam` | `reviews/task-05/161-10030-discovered-candidate-search-seam` | `task-05` | `scan` |
| `review/10031-bootstrap-trace-search-seam` | `reviews/task-05/162-10031-bootstrap-trace-search-seam` | `task-05` | `scan` |
| `review/10032-visible-seed-top-up-search-seam` | `reviews/task-05/163-10032-visible-seed-top-up-search-seam` | `task-05` | `scan` |
| `review/10033-refill-source-search-seam` | `reviews/task-05/164-10033-refill-source-search-seam` | `task-05` | `scan` |
| `review/10034-ef-search-guc-sentinel-fix` | `reviews/task-05/165-10034-ef-search-guc-sentinel-fix` | `task-05` | `scan` |
| `review/10036-scan-test-only-gating-and-plan-status` | `reviews/task-05/166-10036-scan-test-only-gating-and-plan-status` | `task-05` | `scan` |
| `review/10037-graph-first-primary-scan` | `reviews/task-05/167-10037-graph-first-primary-scan` | `task-05` | `scan` |
| `review/10038-unseeded-graph-linear-fallback` | `reviews/task-05/168-10038-unseeded-graph-linear-fallback` | `task-05` | `scan` |
| `review/10039-explicit-graph-fallback-phases` | `reviews/task-05/169-10039-explicit-graph-fallback-phases` | `task-05` | `scan` |
| `review/10040-graph-first-exhaustion-test-contract` | `reviews/task-05/170-10040-graph-first-exhaustion-test-contract` | `task-05` | `scan` |
| `review/10041-phase-dispatched-scan-production` | `reviews/task-05/171-10041-phase-dispatched-scan-production` | `task-05` | `scan` |
| `review/10042-graph-first-rescan-exhaustion-test-contract` | `reviews/task-05/172-10042-graph-first-rescan-exhaustion-test-contract` | `task-05` | `scan` |
| `review/10043-pending-scan-output-seam` | `reviews/task-05/173-10043-pending-scan-output-seam` | `task-05` | `scan` |
| `review/10044-build-element-neighbor-locality` | `reviews/task-05/174-10044-build-element-neighbor-locality` | `task-05` | `scan` |
| `review/10047-neighbor-count-dead-metadata` | `reviews/task-05/175-10047-neighbor-count-dead-metadata` | `task-05` | `scan` |
| `review/11022-phase5d-persisted-graph-reader` | `reviews/task-05/176-11022-phase5d-persisted-graph-reader` | `task-05` | `scan` |
| `review/11023-phase6a-scan-algorithm-shell` | `reviews/task-05/177-11023-phase6a-scan-algorithm-shell` | `task-05` | `scan` |
| `review/11028-scan-entry-point-resolver` | `reviews/task-05/178-11028-scan-entry-point-resolver` | `task-05` | `scan` |
| `review/11031-phase6b1-scan-query-primitives` | `reviews/task-05/179-11031-phase6b1-scan-query-primitives` | `task-05` | `scan` |
| `review/11032-phase6b2-pgrx-scan-callback-wiring` | `reviews/task-05/180-11032-phase6b2-pgrx-scan-callback-wiring` | `task-05` | `scan` |
| `review/11041-phase7i-duplicate-bind-overflow-scan-expansion` | `reviews/task-05/181-11041-phase7i-duplicate-bind-overflow-scan-expansion` | `task-05` | `scan` |
| `review/01-aminsert-groundwork` | `reviews/task-06/001-01-aminsert-groundwork` | `task-06` | `insert` |
| `review/04-build-source-live-insert-rejection` | `reviews/task-06/002-04-build-source-live-insert-rejection` | `task-06` | `insert` |
| `review/74-insert-module-boundary` | `reviews/task-06/003-74-insert-module-boundary` | `task-06` | `insert` |
| `review/199-aminsert-graph-aware-insertion-roadmap` | `reviews/task-06/004-199-aminsert-graph-aware-insertion-roadmap` | `task-06` | `insert` |
| `review/229-a5-insert-level-allocation-promotion` | `reviews/task-06/005-229-a5-insert-level-allocation-promotion` | `task-06` | `insert` |
| `review/230-a5-forward-links-on-new-node` | `reviews/task-06/006-230-a5-forward-links-on-new-node` | `task-06` | `insert` |
| `review/231-a5-layer0-backlinks-lock-ordering` | `reviews/task-06/007-231-a5-layer0-backlinks-lock-ordering` | `task-06` | `insert` |
| `review/232-a5-upper-layer-insert-links` | `reviews/task-06/008-232-a5-upper-layer-insert-links` | `task-06` | `insert` |
| `review/233-a5-overflow-backlink-pruning` | `reviews/task-06/009-233-a5-overflow-backlink-pruning` | `task-06` | `insert` |
| `review/234-a5-insert-drift-accounting` | `reviews/task-06/010-234-a5-insert-drift-accounting` | `task-06` | `insert` |
| `review/235-a5-concurrency-retry-hardening` | `reviews/task-06/011-235-a5-concurrency-retry-hardening` | `task-06` | `insert` |
| `review/239-a5-review-followups` | `reviews/task-06/012-239-a5-review-followups` | `task-06` | `insert` |
| `review/10045-insert-metadata-lock-scope` | `reviews/task-06/013-10045-insert-metadata-lock-scope` | `task-06` | `insert` |
| `review/11033-phase7a-insert-payload-derivation` | `reviews/task-06/014-11033-phase7a-insert-payload-derivation` | `task-06` | `insert` |
| `review/11034-phase7b-empty-index-bootstrap-insert` | `reviews/task-06/015-11034-phase7b-empty-index-bootstrap-insert` | `task-06` | `insert` |
| `review/11036-phase7d-unique-insert-forward-planning-boundary` | `reviews/task-06/016-11036-phase7d-unique-insert-forward-planning-boundary` | `task-06` | `insert` |
| `review/11039-phase7g-insert-metadata-bookkeeping` | `reviews/task-06/017-11039-phase7g-insert-metadata-bookkeeping` | `task-06` | `insert` |
| `review/05-vacuum-noop-callbacks` | `reviews/task-07/001-05-vacuum-noop-callbacks` | `task-07` | `vacuum` |
| `review/10-vacuum-noop-coverage` | `reviews/task-07/002-10-vacuum-noop-coverage` | `task-07` | `vacuum` |
| `review/236-a6-vacuum-pass1-mark` | `reviews/task-07/003-236-a6-vacuum-pass1-mark` | `task-07` | `vacuum` |
| `review/237-a6-vacuum-finalize-duplicate-guard` | `reviews/task-07/004-237-a6-vacuum-finalize-duplicate-guard` | `task-07` | `vacuum` |
| `review/238-a6-vacuum-dead-edge-unlink` | `reviews/task-07/005-238-a6-vacuum-dead-edge-unlink` | `task-07` | `vacuum` |
| `review/240-a6-layer0-replacement-fill` | `reviews/task-07/006-240-a6-layer0-replacement-fill` | `task-07` | `vacuum` |
| `review/241-a6-upper-layer-replacement-fill` | `reviews/task-07/007-241-a6-upper-layer-replacement-fill` | `task-07` | `vacuum` |
| `review/242-a6-vacuum-concurrency-validation` | `reviews/task-07/008-242-a6-vacuum-concurrency-validation` | `task-07` | `vacuum` |
| `review/243-a5-a6-review-followups` | `reviews/task-07/009-243-a5-a6-review-followups` | `task-07` | `vacuum` |
| `review/11021-phase8a-vacuum-primitives` | `reviews/task-07/010-11021-phase8a-vacuum-primitives` | `task-07` | `vacuum` |
| `review/11042-phase8b1-vacuum-callback-strip-unlink-finalize` | `reviews/task-07/011-11042-phase8b1-vacuum-callback-strip-unlink-finalize` | `task-07` | `vacuum` |
| `review/11043-phase8b2-vacuum-fill-only-repair` | `reviews/task-07/012-11043-phase8b2-vacuum-fill-only-repair` | `task-07` | `vacuum` |
| `review/11044-phase8b3-vacuum-stale-rewrite-replan` | `reviews/task-07/013-11044-phase8b3-vacuum-stale-rewrite-replan` | `task-07` | `vacuum` |
| `review/16-query-inner-product-gamma-payload` | `reviews/task-08/001-16-query-inner-product-gamma-payload` | `task-08` | `simd-quantizer` |
| `review/20-encode-dimension-boundary` | `reviews/task-08/002-20-encode-dimension-boundary` | `task-08` | `simd-quantizer` |
| `review/29-gamma-aware-duplicate-coalescing` | `reviews/task-08/003-29-gamma-aware-duplicate-coalescing` | `task-08` | `simd-quantizer` |
| `review/51-persist-gamma-in-element-tuples` | `reviews/task-08/004-51-persist-gamma-in-element-tuples` | `task-08` | `simd-quantizer` |
| `review/52-persisted-gamma-hot-path-cutover` | `reviews/task-08/005-52-persisted-gamma-hot-path-cutover` | `task-08` | `simd-quantizer` |
| `review/63-score-ordered-bootstrap-expansion` | `reviews/task-08/006-63-score-ordered-bootstrap-expansion` | `task-08` | `simd-quantizer` |
| `review/117-scan-quantizer-hot-path-cache` | `reviews/task-08/007-117-scan-quantizer-hot-path-cache` | `task-08` | `simd-quantizer` |
| `review/120-amgettuple-orderby-score-emission` | `reviews/task-08/008-120-amgettuple-orderby-score-emission` | `task-08` | `simd-quantizer` |
| `review/121-amgettuple-orderby-score-lifecycle` | `reviews/task-08/009-121-amgettuple-orderby-score-lifecycle` | `task-08` | `simd-quantizer` |
| `review/195-a4-score-function-divergence` | `reviews/task-08/010-195-a4-score-function-divergence` | `task-08` | `simd-quantizer` |
| `review/197-a4-gamma-in-build-vs-search` | `reviews/task-08/011-197-a4-gamma-in-build-vs-search` | `task-08` | `simd-quantizer` |
| `review/201-a4-quantizer-triage` | `reviews/task-08/012-201-a4-quantizer-triage` | `task-08` | `simd-quantizer` |
| `review/203-a4-1536-tiled-fwht-quantizer` | `reviews/task-08/013-203-a4-1536-tiled-fwht-quantizer` | `task-08` | `simd-quantizer` |
| `review/205-a4-quantizer-impl-mismatch-audit` | `reviews/task-08/014-205-a4-quantizer-impl-mismatch-audit` | `task-08` | `simd-quantizer` |
| `review/207-a4-1536-4bit-qjl-vs-mse` | `reviews/task-08/015-207-a4-1536-4bit-qjl-vs-mse` | `task-08` | `simd-quantizer` |
| `review/208-a4-thin-qjl-marginal-value` | `reviews/task-08/016-208-a4-thin-qjl-marginal-value` | `task-08` | `simd-quantizer` |
| `review/209-a4-4mse-codebook-sweep` | `reviews/task-08/017-209-a4-4mse-codebook-sweep` | `task-08` | `simd-quantizer` |
| `review/227-encode-pack-mse-indices-fast-path` | `reviews/task-08/018-227-encode-pack-mse-indices-fast-path` | `task-08` | `simd-quantizer` |
| `review/228-encode-nearest-centroid-branchless` | `reviews/task-08/019-228-encode-nearest-centroid-branchless` | `task-08` | `simd-quantizer` |
| `review/229-b1-simd-accel-merge` | `reviews/task-08/020-229-b1-simd-accel-merge` | `task-08` | `simd-quantizer` |
| `review/254-c1-scan-score-cache` | `reviews/task-08/021-254-c1-scan-score-cache` | `task-08` | `simd-quantizer` |
| `review/257-c1-qjl-disabled-score-fast-path` | `reviews/task-08/022-257-c1-qjl-disabled-score-fast-path` | `task-08` | `simd-quantizer` |
| `review/266-c1-avx2-no-qjl-4bit-score` | `reviews/task-08/023-266-c1-avx2-no-qjl-4bit-score` | `task-08` | `simd-quantizer` |
| `review/290-c1-adr031-tier2-pin-hold-borrowed-score` | `reviews/task-08/024-290-c1-adr031-tier2-pin-hold-borrowed-score` | `task-08` | `simd-quantizer` |
| `review/455-c1-native-build-query-score-cache` | `reviews/task-08/025-455-c1-native-build-query-score-cache` | `task-08` | `simd-quantizer` |
| `review/456-c1-native-build-backlink-score-cache` | `reviews/task-08/026-456-c1-native-build-backlink-score-cache` | `task-08` | `simd-quantizer` |
| `review/660-c1-source-score-avx-build-timing` | `reviews/task-08/027-660-c1-source-score-avx-build-timing` | `task-08` | `simd-quantizer` |
| `review/662-c1-source-score-avx-accumulator-timing` | `reviews/task-08/028-662-c1-source-score-avx-accumulator-timing` | `task-08` | `simd-quantizer` |
| `review/664-c1-avx-tail-feedback-followup` | `reviews/task-08/029-664-c1-avx-tail-feedback-followup` | `task-08` | `simd-quantizer` |
| `review/10046-spec-fr007-gamma-field` | `reviews/task-08/030-10046-spec-fr007-gamma-field` | `task-08` | `simd-quantizer` |
| `review/10050-nan-gamma-validation` | `reviews/task-08/031-10050-nan-gamma-validation` | `task-08` | `simd-quantizer` |
| `review/11005-phase1-quantizer-trait-seam` | `reviews/task-08/032-11005-phase1-quantizer-trait-seam` | `task-08` | `simd-quantizer` |
| `review/11029-phase5c3-codebook-chain-staging` | `reviews/task-08/033-11029-phase5c3-codebook-chain-staging` | `task-08` | `simd-quantizer` |
| `review/c1-task10065-native-hnsw-build-final-review.md` | `reviews/task-10065/001-c1-task10065-native-hnsw-build-final-review/request.md` | `task-10065` | `task-token` |
| `review/31-am-mod-cost-vacuum-split` | `reviews/task-11/001-31-am-mod-cost-vacuum-split` | `task-11` | `planner` |
| `review/127-admin-snapshot-for-planner-and-insert-stats.md` | `reviews/task-11/002-127-admin-snapshot-for-planner-and-insert-stats/request.md` | `task-11` | `planner` |
| `review/134-planner-cost-model-scaffolding.md` | `reviews/task-11/003-134-planner-cost-model-scaffolding/request.md` | `task-11` | `planner` |
| `review/135-cost-snapshot-for-gated-planner-model.md` | `reviews/task-11/004-135-cost-snapshot-for-gated-planner-model/request.md` | `task-11` | `planner` |
| `review/136-explicit-tree-height-fallback-in-cost-snapshot.md` | `reviews/task-11/005-136-explicit-tree-height-fallback-in-cost-snapshot/request.md` | `task-11` | `planner` |
| `review/137-strategy-translation-scaffolding-in-explain-snapshot.md` | `reviews/task-11/006-137-strategy-translation-scaffolding-in-explain-snapshot/request.md` | `task-11` | `planner` |
| `review/138-custom-explain-scaffolding-in-explain-snapshot.md` | `reviews/task-11/007-138-custom-explain-scaffolding-in-explain-snapshot/request.md` | `task-11` | `planner` |
| `review/139-statistics-scaffolding-snapshot.md` | `reviews/task-11/008-139-statistics-scaffolding-snapshot/request.md` | `task-11` | `planner` |
| `review/140-pg18-upgrade-boundary-snapshot.md` | `reviews/task-11/009-140-pg18-upgrade-boundary-snapshot/request.md` | `task-11` | `planner` |
| `review/141-consolidated-pg18-diagnostics-snapshot.md` | `reviews/task-11/010-141-consolidated-pg18-diagnostics-snapshot/request.md` | `task-11` | `planner` |
| `review/142-planner-integration-blockers-snapshot.md` | `reviews/task-11/011-142-planner-integration-blockers-snapshot/request.md` | `task-11` | `planner` |
| `review/167-planner-batch-review-and-alignment` | `reviews/task-11/012-167-planner-batch-review-and-alignment` | `task-11` | `planner` |
| `review/168-planner-pure-callback-batch-review` | `reviews/task-11/013-168-planner-pure-callback-batch-review` | `task-11` | `planner` |
| `review/249-c1-per-cell-planner-verification` | `reviews/task-11/014-249-c1-per-cell-planner-verification` | `task-11` | `planner` |
| `review/250-c1-ef200-planner-cost-crossover` | `reviews/task-11/015-250-c1-ef200-planner-cost-crossover` | `task-11` | `planner` |
| `review/462-c1-pg18-shared-infra-merge` | `reviews/task-11/016-462-c1-pg18-shared-infra-merge` | `task-11` | `planner` |
| `review/466-c1-pg18-preload-pgstat-validation` | `reviews/task-11/017-466-c1-pg18-preload-pgstat-validation` | `task-11` | `planner` |
| `review/609-c1-pg18-scale-invariant-drain` | `reviews/task-11/018-609-c1-pg18-scale-invariant-drain` | `task-11` | `planner` |
| `review/10001-ef-search-control-surface-and-planner-gate-scaffolding` | `reviews/task-11/019-10001-ef-search-control-surface-and-planner-gate-scaffolding` | `task-11` | `planner` |
| `review/10002-admin-snapshot-for-planner-and-insert-stats` | `reviews/task-11/020-10002-admin-snapshot-for-planner-and-insert-stats` | `task-11` | `planner` |
| `review/10003-explain-snapshot-for-planner-gate` | `reviews/task-11/021-10003-explain-snapshot-for-planner-gate` | `task-11` | `planner` |
| `review/10004-planner-cost-model-scaffolding` | `reviews/task-11/022-10004-planner-cost-model-scaffolding` | `task-11` | `planner` |
| `review/10005-cost-snapshot-for-gated-planner-model` | `reviews/task-11/023-10005-cost-snapshot-for-gated-planner-model` | `task-11` | `planner` |
| `review/10006-explicit-tree-height-fallback-in-cost-snapshot` | `reviews/task-11/024-10006-explicit-tree-height-fallback-in-cost-snapshot` | `task-11` | `planner` |
| `review/10007-strategy-translation-scaffolding-in-explain-snapshot` | `reviews/task-11/025-10007-strategy-translation-scaffolding-in-explain-snapshot` | `task-11` | `planner` |
| `review/10008-custom-explain-scaffolding-in-explain-snapshot` | `reviews/task-11/026-10008-custom-explain-scaffolding-in-explain-snapshot` | `task-11` | `planner` |
| `review/10009-statistics-scaffolding-snapshot` | `reviews/task-11/027-10009-statistics-scaffolding-snapshot` | `task-11` | `planner` |
| `review/10010-pg18-upgrade-boundary-snapshot` | `reviews/task-11/028-10010-pg18-upgrade-boundary-snapshot` | `task-11` | `planner` |
| `review/10011-consolidated-pg18-diagnostics-snapshot` | `reviews/task-11/029-10011-consolidated-pg18-diagnostics-snapshot` | `task-11` | `planner` |
| `review/10012-planner-integration-blockers-snapshot` | `reviews/task-11/030-10012-planner-integration-blockers-snapshot` | `task-11` | `planner` |
| `review/10013-read-stream-scaffolding-snapshot` | `reviews/task-11/031-10013-read-stream-scaffolding-snapshot` | `task-11` | `planner` |
| `review/10014-explain-counter-scaffolding-snapshot` | `reviews/task-11/032-10014-explain-counter-scaffolding-snapshot` | `task-11` | `planner` |
| `review/10015-read-stream-callback-signature-scaffolding` | `reviews/task-11/033-10015-read-stream-callback-signature-scaffolding` | `task-11` | `planner` |
| `review/10016-reusable-explain-counter-struct` | `reviews/task-11/034-10016-reusable-explain-counter-struct` | `task-11` | `planner` |
| `review/10017-pure-tree-height-callback-helper` | `reviews/task-11/035-10017-pure-tree-height-callback-helper` | `task-11` | `planner` |
| `review/10018-expand-pure-strategy-compare-type-contract` | `reviews/task-11/036-10018-expand-pure-strategy-compare-type-contract` | `task-11` | `planner` |
| `review/10019-explain-property-emission-skeleton` | `reviews/task-11/037-10019-explain-property-emission-skeleton` | `task-11` | `planner` |
| `review/10020-reusable-cumulative-stats-struct` | `reviews/task-11/038-10020-reusable-cumulative-stats-struct` | `task-11` | `planner` |
| `review/10021-read-stream-pure-callback-functions` | `reviews/task-11/039-10021-read-stream-pure-callback-functions` | `task-11` | `planner` |
| `review/10022-align-pure-callback-helper-names-with-pg18` | `reviews/task-11/040-10022-align-pure-callback-helper-names-with-pg18` | `task-11` | `planner` |
| `review/10023-explain-output-group-contract` | `reviews/task-11/041-10023-explain-output-group-contract` | `task-11` | `planner` |
| `review/10024-stats-summary-derived-rates` | `reviews/task-11/042-10024-stats-summary-derived-rates` | `task-11` | `planner` |
| `review/10025-explain-hook-context-gate` | `reviews/task-11/043-10025-explain-hook-context-gate` | `task-11` | `planner` |
| `review/10026-reusable-read-stream-state-reset` | `reviews/task-11/044-10026-reusable-read-stream-state-reset` | `task-11` | `planner` |
| `review/10035-consolidate-planner-snapshot-sql-surface` | `reviews/task-11/045-10035-consolidate-planner-snapshot-sql-surface` | `task-11` | `planner` |
| `review/10070-d2-planner-cost-activation` | `reviews/task-11/046-10070-d2-planner-cost-activation` | `task-11` | `planner` |
| `review/11045-phase9-planner-cost-activation` | `reviews/task-11/047-11045-phase9-planner-cost-activation` | `task-11` | `planner` |
| `review/30624-suite-explain-planner-cost-results` | `reviews/task-11/048-30624-suite-explain-planner-cost-results` | `task-11` | `planner` |
| `review/31143-c1-readstream-callback-unsafe-hardening` | `reviews/task-11/049-31143-c1-readstream-callback-unsafe-hardening` | `task-11` | `planner` |
| `review/31144-c1-pg18-pgstat-shim-unsafe-boundary` | `reviews/task-11/050-31144-c1-pg18-pgstat-shim-unsafe-boundary` | `task-11` | `planner` |
| `review/31148-c1-diskann-planner-cost-relation-guard` | `reviews/task-11/051-31148-c1-diskann-planner-cost-relation-guard` | `task-11` | `planner` |
| `review/31149-c1-ivf-planner-cost-relation-guard` | `reviews/task-11/052-31149-c1-ivf-planner-cost-relation-guard` | `task-11` | `planner` |
| `review/405-c1-adr030-v2-task15-landing-proof` | `reviews/task-15/001-405-c1-adr030-v2-task15-landing-proof` | `task-15` | `task-token` |
| `review/204-a4-full-vs-tiled-fwht-turboquantdb` | `reviews/task-16/001-204-a4-full-vs-tiled-fwht-turboquantdb` | `task-16` | `task16-ish` |
| `review/280-c1-adr030-grouped-feasibility-study` | `reviews/task-16/002-280-c1-adr030-grouped-feasibility-study` | `task-16` | `task16-ish` |
| `review/310-c1-adr030-v2-design-checkpoint` | `reviews/task-16/003-310-c1-adr030-v2-design-checkpoint` | `task-16` | `task16-ish` |
| `review/311-c1-adr030-v2-grouped-pq-feasibility` | `reviews/task-16/004-311-c1-adr030-v2-grouped-pq-feasibility` | `task-16` | `task16-ish` |
| `review/312-c1-adr030-v2-metadata-contract` | `reviews/task-16/005-312-c1-adr030-v2-metadata-contract` | `task-16` | `task16-ish` |
| `review/313-c1-adr030-v2-hot-cold-payload-contract` | `reviews/task-16/006-313-c1-adr030-v2-hot-cold-payload-contract` | `task-16` | `task16-ish` |
| `review/314-c1-adr030-v2-page-placement-contract` | `reviews/task-16/007-314-c1-adr030-v2-page-placement-contract` | `task-16` | `task16-ish` |
| `review/315-c1-adr030-v2-build-assembly-seam` | `reviews/task-16/008-315-c1-adr030-v2-build-assembly-seam` | `task-16` | `task16-ish` |
| `review/316-c1-adr030-v2-alternate-write-path` | `reviews/task-16/009-316-c1-adr030-v2-alternate-write-path` | `task-16` | `task16-ish` |
| `review/317-c1-adr030-v2-grouped-code-generation-seam` | `reviews/task-16/010-317-c1-adr030-v2-grouped-code-generation-seam` | `task-16` | `task16-ish` |
| `review/318-c1-adr030-v2-generated-code-write-path` | `reviews/task-16/011-318-c1-adr030-v2-generated-code-write-path` | `task-16` | `task16-ish` |
| `review/319-c1-adr030-v2-source-build-plan` | `reviews/task-16/012-319-c1-adr030-v2-source-build-plan` | `task-16` | `task16-ish` |
| `review/320-c1-adr030-v2-guarded-flush-output` | `reviews/task-16/013-320-c1-adr030-v2-guarded-flush-output` | `task-16` | `task16-ish` |
| `review/321-c1-adr030-v2-experimental-build-gate` | `reviews/task-16/014-321-c1-adr030-v2-experimental-build-gate` | `task-16` | `task16-ish` |
| `review/322-c1-adr030-v2-gated-raw-page-validation` | `reviews/task-16/015-322-c1-adr030-v2-gated-raw-page-validation` | `task-16` | `task16-ish` |
| `review/323-c1-adr030-v2-runtime-format-gate` | `reviews/task-16/016-323-c1-adr030-v2-runtime-format-gate` | `task-16` | `task16-ish` |
| `review/324-c1-adr030-v2-grouped-read-scaffolding` | `reviews/task-16/017-324-c1-adr030-v2-grouped-read-scaffolding` | `task-16` | `task16-ish` |
| `review/325-c1-adr030-v2-scan-storage-descriptor-seam` | `reviews/task-16/018-325-c1-adr030-v2-scan-storage-descriptor-seam` | `task-16` | `task16-ish` |
| `review/326-c1-adr030-v2-loaded-state-unavailable-seam` | `reviews/task-16/019-326-c1-adr030-v2-loaded-state-unavailable-seam` | `task-16` | `task16-ish` |
| `review/327-c1-adr030-v2-grouped-hot-payload-cache-seam` | `reviews/task-16/020-327-c1-adr030-v2-grouped-hot-payload-cache-seam` | `task-16` | `task16-ish` |
| `review/328-c1-adr030-v2-grouped-score-input-seam` | `reviews/task-16/021-328-c1-adr030-v2-grouped-score-input-seam` | `task-16` | `task16-ish` |
| `review/329-c1-adr030-v2-candidate-score-dispatch-seam` | `reviews/task-16/022-329-c1-adr030-v2-candidate-score-dispatch-seam` | `task-16` | `task16-ish` |
| `review/330-c1-adr030-v2-grouped-score-helper-stub` | `reviews/task-16/023-330-c1-adr030-v2-grouped-score-helper-stub` | `task-16` | `task16-ish` |
| `review/331-c1-adr030-v2-grouped-score-shape-seam` | `reviews/task-16/024-331-c1-adr030-v2-grouped-score-shape-seam` | `task-16` | `task16-ish` |
| `review/332-c1-adr030-v2-grouped-score-context-seam` | `reviews/task-16/025-332-c1-adr030-v2-grouped-score-context-seam` | `task-16` | `task16-ish` |
| `review/333-c1-adr030-v2-grouped-score-helper-context` | `reviews/task-16/026-333-c1-adr030-v2-grouped-score-helper-context` | `task-16` | `task16-ish` |
| `review/334-c1-adr030-v2-grouped-payload-view` | `reviews/task-16/027-334-c1-adr030-v2-grouped-payload-view` | `task-16` | `task16-ish` |
| `review/335-c1-adr030-v2-review-driven-plan-adjustment` | `reviews/task-16/028-335-c1-adr030-v2-review-driven-plan-adjustment` | `task-16` | `task16-ish` |
| `review/336-c1-adr030-v2-shared-grouped-encoder-contract` | `reviews/task-16/029-336-c1-adr030-v2-shared-grouped-encoder-contract` | `task-16` | `task16-ish` |
| `review/337-c1-adr030-v2-insert-format-gate` | `reviews/task-16/030-337-c1-adr030-v2-insert-format-gate` | `task-16` | `task16-ish` |
| `review/338-c1-adr030-v2-vacuum-format-gate` | `reviews/task-16/031-338-c1-adr030-v2-vacuum-format-gate` | `task-16` | `task16-ish` |
| `review/339-c1-adr030-v2-cold-rerank-fetch-seam` | `reviews/task-16/032-339-c1-adr030-v2-cold-rerank-fetch-seam` | `task-16` | `task16-ish` |
| `review/340-c1-adr030-v2-grouped-rerank-payload-seam` | `reviews/task-16/033-340-c1-adr030-v2-grouped-rerank-payload-seam` | `task-16` | `task16-ish` |
| `review/341-c1-adr030-v2-grouped-metadata-payload-validation` | `reviews/task-16/034-341-c1-adr030-v2-grouped-metadata-payload-validation` | `task-16` | `task16-ish` |
| `review/342-c1-adr030-v2-grouped-exact-rerank-helper` | `reviews/task-16/035-342-c1-adr030-v2-grouped-exact-rerank-helper` | `task-16` | `task16-ish` |
| `review/343-c1-adr030-v2-shared-grouped-pq-scorer` | `reviews/task-16/036-343-c1-adr030-v2-shared-grouped-pq-scorer` | `task-16` | `task16-ish` |
| `review/344-c1-adr030-v2-persisted-grouped-codebook-basis` | `reviews/task-16/037-344-c1-adr030-v2-persisted-grouped-codebook-basis` | `task-16` | `task16-ish` |
| `review/345-c1-adr030-v2-gated-grouped-scan-runtime` | `reviews/task-16/038-345-c1-adr030-v2-gated-grouped-scan-runtime` | `task-16` | `task16-ish` |
| `review/346-c1-adr030-v2-grouped-rerank-comparison-output` | `reviews/task-16/039-346-c1-adr030-v2-grouped-rerank-comparison-output` | `task-16` | `task16-ish` |
| `review/347-c1-adr030-v2-grouped-divergence-summary-diagnostics` | `reviews/task-16/040-347-c1-adr030-v2-grouped-divergence-summary-diagnostics` | `task-16` | `task16-ish` |
| `review/348-c1-adr030-v2-grouped-order-drift-diagnostics` | `reviews/task-16/041-348-c1-adr030-v2-grouped-order-drift-diagnostics` | `task-16` | `task16-ish` |
| `review/349-c1-adr030-v2-grouped-window-evidence-summary` | `reviews/task-16/042-349-c1-adr030-v2-grouped-window-evidence-summary` | `task-16` | `task16-ish` |
| `review/350-c1-adr030-v2-grouped-window-simulation-diagnostics` | `reviews/task-16/043-350-c1-adr030-v2-grouped-window-simulation-diagnostics` | `task-16` | `task16-ish` |
| `review/351-c1-adr030-v2-grouped-live-rerank-window` | `reviews/task-16/044-351-c1-adr030-v2-grouped-live-rerank-window` | `task-16` | `task16-ish` |
| `review/352-c1-adr030-v2-staged-1k-10k-50k-runtime-validation` | `reviews/task-16/045-352-c1-adr030-v2-staged-1k-10k-50k-runtime-validation` | `task-16` | `task16-ish` |
| `review/353-c1-adr030-v2-configurable-live-rerank-window` | `reviews/task-16/046-353-c1-adr030-v2-configurable-live-rerank-window` | `task-16` | `task16-ish` |
| `review/355-c1-adr030-v2-exact-traversal-scan-mode` | `reviews/task-16/047-355-c1-adr030-v2-exact-traversal-scan-mode` | `task-16` | `task16-ish` |
| `review/356-c1-adr030-v2-selective-exact-traversal-budget` | `reviews/task-16/048-356-c1-adr030-v2-selective-exact-traversal-budget` | `task-16` | `task16-ish` |
| `review/358-c1-adr030-v2-frontier-head-exact-traversal` | `reviews/task-16/049-358-c1-adr030-v2-frontier-head-exact-traversal` | `task-16` | `task16-ish` |
| `review/359-c1-adr030-v2-binary-traversal-score-mode` | `reviews/task-16/050-359-c1-adr030-v2-binary-traversal-score-mode` | `task-16` | `task16-ish` |
| `review/360-c1-adr030-v2-binary-window64-operating-point` | `reviews/task-16/051-360-c1-adr030-v2-binary-window64-operating-point` | `task-16` | `task16-ish` |
| `review/361-c1-adr030-v2-deterministic-grouped-graph-build` | `reviews/task-16/052-361-c1-adr030-v2-deterministic-grouped-graph-build` | `task-16` | `task16-ish` |
| `review/362-c1-adr030-v2-real-50k-m16-runtime-validation` | `reviews/task-16/053-362-c1-adr030-v2-real-50k-m16-runtime-validation` | `task-16` | `task16-ish` |
| `review/363-c1-adr030-v2-pgvector-size-and-runtime-baseline` | `reviews/task-16/054-363-c1-adr030-v2-pgvector-size-and-runtime-baseline` | `task-16` | `task16-ish` |
| `review/365-c1-adr030-v2-tqvector-sql-overhead-breakdown` | `reviews/task-16/055-365-c1-adr030-v2-tqvector-sql-overhead-breakdown` | `task-16` | `task16-ish` |
| `review/366-c1-adr030-v2-heap-fetch-projection-breakdown` | `reviews/task-16/056-366-c1-adr030-v2-heap-fetch-projection-breakdown` | `task-16` | `task16-ish` |
| `review/367-c1-adr030-v2-corrected-sql-overhead-session-reuse` | `reviews/task-16/057-367-c1-adr030-v2-corrected-sql-overhead-session-reuse` | `task-16` | `task16-ish` |
| `review/368-c1-adr030-v2-matched-session-sql-comparison` | `reviews/task-16/058-368-c1-adr030-v2-matched-session-sql-comparison` | `task-16` | `task16-ish` |
| `review/369-c1-adr030-v2-matched-session-operating-point-verdict` | `reviews/task-16/059-369-c1-adr030-v2-matched-session-operating-point-verdict` | `task-16` | `task16-ish` |
| `review/370-c1-adr030-v2-isolated-m16-lane-reconciliation` | `reviews/task-16/060-370-c1-adr030-v2-isolated-m16-lane-reconciliation` | `task-16` | `task16-ish` |
| `review/371-c1-adr030-v2-heap-f32-rerank-feasibility` | `reviews/task-16/061-371-c1-adr030-v2-heap-f32-rerank-feasibility` | `task-16` | `task16-ish` |
| `review/373-c1-adr030-v2-zero-copy-heap-array-decode` | `reviews/task-16/062-373-c1-adr030-v2-zero-copy-heap-array-decode` | `task-16` | `task16-ish` |
| `review/374-c1-adr030-v2-scan-only-bytea-rerank-source` | `reviews/task-16/063-374-c1-adr030-v2-scan-only-bytea-rerank-source` | `task-16` | `task16-ish` |
| `review/375-c1-adr030-v2-runtime-adapters-and-shared-grouped-encoder` | `reviews/task-16/064-375-c1-adr030-v2-runtime-adapters-and-shared-grouped-encoder` | `task-16` | `task16-ish` |
| `review/376-c1-adr030-v2-shared-source-metric-and-live-source-insert` | `reviews/task-16/065-376-c1-adr030-v2-shared-source-metric-and-live-source-insert` | `task-16` | `task16-ish` |
| `review/377-c1-adr030-v2-source-backed-vacuum-repair` | `reviews/task-16/066-377-c1-adr030-v2-source-backed-vacuum-repair` | `task-16` | `task16-ish` |
| `review/378-c1-adr030-v2-storage-format-reloption-build-selection` | `reviews/task-16/067-378-c1-adr030-v2-storage-format-reloption-build-selection` | `task-16` | `task16-ish` |
| `review/379-c1-adr030-v2-scan-runtime-gate-removal` | `reviews/task-16/068-379-c1-adr030-v2-scan-runtime-gate-removal` | `task-16` | `task16-ish` |
| `review/380-c1-adr030-v2-runtime-descriptor-rename` | `reviews/task-16/069-380-c1-adr030-v2-runtime-descriptor-rename` | `task-16` | `task16-ish` |
| `review/381-c1-adr030-v2-storage-aware-exact-graph-search` | `reviews/task-16/070-381-c1-adr030-v2-storage-aware-exact-graph-search` | `task-16` | `task16-ish` |
| `review/382-c1-adr030-v2-pqfastscan-live-insert-built-indexes` | `reviews/task-16/071-382-c1-adr030-v2-pqfastscan-live-insert-built-indexes` | `task-16` | `task16-ish` |
| `review/383-c1-adr030-v2-pqfastscan-vacuum-built-indexes` | `reviews/task-16/072-383-c1-adr030-v2-pqfastscan-vacuum-built-indexes` | `task-16` | `task16-ish` |
| `review/384-c1-adr030-v2-core-storage-format-naming` | `reviews/task-16/073-384-c1-adr030-v2-core-storage-format-naming` | `task-16` | `task16-ish` |
| `review/385-c1-adr030-v2-pqfastscan-vacuum-linear-top-up` | `reviews/task-16/074-385-c1-adr030-v2-pqfastscan-vacuum-linear-top-up` | `task-16` | `task16-ish` |
| `review/386-c1-adr030-v2-pqfastscan-runtime-test-helper-names` | `reviews/task-16/075-386-c1-adr030-v2-pqfastscan-runtime-test-helper-names` | `task-16` | `task16-ish` |
| `review/387-c1-adr030-v2-first-class-storage-format-docs-and-tests` | `reviews/task-16/076-387-c1-adr030-v2-first-class-storage-format-docs-and-tests` | `task-16` | `task16-ish` |
| `review/388-c1-adr030-v2-pqfastscan-pg-test-surface-rename` | `reviews/task-16/077-388-c1-adr030-v2-pqfastscan-pg-test-surface-rename` | `task-16` | `task16-ish` |
| `review/389-c1-adr030-v2-pqfastscan-insert-vacuum-scan-assertions` | `reviews/task-16/078-389-c1-adr030-v2-pqfastscan-insert-vacuum-scan-assertions` | `task-16` | `task16-ish` |
| `review/390-c1-adr030-v2-pqfastscan-empty-bootstrap-and-small-builds` | `reviews/task-16/079-390-c1-adr030-v2-pqfastscan-empty-bootstrap-and-small-builds` | `task-16` | `task16-ish` |
| `review/391-c1-adr030-v2-pqfastscan-small-dimension-group-size` | `reviews/task-16/080-391-c1-adr030-v2-pqfastscan-small-dimension-group-size` | `task-16` | `task16-ish` |
| `review/392-c1-adr030-v2-pqfastscan-runtime-env-surface-rename` | `reviews/task-16/081-392-c1-adr030-v2-pqfastscan-runtime-env-surface-rename` | `task-16` | `task16-ish` |
| `review/393-c1-adr030-v2-storage-format-round-trip-proof` | `reviews/task-16/082-393-c1-adr030-v2-storage-format-round-trip-proof` | `task-16` | `task16-ish` |
| `review/394-c1-adr030-v2-pqfastscan-runtime-debug-surface-rename` | `reviews/task-16/083-394-c1-adr030-v2-pqfastscan-runtime-debug-surface-rename` | `task-16` | `task16-ish` |
| `review/395-c1-adr030-v2-pqfastscan-scan-analysis-debug-surface-rename` | `reviews/task-16/084-395-c1-adr030-v2-pqfastscan-scan-analysis-debug-surface-rename` | `task-16` | `task16-ish` |
| `review/397-c1-adr030-v2-pqfastscan-runtime-test-env-surface-rename` | `reviews/task-16/085-397-c1-adr030-v2-pqfastscan-runtime-test-env-surface-rename` | `task-16` | `task16-ish` |
| `review/400-c1-adr030-v2-explicit-format-scratch-wrapper-support` | `reviews/task-16/086-400-c1-adr030-v2-explicit-format-scratch-wrapper-support` | `task-16` | `task16-ish` |
| `review/401-c1-adr030-v2-pqfastscan-default-runtime-lane` | `reviews/task-16/087-401-c1-adr030-v2-pqfastscan-default-runtime-lane` | `task-16` | `task16-ish` |
| `review/402-c1-adr030-v2-scratch-socket-target-guardrails` | `reviews/task-16/088-402-c1-adr030-v2-scratch-socket-target-guardrails` | `task-16` | `task16-ish` |
| `review/403-c1-adr030-v2-storage-format-reindex-guardrail` | `reviews/task-16/089-403-c1-adr030-v2-storage-format-reindex-guardrail` | `task-16` | `task16-ish` |
| `review/404-c1-adr030-v2-source-backed-pqfastscan-default-heap-rerank` | `reviews/task-16/090-404-c1-adr030-v2-source-backed-pqfastscan-default-heap-rerank` | `task-16` | `task16-ish` |
| `review/406-c1-adr030-v2-scratch-socket-override-visibility` | `reviews/task-16/091-406-c1-adr030-v2-scratch-socket-override-visibility` | `task-16` | `task16-ish` |
| `review/407-c1-adr030-v2-real-50k-m16-current-head-validation` | `reviews/task-16/092-407-c1-adr030-v2-real-50k-m16-current-head-validation` | `task-16` | `task16-ish` |
| `review/408-c1-adr030-v2-pqfastscan-index-runtime-fallback-visibility` | `reviews/task-16/093-408-c1-adr030-v2-pqfastscan-index-runtime-fallback-visibility` | `task-16` | `task16-ish` |
| `review/409-c1-adr030-v2-storage-format-reindex-insert-vacuum-coverage` | `reviews/task-16/094-409-c1-adr030-v2-storage-format-reindex-insert-vacuum-coverage` | `task-16` | `task16-ish` |
| `review/410-c1-adr030-v2-pqfastscan-index-rerank-runtime-visibility` | `reviews/task-16/095-410-c1-adr030-v2-pqfastscan-index-rerank-runtime-visibility` | `task-16` | `task16-ish` |
| `review/411-c1-adr030-v2-pqfastscan-default-rerank-parity` | `reviews/task-16/096-411-c1-adr030-v2-pqfastscan-default-rerank-parity` | `task-16` | `task16-ish` |
| `review/412-c1-adr030-v2-formatting-cleanup-runtime-helpers` | `reviews/task-16/097-412-c1-adr030-v2-formatting-cleanup-runtime-helpers` | `task-16` | `task16-ish` |
| `review/413-c1-adr030-v2-scratch-explicit-format-runtime-matrix` | `reviews/task-16/098-413-c1-adr030-v2-scratch-explicit-format-runtime-matrix` | `task-16` | `task16-ish` |
| `review/414-c1-adr030-v2-scratch-isolated-explicit-format-sql-matrix` | `reviews/task-16/099-414-c1-adr030-v2-scratch-isolated-explicit-format-sql-matrix` | `task-16` | `task16-ish` |
| `review/416-c1-adr030-v2-debug-heap-backed-scan-and-vacuum-repair` | `reviews/task-16/100-416-c1-adr030-v2-debug-heap-backed-scan-and-vacuum-repair` | `task-16` | `task16-ish` |
| `review/417-c1-adr030-v2-pqfastscan-runtime-fixture-contract-alignment` | `reviews/task-16/101-417-c1-adr030-v2-pqfastscan-runtime-fixture-contract-alignment` | `task-16` | `task16-ish` |
| `review/418-c1-qjl-build-offset-and-grouped-pq-study-alignment` | `reviews/task-16/102-418-c1-qjl-build-offset-and-grouped-pq-study-alignment` | `task-16` | `task16-ish` |
| `review/419-c1-adr030-v2-merge-readiness-assessment` | `reviews/task-16/103-419-c1-adr030-v2-merge-readiness-assessment` | `task-16` | `task16-ish` |
| `review/420-c1-adr030-v2-storage-format-matrix-and-runtime-gate-closure` | `reviews/task-16/104-420-c1-adr030-v2-storage-format-matrix-and-runtime-gate-closure` | `task-16` | `task16-ish` |
| `review/421-c1-adr030-v2-final-local-landing-proof-artifact` | `reviews/task-16/105-421-c1-adr030-v2-final-local-landing-proof-artifact` | `task-16` | `task16-ish` |
| `review/422-c1-task16-buildcodedistance-50k-build-row` | `reviews/task-16/106-422-c1-task16-buildcodedistance-50k-build-row` | `task-16` | `task-token` |
| `review/424-c1-task16-turboquant-live-rerank` | `reviews/task-16/107-424-c1-task16-turboquant-live-rerank` | `task-16` | `task-token` |
| `review/425-c1-task16-turboquant-quantized-default` | `reviews/task-16/108-425-c1-task16-turboquant-quantized-default` | `task-16` | `task-token` |
| `review/427-c1-task16-turboquant-v3-page-layout-groundwork` | `reviews/task-16/109-427-c1-task16-turboquant-v3-page-layout-groundwork` | `task-16` | `task-token` |
| `review/428-c1-task16-turboquant-v3-runtime-wiring` | `reviews/task-16/110-428-c1-task16-turboquant-v3-runtime-wiring` | `task-16` | `task-token` |
| `review/431-c1-task16-persisted-rerank-source-column` | `reviews/task-16/111-431-c1-task16-persisted-rerank-source-column` | `task-16` | `task-token` |
| `review/433-c1-task16-lever4-lever5-option-matrix` | `reviews/task-16/112-433-c1-task16-lever4-lever5-option-matrix` | `task-16` | `task-token` |
| `review/434-c1-task16-turboquant-int8-exact-score-experiment` | `reviews/task-16/113-434-c1-task16-turboquant-int8-exact-score-experiment` | `task-16` | `task-token` |
| `review/435-c1-task16-args-only-scratch-wrapper-targeting` | `reviews/task-16/114-435-c1-task16-args-only-scratch-wrapper-targeting` | `task-16` | `task-token` |
| `review/436-c1-task16-turboquant-lut-score-experiments` | `reviews/task-16/115-436-c1-task16-turboquant-lut-score-experiments` | `task-16` | `task-token` |
| `review/437-c1-task16-turboquant-live-score-mode-matrix` | `reviews/task-16/116-437-c1-task16-turboquant-live-score-mode-matrix` | `task-16` | `task-token` |
| `review/438-c1-task16-vacuum-entry-repair-and-scan-fallback` | `reviews/task-16/117-438-c1-task16-vacuum-entry-repair-and-scan-fallback` | `task-16` | `task-token` |
| `review/439-c1-task16-turboquant-rerank-source-column` | `reviews/task-16/118-439-c1-task16-turboquant-rerank-source-column` | `task-16` | `task-token` |
| `review/442-c1-task16-ecvector-canonical-row-model` | `reviews/task-16/119-442-c1-task16-ecvector-canonical-row-model` | `task-16` | `task-token` |
| `review/443-c1-task16-tqvector-quant-artifact-rename` | `reviews/task-16/120-443-c1-task16-tqvector-quant-artifact-rename` | `task-16` | `task-token` |
| `review/444-c1-task16-scratch-wrapper-db-targeting` | `reviews/task-16/121-444-c1-task16-scratch-wrapper-db-targeting` | `task-16` | `task-token` |
| `review/445-c1-task16-tqvector-compact-artifact-layout` | `reviews/task-16/122-445-c1-task16-tqvector-compact-artifact-layout` | `task-16` | `task-token` |
| `review/447-c1-task16-ecvector-inline-storage-tradeoff` | `reviews/task-16/123-447-c1-task16-ecvector-inline-storage-tradeoff` | `task-16` | `task-token` |
| `review/448-c1-task16-rerank-source-reset-regression` | `reviews/task-16/124-448-c1-task16-rerank-source-reset-regression` | `task-16` | `task-token` |
| `review/449-c1-task16-adr044-storage-policy-proposal` | `reviews/task-16/125-449-c1-task16-adr044-storage-policy-proposal` | `task-16` | `task-token` |
| `review/450-c1-task16-install-script-backend-assertion` | `reviews/task-16/126-450-c1-task16-install-script-backend-assertion` | `task-16` | `task-token` |
| `review/451-c1-task16-ecvector-storage-code-correction` | `reviews/task-16/127-451-c1-task16-ecvector-storage-code-correction` | `task-16` | `task-token` |
| `review/452-c1-task16-storage-policy-deferral-to-native-build` | `reviews/task-16/128-452-c1-task16-storage-policy-deferral-to-native-build` | `task-16` | `task-token` |
| `review/453-c1-task16-follow-on-tracking-hardening` | `reviews/task-16/129-453-c1-task16-follow-on-tracking-hardening` | `task-16` | `task-token` |
| `review/458-c1-ecvector-extension-upgrade-path` | `reviews/task-16/130-458-c1-ecvector-extension-upgrade-path` | `task-16` | `task16-ish` |
| `review/460-c1-task17-shared-am-module-split` | `reviews/task-17/001-460-c1-task17-shared-am-module-split` | `task-17` | `task-token` |
| `review/11001-diskann-task17-plan` | `reviews/task-17/002-11001-diskann-task17-plan` | `task-17` | `task-token` |
| `review/11002-adr046-vamana-insert-lock-ordering` | `reviews/task-17/003-11002-adr046-vamana-insert-lock-ordering` | `task-17` | `diskann-legacy` |
| `review/11003-adr047-vamana-vacuum-lock-ordering` | `reviews/task-17/004-11003-adr047-vamana-vacuum-lock-ordering` | `task-17` | `diskann-legacy` |
| `review/11004-diskann-build-algorithm-design` | `reviews/task-17/005-11004-diskann-build-algorithm-design` | `task-17` | `diskann-legacy` |
| `review/11015-phase5a-vamana-algorithm-core` | `reviews/task-17/006-11015-phase5a-vamana-algorithm-core` | `task-17` | `diskann-legacy` |
| `review/11046-task17-status-doc-refresh` | `reviews/task-17/007-11046-task17-status-doc-refresh` | `task-17` | `task-token` |
| `review/11048-task17-strict-clippy-hygiene` | `reviews/task-17/008-11048-task17-strict-clippy-hygiene` | `task-17` | `task-token` |
| `review/11053-task17-revert-script-work-deprecated` | `reviews/task-17/009-11053-task17-revert-script-work-deprecated` | `task-17` | `task-token` |
| `review/11054-task17-unknown-reloption-warn` | `reviews/task-17/010-11054-task17-unknown-reloption-warn` | `task-17` | `task-token` |
| `review/11056-task17-reloption-flag-collision` | `reviews/task-17/011-11056-task17-reloption-flag-collision` | `task-17` | `task-token` |
| `review/11057-task17-sweep-axis-label` | `reviews/task-17/012-11057-task17-sweep-axis-label` | `task-17` | `task-token` |
| `review/11060-task17-compare-am-preflight` | `reviews/task-17/013-11060-task17-compare-am-preflight` | `task-17` | `task-token` |
| `review/11062-task17-compare-axis-labels` | `reviews/task-17/014-11062-task17-compare-axis-labels` | `task-17` | `task-token` |
| `review/11065-task17-compare-progress-labels` | `reviews/task-17/015-11065-task17-compare-progress-labels` | `task-17` | `task-token` |
| `review/11066-task17-diskann-m-guidance` | `reviews/task-17/016-11066-task17-diskann-m-guidance` | `task-17` | `task-token` |
| `review/11067-task17-diskann-ef-construction` | `reviews/task-17/017-11067-task17-diskann-ef-construction` | `task-17` | `task-token` |
| `review/11068-task17-diskann-reload-preflight` | `reviews/task-17/018-11068-task17-diskann-reload-preflight` | `task-17` | `task-token` |
| `review/11069-task17-diskann-hnsw-reloption-preflight` | `reviews/task-17/019-11069-task17-diskann-hnsw-reloption-preflight` | `task-17` | `task-token` |
| `review/11070-task17-ecaz-cli-connection-targeting` | `reviews/task-17/020-11070-task17-ecaz-cli-connection-targeting` | `task-17` | `task-token` |
| `review/11072-task17-psql-relkind-char` | `reviews/task-17/021-11072-task17-psql-relkind-char` | `task-17` | `task-token` |
| `review/11074-task17-ecaz-log-file` | `reviews/task-17/022-11074-task17-ecaz-log-file` | `task-17` | `task-token` |
| `review/11075-task17-loader-ensure-ecaz-extension` | `reviews/task-17/023-11075-task17-loader-ensure-ecaz-extension` | `task-17` | `task-token` |
| `review/11076-task17-knn-operator-resolution` | `reviews/task-17/024-11076-task17-knn-operator-resolution` | `task-17` | `task-token` |
| `review/11079-task17-diskann-sql-limit-cap` | `reviews/task-17/025-11079-task17-diskann-sql-limit-cap` | `task-17` | `task-token` |
| `review/11080-task17-build-duplicate-coalescing` | `reviews/task-17/026-11080-task17-build-duplicate-coalescing` | `task-17` | `task-token` |
| `review/11082-task17-vacuum-pq-frontier` | `reviews/task-17/027-11082-task17-vacuum-pq-frontier` | `task-17` | `task-token` |
| `review/11083-task17-diskann-post-vacuum-smoke` | `reviews/task-17/028-11083-task17-diskann-post-vacuum-smoke` | `task-17` | `task-token` |
| `review/11084-task17-status-refresh` | `reviews/task-17/029-11084-task17-status-refresh` | `task-17` | `task-token` |
| `review/11085-task17-unit-norm-contract` | `reviews/task-17/030-11085-task17-unit-norm-contract` | `task-17` | `task-token` |
| `review/11086-task17-handoff-contract` | `reviews/task-17/031-11086-task17-handoff-contract` | `task-17` | `task-token` |
| `review/30155-user-docs-ivf-diskann` | `reviews/task-17/032-30155-user-docs-ivf-diskann` | `task-17` | `diskann-legacy` |
| `review/20000-task25-rabitq-module-skeleton` | `reviews/task-25/001-20000-task25-rabitq-module-skeleton` | `task-25` | `task-token` |
| `review/20001-task25-rabitq-graduate-adr031` | `reviews/task-25/002-20001-task25-rabitq-graduate-adr031` | `task-25` | `task-token` |
| `review/20002-task25-rabitq-rotation-seam` | `reviews/task-25/003-20002-task25-rabitq-rotation-seam` | `task-25` | `task-token` |
| `review/20003-task25-rabitq-estimator` | `reviews/task-25/004-20003-task25-rabitq-estimator` | `task-25` | `task-token` |
| `review/20004-task25-rabitq-feasibility-binary` | `reviews/task-25/005-20004-task25-rabitq-feasibility-binary` | `task-25` | `task-token` |
| `review/20005-task25-task27-handoff-contract` | `reviews/task-25/006-20005-task25-task27-handoff-contract` | `task-25` | `task-token` |
| `review/20006-task25-ecaz-cli-quant-feasibility` | `reviews/task-25/007-20006-task25-ecaz-cli-quant-feasibility` | `task-25` | `task-token` |
| `review/20007-task25-rabitq-gate-verdict` | `reviews/task-25/008-20007-task25-rabitq-gate-verdict` | `task-25` | `task-token` |
| `review/20008-task25-rabitq-paper-faithful-port` | `reviews/task-25/009-20008-task25-rabitq-paper-faithful-port` | `task-25` | `task-token` |
| `review/20009-task25-rabitq-gate-verdict-rerun` | `reviews/task-25/010-20009-task25-rabitq-gate-verdict-rerun` | `task-25` | `task-token` |
| `review/20010-task25-feasibility-rerank` | `reviews/task-25/011-20010-task25-feasibility-rerank` | `task-25` | `task-token` |
| `review/20011-task25-qbit-sweep` | `reviews/task-25/012-20011-task25-qbit-sweep` | `task-25` | `task-token` |
| `review/20012-task25-srht-seed-plumbing` | `reviews/task-25/013-20012-task25-srht-seed-plumbing` | `task-25` | `task-token` |
| `review/20013-task25-symphony-prerequisite-finding` | `reviews/task-25/014-20013-task25-symphony-prerequisite-finding` | `task-25` | `task-token` |
| `review/20014-task25-centered-api` | `reviews/task-25/015-20014-task25-centered-api` | `task-25` | `task-token` |
| `review/20015-task25-task27-handoff-contract-v2` | `reviews/task-25/016-20015-task25-task27-handoff-contract-v2` | `task-25` | `task-token` |
| `review/20016-task25-reviewer-feedback-response` | `reviews/task-25/017-20016-task25-reviewer-feedback-response` | `task-25` | `task-token` |
| `review/30158-rabitq-symphony-status-refresh` | `reviews/task-25/018-30158-rabitq-symphony-status-refresh` | `task-25` | `rabitq` |
| `review/467-c1-parallel-scan-callback-surface` | `reviews/task-26/001-467-c1-parallel-scan-callback-surface` | `task-26` | `parallel-build` |
| `review/468-c1-parallel-scan-dsm-layout` | `reviews/task-26/002-468-c1-parallel-scan-dsm-layout` | `task-26` | `parallel-build` |
| `review/469-c1-parallel-scan-worker-slot-claiming` | `reviews/task-26/003-469-c1-parallel-scan-worker-slot-claiming` | `task-26` | `parallel-build` |
| `review/470-c1-parallel-scan-worker-runtime-snapshots` | `reviews/task-26/004-470-c1-parallel-scan-worker-runtime-snapshots` | `task-26` | `parallel-build` |
| `review/618-c1-parallel-index-build-coordinator-scaffold` | `reviews/task-26/005-618-c1-parallel-index-build-coordinator-scaffold` | `task-26` | `parallel-build` |
| `review/619-c1-parallel-index-build-ingestion` | `reviews/task-26/006-619-c1-parallel-index-build-ingestion` | `task-26` | `parallel-build` |
| `review/621-c1-parallel-index-build-phase-timing` | `reviews/task-26/007-621-c1-parallel-index-build-phase-timing` | `task-26` | `parallel-build` |
| `review/632-c1-parallel-hnsw-build-graph-assembly-adr` | `reviews/task-26/008-632-c1-parallel-hnsw-build-graph-assembly-adr` | `task-26` | `parallel-build` |
| `review/634-c1-concurrent-dsm-graph-layout` | `reviews/task-26/009-634-c1-concurrent-dsm-graph-layout` | `task-26` | `parallel-build` |
| `review/635-c1-concurrent-dsm-node-slot-plan` | `reviews/task-26/010-635-c1-concurrent-dsm-node-slot-plan` | `task-26` | `parallel-build` |
| `review/637-c1-concurrent-dsm-preassembly-plan` | `reviews/task-26/011-637-c1-concurrent-dsm-preassembly-plan` | `task-26` | `parallel-build` |
| `review/638-c1-concurrent-dsm-image-initializer` | `reviews/task-26/012-638-c1-concurrent-dsm-image-initializer` | `task-26` | `parallel-build` |
| `review/639-c1-concurrent-dsm-insertion-ranges` | `reviews/task-26/013-639-c1-concurrent-dsm-insertion-ranges` | `task-26` | `parallel-build` |
| `review/640-c1-concurrent-dsm-graph-readback` | `reviews/task-26/014-640-c1-concurrent-dsm-graph-readback` | `task-26` | `parallel-build` |
| `review/641-c1-concurrent-dsm-node-insert` | `reviews/task-26/015-641-c1-concurrent-dsm-node-insert` | `task-26` | `parallel-build` |
| `review/642-c1-concurrent-dsm-partition-insert` | `reviews/task-26/016-642-c1-concurrent-dsm-partition-insert` | `task-26` | `parallel-build` |
| `review/643-c1-concurrent-dsm-page-staging` | `reviews/task-26/017-643-c1-concurrent-dsm-page-staging` | `task-26` | `parallel-build` |
| `review/644-c1-concurrent-dsm-layout-reattach` | `reviews/task-26/018-644-c1-concurrent-dsm-layout-reattach` | `task-26` | `parallel-build` |
| `review/645-c1-concurrent-dsm-insert-config-header` | `reviews/task-26/019-645-c1-concurrent-dsm-insert-config-header` | `task-26` | `parallel-build` |
| `review/646-c1-concurrent-dsm-graph-attachment` | `reviews/task-26/020-646-c1-concurrent-dsm-graph-attachment` | `task-26` | `parallel-build` |
| `review/647-c1-parallel-concurrent-dsm-graph-workers` | `reviews/task-26/021-647-c1-parallel-concurrent-dsm-graph-workers` | `task-26` | `parallel-build` |
| `review/649-c1-concurrent-dsm-graph-timing-accounting` | `reviews/task-26/022-649-c1-concurrent-dsm-graph-timing-accounting` | `task-26` | `parallel-build` |
| `review/653-c1-concurrent-dsm-striped-graph-insertion` | `reviews/task-26/023-653-c1-concurrent-dsm-striped-graph-insertion` | `task-26` | `parallel-build` |
| `review/657-c1-concurrent-dsm-source-scored-graph-build` | `reviews/task-26/024-657-c1-concurrent-dsm-source-scored-graph-build` | `task-26` | `parallel-build` |
| `review/658-c1-concurrent-dsm-source-real-50k-rerun` | `reviews/task-26/025-658-c1-concurrent-dsm-source-real-50k-rerun` | `task-26` | `parallel-build` |
| `review/659-c1-source-dsm-real-50k-build-timing` | `reviews/task-26/026-659-c1-source-dsm-real-50k-build-timing` | `task-26` | `parallel-build` |
| `review/663-c1-concurrent-dsm-backlink-score-cache-timing` | `reviews/task-26/027-663-c1-concurrent-dsm-backlink-score-cache-timing` | `task-26` | `parallel-build` |
| `review/665-c1-concurrent-dsm-default-switch` | `reviews/task-26/028-665-c1-concurrent-dsm-default-switch` | `task-26` | `parallel-build` |
| `review/667-c1-parallel-build-doc-alignment` | `reviews/task-26/029-667-c1-parallel-build-doc-alignment` | `task-26` | `parallel-build` |
| `review/668-c1-concurrent-dsm-worker-sweep` | `reviews/task-26/030-668-c1-concurrent-dsm-worker-sweep` | `task-26` | `parallel-build` |
| `review/669-c1-concurrent-dsm-real990k-scale` | `reviews/task-26/031-669-c1-concurrent-dsm-real990k-scale` | `task-26` | `parallel-build` |
| `review/670-c1-task26-worker-sweep-followup` | `reviews/task-26/032-670-c1-task26-worker-sweep-followup` | `task-26` | `task-token` |
| `review/671-c1-parallel-build-worker-timing-split` | `reviews/task-26/033-671-c1-parallel-build-worker-timing-split` | `task-26` | `parallel-build` |
| `review/672-c1-concurrent-dsm-w8-headroom` | `reviews/task-26/034-672-c1-concurrent-dsm-w8-headroom` | `task-26` | `parallel-build` |
| `review/30000-task28-ivf-design-freeze` | `reviews/task-28/001-30000-task28-ivf-design-freeze` | `task-28` | `task-token` |
| `review/30001-task28-ivf-am-scaffold` | `reviews/task-28/002-30001-task28-ivf-am-scaffold` | `task-28` | `task-token` |
| `review/30002-task28-ivf-empty-index` | `reviews/task-28/003-30002-task28-ivf-empty-index` | `task-28` | `task-token` |
| `review/30003-task28-ivf-page-codecs` | `reviews/task-28/004-30003-task28-ivf-page-codecs` | `task-28` | `task-token` |
| `review/30004-task28-ivf-spherical-trainer` | `reviews/task-28/005-30004-task28-ivf-spherical-trainer` | `task-28` | `task-token` |
| `review/30005-task28-ivf-training-sample` | `reviews/task-28/006-30005-task28-ivf-training-sample` | `task-28` | `task-token` |
| `review/30006-task28-ivf-bulk-assignment` | `reviews/task-28/007-30006-task28-ivf-bulk-assignment` | `task-28` | `task-token` |
| `review/30007-task28-ivf-populated-build-pages` | `reviews/task-28/008-30007-task28-ivf-populated-build-pages` | `task-28` | `task-token` |
| `review/30008-task28-ivf-build-stats` | `reviews/task-28/009-30008-task28-ivf-build-stats` | `task-28` | `task-token` |
| `review/30009-task28-ivf-build-smoke-coverage` | `reviews/task-28/010-30009-task28-ivf-build-smoke-coverage` | `task-28` | `task-token` |
| `review/30010-task28-ivf-query-prep` | `reviews/task-28/011-30010-task28-ivf-query-prep` | `task-28` | `task-token` |
| `review/30011-task28-ivf-probe-candidates` | `reviews/task-28/012-30011-task28-ivf-probe-candidates` | `task-28` | `task-token` |
| `review/30012-task28-ivf-result-emission` | `reviews/task-28/013-30012-task28-ivf-result-emission` | `task-28` | `task-token` |
| `review/30013-task28-ivf-bounded-probe-heap` | `reviews/task-28/014-30013-task28-ivf-bounded-probe-heap` | `task-28` | `task-token` |
| `review/30014-task28-ivf-v1-rerank-mode` | `reviews/task-28/015-30014-task28-ivf-v1-rerank-mode` | `task-28` | `task-token` |
| `review/30016-task28-ivf-live-insert` | `reviews/task-28/016-30016-task28-ivf-live-insert` | `task-28` | `task-token` |
| `review/30017-task28-ivf-empty-insert-bootstrap` | `reviews/task-28/017-30017-task28-ivf-empty-insert-bootstrap` | `task-28` | `task-token` |
| `review/30018-task28-ivf-same-list-live-insert` | `reviews/task-28/018-30018-task28-ivf-same-list-live-insert` | `task-28` | `task-token` |
| `review/30019-task28-ivf-duplicate-heap-tid` | `reviews/task-28/019-30019-task28-ivf-duplicate-heap-tid` | `task-28` | `task-token` |
| `review/30020-task28-ivf-vacuum-noop` | `reviews/task-28/020-30020-task28-ivf-vacuum-noop` | `task-28` | `task-token` |
| `review/30021-task28-ivf-vacuum-dead-postings` | `reviews/task-28/021-30021-task28-ivf-vacuum-dead-postings` | `task-28` | `task-token` |
| `review/30022-task28-ivf-vacuum-directory-repair` | `reviews/task-28/022-30022-task28-ivf-vacuum-directory-repair` | `task-28` | `task-token` |
| `review/30023-task28-ivf-drift-snapshot` | `reviews/task-28/023-30023-task28-ivf-drift-snapshot` | `task-28` | `task-token` |
| `review/30024-task28-ivf-vacuum-safety` | `reviews/task-28/024-30024-task28-ivf-vacuum-safety` | `task-28` | `task-token` |
| `review/30025-task28-ivf-shape-validation` | `reviews/task-28/025-30025-task28-ivf-shape-validation` | `task-28` | `task-token` |
| `review/30026-task28-ivf-concurrent-inserts` | `reviews/task-28/026-30026-task28-ivf-concurrent-inserts` | `task-28` | `task-token` |
| `review/30027-task28-ivf-admin-snapshot` | `reviews/task-28/027-30027-task28-ivf-admin-snapshot` | `task-28` | `task-token` |
| `review/30028-task28-ivf-cost-model` | `reviews/task-28/028-30028-task28-ivf-cost-model` | `task-28` | `task-token` |
| `review/30029-task28-ivf-explain-counters` | `reviews/task-28/029-30029-task28-ivf-explain-counters` | `task-28` | `task-token` |
| `review/30030-task28-ivf-pg18-planner-callbacks` | `reviews/task-28/030-30030-task28-ivf-pg18-planner-callbacks` | `task-28` | `task-token` |
| `review/30031-task28-ivf-pg18-stats` | `reviews/task-28/031-30031-task28-ivf-pg18-stats` | `task-28` | `task-token` |
| `review/30032-task28-ivf-readstream-postings` | `reviews/task-28/032-30032-task28-ivf-readstream-postings` | `task-28` | `task-token` |
| `review/30033-task28-ivf-pg18-validation` | `reviews/task-28/033-30033-task28-ivf-pg18-validation` | `task-28` | `task-token` |
| `review/30034-task28-ivf-optimization-handoff` | `reviews/task-28/034-30034-task28-ivf-optimization-handoff` | `task-28` | `task-token` |
| `review/30036-task28-ivf-anchor50k-nprobe-debug` | `reviews/task-28/035-30036-task28-ivf-anchor50k-nprobe-debug` | `task-28` | `task-token` |
| `review/30037-task28-ivf-fullprobe-scorer-alignment` | `reviews/task-28/036-30037-task28-ivf-fullprobe-scorer-alignment` | `task-28` | `task-token` |
| `review/30038-task28-ivf-heap-rerank-smoke` | `reviews/task-28/037-30038-task28-ivf-heap-rerank-smoke` | `task-28` | `task-token` |
| `review/30039-task28-ivf-rerank-width-sweep` | `reviews/task-28/038-30039-task28-ivf-rerank-width-sweep` | `task-28` | `task-token` |
| `review/30040-task28-ivf-nprobe-rerank-width-grid` | `reviews/task-28/039-30040-task28-ivf-nprobe-rerank-width-grid` | `task-28` | `task-token` |
| `review/30041-task28-ivf-nlists-routing-grid` | `reviews/task-28/040-30041-task28-ivf-nlists-routing-grid` | `task-28` | `task-token` |
| `review/30042-task28-ivf-anchor100-query-check` | `reviews/task-28/041-30042-task28-ivf-anchor100-query-check` | `task-28` | `task-token` |
| `review/30043-task28-ivf-anchor100-midprobe` | `reviews/task-28/042-30043-task28-ivf-anchor100-midprobe` | `task-28` | `task-token` |
| `review/30044-task28-ivf-anchor25k-candidate-check` | `reviews/task-28/043-30044-task28-ivf-anchor25k-candidate-check` | `task-28` | `task-token` |
| `review/30045-task28-ivf-anchor25k-routing-followup` | `reviews/task-28/044-30045-task28-ivf-anchor25k-routing-followup` | `task-28` | `task-token` |
| `review/30046-task28-ivf-initial-tuning-summary` | `reviews/task-28/045-30046-task28-ivf-initial-tuning-summary` | `task-28` | `task-token` |
| `review/30047-task28-ivf-review-followups` | `reviews/task-28/046-30047-task28-ivf-review-followups` | `task-28` | `task-token` |
| `review/30048-task28-ivf-rerank-prefetch-internal-score` | `reviews/task-28/047-30048-task28-ivf-rerank-prefetch-internal-score` | `task-28` | `task-token` |
| `review/30049-task28-ivf-quantizer-dispatch` | `reviews/task-28/048-30049-task28-ivf-quantizer-dispatch` | `task-28` | `task-token` |
| `review/30050-task28-ivf-candidate-dedup-pool` | `reviews/task-28/049-30050-task28-ivf-candidate-dedup-pool` | `task-28` | `task-token` |
| `review/30051-task28-ivf-postopt-smoke` | `reviews/task-28/050-30051-task28-ivf-postopt-smoke` | `task-28` | `task-token` |
| `review/30052-task28-ivf-nlists64-postopt` | `reviews/task-28/051-30052-task28-ivf-nlists64-postopt` | `task-28` | `task-token` |
| `review/30053-task28-ivf-nlists128-postopt` | `reviews/task-28/052-30053-task28-ivf-nlists128-postopt` | `task-28` | `task-token` |
| `review/30054-task28-ivf-nlists128-forced-index` | `reviews/task-28/053-30054-task28-ivf-nlists128-forced-index` | `task-28` | `task-token` |
| `review/30055-task28-ivf-rerank-width-postopt` | `reviews/task-28/054-30055-task28-ivf-rerank-width-postopt` | `task-28` | `task-token` |
| `review/30056-task28-ivf-build-training-vacuum-insert` | `reviews/task-28/055-30056-task28-ivf-build-training-vacuum-insert` | `task-28` | `task-token` |
| `review/30057-task28-ivf-insert-hotpath` | `reviews/task-28/056-30057-task28-ivf-insert-hotpath` | `task-28` | `task-token` |
| `review/30058-task28-ivf-cost-model-posting-scale` | `reviews/task-28/057-30058-task28-ivf-cost-model-posting-scale` | `task-28` | `task-token` |
| `review/30059-task28-ivf-insert-centroid-cache` | `reviews/task-28/058-30059-task28-ivf-insert-centroid-cache` | `task-28` | `task-token` |
| `review/30060-task28-ivf-insert-directory-tid-cache` | `reviews/task-28/059-30060-task28-ivf-insert-directory-tid-cache` | `task-28` | `task-token` |
| `review/30060-task28-ivf-insert-normalize-once` | `reviews/task-28/060-30060-task28-ivf-insert-normalize-once` | `task-28` | `task-token` |
| `review/30060-task28-ivf-insert-stream-duplicate-check` | `reviews/task-28/061-30060-task28-ivf-insert-stream-duplicate-check` | `task-28` | `task-token` |
| `review/30061-task28-ivf-insert-admin-snapshot-required` | `reviews/task-28/062-30061-task28-ivf-insert-admin-snapshot-required` | `task-28` | `task-token` |
| `review/30062-task28-ivf-insert-combined-stats-wal` | `reviews/task-28/063-30062-task28-ivf-insert-combined-stats-wal` | `task-28` | `task-token` |
| `review/30063-task28-ivf-insert-single-posting-encode` | `reviews/task-28/064-30063-task28-ivf-insert-single-posting-encode` | `task-28` | `task-token` |
| `review/30064-task28-ivf-insert-dimension-harness` | `reviews/task-28/065-30064-task28-ivf-insert-dimension-harness` | `task-28` | `task-token` |
| `review/30065-task28-ivf-insert-dim1536-baseline` | `reviews/task-28/066-30065-task28-ivf-insert-dim1536-baseline` | `task-28` | `task-token` |
| `review/30066-task28-ivf-insert-assign-without-normalize` | `reviews/task-28/067-30066-task28-ivf-insert-assign-without-normalize` | `task-28` | `task-token` |
| `review/30067-task28-ivf-prepared-quantizer-scan` | `reviews/task-28/068-30067-task28-ivf-prepared-quantizer-scan` | `task-28` | `task-token` |
| `review/30068-task28-ivf-prerank-topk` | `reviews/task-28/069-30068-task28-ivf-prerank-topk` | `task-28` | `task-token` |
| `review/30069-task28-ivf-borrowed-posting-scan` | `reviews/task-28/070-30069-task28-ivf-borrowed-posting-scan` | `task-28` | `task-token` |
| `review/30071-task28-ivf-n128-cost-diagnostic` | `reviews/task-28/071-30071-task28-ivf-n128-cost-diagnostic` | `task-28` | `task-token` |
| `review/30072-task28-ivf-frontier-prune` | `reviews/task-28/072-30072-task28-ivf-frontier-prune` | `task-28` | `task-token` |
| `review/30073-task28-ivf-turboquant-lut` | `reviews/task-28/073-30073-task28-ivf-turboquant-lut` | `task-28` | `task-token` |
| `review/30074-task28-ivf-typed-score-mode` | `reviews/task-28/074-30074-task28-ivf-typed-score-mode` | `task-28` | `task-token` |
| `review/30075-task28-ivf-quantizer-cache-audit` | `reviews/task-28/075-30075-task28-ivf-quantizer-cache-audit` | `task-28` | `task-token` |
| `review/30076-task28-ivf-cost-model-audit` | `reviews/task-28/076-30076-task28-ivf-cost-model-audit` | `task-28` | `task-token` |
| `review/30077-task28-ivf-planner-cross-matrix` | `reviews/task-28/077-30077-task28-ivf-planner-cross-matrix` | `task-28` | `task-token` |
| `review/30078-task28-ivf-score-bound-prune` | `reviews/task-28/078-30078-task28-ivf-score-bound-prune` | `task-28` | `task-token` |
| `review/30079-task28-ivf-streaming-vacuum` | `reviews/task-28/079-30079-task28-ivf-streaming-vacuum` | `task-28` | `task-token` |
| `review/30080-task28-ivf-vacuum-compaction` | `reviews/task-28/080-30080-task28-ivf-vacuum-compaction` | `task-28` | `task-token` |
| `review/30083-task28-ivf-pqfastscan-model-cache` | `reviews/task-28/081-30083-task28-ivf-pqfastscan-model-cache` | `task-28` | `task-token` |
| `review/30084-task28-ivf-quantizer-headtohead-smoke` | `reviews/task-28/082-30084-task28-ivf-quantizer-headtohead-smoke` | `task-28` | `task-token` |
| `review/30085-task28-ivf-pqfastscan-rerank-diagnostic` | `reviews/task-28/083-30085-task28-ivf-pqfastscan-rerank-diagnostic` | `task-28` | `task-token` |
| `review/30086-task28-ivf-pqfastscan-group-size` | `reviews/task-28/084-30086-task28-ivf-pqfastscan-group-size` | `task-28` | `task-token` |
| `review/30087-task28-ivf-pqfastscan-group-size-smoke` | `reviews/task-28/085-30087-task28-ivf-pqfastscan-group-size-smoke` | `task-28` | `task-token` |
| `review/30088-task28-ivf-pqfastscan-g8-25k-smoke` | `reviews/task-28/086-30088-task28-ivf-pqfastscan-g8-25k-smoke` | `task-28` | `task-token` |
| `review/30089-task28-ivf-pqfastscan-g8-rerank-narrowing` | `reviews/task-28/087-30089-task28-ivf-pqfastscan-g8-rerank-narrowing` | `task-28` | `task-token` |
| `review/30090-task28-ivf-pqfastscan-g8-100k-smoke` | `reviews/task-28/088-30090-task28-ivf-pqfastscan-g8-100k-smoke` | `task-28` | `task-token` |
| `review/30091-task28-ivf-100k-pqfastscan-turboquant-comparison` | `reviews/task-28/089-30091-task28-ivf-100k-pqfastscan-turboquant-comparison` | `task-28` | `task-token` |
| `review/30092-task28-ivf-pqfastscan-g8-100k-nlists128` | `reviews/task-28/090-30092-task28-ivf-pqfastscan-g8-100k-nlists128` | `task-28` | `task-token` |
| `review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank` | `reviews/task-28/091-30093-task28-ivf-pqfastscan-g8-100k-n128-rerank` | `task-28` | `task-token` |
| `review/30094-task28-ivf-pqfastscan-g8-100k-n128-nprobe-middle` | `reviews/task-28/092-30094-task28-ivf-pqfastscan-g8-100k-n128-nprobe-middle` | `task-28` | `task-token` |
| `review/30095-task28-ivf-pqfastscan-g8-100k-nlists256` | `reviews/task-28/093-30095-task28-ivf-pqfastscan-g8-100k-nlists256` | `task-28` | `task-token` |
| `review/30096-task28-ivf-current-auto-recommendation` | `reviews/task-28/094-30096-task28-ivf-current-auto-recommendation` | `task-28` | `task-token` |
| `review/30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh` | `reviews/task-28/095-30097-task28-ivf-pqfastscan-g8-10k-25k-a10-refresh` | `task-28` | `task-token` |
| `review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128` | `reviews/task-28/096-30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128` | `task-28` | `task-token` |
| `review/30099-task28-ivf-rerank-width-session-guc` | `reviews/task-28/097-30099-task28-ivf-rerank-width-session-guc` | `task-28` | `task-token` |
| `review/30100-task28-ivf-rerank-width-cli-flag` | `reviews/task-28/098-30100-task28-ivf-rerank-width-cli-flag` | `task-28` | `task-token` |
| `review/30101-task28-ivf-rerank-width-flag-smoke` | `reviews/task-28/099-30101-task28-ivf-rerank-width-flag-smoke` | `task-28` | `task-token` |
| `review/30102-task28-ivf-a4-a5-typed-cache-audit` | `reviews/task-28/100-30102-task28-ivf-a4-a5-typed-cache-audit` | `task-28` | `task-token` |
| `review/30103-task28-ivf-vacuum-posting-page-compaction` | `reviews/task-28/101-30103-task28-ivf-vacuum-posting-page-compaction` | `task-28` | `task-token` |
| `review/30104-task28-ivf-vacuum-churn-smoke` | `reviews/task-28/102-30104-task28-ivf-vacuum-churn-smoke` | `task-28` | `task-token` |
| `review/30105-task28-ivf-vacuum-range-reuse-smoke` | `reviews/task-28/103-30105-task28-ivf-vacuum-range-reuse-smoke` | `task-28` | `task-token` |
| `review/30106-task28-ivf-vacuum-replacement-reuse-smoke` | `reviews/task-28/104-30106-task28-ivf-vacuum-replacement-reuse-smoke` | `task-28` | `task-token` |
| `review/30107-task28-ivf-empty-range-reuse-smoke` | `reviews/task-28/105-30107-task28-ivf-empty-range-reuse-smoke` | `task-28` | `task-token` |
| `review/30108-task28-ivf-vacuum-scale-harness` | `reviews/task-28/106-30108-task28-ivf-vacuum-scale-harness` | `task-28` | `task-token` |
| `review/30109-task28-ivf-a2-1m-vacuum-scale` | `reviews/task-28/107-30109-task28-ivf-a2-1m-vacuum-scale` | `task-28` | `task-token` |
| `review/30110-task28-ivf-selected-list-live-stop` | `reviews/task-28/108-30110-task28-ivf-selected-list-live-stop` | `task-28` | `task-token` |
| `review/30111-task28-ivf-a9-100k-current-head` | `reviews/task-28/109-30111-task28-ivf-a9-100k-current-head` | `task-28` | `task-token` |
| `review/30115-task28-ivf-pqfastscan-bound-prune` | `reviews/task-28/110-30115-task28-ivf-pqfastscan-bound-prune` | `task-28` | `task-token` |
| `review/30116-task28-ivf-pqfastscan-bound-prune-smoke` | `reviews/task-28/111-30116-task28-ivf-pqfastscan-bound-prune-smoke` | `task-28` | `task-token` |
| `review/30118-task28-ivf-a9-current-size-cache` | `reviews/task-28/112-30118-task28-ivf-a9-current-size-cache` | `task-28` | `task-token` |
| `review/30119-task28-ivf-a9-100k-current-build` | `reviews/task-28/113-30119-task28-ivf-a9-100k-current-build` | `task-28` | `task-token` |
| `review/30120-task28-ivf-vacuum-sustained-churn` | `reviews/task-28/114-30120-task28-ivf-vacuum-sustained-churn` | `task-28` | `task-token` |
| `review/30121-task28-ivf-fsm-range-reuse` | `reviews/task-28/115-30121-task28-ivf-fsm-range-reuse` | `task-28` | `task-token` |
| `review/30122-task28-ivf-vacuum-free-block-set` | `reviews/task-28/116-30122-task28-ivf-vacuum-free-block-set` | `task-28` | `task-token` |
| `review/30123-task28-ivf-vacuum-cursor-reuse` | `reviews/task-28/117-30123-task28-ivf-vacuum-cursor-reuse` | `task-28` | `task-token` |
| `review/30124-task28-ivf-vacuum-same-slice-churn` | `reviews/task-28/118-30124-task28-ivf-vacuum-same-slice-churn` | `task-28` | `task-token` |
| `review/30125-task28-ivf-adjacent-page-reuse` | `reviews/task-28/119-30125-task28-ivf-adjacent-page-reuse` | `task-28` | `task-token` |
| `review/30126-task28-ivf-a9-100k-current-refresh` | `reviews/task-28/120-30126-task28-ivf-a9-100k-current-refresh` | `task-28` | `task-token` |
| `review/30127-task28-ivf-a10-current-recommendation` | `reviews/task-28/121-30127-task28-ivf-a10-current-recommendation` | `task-28` | `task-token` |
| `review/30128-task28-ivf-a9-remaining-inventory` | `reviews/task-28/122-30128-task28-ivf-a9-remaining-inventory` | `task-28` | `task-token` |
| `review/30129-task28-ivf-a2-vacuum-scale` | `reviews/task-28/123-30129-task28-ivf-a2-vacuum-scale` | `task-28` | `task-token` |
| `review/30130-task28-ivf-a9-990k-ivf-surface` | `reviews/task-28/124-30130-task28-ivf-a9-990k-ivf-surface` | `task-28` | `task-token` |
| `review/30131-task28-ivf-current-gate-status` | `reviews/task-28/125-30131-task28-ivf-current-gate-status` | `task-28` | `task-token` |
| `review/30134-task28-ivf-scan-volume-counters` | `reviews/task-28/126-30134-task28-ivf-scan-volume-counters` | `task-28` | `task-token` |
| `review/30135-task28-ivf-990k-scan-volume` | `reviews/task-28/127-30135-task28-ivf-990k-scan-volume` | `task-28` | `task-token` |
| `review/30136-task28-ivf-990k-rerank-width` | `reviews/task-28/128-30136-task28-ivf-990k-rerank-width` | `task-28` | `task-token` |
| `review/30137-task28-ivf-a7-10k-25k-bound-closure` | `reviews/task-28/129-30137-task28-ivf-a7-10k-25k-bound-closure` | `task-28` | `task-token` |
| `review/30138-task28-ivf-bound-prune-cleanups` | `reviews/task-28/130-30138-task28-ivf-bound-prune-cleanups` | `task-28` | `task-token` |
| `review/30139-task28-ivf-a3-page-ownership-diagnostic` | `reviews/task-28/131-30139-task28-ivf-a3-page-ownership-diagnostic` | `task-28` | `task-token` |
| `review/30140-task28-ivf-a3-list-segregated-build` | `reviews/task-28/132-30140-task28-ivf-a3-list-segregated-build` | `task-28` | `task-token` |
| `review/30141-task28-ivf-a3-100k-sustained-churn` | `reviews/task-28/133-30141-task28-ivf-a3-100k-sustained-churn` | `task-28` | `task-token` |
| `review/30142-task28-ivf-a3-posting-slack-churn` | `reviews/task-28/134-30142-task28-ivf-a3-posting-slack-churn` | `task-28` | `task-token` |
| `review/30143-task28-ivf-a10-turboquant-memory` | `reviews/task-28/135-30143-task28-ivf-a10-turboquant-memory` | `task-28` | `task-token` |
| `review/30144-task28-ivf-a10-rabitq-fill` | `reviews/task-28/136-30144-task28-ivf-a10-rabitq-fill` | `task-28` | `task-token` |
| `review/30145-task28-ivf-a10-current-closure` | `reviews/task-28/137-30145-task28-ivf-a10-current-closure` | `task-28` | `task-token` |
| `review/30146-task28-ivf-a9-100k-hnsw-reference` | `reviews/task-28/138-30146-task28-ivf-a9-100k-hnsw-reference` | `task-28` | `task-token` |
| `review/30150-task28-ivf-local-990k-deferral` | `reviews/task-28/139-30150-task28-ivf-local-990k-deferral` | `task-28` | `task-token` |
| `review/30151-task28-ivf-local-landing-status` | `reviews/task-28/140-30151-task28-ivf-local-landing-status` | `task-28` | `task-token` |
| `review/30152-task28-ivf-rabitq-score-hotpath` | `reviews/task-28/141-30152-task28-ivf-rabitq-score-hotpath` | `task-28` | `task-token` |
| `review/30153-task28-ivf-h-i-cleanups` | `reviews/task-28/142-30153-task28-ivf-h-i-cleanups` | `task-28` | `task-token` |
| `review/674-c1-task29-diskann-rebase-verification` | `reviews/task-29/001-674-c1-task29-diskann-rebase-verification` | `task-29` | `task-token` |
| `review/675-c1-task29-diskann-real10k-baseline` | `reviews/task-29/002-675-c1-task29-diskann-real10k-baseline` | `task-29` | `task-token` |
| `review/676-c1-task29-isolated-real10k-baseline` | `reviews/task-29/003-676-c1-task29-isolated-real10k-baseline` | `task-29` | `task-token` |
| `review/677-c1-task29-diskann-alpha20-probe` | `reviews/task-29/004-677-c1-task29-diskann-alpha20-probe` | `task-29` | `task-token` |
| `review/678-c1-task29-diskann-prior-neighbor-probe` | `reviews/task-29/005-678-c1-task29-diskann-prior-neighbor-probe` | `task-29` | `task-token` |
| `review/11087-task29-diskann-graph-diagnostics` | `reviews/task-29/006-11087-task29-diskann-graph-diagnostics` | `task-29` | `task-token` |
| `review/11088-task29-diskann-seeded-build-probe` | `reviews/task-29/007-11088-task29-diskann-seeded-build-probe` | `task-29` | `task-token` |
| `review/11089-task29-diskann-build-probe` | `reviews/task-29/008-11089-task29-diskann-build-probe` | `task-29` | `task-token` |
| `review/11090-task29-diskann-pass1-sample-probe` | `reviews/task-29/009-11090-task29-diskann-pass1-sample-probe` | `task-29` | `task-token` |
| `review/11091-task29-diskann-sql-vs-memory-compare` | `reviews/task-29/010-11091-task29-diskann-sql-vs-memory-compare` | `task-29` | `task-token` |
| `review/11092-task29-diskann-rerank-budget-probe` | `reviews/task-29/011-11092-task29-diskann-rerank-budget-probe` | `task-29` | `task-token` |
| `review/11093-task29-diskann-rerank200-probe` | `reviews/task-29/012-11093-task29-diskann-rerank200-probe` | `task-29` | `task-token` |
| `review/11094-task29-diskann-grouped-frontier-probe` | `reviews/task-29/013-11094-task29-diskann-grouped-frontier-probe` | `task-29` | `task-token` |
| `review/11095-task29-diskann-pgvectorscale-comparison` | `reviews/task-29/014-11095-task29-diskann-pgvectorscale-comparison` | `task-29` | `task-token` |
| `review/11099-task29-diskann-landing-readiness` | `reviews/task-29/015-11099-task29-diskann-landing-readiness` | `task-29` | `task-token` |
| `review/11103-task29-landing-readiness-refresh` | `reviews/task-29/016-11103-task29-landing-readiness-refresh` | `task-29` | `task-token` |
| `review/30204-task29-diskann-m5-neon-rerank` | `reviews/task-29/017-30204-task29-diskann-m5-neon-rerank` | `task-29` | `task-token` |
| `review/30205-task29-diskann-m5-rerank-heap-order` | `reviews/task-29/018-30205-task29-diskann-m5-rerank-heap-order` | `task-29` | `task-token` |
| `review/30206-task29-diskann-m5-rerank-prefetch` | `reviews/task-29/019-30206-task29-diskann-m5-rerank-prefetch` | `task-29` | `task-token` |
| `review/30207-task29-diskann-m5-decision` | `reviews/task-29/020-30207-task29-diskann-m5-decision` | `task-29` | `task-token` |
| `review/30208-task29-diskann-m5-build-neon-followup` | `reviews/task-29/021-30208-task29-diskann-m5-build-neon-followup` | `task-29` | `task-token` |
| `review/30209-task29-diskann-m5-cold-cache-100k` | `reviews/task-29/022-30209-task29-diskann-m5-cold-cache-100k` | `task-29` | `task-token` |
| `review/11096-task29a-diskann-binary-sidecar-prefilter` | `reviews/task-29a/001-11096-task29a-diskann-binary-sidecar-prefilter` | `task-29a` | `task-token` |
| `review/11100-task29b-diskann-vacuum-prefilter-consistency` | `reviews/task-29b/001-11100-task29b-diskann-vacuum-prefilter-consistency` | `task-29b` | `task-token` |
| `review/11106-task29d-build-heap-frontier-ab` | `reviews/task-29d/001-11106-task29d-build-heap-frontier-ab` | `task-29d` | `task-token` |
| `review/11108-task29d-build-distance-simd` | `reviews/task-29d/002-11108-task29d-build-distance-simd` | `task-29d` | `task-token` |
| `review/11109-task29d-final-readiness` | `reviews/task-29d/003-11109-task29d-final-readiness` | `task-29d` | `task-token` |
| `review/11110-task29e-rerank-borrowed-simd` | `reviews/task-29e/001-11110-task29e-rerank-borrowed-simd` | `task-29e` | `task-token` |
| `review/679-c1-spire-phase12c-customscan-callback-pins` | `reviews/task-30/001-679-c1-spire-phase12c-customscan-callback-pins` | `task-30` | `spire` |
| `review/680-c1-spire-typed-tuple-edge-coverage` | `reviews/task-30/002-680-c1-spire-typed-tuple-edge-coverage` | `task-30` | `spire` |
| `review/681-c1-spire-stage-e-contract-boundary` | `reviews/task-30/003-681-c1-spire-stage-e-contract-boundary` | `task-30` | `spire` |
| `review/682-c1-spire-remote-heap-score-coverage` | `reviews/task-30/004-682-c1-spire-remote-heap-score-coverage` | `task-30` | `spire` |
| `review/683-c1-spire-strict-receive-category-coverage` | `reviews/task-30/005-683-c1-spire-strict-receive-category-coverage` | `task-30` | `spire` |
| `review/684-c1-spire-customscan-json-explain-tightening` | `reviews/task-30/006-684-c1-spire-customscan-json-explain-tightening` | `task-30` | `spire` |
| `review/685-c1-spire-scan-data-shape-edges` | `reviews/task-30/007-685-c1-spire-scan-data-shape-edges` | `task-30` | `spire` |
| `review/686-c1-spire-no-active-epoch-planner-fallback` | `reviews/task-30/008-686-c1-spire-no-active-epoch-planner-fallback` | `task-30` | `spire` |
| `review/687-c1-spire-dml-numeric-pk-predicate-rejection` | `reviews/task-30/009-687-c1-spire-dml-numeric-pk-predicate-rejection` | `task-30` | `spire` |
| `review/688-c1-spire-customscan-json-node-counters` | `reviews/task-30/010-688-c1-spire-customscan-json-node-counters` | `task-30` | `spire` |
| `review/689-c1-spire-local-store-execution-snapshot` | `reviews/task-30/011-689-c1-spire-local-store-execution-snapshot` | `task-30` | `spire` |
| `review/690-c1-spire-dropped-index-diagnostics` | `reviews/task-30/012-690-c1-spire-dropped-index-diagnostics` | `task-30` | `spire` |
| `review/691-c1-spire-numerical-edge-scan-coverage` | `reviews/task-30/013-691-c1-spire-numerical-edge-scan-coverage` | `task-30` | `spire` |
| `review/692-c1-spire-customscan-selected-pid-payloads` | `reviews/task-30/014-692-c1-spire-customscan-selected-pid-payloads` | `task-30` | `spire` |
| `review/693-c1-spire-multistore-scan-widths` | `reviews/task-30/015-693-c1-spire-multistore-scan-widths` | `task-30` | `spire` |
| `review/694-c1-spire-dml-non-pk-select-pass-through` | `reviews/task-30/016-694-c1-spire-dml-non-pk-select-pass-through` | `task-30` | `spire` |
| `review/695-c1-spire-dml-composite-pk-rejection` | `reviews/task-30/017-695-c1-spire-dml-composite-pk-rejection` | `task-30` | `spire` |
| `review/696-c1-spire-dml-float-pk-rejection` | `reviews/task-30/018-696-c1-spire-dml-float-pk-rejection` | `task-30` | `spire` |
| `review/697-c1-spire-text-nul-projection-boundary` | `reviews/task-30/019-697-c1-spire-text-nul-projection-boundary` | `task-30` | `spire` |
| `review/698-c1-spire-customscan-json-explain-field-set` | `reviews/task-30/020-698-c1-spire-customscan-json-explain-field-set` | `task-30` | `spire` |
| `review/699-c1-spire-wide-typed-payload-projection` | `reviews/task-30/021-699-c1-spire-wide-typed-payload-projection` | `task-30` | `spire` |
| `review/700-c1-spire-large-typed-payload-text` | `reviews/task-30/022-700-c1-spire-large-typed-payload-text` | `task-30` | `spire` |
| `review/701-c1-spire-payload-batch-preflight-caps` | `reviews/task-30/023-701-c1-spire-payload-batch-preflight-caps` | `task-30` | `spire` |
| `review/702-c1-spire-idempotent-remote-delete-payload` | `reviews/task-30/024-702-c1-spire-idempotent-remote-delete-payload` | `task-30` | `spire` |
| `review/703-c1-spire-cost-tuning-model-coverage` | `reviews/task-30/025-703-c1-spire-cost-tuning-model-coverage` | `task-30` | `spire` |
| `review/704-c1-spire-prepared-xact-intent-transitions` | `reviews/task-30/026-704-c1-spire-prepared-xact-intent-transitions` | `task-30` | `spire` |
| `review/706-c1-spire-cost-tuning-snapshot-gucs` | `reviews/task-30/027-706-c1-spire-cost-tuning-snapshot-gucs` | `task-30` | `spire` |
| `review/707-c1-spire-remote-oom-transport-fault` | `reviews/task-30/028-707-c1-spire-remote-oom-transport-fault` | `task-30` | `spire` |
| `review/708-c1-spire-prepared-xact-reaper-in-doubt` | `reviews/task-30/029-708-c1-spire-prepared-xact-reaper-in-doubt` | `task-30` | `spire` |
| `review/709-c1-spire-network-partition-transport-fault` | `reviews/task-30/030-709-c1-spire-network-partition-transport-fault` | `task-30` | `spire` |
| `review/710-c1-spire-drop-index-pre-dispatch-lifecycle` | `reviews/task-30/031-710-c1-spire-drop-index-pre-dispatch-lifecycle` | `task-30` | `spire` |
| `review/711-c1-spire-remote-timeout-matrix-actions` | `reviews/task-30/032-711-c1-spire-remote-timeout-matrix-actions` | `task-30` | `spire` |
| `review/712-c1-spire-reindex-pre-dispatch-lifecycle` | `reviews/task-30/033-712-c1-spire-reindex-pre-dispatch-lifecycle` | `task-30` | `spire` |
| `review/713-c1-spire-inflight-lifecycle-faults` | `reviews/task-30/034-713-c1-spire-inflight-lifecycle-faults` | `task-30` | `spire` |
| `review/714-c1-spire-transport-cancel-matrix-actions` | `reviews/task-30/035-714-c1-spire-transport-cancel-matrix-actions` | `task-30` | `spire` |
| `review/715-c1-spire-cic-descriptor-defer-receive` | `reviews/task-30/036-715-c1-spire-cic-descriptor-defer-receive` | `task-30` | `spire` |
| `review/716-c1-spire-12c-tracker-reconciliation` | `reviews/task-30/037-716-c1-spire-12c-tracker-reconciliation` | `task-30` | `spire` |
| `review/717-c1-spire-tuple-transport-heap-matrix-actions` | `reviews/task-30/038-717-c1-spire-tuple-transport-heap-matrix-actions` | `task-30` | `spire` |
| `review/718-c1-spire-concurrent-delete-collision` | `reviews/task-30/039-718-c1-spire-concurrent-delete-collision` | `task-30` | `spire` |
| `review/719-c1-spire-delete-idempotency-no-redispatch` | `reviews/task-30/040-719-c1-spire-delete-idempotency-no-redispatch` | `task-30` | `spire` |
| `review/720-c1-spire-dml-frontdoor-tracker-reconciliation` | `reviews/task-30/041-720-c1-spire-dml-frontdoor-tracker-reconciliation` | `task-30` | `spire` |
| `review/721-c1-spire-stage-e-contract-doc-tracker` | `reviews/task-30/042-721-c1-spire-stage-e-contract-doc-tracker` | `task-30` | `spire` |
| `review/722-c1-spire-sign-convention-tracker-reconciliation` | `reviews/task-30/043-722-c1-spire-sign-convention-tracker-reconciliation` | `task-30` | `spire` |
| `review/723-c1-spire-customscan-cost-ratio-tracker` | `reviews/task-30/044-723-c1-spire-customscan-cost-ratio-tracker` | `task-30` | `spire` |
| `review/724-c1-spire-selected-pid-tracker-reconciliation` | `reviews/task-30/045-724-c1-spire-selected-pid-tracker-reconciliation` | `task-30` | `spire` |
| `review/725-c1-spire-cost-guc-explain-reflection` | `reviews/task-30/046-725-c1-spire-cost-guc-explain-reflection` | `task-30` | `spire` |
| `review/726-c1-spire-explain-tracker-reconciliation` | `reviews/task-30/047-726-c1-spire-explain-tracker-reconciliation` | `task-30` | `spire` |
| `review/727-c1-spire-typed-tuple-tracker-reconciliation` | `reviews/task-30/048-727-c1-spire-typed-tuple-tracker-reconciliation` | `task-30` | `spire` |
| `review/728-c1-spire-diagnostics-shape-tracker-reconciliation` | `reviews/task-30/049-728-c1-spire-diagnostics-shape-tracker-reconciliation` | `task-30` | `spire` |
| `review/729-c1-spire-customscan-wide-projection` | `reviews/task-30/050-729-c1-spire-customscan-wide-projection` | `task-30` | `spire` |
| `review/730-c1-spire-multistore-counter-sums` | `reviews/task-30/051-730-c1-spire-multistore-counter-sums` | `task-30` | `spire` |
| `review/731-c1-spire-customscan-large-text-projection` | `reviews/task-30/052-731-c1-spire-customscan-large-text-projection` | `task-30` | `spire` |
| `review/732-c1-spire-customscan-idle-cursor-timeout` | `reviews/task-30/053-732-c1-spire-customscan-idle-cursor-timeout` | `task-30` | `spire` |
| `review/733-c1-spire-customscan-remote-restart-rejoin` | `reviews/task-30/054-733-c1-spire-customscan-remote-restart-rejoin` | `task-30` | `spire` |
| `review/734-c1-spire-customscan-coordinator-drop-index-lock` | `reviews/task-30/055-734-c1-spire-customscan-coordinator-drop-index-lock` | `task-30` | `spire` |
| `review/735-c1-spire-descriptor-race-tightening` | `reviews/task-30/056-735-c1-spire-descriptor-race-tightening` | `task-30` | `spire` |
| `review/736-c1-spire-customscan-multiremote-fanout` | `reviews/task-30/057-736-c1-spire-customscan-multiremote-fanout` | `task-30` | `spire` |
| `review/737-c1-spire-dml-schema-drift-split` | `reviews/task-30/058-737-c1-spire-dml-schema-drift-split` | `task-30` | `spire` |
| `review/738-c1-spire-empty-customscan-cleanup` | `reviews/task-30/059-738-c1-spire-empty-customscan-cleanup` | `task-30` | `spire` |
| `review/740-c1-spire-cic-refreshed-customscan` | `reviews/task-30/060-740-c1-spire-cic-refreshed-customscan` | `task-30` | `spire` |
| `review/741-c1-spire-customscan-executor-positive` | `reviews/task-30/061-741-c1-spire-customscan-executor-positive` | `task-30` | `spire` |
| `review/742-c1-spire-customscan-cursor-rescan` | `reviews/task-30/062-742-c1-spire-customscan-cursor-rescan` | `task-30` | `spire` |
| `review/743-c1-spire-customscan-dml-panic-cleanup` | `reviews/task-30/063-743-c1-spire-customscan-dml-panic-cleanup` | `task-30` | `spire` |
| `review/744-c1-spire-stage-e-fault-tracker-reconciliation` | `reviews/task-30/064-744-c1-spire-stage-e-fault-tracker-reconciliation` | `task-30` | `spire` |
| `review/745-c1-spire-tuple-transport-retired-live` | `reviews/task-30/065-745-c1-spire-tuple-transport-retired-live` | `task-30` | `spire` |
| `review/746-c1-spire-payload-too-large-strict-reconciliation` | `reviews/task-30/066-746-c1-spire-payload-too-large-strict-reconciliation` | `task-30` | `spire` |
| `review/747-c1-spire-customscan-cancel-cleanup` | `reviews/task-30/067-747-c1-spire-customscan-cancel-cleanup` | `task-30` | `spire` |
| `review/748-c1-spire-customscan-local-statement-timeout` | `reviews/task-30/068-748-c1-spire-customscan-local-statement-timeout` | `task-30` | `spire` |
| `review/749-c1-spire-customscan-planner-exclusions` | `reviews/task-30/069-749-c1-spire-customscan-planner-exclusions` | `task-30` | `spire` |
| `review/750-c1-spire-storage-snapshot-reindex` | `reviews/task-30/070-750-c1-spire-storage-snapshot-reindex` | `task-30` | `spire` |
| `review/751-c1-spire-stage-e-matrix-reconciliation` | `reviews/task-30/071-751-c1-spire-stage-e-matrix-reconciliation` | `task-30` | `spire` |
| `review/752-c1-spire-degraded-payload-cap-counter` | `reviews/task-30/072-752-c1-spire-degraded-payload-cap-counter` | `task-30` | `spire` |
| `review/753-c1-spire-stale-manifest-endpoint-status` | `reviews/task-30/073-753-c1-spire-stale-manifest-endpoint-status` | `task-30` | `spire` |
| `review/754-c1-spire-stale-read-recheck-cross-reference` | `reviews/task-30/074-754-c1-spire-stale-read-recheck-cross-reference` | `task-30` | `spire` |
| `review/755-c1-spire-customscan-memory-context-cleanup` | `reviews/task-30/075-755-c1-spire-customscan-memory-context-cleanup` | `task-30` | `spire` |
| `review/756-c1-spire-select-for-update-stale-read` | `reviews/task-30/076-756-c1-spire-select-for-update-stale-read` | `task-30` | `spire` |
| `review/757-c1-spire-dropped-index-no-descriptor-refresh` | `reviews/task-30/077-757-c1-spire-dropped-index-no-descriptor-refresh` | `task-30` | `spire` |
| `review/758-c1-spire-read-schema-drift-scope-deferral` | `reviews/task-30/078-758-c1-spire-read-schema-drift-scope-deferral` | `task-30` | `spire` |
| `review/759-c1-spire-read-schema-drift-phase13-gate` | `reviews/task-30/079-759-c1-spire-read-schema-drift-phase13-gate` | `task-30` | `spire` |
| `review/760-c1-spire-test-aggregator-split` | `reviews/task-30/080-760-c1-spire-test-aggregator-split` | `task-30` | `spire` |
| `review/761-c1-spire-remote-search-contract-split` | `reviews/task-30/081-761-c1-spire-remote-search-contract-split` | `task-30` | `spire` |
| `review/762-c1-spire-phase12c-closeout-audit` | `reviews/task-30/082-762-c1-spire-phase12c-closeout-audit` | `task-30` | `spire` |
| `review/763-c1-spire-read-schema-drift-guard` | `reviews/task-30/083-763-c1-spire-read-schema-drift-guard` | `task-30` | `spire` |
| `review/764-c1-spire-phase12c-final-closeout` | `reviews/task-30/084-764-c1-spire-phase12c-final-closeout` | `task-30` | `spire` |
| `review/766-c1-spire-phase13d-read-efficiency-observability` | `reviews/task-30/085-766-c1-spire-phase13d-read-efficiency-observability` | `task-30` | `spire` |
| `review/external/2026-05-09-phase-7-8-final-review` | `reviews/task-30/086-2026-05-09-phase-7-8-final-review` | `task-30` | `external-task30` |
| `review/external/2026-05-09-phase-9-closeout-requirements` | `reviews/task-30/087-2026-05-09-phase-9-closeout-requirements` | `task-30` | `external-task30` |
| `review/30159-spire-ivf-foundation-adr` | `reviews/task-30/088-30159-spire-ivf-foundation-adr` | `task-30` | `spire` |
| `review/30160-spire-task-plan` | `reviews/task-30/089-30160-spire-task-plan` | `task-30` | `spire` |
| `review/30161-spire-partition-object-design` | `reviews/task-30/090-30161-spire-partition-object-design` | `task-30` | `spire` |
| `review/30162-spire-phase0-partition-object-storage` | `reviews/task-30/091-30162-spire-phase0-partition-object-storage` | `task-30` | `spire` |
| `review/30163-spire-am-scaffold` | `reviews/task-30/092-30163-spire-am-scaffold` | `task-30` | `spire` |
| `review/30164-spire-storage-codecs` | `reviews/task-30/093-30164-spire-storage-codecs` | `task-30` | `spire` |
| `review/30165-spire-metadata-codecs` | `reviews/task-30/094-30165-spire-metadata-codecs` | `task-30` | `spire` |
| `review/30166-spire-leaf-object-codec` | `reviews/task-30/095-30166-spire-leaf-object-codec` | `task-30` | `spire` |
| `review/30167-spire-placement-directory-codec` | `reviews/task-30/096-30167-spire-placement-directory-codec` | `task-30` | `spire` |
| `review/30168-spire-local-object-store` | `reviews/task-30/097-30168-spire-local-object-store` | `task-30` | `spire` |
| `review/30169-spire-object-manifest-codec` | `reviews/task-30/098-30169-spire-object-manifest-codec` | `task-30` | `spire` |
| `review/30170-spire-local-vec-id-allocator` | `reviews/task-30/099-30170-spire-local-vec-id-allocator` | `task-30` | `spire` |
| `review/30171-spire-primary-assignment-builder` | `reviews/task-30/100-30171-spire-primary-assignment-builder` | `task-30` | `spire` |
| `review/30172-spire-root-control-state-codec` | `reviews/task-30/101-30172-spire-root-control-state-codec` | `task-30` | `spire` |
| `review/30173-spire-pid-allocator` | `reviews/task-30/102-30173-spire-pid-allocator` | `task-30` | `spire` |
| `review/30174-spire-published-epoch-snapshot` | `reviews/task-30/103-30174-spire-published-epoch-snapshot` | `task-30` | `spire` |
| `review/30175-spire-single-level-build-draft` | `reviews/task-30/104-30175-spire-single-level-build-draft` | `task-30` | `spire` |
| `review/30176-spire-draft-root-control-bridge` | `reviews/task-30/105-30176-spire-draft-root-control-bridge` | `task-30` | `spire` |
| `review/30177-spire-epoch-cleanup-eligibility` | `reviews/task-30/106-30177-spire-epoch-cleanup-eligibility` | `task-30` | `spire` |
| `review/30178-spire-snapshot-leaf-row-collector` | `reviews/task-30/107-30178-spire-snapshot-leaf-row-collector` | `task-30` | `spire` |
| `review/30179-spire-visible-primary-scan-filter` | `reviews/task-30/108-30179-spire-visible-primary-scan-filter` | `task-30` | `spire` |
| `review/30180-spire-epoch-cleanup-planner` | `reviews/task-30/109-30180-spire-epoch-cleanup-planner` | `task-30` | `spire` |
| `review/30181-spire-build-draft-manifest-bundle` | `reviews/task-30/110-30181-spire-build-draft-manifest-bundle` | `task-30` | `spire` |
| `review/30182-spire-encoded-publish-bundle` | `reviews/task-30/111-30182-spire-encoded-publish-bundle` | `task-30` | `spire` |
| `review/30183-spire-delta-partition-object-codec` | `reviews/task-30/112-30183-spire-delta-partition-object-codec` | `task-30` | `spire` |
| `review/30184-spire-local-delta-object-store` | `reviews/task-30/113-30184-spire-local-delta-object-store` | `task-30` | `spire` |
| `review/30185-spire-delta-assignment-builders` | `reviews/task-30/114-30185-spire-delta-assignment-builders` | `task-30` | `spire` |
| `review/30186-spire-delta-epoch-draft-helper` | `reviews/task-30/115-30186-spire-delta-epoch-draft-helper` | `task-30` | `spire` |
| `review/30187-spire-delta-draft-snapshot-carry-forward` | `reviews/task-30/116-30187-spire-delta-draft-snapshot-carry-forward` | `task-30` | `spire` |
| `review/30188-spire-scan-object-kind-dispatch` | `reviews/task-30/117-30188-spire-scan-object-kind-dispatch` | `task-30` | `spire` |
| `review/30189-spire-visible-delta-scan-overlay` | `reviews/task-30/118-30189-spire-visible-delta-scan-overlay` | `task-30` | `spire` |
| `review/30190-spire-delta-publish-bundle` | `reviews/task-30/119-30190-spire-delta-publish-bundle` | `task-30` | `spire` |
| `review/30191-spire-local-vec-id-zero-guard` | `reviews/task-30/120-30191-spire-local-vec-id-zero-guard` | `task-30` | `spire` |
| `review/30192-spire-delta-draft-base-vec-id-observation` | `reviews/task-30/121-30192-spire-delta-draft-base-vec-id-observation` | `task-30` | `spire` |
| `review/30193-spire-delta-draft-epoch-order` | `reviews/task-30/122-30193-spire-delta-draft-epoch-order` | `task-30` | `spire` |
| `review/30194-spire-delete-delta-target-validation` | `reviews/task-30/123-30194-spire-delete-delta-target-validation` | `task-30` | `spire` |
| `review/30195-spire-delete-delta-visible-target-validation` | `reviews/task-30/124-30195-spire-delete-delta-visible-target-validation` | `task-30` | `spire` |
| `review/30196-spire-delete-delta-duplicate-target-guard` | `reviews/task-30/125-30196-spire-delete-delta-duplicate-target-guard` | `task-30` | `spire` |
| `review/30197-spire-delete-delta-row-locator-validation` | `reviews/task-30/126-30197-spire-delete-delta-row-locator-validation` | `task-30` | `spire` |
| `review/30198-spire-delta-leaf-base-guard` | `reviews/task-30/127-30198-spire-delta-leaf-base-guard` | `task-30` | `spire` |
| `review/30199-spire-delta-object-vec-id-uniqueness` | `reviews/task-30/128-30199-spire-delta-object-vec-id-uniqueness` | `task-30` | `spire` |
| `review/30200-spire-leaf-object-vec-id-uniqueness` | `reviews/task-30/129-30200-spire-leaf-object-vec-id-uniqueness` | `task-30` | `spire` |
| `review/30201-spire-leaf-object-delta-flag-guard` | `reviews/task-30/130-30201-spire-leaf-object-delta-flag-guard` | `task-30` | `spire` |
| `review/30202-spire-delta-assignment-role-flags` | `reviews/task-30/131-30202-spire-delta-assignment-role-flags` | `task-30` | `spire` |
| `review/30203-spire-delete-delta-payload-shape` | `reviews/task-30/132-30203-spire-delete-delta-payload-shape` | `task-30` | `spire` |
| `review/30204-spire-stale-locator-scan-filter` | `reviews/task-30/133-30204-spire-stale-locator-scan-filter` | `task-30` | `spire` |
| `review/30205-spire-delta-stale-locator-guard` | `reviews/task-30/134-30205-spire-delta-stale-locator-guard` | `task-30` | `spire` |
| `review/30206-spire-stale-delete-target-guard` | `reviews/task-30/135-30206-spire-stale-delete-target-guard` | `task-30` | `spire` |
| `review/30207-spire-leaf-assignment-role-flags` | `reviews/task-30/136-30207-spire-leaf-assignment-role-flags` | `task-30` | `spire` |
| `review/30208-spire-leaf-assignment-foundation-status` | `reviews/task-30/137-30208-spire-leaf-assignment-foundation-status` | `task-30` | `spire` |
| `review/30209-spire-placement-manifest-exactness` | `reviews/task-30/138-30209-spire-placement-manifest-exactness` | `task-30` | `spire` |
| `review/30210-spire-visible-vec-id-uniqueness` | `reviews/task-30/139-30210-spire-visible-vec-id-uniqueness` | `task-30` | `spire` |
| `review/30211-spire-delta-degraded-base-placement-guard` | `reviews/task-30/140-30211-spire-delta-degraded-base-placement-guard` | `task-30` | `spire` |
| `review/30212-spire-common-spherical-kmeans-training` | `reviews/task-30/141-30212-spire-common-spherical-kmeans-training` | `task-30` | `spire` |
| `review/30213-spire-foundation-task-status` | `reviews/task-30/142-30213-spire-foundation-task-status` | `task-30` | `spire` |
| `review/30214-spire-single-level-centroid-plan` | `reviews/task-30/143-30214-spire-single-level-centroid-plan` | `task-30` | `spire` |
| `review/30215-spire-partitioned-leaf-draft` | `reviews/task-30/144-30215-spire-partitioned-leaf-draft` | `task-30` | `spire` |
| `review/30216-spire-single-level-route-map` | `reviews/task-30/145-30216-spire-single-level-route-map` | `task-30` | `spire` |
| `review/30217-spire-routing-object-codec` | `reviews/task-30/146-30217-spire-routing-object-codec` | `task-30` | `spire` |
| `review/30218-spire-root-routing-draft-object` | `reviews/task-30/147-30218-spire-root-routing-draft-object` | `task-30` | `spire` |
| `review/30219-spire-foundation-progress-status` | `reviews/task-30/148-30219-spire-foundation-progress-status` | `task-30` | `spire` |
| `review/30220-spire-root-routed-scan-helper` | `reviews/task-30/149-30220-spire-root-routed-scan-helper` | `task-30` | `spire` |
| `review/30221-spire-routed-nprobe-helper` | `reviews/task-30/150-30221-spire-routed-nprobe-helper` | `task-30` | `spire` |
| `review/30222-spire-scan-foundation-status` | `reviews/task-30/151-30222-spire-scan-foundation-status` | `task-30` | `spire` |
| `review/30223-spire-routed-candidate-ranking` | `reviews/task-30/152-30223-spire-routed-candidate-ranking` | `task-30` | `spire` |
| `review/30224-spire-candidate-rerank-seam` | `reviews/task-30/153-30224-spire-candidate-rerank-seam` | `task-30` | `spire` |
| `review/30225-spire-scan-path-status` | `reviews/task-30/154-30225-spire-scan-path-status` | `task-30` | `spire` |
| `review/30226-spire-ranked-scan-cursor` | `reviews/task-30/155-30226-spire-ranked-scan-cursor` | `task-30` | `spire` |
| `review/30227-spire-snapshot-diagnostics` | `reviews/task-30/156-30227-spire-snapshot-diagnostics` | `task-30` | `spire` |
| `review/30228-spire-diagnostics-status` | `reviews/task-30/157-30228-spire-diagnostics-status` | `task-30` | `spire` |
| `review/30229-spire-assignment-payload-formats` | `reviews/task-30/158-30229-spire-assignment-payload-formats` | `task-30` | `spire` |
| `review/30230-spire-assignment-quantizer-scorer` | `reviews/task-30/159-30230-spire-assignment-quantizer-scorer` | `task-30` | `spire` |
| `review/30231-spire-quantizer-reuse-status` | `reviews/task-30/160-30231-spire-quantizer-reuse-status` | `task-30` | `spire` |
| `review/30232-spire-routed-scan-quantizer-binding` | `reviews/task-30/161-30232-spire-routed-scan-quantizer-binding` | `task-30` | `spire` |
| `review/30233-spire-routed-scorer-status` | `reviews/task-30/162-30233-spire-routed-scorer-status` | `task-30` | `spire` |
| `review/30234-spire-quantized-assignment-input` | `reviews/task-30/163-30234-spire-quantized-assignment-input` | `task-30` | `spire` |
| `review/30235-spire-assignment-input-status` | `reviews/task-30/164-30235-spire-assignment-input-status` | `task-30` | `spire` |
| `review/30236-spire-quantized-rerank-helper` | `reviews/task-30/165-30236-spire-quantized-rerank-helper` | `task-30` | `spire` |
| `review/30237-spire-quantized-rerank-status` | `reviews/task-30/166-30237-spire-quantized-rerank-status` | `task-30` | `spire` |
| `review/30238-spire-scan-option-plumbing` | `reviews/task-30/167-30238-spire-scan-option-plumbing` | `task-30` | `spire` |
| `review/30239-spire-option-status` | `reviews/task-30/168-30239-spire-option-status` | `task-30` | `spire` |
| `review/30240-spire-single-level-scan-option-plan` | `reviews/task-30/169-30240-spire-single-level-scan-option-plan` | `task-30` | `spire` |
| `review/30241-spire-scan-option-plan-status` | `reviews/task-30/170-30241-spire-scan-option-plan-status` | `task-30` | `spire` |
| `review/30242-spire-scan-plan-helper-binding` | `reviews/task-30/171-30242-spire-scan-plan-helper-binding` | `task-30` | `spire` |
| `review/30243-spire-scan-plan-binding-status` | `reviews/task-30/172-30243-spire-scan-plan-binding-status` | `task-30` | `spire` |
| `review/30244-spire-scan-output-bridge` | `reviews/task-30/173-30244-spire-scan-output-bridge` | `task-30` | `spire` |
| `review/30245-spire-scan-output-status` | `reviews/task-30/174-30245-spire-scan-output-status` | `task-30` | `spire` |
| `review/30246-spire-scan-opaque-lifecycle` | `reviews/task-30/175-30246-spire-scan-opaque-lifecycle` | `task-30` | `spire` |
| `review/30247-spire-scan-opaque-status` | `reviews/task-30/176-30247-spire-scan-opaque-status` | `task-30` | `spire` |
| `review/30248-spire-snapshot-leaf-count-helper` | `reviews/task-30/177-30248-spire-snapshot-leaf-count-helper` | `task-30` | `spire` |
| `review/30249-spire-scan-leaf-count-status` | `reviews/task-30/178-30249-spire-scan-leaf-count-status` | `task-30` | `spire` |
| `review/30250-spire-validated-scan-query-state` | `reviews/task-30/179-30250-spire-validated-scan-query-state` | `task-30` | `spire` |
| `review/30251-spire-scan-query-state-status` | `reviews/task-30/180-30251-spire-scan-query-state-status` | `task-30` | `spire` |
| `review/30252-spire-amrescan-query-parsing` | `reviews/task-30/181-30252-spire-amrescan-query-parsing` | `task-30` | `spire` |
| `review/30253-spire-amrescan-query-parsing-status` | `reviews/task-30/182-30253-spire-amrescan-query-parsing-status` | `task-30` | `spire` |
| `review/30254-spire-snapshot-scan-preparation` | `reviews/task-30/183-30254-spire-snapshot-scan-preparation` | `task-30` | `spire` |
| `review/30255-spire-foundation-architecture-response` | `reviews/task-30/184-30255-spire-foundation-architecture-response` | `task-30` | `spire` |
| `review/30256-spire-leaf-v2-segmented-store-path` | `reviews/task-30/185-30256-spire-leaf-v2-segmented-store-path` | `task-30` | `spire` |
| `review/30257-spire-borrowed-assignment-row-refs` | `reviews/task-30/186-30257-spire-borrowed-assignment-row-refs` | `task-30` | `spire` |
| `review/30258-spire-validated-snapshot-lookup-cache` | `reviews/task-30/187-30258-spire-validated-snapshot-lookup-cache` | `task-30` | `spire` |
| `review/30259-spire-bounded-scan-heaps` | `reviews/task-30/188-30259-spire-bounded-scan-heaps` | `task-30` | `spire` |
| `review/30260-spire-explicit-dedupe-mode` | `reviews/task-30/189-30260-spire-explicit-dedupe-mode` | `task-30` | `spire` |
| `review/30261-spire-validated-snapshot-publication-helpers` | `reviews/task-30/190-30261-spire-validated-snapshot-publication-helpers` | `task-30` | `spire` |
| `review/30262-spire-flat-routing-object-layout` | `reviews/task-30/191-30262-spire-flat-routing-object-layout` | `task-30` | `spire` |
| `review/30263-spire-batch-assignment-scoring` | `reviews/task-30/192-30263-spire-batch-assignment-scoring` | `task-30` | `spire` |
| `review/30264-spire-leaf-v2-column-views` | `reviews/task-30/193-30264-spire-leaf-v2-column-views` | `task-30` | `spire` |
| `review/30265-spire-codec-validation-split` | `reviews/task-30/194-30265-spire-codec-validation-split` | `task-30` | `spire` |
| `review/30266-spire-publish-coordinator` | `reviews/task-30/195-30266-spire-publish-coordinator` | `task-30` | `spire` |
| `review/30267-spire-placement-state-constructors` | `reviews/task-30/196-30267-spire-placement-state-constructors` | `task-30` | `spire` |
| `review/30268-spire-allocator-exhaustion-diagnostics` | `reviews/task-30/197-30268-spire-allocator-exhaustion-diagnostics` | `task-30` | `spire` |
| `review/30269-spire-object-byte-diagnostics` | `reviews/task-30/198-30269-spire-object-byte-diagnostics` | `task-30` | `spire` |
| `review/30270-spire-object-reader-trait` | `reviews/task-30/199-30270-spire-object-reader-trait` | `task-30` | `spire` |
| `review/30271-spire-v2-leaf-build-scan` | `reviews/task-30/200-30271-spire-v2-leaf-build-scan` | `task-30` | `spire` |
| `review/30272-spire-v2-column-batch-scan` | `reviews/task-30/201-30272-spire-v2-column-batch-scan` | `task-30` | `spire` |
| `review/30273-spire-object-epoch-backrefs` | `reviews/task-30/202-30273-spire-object-epoch-backrefs` | `task-30` | `spire` |
| `review/30274-spire-payload-helper-return` | `reviews/task-30/203-30274-spire-payload-helper-return` | `task-30` | `spire` |
| `review/30275-spire-object-reader-scan-update` | `reviews/task-30/204-30275-spire-object-reader-scan-update` | `task-30` | `spire` |
| `review/30276-spire-candidate-tie-breaks` | `reviews/task-30/205-30276-spire-candidate-tie-breaks` | `task-30` | `spire` |
| `review/30277-spire-empty-root-control` | `reviews/task-30/206-30277-spire-empty-root-control` | `task-30` | `spire` |
| `review/30278-spire-relation-object-tuples` | `reviews/task-30/207-30278-spire-relation-object-tuples` | `task-30` | `spire` |
| `review/30279-spire-relation-object-store` | `reviews/task-30/208-30279-spire-relation-object-store` | `task-30` | `spire` |
| `review/30280-spire-relation-v2-leaf-store` | `reviews/task-30/209-30280-spire-relation-v2-leaf-store` | `task-30` | `spire` |
| `review/30281-spire-manifest-bundle-publish` | `reviews/task-30/210-30281-spire-manifest-bundle-publish` | `task-30` | `spire` |
| `review/30282-spire-pinned-relation-tuple-reads` | `reviews/task-30/211-30282-spire-pinned-relation-tuple-reads` | `task-30` | `spire` |
| `review/30283-spire-root-control-scan-cache` | `reviews/task-30/212-30283-spire-root-control-scan-cache` | `task-30` | `spire` |
| `review/30284-spire-build-root-control-order` | `reviews/task-30/213-30284-spire-build-root-control-order` | `task-30` | `spire` |
| `review/30285-spire-populated-build-tuple-collection` | `reviews/task-30/214-30285-spire-populated-build-tuple-collection` | `task-30` | `spire` |
| `review/30286-spire-placement-entry-locators` | `reviews/task-30/215-30286-spire-placement-entry-locators` | `task-30` | `spire` |
| `review/30287-spire-populated-build-publish` | `reviews/task-30/216-30287-spire-populated-build-publish` | `task-30` | `spire` |
| `review/30288-spire-relation-snapshot-scan` | `reviews/task-30/217-30288-spire-relation-snapshot-scan` | `task-30` | `spire` |
| `review/30289-spire-active-snapshot-diagnostics` | `reviews/task-30/218-30289-spire-active-snapshot-diagnostics` | `task-30` | `spire` |
| `review/30290-spire-active-scan-heap-rerank` | `reviews/task-30/219-30290-spire-active-scan-heap-rerank` | `task-30` | `spire` |
| `review/30291-spire-insert-delta-epochs` | `reviews/task-30/220-30291-spire-insert-delta-epochs` | `task-30` | `spire` |
| `review/30292-spire-vacuum-delete-deltas` | `reviews/task-30/221-30292-spire-vacuum-delete-deltas` | `task-30` | `spire` |
| `review/30293-spire-empty-insert-bootstrap` | `reviews/task-30/222-30293-spire-empty-insert-bootstrap` | `task-30` | `spire` |
| `review/30294-spire-active-diagnostics-sql` | `reviews/task-30/223-30294-spire-active-diagnostics-sql` | `task-30` | `spire` |
| `review/30295-spire-options-diagnostics-sql` | `reviews/task-30/224-30295-spire-options-diagnostics-sql` | `task-30` | `spire` |
| `review/30296-spire-vacuum-delta-compaction` | `reviews/task-30/225-30296-spire-vacuum-delta-compaction` | `task-30` | `spire` |
| `review/30297-spire-health-diagnostics-sql` | `reviews/task-30/226-30297-spire-health-diagnostics-sql` | `task-30` | `spire` |
| `review/30298-spire-effective-options-sql` | `reviews/task-30/227-30298-spire-effective-options-sql` | `task-30` | `spire` |
| `review/30299-spire-placement-diagnostics-sql` | `reviews/task-30/228-30299-spire-placement-diagnostics-sql` | `task-30` | `spire` |
| `review/30300-spire-scan-placement-diagnostics-sql` | `reviews/task-30/229-30300-spire-scan-placement-diagnostics-sql` | `task-30` | `spire` |
| `review/30301-spire-v2-column-segment-iterator` | `reviews/task-30/230-30301-spire-v2-column-segment-iterator` | `task-30` | `spire` |
| `review/30302-spire-root-routing-diagnostics-sql` | `reviews/task-30/231-30302-spire-root-routing-diagnostics-sql` | `task-30` | `spire` |
| `review/30303-spire-pqfastscan-deferral-diagnostics` | `reviews/task-30/232-30303-spire-pqfastscan-deferral-diagnostics` | `task-30` | `spire` |
| `review/30304-spire-relation-storage-debt-diagnostics` | `reviews/task-30/233-30304-spire-relation-storage-debt-diagnostics` | `task-30` | `spire` |
| `review/30305-spire-scan-sanity-diagnostics` | `reviews/task-30/234-30305-spire-scan-sanity-diagnostics` | `task-30` | `spire` |
| `review/30306-spire-epoch-cleanup-diagnostics` | `reviews/task-30/235-30306-spire-epoch-cleanup-diagnostics` | `task-30` | `spire` |
| `review/30307-spire-retired-epoch-manifests` | `reviews/task-30/236-30307-spire-retired-epoch-manifests` | `task-30` | `spire` |
| `review/30308-spire-leaf-partition-diagnostics` | `reviews/task-30/237-30308-spire-leaf-partition-diagnostics` | `task-30` | `spire` |
| `review/30309-spire-leaf-maintenance-thresholds` | `reviews/task-30/238-30309-spire-leaf-maintenance-thresholds` | `task-30` | `spire` |
| `review/30310-spire-insert-batching-debt-diagnostics` | `reviews/task-30/239-30310-spire-insert-batching-debt-diagnostics` | `task-30` | `spire` |
| `review/30311-spire-hierarchy-diagnostics` | `reviews/task-30/240-30311-spire-hierarchy-diagnostics` | `task-30` | `spire` |
| `review/30312-spire-partition-object-diagnostics` | `reviews/task-30/241-30312-spire-partition-object-diagnostics` | `task-30` | `spire` |
| `review/30313-spire-delta-diagnostics` | `reviews/task-30/242-30313-spire-delta-diagnostics` | `task-30` | `spire` |
| `review/30314-spire-delete-delta-diagnostics-coverage` | `reviews/task-30/243-30314-spire-delete-delta-diagnostics-coverage` | `task-30` | `spire` |
| `review/30315-spire-allocator-diagnostics` | `reviews/task-30/244-30315-spire-allocator-diagnostics` | `task-30` | `spire` |
| `review/30316-spire-root-routing-defensive-coverage` | `reviews/task-30/245-30316-spire-root-routing-defensive-coverage` | `task-30` | `spire` |
| `review/30317-spire-placement-delta-diagnostics-coverage` | `reviews/task-30/246-30317-spire-placement-delta-diagnostics-coverage` | `task-30` | `spire` |
| `review/30318-spire-insert-error-path-coverage` | `reviews/task-30/247-30318-spire-insert-error-path-coverage` | `task-30` | `spire` |
| `review/30319-spire-multi-row-insert-epoch-coverage` | `reviews/task-30/248-30319-spire-multi-row-insert-epoch-coverage` | `task-30` | `spire` |
| `review/30320-spire-vacuum-compaction-leaf-pid-guard` | `reviews/task-30/249-30320-spire-vacuum-compaction-leaf-pid-guard` | `task-30` | `spire` |
| `review/30321-spire-scan-root-control-cache-refresh` | `reviews/task-30/250-30321-spire-scan-root-control-cache-refresh` | `task-30` | `spire` |
| `review/30322-spire-storage-debt-compaction-coverage` | `reviews/task-30/251-30322-spire-storage-debt-compaction-coverage` | `task-30` | `spire` |
| `review/30323-spire-epoch-diagnostics-compaction-coverage` | `reviews/task-30/252-30323-spire-epoch-diagnostics-compaction-coverage` | `task-30` | `spire` |
| `review/30324-spire-null-insert-error-path-coverage` | `reviews/task-30/253-30324-spire-null-insert-error-path-coverage` | `task-30` | `spire` |
| `review/30325-spire-replica-deferral-checkpoint` | `reviews/task-30/254-30325-spire-replica-deferral-checkpoint` | `task-30` | `spire` |
| `review/30326-spire-single-store-placement-complete` | `reviews/task-30/255-30326-spire-single-store-placement-complete` | `task-30` | `spire` |
| `review/30327-spire-pqfastscan-build-deferral-coverage` | `reviews/task-30/256-30327-spire-pqfastscan-build-deferral-coverage` | `task-30` | `spire` |
| `review/30328-spire-build-path-complete` | `reviews/task-30/257-30328-spire-build-path-complete` | `task-30` | `spire` |
| `review/30329-spire-empty-pqfastscan-scan-coverage` | `reviews/task-30/258-30329-spire-empty-pqfastscan-scan-coverage` | `task-30` | `spire` |
| `review/30330-spire-scan-path-complete` | `reviews/task-30/259-30330-spire-scan-path-complete` | `task-30` | `spire` |
| `review/30331-spire-diagnostics-complete` | `reviews/task-30/260-30331-spire-diagnostics-complete` | `task-30` | `spire` |
| `review/30332-spire-leaf-maintenance-triggers-complete` | `reviews/task-30/261-30332-spire-leaf-maintenance-triggers-complete` | `task-30` | `spire` |
| `review/30333-spire-insert-path-complete` | `reviews/task-30/262-30333-spire-insert-path-complete` | `task-30` | `spire` |
| `review/30334-spire-vacuum-path-complete` | `reviews/task-30/263-30334-spire-vacuum-path-complete` | `task-30` | `spire` |
| `review/30335-spire-update-mechanics-plan` | `reviews/task-30/264-30335-spire-update-mechanics-plan` | `task-30` | `spire` |
| `review/30336-spire-concurrent-insert-coverage` | `reviews/task-30/265-30336-spire-concurrent-insert-coverage` | `task-30` | `spire` |
| `review/30337-spire-sql-vacuum-rerank-visibility` | `reviews/task-30/266-30337-spire-sql-vacuum-rerank-visibility` | `task-30` | `spire` |
| `review/30338-spire-leaf-diagnostics-order` | `reviews/task-30/267-30338-spire-leaf-diagnostics-order` | `task-30` | `spire` |
| `review/30339-spire-root-control-rescan-refresh` | `reviews/task-30/268-30339-spire-root-control-rescan-refresh` | `task-30` | `spire` |
| `review/30340-spire-update-mechanics-review-followup` | `reviews/task-30/269-30340-spire-update-mechanics-review-followup` | `task-30` | `spire` |
| `review/30341-spire-concurrent-insert-waiters` | `reviews/task-30/270-30341-spire-concurrent-insert-waiters` | `task-30` | `spire` |
| `review/30342-spire-retired-epoch-residue-diagnostics` | `reviews/task-30/271-30342-spire-retired-epoch-residue-diagnostics` | `task-30` | `spire` |
| `review/30343-spire-replacement-epoch-publish-helper` | `reviews/task-30/272-30343-spire-replacement-epoch-publish-helper` | `task-30` | `spire` |
| `review/30344-spire-vacuum-compaction-version-guard` | `reviews/task-30/273-30344-spire-vacuum-compaction-version-guard` | `task-30` | `spire` |
| `review/30345-spire-diagnostics-overview-doc` | `reviews/task-30/274-30345-spire-diagnostics-overview-doc` | `task-30` | `spire` |
| `review/30346-spire-placement-state-diagnostics-coverage` | `reviews/task-30/275-30346-spire-placement-state-diagnostics-coverage` | `task-30` | `spire` |
| `review/30347-spire-scan-path-checklist-scope` | `reviews/task-30/276-30347-spire-scan-path-checklist-scope` | `task-30` | `spire` |
| `review/30348-spire-leaf-maintenance-threshold-constants` | `reviews/task-30/277-30348-spire-leaf-maintenance-threshold-constants` | `task-30` | `spire` |
| `review/30349-spire-object-tuple-scan-contract` | `reviews/task-30/278-30349-spire-object-tuple-scan-contract` | `task-30` | `spire` |
| `review/30350-spire-multi-row-insert-test-contract` | `reviews/task-30/279-30350-spire-multi-row-insert-test-contract` | `task-30` | `spire` |
| `review/30351-spire-epoch-manifest-magic` | `reviews/task-30/280-30351-spire-epoch-manifest-magic` | `task-30` | `spire` |
| `review/30352-spire-heterogeneous-concurrency` | `reviews/task-30/281-30352-spire-heterogeneous-concurrency` | `task-30` | `spire` |
| `review/30353-spire-diagnostic-status-labels` | `reviews/task-30/282-30353-spire-diagnostic-status-labels` | `task-30` | `spire` |
| `review/30354-spire-retired-manifest-precondition` | `reviews/task-30/283-30354-spire-retired-manifest-precondition` | `task-30` | `spire` |
| `review/30355-spire-placement-delta-sql-coverage` | `reviews/task-30/284-30355-spire-placement-delta-sql-coverage` | `task-30` | `spire` |
| `review/30356-spire-diagnostic-label-constants` | `reviews/task-30/285-30356-spire-diagnostic-label-constants` | `task-30` | `spire` |
| `review/30357-spire-bundle-residue-diagnostics` | `reviews/task-30/286-30357-spire-bundle-residue-diagnostics` | `task-30` | `spire` |
| `review/30358-spire-scan-root-cache-seed-coverage` | `reviews/task-30/287-30358-spire-scan-root-cache-seed-coverage` | `task-30` | `spire` |
| `review/30359-spire-phase1-pqfastscan-deferral-scope` | `reviews/task-30/288-30359-spire-phase1-pqfastscan-deferral-scope` | `task-30` | `spire` |
| `review/30360-spire-phase1-validation-checklist` | `reviews/task-30/289-30360-spire-phase1-validation-checklist` | `task-30` | `spire` |
| `review/30361-spire-phase1-landing` | `reviews/task-30/290-30361-spire-phase1-landing` | `task-30` | `spire` |
| `review/30362-spire-replacement-leaf-planning` | `reviews/task-30/291-30362-spire-replacement-leaf-planning` | `task-30` | `spire` |
| `review/30363-spire-replacement-routing-rewrite` | `reviews/task-30/292-30363-spire-replacement-routing-rewrite` | `task-30` | `spire` |
| `review/30364-spire-replacement-placement-directory` | `reviews/task-30/293-30364-spire-replacement-placement-directory` | `task-30` | `spire` |
| `review/30365-spire-replacement-publish-draft` | `reviews/task-30/294-30365-spire-replacement-publish-draft` | `task-30` | `spire` |
| `review/30366-spire-local-replacement-object-writes` | `reviews/task-30/295-30366-spire-local-replacement-object-writes` | `task-30` | `spire` |
| `review/30367-spire-relation-replacement-object-writer` | `reviews/task-30/296-30367-spire-relation-replacement-object-writer` | `task-30` | `spire` |
| `review/30368-spire-replacement-publish-assembly` | `reviews/task-30/297-30368-spire-replacement-publish-assembly` | `task-30` | `spire` |
| `review/30369-spire-relation-replacement-publish-helper` | `reviews/task-30/298-30369-spire-relation-replacement-publish-helper` | `task-30` | `spire` |
| `review/30370-spire-replacement-scheduler-choice` | `reviews/task-30/299-30370-spire-replacement-scheduler-choice` | `task-30` | `spire` |
| `review/30371-spire-scheduled-replacement-pid-planning` | `reviews/task-30/300-30371-spire-scheduled-replacement-pid-planning` | `task-30` | `spire` |
| `review/30372-spire-replacement-scheduler-recheck` | `reviews/task-30/301-30372-spire-replacement-scheduler-recheck` | `task-30` | `spire` |
| `review/30373-spire-merge-replacement-leaf-input` | `reviews/task-30/302-30373-spire-merge-replacement-leaf-input` | `task-30` | `spire` |
| `review/30374-spire-split-replacement-leaf-input` | `reviews/task-30/303-30374-spire-split-replacement-leaf-input` | `task-30` | `spire` |
| `review/30375-spire-scheduled-routing-replacement-children` | `reviews/task-30/304-30375-spire-scheduled-routing-replacement-children` | `task-30` | `spire` |
| `review/30376-spire-scheduled-routing-rewrite` | `reviews/task-30/305-30376-spire-scheduled-routing-rewrite` | `task-30` | `spire` |
| `review/30377-spire-scheduled-replacement-publish-draft` | `reviews/task-30/306-30377-spire-scheduled-replacement-publish-draft` | `task-30` | `spire` |
| `review/30378-spire-scheduled-replacement-object-writer` | `reviews/task-30/307-30378-spire-scheduled-replacement-object-writer` | `task-30` | `spire` |
| `review/30379-spire-scheduled-replacement-successor-epoch` | `reviews/task-30/308-30379-spire-scheduled-replacement-successor-epoch` | `task-30` | `spire` |
| `review/30380-spire-scheduled-pid-plan-output-validation` | `reviews/task-30/309-30380-spire-scheduled-pid-plan-output-validation` | `task-30` | `spire` |
| `review/30381-spire-local-scheduled-replacement-execution-draft` | `reviews/task-30/310-30381-spire-local-scheduled-replacement-execution-draft` | `task-30` | `spire` |
| `review/30382-spire-relation-scheduled-replacement-publish` | `reviews/task-30/311-30382-spire-relation-scheduled-replacement-publish` | `task-30` | `spire` |
| `review/30383-spire-scheduled-replacement-publish-plan` | `reviews/task-30/312-30383-spire-scheduled-replacement-publish-plan` | `task-30` | `spire` |
| `review/30384-spire-scheduled-replacement-consistency-mode` | `reviews/task-30/313-30384-spire-scheduled-replacement-consistency-mode` | `task-30` | `spire` |
| `review/30385-spire-scheduled-relation-publish-input-builder` | `reviews/task-30/314-30385-spire-scheduled-relation-publish-input-builder` | `task-30` | `spire` |
| `review/30386-spire-scheduled-relation-publish-plan-validation` | `reviews/task-30/315-30386-spire-scheduled-relation-publish-plan-validation` | `task-30` | `spire` |
| `review/30387-spire-local-scheduled-publish-input-builder` | `reviews/task-30/316-30387-spire-local-scheduled-publish-input-builder` | `task-30` | `spire` |
| `review/30388-spire-local-scheduled-publish-plan-validation` | `reviews/task-30/317-30388-spire-local-scheduled-publish-plan-validation` | `task-30` | `spire` |
| `review/30389-spire-scheduled-execution-decision-validation` | `reviews/task-30/318-30389-spire-scheduled-execution-decision-validation` | `task-30` | `spire` |
| `review/30390-spire-scheduled-execution-leaf-version-validation` | `reviews/task-30/319-30390-spire-scheduled-execution-leaf-version-validation` | `task-30` | `spire` |
| `review/30391-spire-scheduled-replacement-pid-cursor-bounds` | `reviews/task-30/320-30391-spire-scheduled-replacement-pid-cursor-bounds` | `task-30` | `spire` |
| `review/30392-spire-scheduled-replacement-pid-plan-shape` | `reviews/task-30/321-30392-spire-scheduled-replacement-pid-plan-shape` | `task-30` | `spire` |
| `review/30393-spire-scheduled-execution-successor-epoch` | `reviews/task-30/322-30393-spire-scheduled-execution-successor-epoch` | `task-30` | `spire` |
| `review/30394-spire-scheduled-execution-publish-timestamp` | `reviews/task-30/323-30394-spire-scheduled-execution-publish-timestamp` | `task-30` | `spire` |
| `review/30395-spire-scheduled-execution-active-snapshot` | `reviews/task-30/324-30395-spire-scheduled-execution-active-snapshot` | `task-30` | `spire` |
| `review/30396-spire-scheduled-object-writer-successor-epoch` | `reviews/task-30/325-30396-spire-scheduled-object-writer-successor-epoch` | `task-30` | `spire` |
| `review/30397-spire-scheduled-routing-object-version` | `reviews/task-30/326-30397-spire-scheduled-routing-object-version` | `task-30` | `spire` |
| `review/30398-spire-scheduled-routing-pid-cursor` | `reviews/task-30/327-30398-spire-scheduled-routing-pid-cursor` | `task-30` | `spire` |
| `review/30399-spire-split-replacement-pid-cursor` | `reviews/task-30/328-30399-spire-split-replacement-pid-cursor` | `task-30` | `spire` |
| `review/30400-spire-merge-replacement-pid-cursor` | `reviews/task-30/329-30400-spire-merge-replacement-pid-cursor` | `task-30` | `spire` |
| `review/30401-spire-scheduled-execution-parent-contents` | `reviews/task-30/330-30401-spire-scheduled-execution-parent-contents` | `task-30` | `spire` |
| `review/30402-spire-scheduler-recheck-selector-contract` | `reviews/task-30/331-30402-spire-scheduler-recheck-selector-contract` | `task-30` | `spire` |
| `review/30403-spire-scheduled-merge-centroid-helper` | `reviews/task-30/332-30403-spire-scheduled-merge-centroid-helper` | `task-30` | `spire` |
| `review/30404-spire-merge-centroid-duplicate-row-guard` | `reviews/task-30/333-30404-spire-merge-centroid-duplicate-row-guard` | `task-30` | `spire` |
| `review/30405-spire-merge-centroid-recommendation-guard` | `reviews/task-30/334-30405-spire-merge-centroid-recommendation-guard` | `task-30` | `spire` |
| `review/30406-spire-scheduler-duplicate-row-guard` | `reviews/task-30/335-30406-spire-scheduler-duplicate-row-guard` | `task-30` | `spire` |
| `review/30407-spire-merge-centroid-shared-duplicate-validation` | `reviews/task-30/336-30407-spire-merge-centroid-shared-duplicate-validation` | `task-30` | `spire` |
| `review/30408-spire-scheduled-parent-routing-loader` | `reviews/task-30/337-30408-spire-scheduled-parent-routing-loader` | `task-30` | `spire` |
| `review/30409-spire-scheduled-merge-routing-parts` | `reviews/task-30/338-30409-spire-scheduled-merge-routing-parts` | `task-30` | `spire` |
| `review/30410-spire-relation-scheduled-merge-execution-parts` | `reviews/task-30/339-30410-spire-relation-scheduled-merge-execution-parts` | `task-30` | `spire` |
| `review/30411-spire-relation-scheduled-merge-execution-input` | `reviews/task-30/340-30411-spire-relation-scheduled-merge-execution-input` | `task-30` | `spire` |
| `review/30412-spire-relation-scheduled-split-execution-parts` | `reviews/task-30/341-30412-spire-relation-scheduled-split-execution-parts` | `task-30` | `spire` |
| `review/30413-spire-relation-scheduled-split-execution-input` | `reviews/task-30/342-30413-spire-relation-scheduled-split-execution-input` | `task-30` | `spire` |
| `review/30414-spire-local-scheduled-merge-execution-input` | `reviews/task-30/343-30414-spire-local-scheduled-merge-execution-input` | `task-30` | `spire` |
| `review/30415-spire-local-scheduled-split-execution-input` | `reviews/task-30/344-30415-spire-local-scheduled-split-execution-input` | `task-30` | `spire` |
| `review/30416-spire-local-scheduled-execution-parts-conversion` | `reviews/task-30/345-30416-spire-local-scheduled-execution-parts-conversion` | `task-30` | `spire` |
| `review/30417-spire-scheduled-publish-lock-allocation` | `reviews/task-30/346-30417-spire-scheduled-publish-lock-allocation` | `task-30` | `spire` |
| `review/30418-spire-rechecked-scheduled-publish-lock` | `reviews/task-30/347-30418-spire-rechecked-scheduled-publish-lock` | `task-30` | `spire` |
| `review/30419-spire-selected-scheduled-publish-lock` | `reviews/task-30/348-30419-spire-selected-scheduled-publish-lock` | `task-30` | `spire` |
| `review/30420-spire-selected-relation-merge-execution-input` | `reviews/task-30/349-30420-spire-selected-relation-merge-execution-input` | `task-30` | `spire` |
| `review/30421-spire-selected-relation-split-execution-input` | `reviews/task-30/350-30421-spire-selected-relation-split-execution-input` | `task-30` | `spire` |
| `review/30422-spire-selected-local-merge-execution-input` | `reviews/task-30/351-30422-spire-selected-local-merge-execution-input` | `task-30` | `spire` |
| `review/30423-spire-selected-local-split-execution-input` | `reviews/task-30/352-30423-spire-selected-local-split-execution-input` | `task-30` | `spire` |
| `review/30424-spire-selected-local-replacement-draft` | `reviews/task-30/353-30424-spire-selected-local-replacement-draft` | `task-30` | `spire` |
| `review/30425-spire-selected-execution-input-validators` | `reviews/task-30/354-30425-spire-selected-execution-input-validators` | `task-30` | `spire` |
| `review/30426-spire-selected-execution-snapshot-validator` | `reviews/task-30/355-30426-spire-selected-execution-snapshot-validator` | `task-30` | `spire` |
| `review/30427-spire-selected-relation-publish-preflight` | `reviews/task-30/356-30427-spire-selected-relation-publish-preflight` | `task-30` | `spire` |
| `review/30428-spire-selected-local-draft-preflight` | `reviews/task-30/357-30428-spire-selected-local-draft-preflight` | `task-30` | `spire` |
| `review/30429-spire-selected-local-split-draft` | `reviews/task-30/358-30429-spire-selected-local-split-draft` | `task-30` | `spire` |
| `review/30430-spire-selected-local-merge-draft` | `reviews/task-30/359-30430-spire-selected-local-merge-draft` | `task-30` | `spire` |
| `review/30431-spire-selected-replacement-snapshot-loaders` | `reviews/task-30/360-30431-spire-selected-replacement-snapshot-loaders` | `task-30` | `spire` |
| `review/30432-spire-selected-merge-snapshot-draft` | `reviews/task-30/361-30432-spire-selected-merge-snapshot-draft` | `task-30` | `spire` |
| `review/30433-spire-selected-split-snapshot-draft` | `reviews/task-30/362-30433-spire-selected-split-snapshot-draft` | `task-30` | `spire` |
| `review/30434-spire-helper-arc-feedback-response` | `reviews/task-30/363-30434-spire-helper-arc-feedback-response` | `task-30` | `spire` |
| `review/30435-spire-selected-relation-merge-snapshot-input` | `reviews/task-30/364-30435-spire-selected-relation-merge-snapshot-input` | `task-30` | `spire` |
| `review/30436-spire-selected-relation-split-snapshot-input` | `reviews/task-30/365-30436-spire-selected-relation-split-snapshot-input` | `task-30` | `spire` |
| `review/30437-spire-selected-local-merge-snapshot-input` | `reviews/task-30/366-30437-spire-selected-local-merge-snapshot-input` | `task-30` | `spire` |
| `review/30438-spire-selected-local-split-snapshot-input` | `reviews/task-30/367-30438-spire-selected-local-split-snapshot-input` | `task-30` | `spire` |
| `review/30439-spire-maintenance-plan-snapshot` | `reviews/task-30/368-30439-spire-maintenance-plan-snapshot` | `task-30` | `spire` |
| `review/30440-spire-maintenance-plan-merge-coverage` | `reviews/task-30/369-30440-spire-maintenance-plan-merge-coverage` | `task-30` | `spire` |
| `review/30441-spire-shared-publish-lock` | `reviews/task-30/370-30441-spire-shared-publish-lock` | `task-30` | `spire` |
| `review/30442-spire-locked-maintenance-plan-snapshot` | `reviews/task-30/371-30442-spire-locked-maintenance-plan-snapshot` | `task-30` | `spire` |
| `review/30443-spire-split-replacement-materialization` | `reviews/task-30/372-30443-spire-split-replacement-materialization` | `task-30` | `spire` |
| `review/30444-spire-split-source-row-hydration` | `reviews/task-30/373-30444-spire-split-source-row-hydration` | `task-30` | `spire` |
| `review/30445-spire-split-materialization-from-rows` | `reviews/task-30/374-30445-spire-split-materialization-from-rows` | `task-30` | `spire` |
| `review/30446-spire-split-source-vector-fetch` | `reviews/task-30/375-30446-spire-split-source-vector-fetch` | `task-30` | `spire` |
| `review/30447-spire-selected-split-input-from-sources` | `reviews/task-30/376-30447-spire-selected-split-input-from-sources` | `task-30` | `spire` |
| `review/30448-spire-selected-split-input-from-heap-sources` | `reviews/task-30/377-30448-spire-selected-split-input-from-heap-sources` | `task-30` | `spire` |
| `review/30449-spire-split-heap-dead-row-contract` | `reviews/task-30/378-30449-spire-split-heap-dead-row-contract` | `task-30` | `spire` |
| `review/30450-spire-maintenance-feedback-polish` | `reviews/task-30/379-30450-spire-maintenance-feedback-polish` | `task-30` | `spire` |
| `review/30451-spire-maintenance-run-result-shape` | `reviews/task-30/380-30451-spire-maintenance-run-result-shape` | `task-30` | `spire` |
| `review/30452-spire-locked-maintenance-run-plan` | `reviews/task-30/381-30452-spire-locked-maintenance-run-plan` | `task-30` | `spire` |
| `review/30453-spire-scheduled-replacement-object-versions` | `reviews/task-30/382-30453-spire-scheduled-replacement-object-versions` | `task-30` | `spire` |
| `review/30454-spire-maintenance-run-entrypoint` | `reviews/task-30/383-30454-spire-maintenance-run-entrypoint` | `task-30` | `spire` |
| `review/30455-spire-maintenance-run-empty-sql-smoke` | `reviews/task-30/384-30455-spire-maintenance-run-empty-sql-smoke` | `task-30` | `spire` |
| `review/30456-spire-maintenance-merge-publish-smoke` | `reviews/task-30/385-30456-spire-maintenance-merge-publish-smoke` | `task-30` | `spire` |
| `review/30457-spire-maintenance-split-publish-smoke` | `reviews/task-30/386-30457-spire-maintenance-split-publish-smoke` | `task-30` | `spire` |
| `review/30458-spire-locked-maintenance-run-plan-sql-smoke` | `reviews/task-30/387-30458-spire-locked-maintenance-run-plan-sql-smoke` | `task-30` | `spire` |
| `review/30459-spire-maintenance-merge-rerun-noop-smoke` | `reviews/task-30/388-30459-spire-maintenance-merge-rerun-noop-smoke` | `task-30` | `spire` |
| `review/30460-spire-phase2-scheduler-status-refresh` | `reviews/task-30/389-30460-spire-phase2-scheduler-status-refresh` | `task-30` | `spire` |
| `review/30461-spire-maintenance-publish-scan-visibility` | `reviews/task-30/390-30461-spire-maintenance-publish-scan-visibility` | `task-30` | `spire` |
| `review/30462-spire-maintenance-no-candidate-sql-smoke` | `reviews/task-30/391-30462-spire-maintenance-no-candidate-sql-smoke` | `task-30` | `spire` |
| `review/30463-spire-maintenance-run-volatility` | `reviews/task-30/392-30463-spire-maintenance-run-volatility` | `task-30` | `spire` |
| `review/30464-spire-locked-run-plan-publish-consistency` | `reviews/task-30/393-30464-spire-locked-run-plan-publish-consistency` | `task-30` | `spire` |
| `review/30465-spire-update-mechanics-scheduler-design-refresh` | `reviews/task-30/394-30465-spire-update-mechanics-scheduler-design-refresh` | `task-30` | `spire` |
| `review/30466-spire-deferred-scheduler-reclamation-accounting` | `reviews/task-30/395-30466-spire-deferred-scheduler-reclamation-accounting` | `task-30` | `spire` |
| `review/30467-spire-phase2-local-scheduler-readiness` | `reviews/task-30/396-30467-spire-phase2-local-scheduler-readiness` | `task-30` | `spire` |
| `review/30468-spire-phase2-concurrency-validation-status` | `reviews/task-30/397-30468-spire-phase2-concurrency-validation-status` | `task-30` | `spire` |
| `review/30469-spire-recursive-hierarchy-design` | `reviews/task-30/398-30469-spire-recursive-hierarchy-design` | `task-30` | `spire` |
| `review/30470-spire-recursive-hierarchy-shape-validation` | `reviews/task-30/399-30470-spire-recursive-hierarchy-shape-validation` | `task-30` | `spire` |
| `review/30471-spire-recursive-routing-draft-helper` | `reviews/task-30/400-30471-spire-recursive-routing-draft-helper` | `task-30` | `spire` |
| `review/30472-spire-recursive-centroid-records` | `reviews/task-30/401-30472-spire-recursive-centroid-records` | `task-30` | `spire` |
| `review/30473-spire-level-local-route-primitive` | `reviews/task-30/402-30473-spire-level-local-route-primitive` | `task-30` | `spire` |
| `review/30474-spire-recursive-route-coordinator` | `reviews/task-30/403-30474-spire-recursive-route-coordinator` | `task-30` | `spire` |
| `review/30475-spire-flat-recursive-routing-comparison` | `reviews/task-30/404-30475-spire-flat-recursive-routing-comparison` | `task-30` | `spire` |
| `review/30476-spire-recursive-routing-preload` | `reviews/task-30/405-30476-spire-recursive-routing-preload` | `task-30` | `spire` |
| `review/30477-spire-recursive-routed-leaf-rows` | `reviews/task-30/406-30477-spire-recursive-routed-leaf-rows` | `task-30` | `spire` |
| `review/30478-spire-recursive-quantized-candidates` | `reviews/task-30/407-30478-spire-recursive-quantized-candidates` | `task-30` | `spire` |
| `review/30479-spire-recursive-leaf-count` | `reviews/task-30/408-30479-spire-recursive-leaf-count` | `task-30` | `spire` |
| `review/30480-spire-recursive-nprobe-policy` | `reviews/task-30/409-30480-spire-recursive-nprobe-policy` | `task-30` | `spire` |
| `review/30481-spire-flat-recursive-candidate-comparison` | `reviews/task-30/410-30481-spire-flat-recursive-candidate-comparison` | `task-30` | `spire` |
| `review/30482-spire-local-recursive-routing-epoch` | `reviews/task-30/411-30482-spire-local-recursive-routing-epoch` | `task-30` | `spire` |
| `review/30483-spire-recursive-epoch-leaf-parent-validation` | `reviews/task-30/412-30483-spire-recursive-epoch-leaf-parent-validation` | `task-30` | `spire` |
| `review/30484-spire-materialized-recursive-scan-proof` | `reviews/task-30/413-30484-spire-materialized-recursive-scan-proof` | `task-30` | `spire` |
| `review/30485-spire-recursive-relation-writer-seam` | `reviews/task-30/414-30485-spire-recursive-relation-writer-seam` | `task-30` | `spire` |
| `review/30486-spire-recursive-leaf-input-writer` | `reviews/task-30/415-30486-spire-recursive-leaf-input-writer` | `task-30` | `spire` |
| `review/30487-spire-recursive-build-input-coordinator` | `reviews/task-30/416-30487-spire-recursive-build-input-coordinator` | `task-30` | `spire` |
| `review/30488-spire-recursive-epoch-centroid-records` | `reviews/task-30/417-30488-spire-recursive-epoch-centroid-records` | `task-30` | `spire` |
| `review/30489-spire-recursive-epoch-publish-bundle` | `reviews/task-30/418-30489-spire-recursive-epoch-publish-bundle` | `task-30` | `spire` |
| `review/30490-spire-recursive-relation-publish-bridge` | `reviews/task-30/419-30490-spire-recursive-relation-publish-bridge` | `task-30` | `spire` |
| `review/30491-spire-recursive-relation-build-composition` | `reviews/task-30/420-30491-spire-recursive-relation-build-composition` | `task-30` | `spire` |
| `review/30492-spire-recursive-fanout-reloption` | `reviews/task-30/421-30492-spire-recursive-fanout-reloption` | `task-30` | `spire` |
| `review/30493-spire-recursive-fanout-build-activation` | `reviews/task-30/422-30493-spire-recursive-fanout-build-activation` | `task-30` | `spire` |
| `review/30494-spire-recursive-routing-support-diagnostic` | `reviews/task-30/423-30494-spire-recursive-routing-support-diagnostic` | `task-30` | `spire` |
| `review/30495-spire-flat-recursive-sql-comparison` | `reviews/task-30/424-30495-spire-flat-recursive-sql-comparison` | `task-30` | `spire` |
| `review/30496-spire-routing-centroid-snapshot` | `reviews/task-30/425-30496-spire-routing-centroid-snapshot` | `task-30` | `spire` |
| `review/30497-spire-recursive-options-diagnostics` | `reviews/task-30/426-30497-spire-recursive-options-diagnostics` | `task-30` | `spire` |
| `review/30498-spire-phase3-coordinator-scan-closeout` | `reviews/task-30/427-30498-spire-phase3-coordinator-scan-closeout` | `task-30` | `spire` |
| `review/30499-spire-level-parameter-diagnostics` | `reviews/task-30/428-30499-spire-level-parameter-diagnostics` | `task-30` | `spire` |
| `review/30500-spire-three-level-routing-coverage` | `reviews/task-30/429-30500-spire-three-level-routing-coverage` | `task-30` | `spire` |
| `review/30501-spire-recursive-draft-invariants` | `reviews/task-30/430-30501-spire-recursive-draft-invariants` | `task-30` | `spire` |
| `review/30502-spire-recursive-fanout-validation` | `reviews/task-30/431-30502-spire-recursive-fanout-validation` | `task-30` | `spire` |
| `review/30503-spire-recursive-maintenance-guard` | `reviews/task-30/432-30503-spire-recursive-maintenance-guard` | `task-30` | `spire` |
| `review/30504-spire-degraded-recursive-routing` | `reviews/task-30/433-30504-spire-degraded-recursive-routing` | `task-30` | `spire` |
| `review/30505-spire-recursive-sql-parity-breadth` | `reviews/task-30/434-30505-spire-recursive-sql-parity-breadth` | `task-30` | `spire` |
| `review/30506-spire-options-nprobe-level-policy` | `reviews/task-30/435-30506-spire-options-nprobe-level-policy` | `task-30` | `spire` |
| `review/30507-spire-recursive-draft-invariant-helper` | `reviews/task-30/436-30507-spire-recursive-draft-invariant-helper` | `task-30` | `spire` |
| `review/30508-spire-recursive-nprobe-docs` | `reviews/task-30/437-30508-spire-recursive-nprobe-docs` | `task-30` | `spire` |
| `review/30509-spire-phase4-local-placement-design` | `reviews/task-30/438-30509-spire-phase4-local-placement-design` | `task-30` | `spire` |
| `review/30510-spire-local-store-config-metadata` | `reviews/task-30/439-30510-spire-local-store-config-metadata` | `task-30` | `spire` |
| `review/30511-spire-pid-hash-placement-planner` | `reviews/task-30/440-30511-spire-pid-hash-placement-planner` | `task-30` | `spire` |
| `review/30512-spire-local-store-count-option` | `reviews/task-30/441-30512-spire-local-store-count-option` | `task-30` | `spire` |
| `review/30513-spire-local-store-tablespaces-option` | `reviews/task-30/442-30513-spire-local-store-tablespaces-option` | `task-30` | `spire` |
| `review/30514-spire-object-store-local-store-id-surface` | `reviews/task-30/443-30514-spire-object-store-local-store-id-surface` | `task-30` | `spire` |
| `review/30515-spire-hash-routed-local-build-writer` | `reviews/task-30/444-30515-spire-hash-routed-local-build-writer` | `task-30` | `spire` |
| `review/30516-spire-local-store-tablespace-plan` | `reviews/task-30/445-30516-spire-local-store-tablespace-plan` | `task-30` | `spire` |
| `review/30517-spire-local-store-relation-name-plan` | `reviews/task-30/446-30517-spire-local-store-relation-name-plan` | `task-30` | `spire` |
| `review/30518-spire-local-store-descriptor-publish-plan` | `reviews/task-30/447-30518-spire-local-store-descriptor-publish-plan` | `task-30` | `spire` |
| `review/30519-spire-scan-leaf-route-store-grouping` | `reviews/task-30/448-30519-spire-scan-leaf-route-store-grouping` | `task-30` | `spire` |
| `review/30520-spire-scan-leaf-delta-read-grouping` | `reviews/task-30/449-30520-spire-scan-leaf-delta-read-grouping` | `task-30` | `spire` |
| `review/30521-spire-update-scan-module-split` | `reviews/task-30/450-30521-spire-update-scan-module-split` | `task-30` | `spire` |
| `review/30522-spire-test-and-publish-chunk-split` | `reviews/task-30/451-30522-spire-test-and-publish-chunk-split` | `task-30` | `spire` |
| `review/30523-spire-storage-module-split` | `reviews/task-30/452-30523-spire-storage-module-split` | `task-30` | `spire` |
| `review/30524-spire-build-module-split` | `reviews/task-30/453-30524-spire-build-module-split` | `task-30` | `spire` |
| `review/30525-spire-root-module-split` | `reviews/task-30/454-30525-spire-root-module-split` | `task-30` | `spire` |
| `review/30526-spire-metadata-module-split` | `reviews/task-30/455-30526-spire-metadata-module-split` | `task-30` | `spire` |
| `review/30527-spire-auxiliary-local-store-relations` | `reviews/task-30/456-30527-spire-auxiliary-local-store-relations` | `task-30` | `spire` |
| `review/30529-spire-large-routing-object-chain` | `reviews/task-30/457-30529-spire-large-routing-object-chain` | `task-30` | `spire` |
| `review/30531-spire-mutation-local-store-routing` | `reviews/task-30/458-30531-spire-mutation-local-store-routing` | `task-30` | `spire` |
| `review/30532-spire-scan-prefetch-placement-resolution` | `reviews/task-30/459-30532-spire-scan-prefetch-placement-resolution` | `task-30` | `spire` |
| `review/30534-spire-readstream-local-fetch` | `reviews/task-30/460-30534-spire-readstream-local-fetch` | `task-30` | `spire` |
| `review/30535-spire-multistore-sql-vacuum` | `reviews/task-30/461-30535-spire-multistore-sql-vacuum` | `task-30` | `spire` |
| `review/30536-spire-storage-debt-multistore` | `reviews/task-30/462-30536-spire-storage-debt-multistore` | `task-30` | `spire` |
| `review/30537-spire-aux-store-autovacuum-guard` | `reviews/task-30/463-30537-spire-aux-store-autovacuum-guard` | `task-30` | `spire` |
| `review/30538-spire-phase4-prelanding-guards` | `reviews/task-30/464-30538-spire-phase4-prelanding-guards` | `task-30` | `spire` |
| `review/30539-spire-larger-multistore-fixture` | `reviews/task-30/465-30539-spire-larger-multistore-fixture` | `task-30` | `spire` |
| `review/30540-spire-aux-store-autovacuum-relcache` | `reviews/task-30/466-30540-spire-aux-store-autovacuum-relcache` | `task-30` | `spire` |
| `review/30541-spire-boundary-replication-design` | `reviews/task-30/467-30541-spire-boundary-replication-design` | `task-30` | `spire` |
| `review/30542-spire-boundary-replication-planning-surface` | `reviews/task-30/468-30542-spire-boundary-replication-planning-surface` | `task-30` | `spire` |
| `review/30543-spire-boundary-replica-build-scan` | `reviews/task-30/469-30543-spire-boundary-replica-build-scan` | `task-30` | `spire` |
| `review/30544-spire-boundary-insert-delta-fanout` | `reviews/task-30/470-30544-spire-boundary-insert-delta-fanout` | `task-30` | `spire` |
| `review/30545-spire-recursive-boundary-replica-build` | `reviews/task-30/471-30545-spire-recursive-boundary-replica-build` | `task-30` | `spire` |
| `review/30546-spire-split-replacement-boundary-fanout` | `reviews/task-30/472-30546-spire-split-replacement-boundary-fanout` | `task-30` | `spire` |
| `review/30547-spire-boundary-storage-accounting` | `reviews/task-30/473-30547-spire-boundary-storage-accounting` | `task-30` | `spire` |
| `review/30549-spire-top-graph-codec` | `reviews/task-30/474-30549-spire-top-graph-codec` | `task-30` | `spire` |
| `review/30549-spire-top-level-graph-design` | `reviews/task-30/475-30549-spire-top-level-graph-design` | `task-30` | `spire` |
| `review/30550-spire-top-graph-build-draft` | `reviews/task-30/476-30550-spire-top-graph-build-draft` | `task-30` | `spire` |
| `review/30551-spire-boundary-review-followups` | `reviews/task-30/477-30551-spire-boundary-review-followups` | `task-30` | `spire` |
| `review/30552-spire-remote-node-model` | `reviews/task-30/478-30552-spire-remote-node-model` | `task-30` | `spire` |
| `review/30553-spire-remote-search-api` | `reviews/task-30/479-30553-spire-remote-search-api` | `task-30` | `spire` |
| `review/30554-spire-remote-candidate-merge` | `reviews/task-30/480-30554-spire-remote-candidate-merge` | `task-30` | `spire` |
| `review/30555-spire-phase7-review-followups` | `reviews/task-30/481-30555-spire-phase7-review-followups` | `task-30` | `spire` |
| `review/30556-spire-remote-search-fail-closed-tests` | `reviews/task-30/482-30556-spire-remote-search-fail-closed-tests` | `task-30` | `spire` |
| `review/30557-spire-remote-search-fanout-planner` | `reviews/task-30/483-30557-spire-remote-search-fanout-planner` | `task-30` | `spire` |
| `review/30558-spire-remote-fanout-diagnostic-sql` | `reviews/task-30/484-30558-spire-remote-fanout-diagnostic-sql` | `task-30` | `spire` |
| `review/30559-spire-remote-candidate-receive-validation` | `reviews/task-30/485-30559-spire-remote-candidate-receive-validation` | `task-30` | `spire` |
| `review/30560-spire-coordinator-local-remote-search` | `reviews/task-30/486-30560-spire-coordinator-local-remote-search` | `task-30` | `spire` |
| `review/30561-spire-remote-search-target-plan` | `reviews/task-30/487-30561-spire-remote-search-target-plan` | `task-30` | `spire` |
| `review/30562-spire-remote-node-snapshot` | `reviews/task-30/488-30562-spire-remote-node-snapshot` | `task-30` | `spire` |
| `review/30563-spire-remote-target-readiness` | `reviews/task-30/489-30563-spire-remote-target-readiness` | `task-30` | `spire` |
| `review/30564-spire-remote-node-capability-plan` | `reviews/task-30/490-30564-spire-remote-node-capability-plan` | `task-30` | `spire` |
| `review/30565-spire-remote-capability-summaries` | `reviews/task-30/491-30565-spire-remote-capability-summaries` | `task-30` | `spire` |
| `review/30566-spire-remote-search-execution-plan` | `reviews/task-30/492-30566-spire-remote-search-execution-plan` | `task-30` | `spire` |
| `review/30567-spire-remote-libpq-request-envelope` | `reviews/task-30/493-30567-spire-remote-libpq-request-envelope` | `task-30` | `spire` |
| `review/30568-spire-remote-receive-merge-contracts` | `reviews/task-30/494-30568-spire-remote-receive-merge-contracts` | `task-30` | `spire` |
| `review/30569-spire-remote-finalization-contracts` | `reviews/task-30/495-30569-spire-remote-finalization-contracts` | `task-30` | `spire` |
| `review/30570-spire-remote-diagnostic-string-registry` | `reviews/task-30/496-30570-spire-remote-diagnostic-string-registry` | `task-30` | `spire` |
| `review/30571-spire-remote-node-contract-string-sharing` | `reviews/task-30/497-30571-spire-remote-node-contract-string-sharing` | `task-30` | `spire` |
| `review/30572-spire-remote-summary-rollups` | `reviews/task-30/498-30572-spire-remote-summary-rollups` | `task-30` | `spire` |
| `review/30573-spire-remote-merge-tie-breaker-invariant` | `reviews/task-30/499-30573-spire-remote-merge-tie-breaker-invariant` | `task-30` | `spire` |
| `review/30574-spire-remote-node-descriptor-contract` | `reviews/task-30/500-30574-spire-remote-node-descriptor-contract` | `task-30` | `spire` |
| `review/30575-spire-remote-node-descriptor-readiness` | `reviews/task-30/501-30575-spire-remote-node-descriptor-readiness` | `task-30` | `spire` |
| `review/30576-spire-remote-epoch-policy-contracts` | `reviews/task-30/502-30576-spire-remote-epoch-policy-contracts` | `task-30` | `spire` |
| `review/30577-spire-remote-coordinator-gate` | `reviews/task-30/503-30577-spire-remote-coordinator-gate` | `task-30` | `spire` |
| `review/30578-spire-remote-heap-resolution-contract` | `reviews/task-30/504-30578-spire-remote-heap-resolution-contract` | `task-30` | `spire` |
| `review/30579-spire-local-heap-resolution-plan` | `reviews/task-30/505-30579-spire-local-heap-resolution-plan` | `task-30` | `spire` |
| `review/30580-spire-heap-resolution-summary` | `reviews/task-30/506-30580-spire-heap-resolution-summary` | `task-30` | `spire` |
| `review/30581-spire-libpq-parameter-contract` | `reviews/task-30/507-30581-spire-libpq-parameter-contract` | `task-30` | `spire` |
| `review/30582-spire-local-heap-candidates` | `reviews/task-30/508-30582-spire-local-heap-candidates` | `task-30` | `spire` |
| `review/30583-spire-degraded-heap-status` | `reviews/task-30/509-30583-spire-degraded-heap-status` | `task-30` | `spire` |
| `review/30584-spire-coordinator-result-summary` | `reviews/task-30/510-30584-spire-coordinator-result-summary` | `task-30` | `spire` |
| `review/30585-spire-descriptor-registration-contract` | `reviews/task-30/511-30585-spire-descriptor-registration-contract` | `task-30` | `spire` |
| `review/30586-spire-descriptor-lifecycle-strings` | `reviews/task-30/512-30586-spire-descriptor-lifecycle-strings` | `task-30` | `spire` |
| `review/30587-spire-local-locator-decode-context` | `reviews/task-30/513-30587-spire-local-locator-decode-context` | `task-30` | `spire` |
| `review/30588-spire-degradation-policy-invariant` | `reviews/task-30/514-30588-spire-degradation-policy-invariant` | `task-30` | `spire` |
| `review/30589-spire-summary-status-precedence` | `reviews/task-30/515-30589-spire-summary-status-precedence` | `task-30` | `spire` |
| `review/30590-spire-coordinator-gate-reuse` | `reviews/task-30/516-30590-spire-coordinator-gate-reuse` | `task-30` | `spire` |
| `review/30591-spire-coordinator-execution-reuse` | `reviews/task-30/517-30591-spire-coordinator-execution-reuse` | `task-30` | `spire` |
| `review/30592-spire-degradation-policy-closeout` | `reviews/task-30/518-30592-spire-degradation-policy-closeout` | `task-30` | `spire` |
| `review/30593-spire-merge-semantics-closeout` | `reviews/task-30/519-30593-spire-merge-semantics-closeout` | `task-30` | `spire` |
| `review/30594-spire-remote-epoch-publish-gate` | `reviews/task-30/520-30594-spire-remote-epoch-publish-gate` | `task-30` | `spire` |
| `review/30595-spire-remote-descriptor-catalog` | `reviews/task-30/521-30595-spire-remote-descriptor-catalog` | `task-30` | `spire` |
| `review/30596-spire-libpq-connection-plan` | `reviews/task-30/522-30596-spire-libpq-connection-plan` | `task-30` | `spire` |
| `review/30597-spire-remote-epoch-manifest-plan` | `reviews/task-30/523-30597-spire-remote-epoch-manifest-plan` | `task-30` | `spire` |
| `review/30598-spire-remote-epoch-manifest-persistence` | `reviews/task-30/524-30598-spire-remote-epoch-manifest-persistence` | `task-30` | `spire` |
| `review/30599-spire-descriptor-state-registry` | `reviews/task-30/525-30599-spire-descriptor-state-registry` | `task-30` | `spire` |
| `review/30600-spire-remote-manifest-catalog-summary` | `reviews/task-30/526-30600-spire-remote-manifest-catalog-summary` | `task-30` | `spire` |
| `review/30601-spire-libpq-executor-readiness` | `reviews/task-30/527-30601-spire-libpq-executor-readiness` | `task-30` | `spire` |
| `review/30602-spire-remote-manifest-publication-plan` | `reviews/task-30/528-30602-spire-remote-manifest-publication-plan` | `task-30` | `spire` |
| `review/30603-spire-contract-drift-invariants` | `reviews/task-30/529-30603-spire-contract-drift-invariants` | `task-30` | `spire` |
| `review/30604-spire-remote-search-bind-plan` | `reviews/task-30/530-30604-spire-remote-search-bind-plan` | `task-30` | `spire` |
| `review/30605-spire-manifest-bind-plan` | `reviews/task-30/531-30605-spire-manifest-bind-plan` | `task-30` | `spire` |
| `review/30606-spire-libpq-bind-summaries` | `reviews/task-30/532-30606-spire-libpq-bind-summaries` | `task-30` | `spire` |
| `review/30607-spire-libpq-executor-work-plans` | `reviews/task-30/533-30607-spire-libpq-executor-work-plans` | `task-30` | `spire` |
| `review/30608-spire-libpq-executor-work-summaries` | `reviews/task-30/534-30608-spire-libpq-executor-work-summaries` | `task-30` | `spire` |
| `review/30609-spire-manifest-libpq-receive-boundary` | `reviews/task-30/535-30609-spire-manifest-libpq-receive-boundary` | `task-30` | `spire` |
| `review/30610-spire-manifest-publication-gate-summary` | `reviews/task-30/536-30610-spire-manifest-publication-gate-summary` | `task-30` | `spire` |
| `review/30611-spire-remote-search-receive-summary` | `reviews/task-30/537-30611-spire-remote-search-receive-summary` | `task-30` | `spire` |
| `review/30612-spire-coordinator-gate-receive-status` | `reviews/task-30/538-30612-spire-coordinator-gate-receive-status` | `task-30` | `spire` |
| `review/30613-spire-result-summary-receive-status` | `reviews/task-30/539-30613-spire-result-summary-receive-status` | `task-30` | `spire` |
| `review/30614-spire-manifest-publication-result-summary` | `reviews/task-30/540-30614-spire-manifest-publication-result-summary` | `task-30` | `spire` |
| `review/30615-spire-manifest-publication-blocked-result` | `reviews/task-30/541-30615-spire-manifest-publication-blocked-result` | `task-30` | `spire` |
| `review/30616-spire-manifest-result-source-contract` | `reviews/task-30/542-30616-spire-manifest-result-source-contract` | `task-30` | `spire` |
| `review/30617-spire-search-result-source-contract` | `reviews/task-30/543-30617-spire-search-result-source-contract` | `task-30` | `spire` |
| `review/30618-spire-search-empty-result-source` | `reviews/task-30/544-30618-spire-search-empty-result-source` | `task-30` | `spire` |
| `review/30619-spire-manifest-result-contract-feedback` | `reviews/task-30/545-30619-spire-manifest-result-contract-feedback` | `task-30` | `spire` |
| `review/30620-spire-planner-cost-model` | `reviews/task-30/546-30620-spire-planner-cost-model` | `task-30` | `spire` |
| `review/30621-spire-planner-cost-snapshot` | `reviews/task-30/547-30621-spire-planner-cost-snapshot` | `task-30` | `spire` |
| `review/30623-spire-suite-config` | `reviews/task-30/548-30623-spire-suite-config` | `task-30` | `spire` |
| `review/30625-spire-maintenance-scheduler-cleanup` | `reviews/task-30/549-30625-spire-maintenance-scheduler-cleanup` | `task-30` | `spire` |
| `review/30626-spire-local-correctness-matrix` | `reviews/task-30/550-30626-spire-local-correctness-matrix` | `task-30` | `spire` |
| `review/30627-spire-operator-docs` | `reviews/task-30/551-30627-spire-operator-docs` | `task-30` | `spire` |
| `review/30628-spire-old-epoch-physical-cleanup` | `reviews/task-30/552-30628-spire-old-epoch-physical-cleanup` | `task-30` | `spire` |
| `review/30629-spire-scale-packet-runbook` | `reviews/task-30/553-30629-spire-scale-packet-runbook` | `task-30` | `spire` |
| `review/30630-spire-remote-operator-contracts` | `reviews/task-30/554-30630-spire-remote-operator-contracts` | `task-30` | `spire` |
| `review/30631-spire-coordinator-pipeline-bundle` | `reviews/task-30/555-30631-spire-coordinator-pipeline-bundle` | `task-30` | `spire` |
| `review/30632-spire-manifest-persist-epoch-guard` | `reviews/task-30/556-30632-spire-manifest-persist-epoch-guard` | `task-30` | `spire` |
| `review/30633-spire-conninfo-secret-contract` | `reviews/task-30/557-30633-spire-conninfo-secret-contract` | `task-30` | `spire` |
| `review/30634-spire-remote-catalog-orphan-cleanup` | `reviews/task-30/558-30634-spire-remote-catalog-orphan-cleanup` | `task-30` | `spire` |
| `review/30635-spire-feedback-followups` | `reviews/task-30/559-30635-spire-feedback-followups` | `task-30` | `spire` |
| `review/30636-spire-remote-catalog-lifecycle-contract` | `reviews/task-30/560-30636-spire-remote-catalog-lifecycle-contract` | `task-30` | `spire` |
| `review/30637-spire-remote-upgrade-catalog-tables` | `reviews/task-30/561-30637-spire-remote-upgrade-catalog-tables` | `task-30` | `spire` |
| `review/30638-spire-conninfo-secret-status` | `reviews/task-30/562-30638-spire-conninfo-secret-status` | `task-30` | `spire` |
| `review/30639-spire-remote-catalog-index-cleanup` | `reviews/task-30/563-30639-spire-remote-catalog-index-cleanup` | `task-30` | `spire` |
| `review/30640-spire-remote-search-secret-plan` | `reviews/task-30/564-30640-spire-remote-search-secret-plan` | `task-30` | `spire` |
| `review/30641-spire-executor-secret-readiness` | `reviews/task-30/565-30641-spire-executor-secret-readiness` | `task-30` | `spire` |
| `review/30642-spire-secret-operator-entrypoints` | `reviews/task-30/566-30642-spire-secret-operator-entrypoints` | `task-30` | `spire` |
| `review/30643-spire-secret-key-collision-guard` | `reviews/task-30/567-30643-spire-secret-key-collision-guard` | `task-30` | `spire` |
| `review/30644-spire-upgrade-state-check-invariant` | `reviews/task-30/568-30644-spire-upgrade-state-check-invariant` | `task-30` | `spire` |
| `review/30645-spire-remote-search-connection-open-plan` | `reviews/task-30/569-30645-spire-remote-search-connection-open-plan` | `task-30` | `spire` |
| `review/30646-spire-remote-search-executor-connection-check` | `reviews/task-30/570-30646-spire-remote-search-executor-connection-check` | `task-30` | `spire` |
| `review/30647-spire-remote-search-libpq-executor-send` | `reviews/task-30/571-30647-spire-remote-search-libpq-executor-send` | `task-30` | `spire` |
| `review/30648-spire-libpq-executor-nonempty-receive` | `reviews/task-30/572-30648-spire-libpq-executor-nonempty-receive` | `task-30` | `spire` |
| `review/30649-spire-manifest-libpq-executor-results` | `reviews/task-30/573-30649-spire-manifest-libpq-executor-results` | `task-30` | `spire` |
| `review/30650-spire-remote-epoch-manifest-apply` | `reviews/task-30/574-30650-spire-remote-epoch-manifest-apply` | `task-30` | `spire` |
| `review/30651-spire-remote-heap-libpq-candidates` | `reviews/task-30/575-30651-spire-remote-heap-libpq-candidates` | `task-30` | `spire` |
| `review/30652-spire-drop-index-catalog-cleanup` | `reviews/task-30/576-30652-spire-drop-index-catalog-cleanup` | `task-30` | `spire` |
| `review/30653-spire-multicluster-pg18-smoke` | `reviews/task-30/577-30653-spire-multicluster-pg18-smoke` | `task-30` | `spire` |
| `review/30654-spire-result-composition-closeout` | `reviews/task-30/578-30654-spire-result-composition-closeout` | `task-30` | `spire` |
| `review/30655-spire-pipeline-steps-consolidation` | `reviews/task-30/579-30655-spire-pipeline-steps-consolidation` | `task-30` | `spire` |
| `review/30656-spire-per-level-nprobe` | `reviews/task-30/580-30656-spire-per-level-nprobe` | `task-30` | `spire` |
| `review/30658-spire-phase9-routing-plan` | `reviews/task-30/581-30658-spire-phase9-routing-plan` | `task-30` | `spire` |
| `review/30659-spire-phase9-phase10-task-files` | `reviews/task-30/582-30659-spire-phase9-phase10-task-files` | `task-30` | `spire` |
| `review/30660-spire-top-graph-frontier-contract` | `reviews/task-30/583-30660-spire-top-graph-frontier-contract` | `task-30` | `spire` |
| `review/30661-spire-top-graph-chain-storage` | `reviews/task-30/584-30661-spire-top-graph-chain-storage` | `task-30` | `spire` |
| `review/30662-spire-borrowed-top-graph-routing` | `reviews/task-30/585-30662-spire-borrowed-top-graph-routing` | `task-30` | `spire` |
| `review/30663-spire-global-recursive-route-budget` | `reviews/task-30/586-30663-spire-global-recursive-route-budget` | `task-30` | `spire` |
| `review/30664-spire-recursive-routing-diagnostics` | `reviews/task-30/587-30664-spire-recursive-routing-diagnostics` | `task-30` | `spire` |
| `review/30665-spire-routing-review-contract-comments` | `reviews/task-30/588-30665-spire-routing-review-contract-comments` | `task-30` | `spire` |
| `review/30666-spire-boundary-replica-scan-diagnostics` | `reviews/task-30/589-30666-spire-boundary-replica-scan-diagnostics` | `task-30` | `spire` |
| `review/30667-spire-vector-identity-contract` | `reviews/task-30/590-30667-spire-vector-identity-contract` | `task-30` | `spire` |
| `review/30668-spire-routing-diagnostics-followups` | `reviews/task-30/591-30668-spire-routing-diagnostics-followups` | `task-30` | `spire` |
| `review/30669-spire-routing-diagnostic-drift-guard` | `reviews/task-30/592-30669-spire-routing-diagnostic-drift-guard` | `task-30` | `spire` |
| `review/30670-spire-bounded-candidate-collection` | `reviews/task-30/593-30670-spire-bounded-candidate-collection` | `task-30` | `spire` |
| `review/30671-spire-vector-dedupe-key-prefixes` | `reviews/task-30/594-30671-spire-vector-dedupe-key-prefixes` | `task-30` | `spire` |
| `review/30672-spire-candidate-truncation-diagnostics` | `reviews/task-30/595-30672-spire-candidate-truncation-diagnostics` | `task-30` | `spire` |
| `review/30673-spire-bounded-tie-break-ordering` | `reviews/task-30/596-30673-spire-bounded-tie-break-ordering` | `task-30` | `spire` |
| `review/30674-spire-routing-diagnostic-depth-guard` | `reviews/task-30/597-30674-spire-routing-diagnostic-depth-guard` | `task-30` | `spire` |
| `review/30675-spire-eager-bounded-scan-contract` | `reviews/task-30/598-30675-spire-eager-bounded-scan-contract` | `task-30` | `spire` |
| `review/30676-spire-heap-rerank-prefetch` | `reviews/task-30/599-30676-spire-heap-rerank-prefetch` | `task-30` | `spire` |
| `review/30677-spire-delta-row-reuse` | `reviews/task-30/600-30677-spire-delta-row-reuse` | `task-30` | `spire` |
| `review/30678-spire-indexed-store-lookup` | `reviews/task-30/601-30678-spire-indexed-store-lookup` | `task-30` | `spire` |
| `review/30679-spire-store-scan-read-diagnostics` | `reviews/task-30/602-30679-spire-store-scan-read-diagnostics` | `task-30` | `spire` |
| `review/30680-spire-top-graph-io-attribution` | `reviews/task-30/603-30680-spire-top-graph-io-attribution` | `task-30` | `spire` |
| `review/30681-spire-local-store-read-scheduling` | `reviews/task-30/604-30681-spire-local-store-read-scheduling` | `task-30` | `spire` |
| `review/30682-spire-remote-libpq-executor-boundary` | `reviews/task-30/605-30682-spire-remote-libpq-executor-boundary` | `task-30` | `spire` |
| `review/30683-spire-remote-heap-resolution-contract` | `reviews/task-30/606-30683-spire-remote-heap-resolution-contract` | `task-30` | `spire` |
| `review/30684-spire-routing-drift-fallback-closeout` | `reviews/task-30/607-30684-spire-routing-drift-fallback-closeout` | `task-30` | `spire` |
| `review/30685-spire-local-scan-pipeline-snapshot` | `reviews/task-30/608-30685-spire-local-scan-pipeline-snapshot` | `task-30` | `spire` |
| `review/30686-spire-phase9-quality-baseline` | `reviews/task-30/609-30686-spire-phase9-quality-baseline` | `task-30` | `spire` |
| `review/30687-spire-adaptive-nprobe` | `reviews/task-30/610-30687-spire-adaptive-nprobe` | `task-30` | `spire` |
| `review/30688-spire-quality-deferrals` | `reviews/task-30/611-30688-spire-quality-deferrals` | `task-30` | `spire` |
| `review/30689-spire-phase-task-overview-alignment` | `reviews/task-30/612-30689-spire-phase-task-overview-alignment` | `task-30` | `spire` |
| `review/30691-spire-phase11-production-parity-plan` | `reviews/task-30/613-30691-spire-phase11-production-parity-plan` | `task-30` | `spire` |
| `review/30692-spire-phase11-paper-parity-gate` | `reviews/task-30/614-30692-spire-phase11-paper-parity-gate` | `task-30` | `spire` |
| `review/30693-spire-vector-identity-allocation-sources` | `reviews/task-30/615-30693-spire-vector-identity-allocation-sources` | `task-30` | `spire` |
| `review/30694-spire-phase11-plan-gap-review` | `reviews/task-30/616-30694-spire-phase11-plan-gap-review` | `task-30` | `spire` |
| `review/30695-spire-writer-identity-contract-status` | `reviews/task-30/617-30695-spire-writer-identity-contract-status` | `task-30` | `spire` |
| `review/30696-spire-phase11-libpq-security-scope` | `reviews/task-30/618-30696-spire-phase11-libpq-security-scope` | `task-30` | `spire` |
| `review/30697-spire-leaf-v2-global-vec-id-storage` | `reviews/task-30/619-30697-spire-leaf-v2-global-vec-id-storage` | `task-30` | `spire` |
| `review/30698-spire-stable-source-identity-contract` | `reviews/task-30/620-30698-spire-stable-source-identity-contract` | `task-30` | `spire` |
| `review/30699-spire-source-identity-provider-adr` | `reviews/task-30/621-30699-spire-source-identity-provider-adr` | `task-30` | `spire` |
| `review/30700-spire-source-identity-include-provider` | `reviews/task-30/622-30700-spire-source-identity-include-provider` | `task-30` | `spire` |
| `review/30701-spire-replacement-global-vec-id-proof` | `reviews/task-30/623-30701-spire-replacement-global-vec-id-proof` | `task-30` | `spire` |
| `review/30702-spire-remote-endpoint-contract-gate` | `reviews/task-30/624-30702-spire-remote-endpoint-contract-gate` | `task-30` | `spire` |
| `review/30703-spire-remote-endpoint-identity-gate` | `reviews/task-30/625-30703-spire-remote-endpoint-identity-gate` | `task-30` | `spire` |
| `review/30704-spire-remote-endpoint-identity-envelope` | `reviews/task-30/626-30704-spire-remote-endpoint-identity-envelope` | `task-30` | `spire` |
| `review/30705-spire-remote-endpoint-ready-gate` | `reviews/task-30/627-30705-spire-remote-endpoint-ready-gate` | `task-30` | `spire` |
| `review/30706-spire-remote-endpoint-strict-rejection` | `reviews/task-30/628-30706-spire-remote-endpoint-strict-rejection` | `task-30` | `spire` |
| `review/30707-spire-endpoint-diagnostic-contract` | `reviews/task-30/629-30707-spire-endpoint-diagnostic-contract` | `task-30` | `spire` |
| `review/30708-spire-libpq-receive-attempt-diagnostics` | `reviews/task-30/630-30708-spire-libpq-receive-attempt-diagnostics` | `task-30` | `spire` |
| `review/30709-spire-remote-heap-endpoint-gate` | `reviews/task-30/631-30709-spire-remote-heap-endpoint-gate` | `task-30` | `spire` |
| `review/30710-spire-remote-descriptor-identity-binding` | `reviews/task-30/632-30710-spire-remote-descriptor-identity-binding` | `task-30` | `spire` |
| `review/30711-spire-remote-capability-search-gates` | `reviews/task-30/633-30711-spire-remote-capability-search-gates` | `task-30` | `spire` |
| `review/30712-spire-libpq-identity-cache-contract` | `reviews/task-30/634-30712-spire-libpq-identity-cache-contract` | `task-30` | `spire` |
| `review/30713-spire-libpq-identity-cache-state` | `reviews/task-30/635-30713-spire-libpq-identity-cache-state` | `task-30` | `spire` |
| `review/30714-spire-libpq-identity-cache-test-matrix` | `reviews/task-30/636-30714-spire-libpq-identity-cache-test-matrix` | `task-30` | `spire` |
| `review/30715-spire-libpq-executor-budget-limits` | `reviews/task-30/637-30715-spire-libpq-executor-budget-limits` | `task-30` | `spire` |
| `review/30716-spire-libpq-degraded-identity-mismatch` | `reviews/task-30/638-30716-spire-libpq-degraded-identity-mismatch` | `task-30` | `spire` |
| `review/30717-spire-libpq-global-dispatch-governance` | `reviews/task-30/639-30717-spire-libpq-global-dispatch-governance` | `task-30` | `spire` |
| `review/30718-spire-pipeline-steps-live-probe` | `reviews/task-30/640-30718-spire-pipeline-steps-live-probe` | `task-30` | `spire` |
| `review/30719-spire-nprobe-parsed-options` | `reviews/task-30/641-30719-spire-nprobe-parsed-options` | `task-30` | `spire` |
| `review/30720-spire-advisory-lock-namespace` | `reviews/task-30/642-30720-spire-advisory-lock-namespace` | `task-30` | `spire` |
| `review/30721-spire-per-node-governance-isolation` | `reviews/task-30/643-30721-spire-per-node-governance-isolation` | `task-30` | `spire` |
| `review/30722-spire-production-coordinator-executor-plan` | `reviews/task-30/644-30722-spire-production-coordinator-executor-plan` | `task-30` | `spire` |
| `review/30723-spire-production-executor-dry-state` | `reviews/task-30/645-30723-spire-production-executor-dry-state` | `task-30` | `spire` |
| `review/30724-spire-production-transport-probe-adapter` | `reviews/task-30/646-30724-spire-production-transport-probe-adapter` | `task-30` | `spire` |
| `review/30725-spire-production-transport-failure-isolation` | `reviews/task-30/647-30725-spire-production-transport-failure-isolation` | `task-30` | `spire` |
| `review/30726-spire-production-transport-state` | `reviews/task-30/648-30726-spire-production-transport-state` | `task-30` | `spire` |
| `review/30727-spire-production-candidate-receive-adapter` | `reviews/task-30/649-30727-spire-production-candidate-receive-adapter` | `task-30` | `spire` |
| `review/30728-spire-production-candidate-receive-state` | `reviews/task-30/650-30728-spire-production-candidate-receive-state` | `task-30` | `spire` |
| `review/30729-spire-production-candidate-receive-isolation` | `reviews/task-30/651-30729-spire-production-candidate-receive-isolation` | `task-30` | `spire` |
| `review/30730-spire-production-heap-handoff-contract` | `reviews/task-30/652-30730-spire-production-heap-handoff-contract` | `task-30` | `spire` |
| `review/30731-spire-production-compact-merge-handoff` | `reviews/task-30/653-30731-spire-production-compact-merge-handoff` | `task-30` | `spire` |
| `review/30732-spire-production-remote-index-resolution` | `reviews/task-30/654-30732-spire-production-remote-index-resolution` | `task-30` | `spire` |
| `review/30733-spire-production-cancellation-batch-cleanup` | `reviews/task-30/655-30733-spire-production-cancellation-batch-cleanup` | `task-30` | `spire` |
| `review/30734-spire-production-receive-request-state` | `reviews/task-30/656-30734-spire-production-receive-request-state` | `task-30` | `spire` |
| `review/30735-spire-production-receive-state-adapter` | `reviews/task-30/657-30735-spire-production-receive-state-adapter` | `task-30` | `spire` |
| `review/30736-spire-scan-selected-leaf-pid-handoff` | `reviews/task-30/658-30736-spire-scan-selected-leaf-pid-handoff` | `task-30` | `spire` |
| `review/30737-spire-stage-c-production-milestone` | `reviews/task-30/659-30737-spire-stage-c-production-milestone` | `task-30` | `spire` |
| `review/30738-spire-remote-statement-timeout-taxonomy` | `reviews/task-30/660-30738-spire-remote-statement-timeout-taxonomy` | `task-30` | `spire` |
| `review/30739-spire-remote-backend-termination-taxonomy` | `reviews/task-30/661-30739-spire-remote-backend-termination-taxonomy` | `task-30` | `spire` |
| `review/30740-spire-remote-cancel-fault-taxonomy` | `reviews/task-30/662-30740-spire-remote-cancel-fault-taxonomy` | `task-30` | `spire` |
| `review/30741-spire-remote-failure-taxonomy-doc-clarity` | `reviews/task-30/663-30741-spire-remote-failure-taxonomy-doc-clarity` | `task-30` | `spire` |
| `review/30742-spire-local-cancel-remote-cancel-primitive` | `reviews/task-30/664-30742-spire-local-cancel-remote-cancel-primitive` | `task-30` | `spire` |
| `review/30743-spire-production-receive-identity-guard` | `reviews/task-30/665-30743-spire-production-receive-identity-guard` | `task-30` | `spire` |
| `review/30744-spire-production-degraded-skip-state` | `reviews/task-30/666-30744-spire-production-degraded-skip-state` | `task-30` | `spire` |
| `review/30745-spire-production-receive-epoch-empty-coverage` | `reviews/task-30/667-30745-spire-production-receive-epoch-empty-coverage` | `task-30` | `spire` |
| `review/30746-spire-production-session-consistency-policy` | `reviews/task-30/668-30746-spire-production-session-consistency-policy` | `task-30` | `spire` |
| `review/30747-spire-production-state-mode-attribution` | `reviews/task-30/669-30747-spire-production-state-mode-attribution` | `task-30` | `spire` |
| `review/30748-spire-production-consistency-policy-preflight` | `reviews/task-30/670-30748-spire-production-consistency-policy-preflight` | `task-30` | `spire` |
| `review/30749-spire-production-pg-interrupt-cancel-bridge` | `reviews/task-30/671-30749-spire-production-pg-interrupt-cancel-bridge` | `task-30` | `spire` |
| `review/30750-spire-production-local-statement-timeout-bridge` | `reviews/task-30/672-30750-spire-production-local-statement-timeout-bridge` | `task-30` | `spire` |
| `review/30751-spire-production-strict-degraded-fault-matrix` | `reviews/task-30/673-30751-spire-production-strict-degraded-fault-matrix` | `task-30` | `spire` |
| `review/30752-spire-multicluster-transport-overlap` | `reviews/task-30/674-30752-spire-multicluster-transport-overlap` | `task-30` | `spire` |
| `review/30753-spire-production-governance-cancel-release` | `reviews/task-30/675-30753-spire-production-governance-cancel-release` | `task-30` | `spire` |
| `review/30754-spire-production-scan-handoff` | `reviews/task-30/676-30754-spire-production-scan-handoff` | `task-30` | `spire` |
| `review/30755-spire-production-heap-resolution` | `reviews/task-30/677-30755-spire-production-heap-resolution` | `task-30` | `spire` |
| `review/30756-spire-production-scan-result-stream` | `reviews/task-30/678-30756-spire-production-scan-result-stream` | `task-30` | `spire` |
| `review/30757-spire-production-am-delivery-contract` | `reviews/task-30/679-30757-spire-production-am-delivery-contract` | `task-30` | `spire` |
| `review/30758-spire-am-remote-placement-gate` | `reviews/task-30/680-30758-spire-am-remote-placement-gate` | `task-30` | `spire` |
| `review/30760-spire-remote-row-materialization-constant` | `reviews/task-30/681-30760-spire-remote-row-materialization-constant` | `task-30` | `spire` |
| `review/30761-spire-row-materialization-contract` | `reviews/task-30/682-30761-spire-row-materialization-contract` | `task-30` | `spire` |
| `review/30762-spire-production-am-output-cursor` | `reviews/task-30/683-30762-spire-production-am-output-cursor` | `task-30` | `spire` |
| `review/30763-spire-row-materialization-lifecycle-adr` | `reviews/task-30/684-30763-spire-row-materialization-lifecycle-adr` | `task-30` | `spire` |
| `review/30764-spire-standalone-pgrx-loader-stubs` | `reviews/task-30/685-30764-spire-standalone-pgrx-loader-stubs` | `task-30` | `spire` |
| `review/30765-spire-row-materialization-mapping-contract` | `reviews/task-30/686-30765-spire-row-materialization-mapping-contract` | `task-30` | `spire` |
| `review/30766-spire-governance-pg-test-isolation` | `reviews/task-30/687-30766-spire-governance-pg-test-isolation` | `task-30` | `spire` |
| `review/30768-spire-am-output-cursor-rescan` | `reviews/task-30/688-30768-spire-am-output-cursor-rescan` | `task-30` | `spire` |
| `review/30769-spire-row-materialization-cleanup-ownership` | `reviews/task-30/689-30769-spire-row-materialization-cleanup-ownership` | `task-30` | `spire` |
| `review/30770-spire-stage-e-fault-matrix` | `reviews/task-30/690-30770-spire-stage-e-fault-matrix` | `task-30` | `spire` |
| `review/30771-spire-operator-diagnostics-rollup` | `reviews/task-30/691-30771-spire-operator-diagnostics-rollup` | `task-30` | `spire` |
| `review/30772-spire-stage-e-lifecycle-matrix` | `reviews/task-30/692-30772-spire-stage-e-lifecycle-matrix` | `task-30` | `spire` |
| `review/30773-spire-stage-e-fixture-evidence-contract` | `reviews/task-30/693-30773-spire-stage-e-fixture-evidence-contract` | `task-30` | `spire` |
| `review/30774-spire-remote-manifest-freshness-diagnostics` | `reviews/task-30/694-30774-spire-remote-manifest-freshness-diagnostics` | `task-30` | `spire` |
| `review/30775-spire-boundary-replica-identity-snapshot` | `reviews/task-30/695-30775-spire-boundary-replica-identity-snapshot` | `task-30` | `spire` |
| `review/30776-spire-cli-multicluster-transport` | `reviews/task-30/696-30776-spire-cli-multicluster-transport` | `task-30` | `spire` |
| `review/30777-spire-cli-multicluster-transport-evidence` | `reviews/task-30/697-30777-spire-cli-multicluster-transport-evidence` | `task-30` | `spire` |
| `review/30778-spire-stage-e-network-partition` | `reviews/task-30/698-30778-spire-stage-e-network-partition` | `task-30` | `spire` |
| `review/30779-spire-stage-e-version-skew` | `reviews/task-30/699-30779-spire-stage-e-version-skew` | `task-30` | `spire` |
| `review/30780-spire-stage-e-epoch-mismatch` | `reviews/task-30/700-30780-spire-stage-e-epoch-mismatch` | `task-30` | `spire` |
| `review/30781-spire-stage-e-missing-remote-index` | `reviews/task-30/701-30781-spire-stage-e-missing-remote-index` | `task-30` | `spire` |
| `review/30782-spire-stage-e-fingerprint-mismatch` | `reviews/task-30/702-30782-spire-stage-e-fingerprint-mismatch` | `task-30` | `spire` |
| `review/30783-spire-stage-e-remote-statement-timeout` | `reviews/task-30/703-30783-spire-stage-e-remote-statement-timeout` | `task-30` | `spire` |
| `review/30784-spire-stage-e-remote-backend-termination` | `reviews/task-30/704-30784-spire-stage-e-remote-backend-termination` | `task-30` | `spire` |
| `review/30785-spire-stage-e-local-cancel` | `reviews/task-30/705-30785-spire-stage-e-local-cancel` | `task-30` | `spire` |
| `review/30786-spire-stage-e-local-statement-timeout` | `reviews/task-30/706-30786-spire-stage-e-local-statement-timeout` | `task-30` | `spire` |
| `review/30787-spire-stage-e-connection-reset-mid-batch` | `reviews/task-30/707-30787-spire-stage-e-connection-reset-mid-batch` | `task-30` | `spire` |
| `review/30788-spire-stage-e-remote-oom` | `reviews/task-30/708-30788-spire-stage-e-remote-oom` | `task-30` | `spire` |
| `review/30789-spire-stage-e-lifecycle-drop-before-fanout` | `reviews/task-30/709-30789-spire-stage-e-lifecycle-drop-before-fanout` | `task-30` | `spire` |
| `review/30790-spire-stage-e-lifecycle-drop-in-flight` | `reviews/task-30/710-30790-spire-stage-e-lifecycle-drop-in-flight` | `task-30` | `spire` |
| `review/30791-spire-stage-e-lifecycle-reindex-before-fanout` | `reviews/task-30/711-30791-spire-stage-e-lifecycle-reindex-before-fanout` | `task-30` | `spire` |
| `review/30792-spire-stage-e-lifecycle-reindex-in-flight` | `reviews/task-30/712-30792-spire-stage-e-lifecycle-reindex-in-flight` | `task-30` | `spire` |
| `review/30793-spire-stage-e-lifecycle-create-missing-descriptor` | `reviews/task-30/713-30793-spire-stage-e-lifecycle-create-missing-descriptor` | `task-30` | `spire` |
| `review/30794-spire-filenode-fingerprint-cache-followup` | `reviews/task-30/714-30794-spire-filenode-fingerprint-cache-followup` | `task-30` | `spire` |
| `review/30795-spire-stage-e-lifecycle-create-new-descriptor` | `reviews/task-30/715-30795-spire-stage-e-lifecycle-create-new-descriptor` | `task-30` | `spire` |
| `review/30796-spire-am-tuple-path-dedupe-blockers` | `reviews/task-30/716-30796-spire-am-tuple-path-dedupe-blockers` | `task-30` | `spire` |
| `review/30797-spire-row-materialization-provider-seam` | `reviews/task-30/717-30797-spire-row-materialization-provider-seam` | `task-30` | `spire` |
| `review/30798-spire-row-materialization-catalog-provider` | `reviews/task-30/718-30798-spire-row-materialization-catalog-provider` | `task-30` | `spire` |
| `review/30799-spire-am-materialized-remote-row` | `reviews/task-30/719-30799-spire-am-materialized-remote-row` | `task-30` | `spire` |
| `review/30800-spire-stage-d-finish-redirect` | `reviews/task-30/720-30800-spire-stage-d-finish-redirect` | `task-30` | `spire` |
| `review/30801-spire-mirror-sync-adr` | `reviews/task-30/721-30801-spire-mirror-sync-adr` | `task-30` | `spire` |
| `review/30802-spire-mirror-sync-contract` | `reviews/task-30/722-30802-spire-mirror-sync-contract` | `task-30` | `spire` |
| `review/30803-spire-customscan-pivot-adrs` | `reviews/task-30/723-30803-spire-customscan-pivot-adrs` | `task-30` | `spire` |
| `review/30804-spire-customscan-pivot-task-rewrite` | `reviews/task-30/724-30804-spire-customscan-pivot-task-rewrite` | `task-30` | `spire` |
| `review/30805-spire-customscan-provider-scaffold` | `reviews/task-30/725-30805-spire-customscan-provider-scaffold` | `task-30` | `spire` |
| `review/30806-spire-customscan-index-eligibility` | `reviews/task-30/726-30806-spire-customscan-index-eligibility` | `task-30` | `spire` |
| `review/30807-spire-remote-tuple-payload` | `reviews/task-30/727-30807-spire-remote-tuple-payload` | `task-30` | `spire` |
| `review/30808-spire-customscan-eligibility-planner-readiness` | `reviews/task-30/728-30808-spire-customscan-eligibility-planner-readiness` | `task-30` | `spire` |
| `review/30809-spire-customscan-planner-path` | `reviews/task-30/729-30809-spire-customscan-planner-path` | `task-30` | `spire` |
| `review/30810-spire-customscan-executor-stream` | `reviews/task-30/730-30810-spire-customscan-executor-stream` | `task-30` | `spire` |
| `review/30811-spire-customscan-parameter-query` | `reviews/task-30/731-30811-spire-customscan-parameter-query` | `task-30` | `spire` |
| `review/30812-spire-tuple-payload-missing-batch` | `reviews/task-30/732-30812-spire-tuple-payload-missing-batch` | `task-30` | `spire` |
| `review/30813-spire-customscan-planner-feedback-polish` | `reviews/task-30/733-30813-spire-customscan-planner-feedback-polish` | `task-30` | `spire` |
| `review/30814-spire-customscan-tuple-payload-slots` | `reviews/task-30/734-30814-spire-customscan-tuple-payload-slots` | `task-30` | `spire` |
| `review/30815-spire-customscan-loopback-read` | `reviews/task-30/735-30815-spire-customscan-loopback-read` | `task-30` | `spire` |
| `review/30816-spire-customscan-payload-scalar-gate` | `reviews/task-30/736-30816-spire-customscan-payload-scalar-gate` | `task-30` | `spire` |
| `review/30817-spire-placement-directory-catalog` | `reviews/task-30/737-30817-spire-placement-directory-catalog` | `task-30` | `spire` |
| `review/30818-spire-classify-centroid-helper` | `reviews/task-30/738-30818-spire-classify-centroid-helper` | `task-30` | `spire` |
| `review/30819-spire-placement-batch-registration` | `reviews/task-30/739-30819-spire-placement-batch-registration` | `task-30` | `spire` |
| `review/30820-spire-customscan-multicluster-read` | `reviews/task-30/740-30820-spire-customscan-multicluster-read` | `task-30` | `spire` |
| `review/30821-spire-customscan-local-only-am-proof` | `reviews/task-30/741-30821-spire-customscan-local-only-am-proof` | `task-30` | `spire` |
| `review/30822-spire-customscan-expression-payload-fallback` | `reviews/task-30/742-30822-spire-customscan-expression-payload-fallback` | `task-30` | `spire` |
| `review/30823-spire-customscan-ecvector-payload-projection` | `reviews/task-30/743-30823-spire-customscan-ecvector-payload-projection` | `task-30` | `spire` |
| `review/30824-spire-placement-local-node-zero` | `reviews/task-30/744-30824-spire-placement-local-node-zero` | `task-30` | `spire` |
| `review/30825-spire-placement-batch-hardening` | `reviews/task-30/745-30825-spire-placement-batch-hardening` | `task-30` | `spire` |
| `review/30826-spire-classifier-leaf-pid-contract` | `reviews/task-30/746-30826-spire-classifier-leaf-pid-contract` | `task-30` | `spire` |
| `review/30827-spire-customscan-cost-model` | `reviews/task-30/747-30827-spire-customscan-cost-model` | `task-30` | `spire` |
| `review/30828-spire-coordinator-insert-plan` | `reviews/task-30/748-30828-spire-coordinator-insert-plan` | `task-30` | `spire` |
| `review/30829-spire-coordinator-insert-dispatch-plan` | `reviews/task-30/749-30829-spire-coordinator-insert-dispatch-plan` | `task-30` | `spire` |
| `review/30830-spire-coordinator-insert-remote-prepare` | `reviews/task-30/750-30830-spire-coordinator-insert-remote-prepare` | `task-30` | `spire` |
| `review/30831-spire-remote-insert-tuple-payload` | `reviews/task-30/751-30831-spire-remote-insert-tuple-payload` | `task-30` | `spire` |
| `review/30832-spire-coordinator-insert-payload-prepare` | `reviews/task-30/752-30832-spire-coordinator-insert-payload-prepare` | `task-30` | `spire` |
| `review/30833-spire-coordinator-insert-sql-helper` | `reviews/task-30/753-30833-spire-coordinator-insert-sql-helper` | `task-30` | `spire` |
| `review/30834-spire-coordinator-insert-read-after-customscan` | `reviews/task-30/754-30834-spire-coordinator-insert-read-after-customscan` | `task-30` | `spire` |
| `review/30835-spire-coordinator-insert-trigger` | `reviews/task-30/755-30835-spire-coordinator-insert-trigger` | `task-30` | `spire` |
| `review/30836-spire-insert-descriptor-refresh` | `reviews/task-30/756-30836-spire-insert-descriptor-refresh` | `task-30` | `spire` |
| `review/30837-spire-trigger-insert-read-after-customscan` | `reviews/task-30/757-30837-spire-trigger-insert-read-after-customscan` | `task-30` | `spire` |
| `review/30838-spire-coordinator-update-forwarding` | `reviews/task-30/758-30838-spire-coordinator-update-forwarding` | `task-30` | `spire` |
| `review/30839-spire-coordinator-delete-forwarding` | `reviews/task-30/759-30839-spire-coordinator-delete-forwarding` | `task-30` | `spire` |
| `review/30840-spire-coordinator-pk-select-forwarding` | `reviews/task-30/760-30840-spire-coordinator-pk-select-forwarding` | `task-30` | `spire` |
| `review/30841-spire-embedding-update-rejection` | `reviews/task-30/761-30841-spire-embedding-update-rejection` | `task-30` | `spire` |
| `review/30842-spire-local-delete-placement` | `reviews/task-30/762-30842-spire-local-delete-placement` | `task-30` | `spire` |
| `review/30843-spire-dml-frontdoor-plan` | `reviews/task-30/763-30843-spire-dml-frontdoor-plan` | `task-30` | `spire` |
| `review/30844-spire-insert-v1-feedback-followups` | `reviews/task-30/764-30844-spire-insert-v1-feedback-followups` | `task-30` | `spire` |
| `review/30845-spire-update-primitive-edge-coverage` | `reviews/task-30/765-30845-spire-update-primitive-edge-coverage` | `task-30` | `spire` |
| `review/30846-spire-pk-select-duplicate-guard` | `reviews/task-30/766-30846-spire-pk-select-duplicate-guard` | `task-30` | `spire` |
| `review/30847-spire-dml-frontdoor-shape-classifier` | `reviews/task-30/767-30847-spire-dml-frontdoor-shape-classifier` | `task-30` | `spire` |
| `review/30848-spire-dml-query-shape-extraction` | `reviews/task-30/768-30848-spire-dml-query-shape-extraction` | `task-30` | `spire` |
| `review/30849-spire-dml-planner-hook-scaffold` | `reviews/task-30/769-30849-spire-dml-planner-hook-scaffold` | `task-30` | `spire` |
| `review/30850-spire-dml-relation-context` | `reviews/task-30/770-30850-spire-dml-relation-context` | `task-30` | `spire` |
| `review/30851-spire-dml-target-relation-extraction` | `reviews/task-30/771-30851-spire-dml-target-relation-extraction` | `task-30` | `spire` |
| `review/30852-spire-dml-relation-context-hardening` | `reviews/task-30/772-30852-spire-dml-relation-context-hardening` | `task-30` | `spire` |
| `review/30853-spire-dml-query-shape-followups` | `reviews/task-30/773-30853-spire-dml-query-shape-followups` | `task-30` | `spire` |
| `review/30854-spire-dml-frontdoor-classify-diagnostic` | `reviews/task-30/774-30854-spire-dml-frontdoor-classify-diagnostic` | `task-30` | `spire` |
| `review/30855-spire-dml-catalog-relation-context` | `reviews/task-30/775-30855-spire-dml-catalog-relation-context` | `task-30` | `spire` |
| `review/30856-spire-dml-hook-classifier-observation` | `reviews/task-30/776-30856-spire-dml-hook-classifier-observation` | `task-30` | `spire` |
| `review/30857-spire-dml-feedback-hardening` | `reviews/task-30/777-30857-spire-dml-feedback-hardening` | `task-30` | `spire` |
| `review/30858-spire-dml-relation-context-docs` | `reviews/task-30/778-30858-spire-dml-relation-context-docs` | `task-30` | `spire` |
| `review/30859-spire-dml-replacement-decision` | `reviews/task-30/779-30859-spire-dml-replacement-decision` | `task-30` | `spire` |
| `review/30860-spire-dml-replacement-argument-shape` | `reviews/task-30/780-30860-spire-dml-replacement-argument-shape` | `task-30` | `spire` |
| `review/30861-spire-dml-frontdoor-fail-closed-guard` | `reviews/task-30/781-30861-spire-dml-frontdoor-fail-closed-guard` | `task-30` | `spire` |
| `review/30862-spire-dml-replacement-pk-argument-shape` | `reviews/task-30/782-30862-spire-dml-replacement-pk-argument-shape` | `task-30` | `spire` |
| `review/30863-spire-dml-context-failclosed-followup` | `reviews/task-30/783-30863-spire-dml-context-failclosed-followup` | `task-30` | `spire` |
| `review/30864-spire-dml-pk-byte-encoding` | `reviews/task-30/784-30864-spire-dml-pk-byte-encoding` | `task-30` | `spire` |
| `review/30865-spire-dml-pk-argument-builder` | `reviews/task-30/785-30865-spire-dml-pk-argument-builder` | `task-30` | `spire` |
| `review/30866-spire-dml-primitive-plan-builder` | `reviews/task-30/786-30866-spire-dml-primitive-plan-builder` | `task-30` | `spire` |
| `review/30867-spire-dml-primitive-pk-bytes` | `reviews/task-30/787-30867-spire-dml-primitive-pk-bytes` | `task-30` | `spire` |
| `review/30868-spire-dml-runtime-pk-parameter-bytes` | `reviews/task-30/788-30868-spire-dml-runtime-pk-parameter-bytes` | `task-30` | `spire` |
| `review/30869-spire-dml-primitive-plan-diagnostic` | `reviews/task-30/789-30869-spire-dml-primitive-plan-diagnostic` | `task-30` | `spire` |
| `review/30870-spire-dml-primitive-invocation-builder` | `reviews/task-30/790-30870-spire-dml-primitive-invocation-builder` | `task-30` | `spire` |
| `review/30871-spire-dml-pk-byte-boundary-feedback` | `reviews/task-30/791-30871-spire-dml-pk-byte-boundary-feedback` | `task-30` | `spire` |
| `review/30872-spire-dml-customscan-expression-handoff` | `reviews/task-30/792-30872-spire-dml-customscan-expression-handoff` | `task-30` | `spire` |
| `review/30873-spire-dml-pk-select-customscan` | `reviews/task-30/793-30873-spire-dml-pk-select-customscan` | `task-30` | `spire` |
| `review/30874-spire-dml-pk-extraction-centralization` | `reviews/task-30/794-30874-spire-dml-pk-extraction-centralization` | `task-30` | `spire` |
| `review/30875-spire-dml-baserel-extraction-followups` | `reviews/task-30/795-30875-spire-dml-baserel-extraction-followups` | `task-30` | `spire` |
| `review/30876-spire-dml-baserel-update-delete-handoff` | `reviews/task-30/796-30876-spire-dml-baserel-update-delete-handoff` | `task-30` | `spire` |
| `review/30877-spire-dml-customscan-mode-plumbing` | `reviews/task-30/797-30877-spire-dml-customscan-mode-plumbing` | `task-30` | `spire` |
| `review/30878-spire-dml-joined-update-feedback` | `reviews/task-30/798-30878-spire-dml-joined-update-feedback` | `task-30` | `spire` |
| `review/30879-spire-dml-baserel-mode-wrappers` | `reviews/task-30/799-30879-spire-dml-baserel-mode-wrappers` | `task-30` | `spire` |
| `review/30880-spire-dml-customscan-column-metadata` | `reviews/task-30/800-30880-spire-dml-customscan-column-metadata` | `task-30` | `spire` |
| `review/30881-spire-dml-customscan-metadata-validation` | `reviews/task-30/801-30881-spire-dml-customscan-metadata-validation` | `task-30` | `spire` |
| `review/30882-spire-dml-customscan-primitive-invocation` | `reviews/task-30/802-30882-spire-dml-customscan-primitive-invocation` | `task-30` | `spire` |
| `review/30883-spire-dml-customscan-pk-select-payload` | `reviews/task-30/803-30883-spire-dml-customscan-pk-select-payload` | `task-30` | `spire` |
| `review/30884-spire-dml-plan-tree-replacement-scaffold` | `reviews/task-30/804-30884-spire-dml-plan-tree-replacement-scaffold` | `task-30` | `spire` |
| `review/30885-spire-dml-plan-metadata-feedback` | `reviews/task-30/805-30885-spire-dml-plan-metadata-feedback` | `task-30` | `spire` |
| `review/30886-spire-dml-customscan-update-executor` | `reviews/task-30/806-30886-spire-dml-customscan-update-executor` | `task-30` | `spire` |
| `review/30887-spire-dml-customscan-delete-executor` | `reviews/task-30/807-30887-spire-dml-customscan-delete-executor` | `task-30` | `spire` |
| `review/30888-spire-dml-plan-tree-adr-limits` | `reviews/task-30/808-30888-spire-dml-plan-tree-adr-limits` | `task-30` | `spire` |
| `review/30889-spire-dml-customscan-remote-fixtures` | `reviews/task-30/809-30889-spire-dml-customscan-remote-fixtures` | `task-30` | `spire` |
| `review/30890-spire-dml-task-reconciliation` | `reviews/task-30/810-30890-spire-dml-task-reconciliation` | `task-30` | `spire` |
| `review/30891-spire-insert-task-reconciliation` | `reviews/task-30/811-30891-spire-insert-task-reconciliation` | `task-30` | `spire` |
| `review/30892-spire-materialization-catalog-cleanup` | `reviews/task-30/812-30892-spire-materialization-catalog-cleanup` | `task-30` | `spire` |
| `review/30893-spire-insert-tracker-followup` | `reviews/task-30/813-30893-spire-insert-tracker-followup` | `task-30` | `spire` |
| `review/30894-spire-materialization-am-cleanup` | `reviews/task-30/814-30894-spire-materialization-am-cleanup` | `task-30` | `spire` |
| `review/30895-spire-stage-e-customscan-matrix` | `reviews/task-30/815-30895-spire-stage-e-customscan-matrix` | `task-30` | `spire` |
| `review/30896-spire-customscan-architecture-review` | `reviews/task-30/816-30896-spire-customscan-architecture-review` | `task-30` | `spire` |
| `review/30909-spire-hardening-task-split` | `reviews/task-30/817-30909-spire-hardening-task-split` | `task-30` | `spire` |
| `review/30910-spire-phase11-closeout` | `reviews/task-30/818-30910-spire-phase11-closeout` | `task-30` | `spire` |
| `review/30911-spire-phase12-operator-compat-cleanup` | `reviews/task-30/819-30911-spire-phase12-operator-compat-cleanup` | `task-30` | `spire` |
| `review/30912-spire-placement-planner-gate-index` | `reviews/task-30/820-30912-spire-placement-planner-gate-index` | `task-30` | `spire` |
| `review/30913-spire-typed-tuple-transport-design` | `reviews/task-30/821-30913-spire-typed-tuple-transport-design` | `task-30` | `spire` |
| `review/30914-spire-prepared-xact-gid-runbook` | `reviews/task-30/822-30914-spire-prepared-xact-gid-runbook` | `task-30` | `spire` |
| `review/30915-spire-typed-tuple-endpoint-scalar` | `reviews/task-30/823-30915-spire-typed-tuple-endpoint-scalar` | `task-30` | `spire` |
| `review/30916-spire-typed-tuple-null-array` | `reviews/task-30/824-30916-spire-typed-tuple-null-array` | `task-30` | `spire` |
| `review/30917-spire-typed-tuple-domain-composite` | `reviews/task-30/825-30917-spire-typed-tuple-domain-composite` | `task-30` | `spire` |
| `review/30918-spire-typed-tuple-feedback-response` | `reviews/task-30/826-30918-spire-typed-tuple-feedback-response` | `task-30` | `spire` |
| `review/30919-spire-typed-tuple-transport-capability` | `reviews/task-30/827-30919-spire-typed-tuple-transport-capability` | `task-30` | `spire` |
| `review/30920-spire-delete-not-found-idempotence` | `reviews/task-30/828-30920-spire-delete-not-found-idempotence` | `task-30` | `spire` |
| `review/30921-spire-prepared-transaction-capacity-hint` | `reviews/task-30/829-30921-spire-prepared-transaction-capacity-hint` | `task-30` | `spire` |
| `review/30922-spire-typed-transport-feedback-notes` | `reviews/task-30/830-30922-spire-typed-transport-feedback-notes` | `task-30` | `spire` |
| `review/30923-spire-prepared-capacity-registration-warning` | `reviews/task-30/831-30923-spire-prepared-capacity-registration-warning` | `task-30` | `spire` |
| `review/30925-spire-review-followups` | `reviews/task-30/832-30925-spire-review-followups` | `task-30` | `spire` |
| `review/30926-spire-bulk-load-registration-docs` | `reviews/task-30/833-30926-spire-bulk-load-registration-docs` | `task-30` | `spire` |
| `review/30927-spire-descriptor-refresh-retry-sqlstate` | `reviews/task-30/834-30927-spire-descriptor-refresh-retry-sqlstate` | `task-30` | `spire` |
| `review/30928-spire-multirow-insert-trigger-fixture` | `reviews/task-30/835-30928-spire-multirow-insert-trigger-fixture` | `task-30` | `spire` |
| `review/30929-spire-dml-pk-predicate-edge-fixture` | `reviews/task-30/836-30929-spire-dml-pk-predicate-edge-fixture` | `task-30` | `spire` |
| `review/30930-spire-multirow-gid-feedback` | `reviews/task-30/837-30930-spire-multirow-gid-feedback` | `task-30` | `spire` |
| `review/30931-spire-ddl-ordering-contract` | `reviews/task-30/838-30931-spire-ddl-ordering-contract` | `task-30` | `spire` |
| `review/30932-spire-trigger-payload-type-fixture` | `reviews/task-30/839-30932-spire-trigger-payload-type-fixture` | `task-30` | `spire` |
| `review/30933-spire-schema-drift-fingerprint` | `reviews/task-30/840-30933-spire-schema-drift-fingerprint` | `task-30` | `spire` |
| `review/30934-spire-schema-drift-docs-followup` | `reviews/task-30/841-30934-spire-schema-drift-docs-followup` | `task-30` | `spire` |
| `review/30935-spire-schema-drift-scope-feedback` | `reviews/task-30/842-30935-spire-schema-drift-scope-feedback` | `task-30` | `spire` |
| `review/30936-spire-update-delete-schema-drift` | `reviews/task-30/843-30936-spire-update-delete-schema-drift` | `task-30` | `spire` |
| `review/30937-spire-remote-pk-read-isolation` | `reviews/task-30/844-30937-spire-remote-pk-read-isolation` | `task-30` | `spire` |
| `review/30938-spire-insert-descriptor-race` | `reviews/task-30/845-30938-spire-insert-descriptor-race` | `task-30` | `spire` |
| `review/30939-spire-read-isolation-runbook` | `reviews/task-30/846-30939-spire-read-isolation-runbook` | `task-30` | `spire` |
| `review/30940-spire-custom-private-metadata` | `reviews/task-30/847-30940-spire-custom-private-metadata` | `task-30` | `spire` |
| `review/30941-spire-dml-relation-context-cache` | `reviews/task-30/848-30941-spire-dml-relation-context-cache` | `task-30` | `spire` |
| `review/30942-spire-dml-pk-byte-buffers` | `reviews/task-30/849-30942-spire-dml-pk-byte-buffers` | `task-30` | `spire` |
| `review/30943-spire-prepared-xact-helper-decision` | `reviews/task-30/850-30943-spire-prepared-xact-helper-decision` | `task-30` | `spire` |
| `review/30944-spire-review-followups-2` | `reviews/task-30/851-30944-spire-review-followups-2` | `task-30` | `spire` |
| `review/30945-spire-local-store-diagnostic-unit` | `reviews/task-30/852-30945-spire-local-store-diagnostic-unit` | `task-30` | `spire` |
| `review/30946-spire-local-store-execution-diagnostics` | `reviews/task-30/853-30946-spire-local-store-execution-diagnostics` | `task-30` | `spire` |
| `review/30947-spire-delta-reuse-coverage` | `reviews/task-30/854-30947-spire-delta-reuse-coverage` | `task-30` | `spire` |
| `review/30948-spire-local-store-read-overlap-harness` | `reviews/task-30/855-30948-spire-local-store-read-overlap-harness` | `task-30` | `spire` |
| `review/30949-spire-local-readiness-evidence-boundaries` | `reviews/task-30/856-30949-spire-local-readiness-evidence-boundaries` | `task-30` | `spire` |
| `review/30950-spire-local-store-review-followups` | `reviews/task-30/857-30950-spire-local-store-review-followups` | `task-30` | `spire` |
| `review/30951-spire-libpq-ops-runbook` | `reviews/task-30/858-30951-spire-libpq-ops-runbook` | `task-30` | `spire` |
| `review/30952-spire-local-capacity-targets` | `reviews/task-30/859-30952-spire-local-capacity-targets` | `task-30` | `spire` |
| `review/30953-spire-typed-fixture-tracker-closure` | `reviews/task-30/860-30953-spire-typed-fixture-tracker-closure` | `task-30` | `spire` |
| `review/30954-spire-degraded-skip-report` | `reviews/task-30/861-30954-spire-degraded-skip-report` | `task-30` | `spire` |
| `review/30955-spire-strict-epoch-tracker-closure` | `reviews/task-30/862-30955-spire-strict-epoch-tracker-closure` | `task-30` | `spire` |
| `review/30956-spire-selected-pid-placement-map` | `reviews/task-30/863-30956-spire-selected-pid-placement-map` | `task-30` | `spire` |
| `review/30957-spire-boundary-replica-remote-identity` | `reviews/task-30/864-30957-spire-boundary-replica-remote-identity` | `task-30` | `spire` |
| `review/30958-spire-boundary-replica-manifest-freshness` | `reviews/task-30/865-30958-spire-boundary-replica-manifest-freshness` | `task-30` | `spire` |
| `review/30959-spire-boundary-replica-placement-diagnostics` | `reviews/task-30/866-30959-spire-boundary-replica-placement-diagnostics` | `task-30` | `spire` |
| `review/30960-spire-placement-diagnostic-doc-feedback` | `reviews/task-30/867-30960-spire-placement-diagnostic-doc-feedback` | `task-30` | `spire` |
| `review/30961-spire-stage-e-periodic-rerun` | `reviews/task-30/868-30961-spire-stage-e-periodic-rerun` | `task-30` | `spire` |
| `review/30962-spire-customscan-read-cli` | `reviews/task-30/869-30962-spire-customscan-read-cli` | `task-30` | `spire` |
| `review/30963-spire-customscan-typed-receive` | `reviews/task-30/870-30963-spire-customscan-typed-receive` | `task-30` | `spire` |
| `review/30965-spire-pipeline-query-metrics` | `reviews/task-30/871-30965-spire-pipeline-query-metrics` | `task-30` | `spire` |
| `review/30966-spire-pipeline-local-store-counters` | `reviews/task-30/872-30966-spire-pipeline-local-store-counters` | `task-30` | `spire` |
| `review/30967-spire-pipeline-remote-readiness-counters` | `reviews/task-30/873-30967-spire-pipeline-remote-readiness-counters` | `task-30` | `spire` |
| `review/30968-spire-multicluster-cli-smoke-insert` | `reviews/task-30/874-30968-spire-multicluster-cli-smoke-insert` | `task-30` | `spire` |
| `review/30969-spire-placement-write-contention` | `reviews/task-30/875-30969-spire-placement-write-contention` | `task-30` | `spire` |
| `review/30970-spire-insert-prepare-local-cancel` | `reviews/task-30/876-30970-spire-insert-prepare-local-cancel` | `task-30` | `spire` |
| `review/30971-spire-harness-surface-tracker` | `reviews/task-30/877-30971-spire-harness-surface-tracker` | `task-30` | `spire` |
| `review/30972-spire-insert-prepare-async-fanout` | `reviews/task-30/878-30972-spire-insert-prepare-async-fanout` | `task-30` | `spire` |
| `review/30973-spire-insert-trigger-batch-prepare` | `reviews/task-30/879-30973-spire-insert-trigger-batch-prepare` | `task-30` | `spire` |
| `review/30974-spire-pipeline-payload-projection-metrics` | `reviews/task-30/880-30974-spire-pipeline-payload-projection-metrics` | `task-30` | `spire` |
| `review/30976-spire-cost-calibration` | `reviews/task-30/881-30976-spire-cost-calibration` | `task-30` | `spire` |
| `review/30977-spire-json-production-retirement` | `reviews/task-30/882-30977-spire-json-production-retirement` | `task-30` | `spire` |
| `review/30978-spire-local-readiness-bundle` | `reviews/task-30/883-30978-spire-local-readiness-bundle` | `task-30` | `spire` |
| `review/30979-spire-trigger-live-fixture` | `reviews/task-30/884-30979-spire-trigger-live-fixture` | `task-30` | `spire` |
| `review/30980-spire-dml-frontdoor-read-pass-through` | `reviews/task-30/885-30980-spire-dml-frontdoor-read-pass-through` | `task-30` | `spire` |
| `review/30981-spire-phase12-local-readiness-closeout` | `reviews/task-30/886-30981-spire-phase12-local-readiness-closeout` | `task-30` | `spire` |
| `review/30982-spire-phase12-final-review` | `reviews/task-30/887-30982-spire-phase12-final-review` | `task-30` | `spire` |
| `review/30983-spire-stage-e-evidence-boundary` | `reviews/task-30/888-30983-spire-stage-e-evidence-boundary` | `task-30` | `spire` |
| `review/30984-spire-retired-tuple-transport-status` | `reviews/task-30/889-30984-spire-retired-tuple-transport-status` | `task-30` | `spire` |
| `review/30985-spire-remote-payload-caps` | `reviews/task-30/890-30985-spire-remote-payload-caps` | `task-30` | `spire` |
| `review/30986-spire-cost-gucs` | `reviews/task-30/891-30986-spire-cost-gucs` | `task-30` | `spire` |
| `review/30987-spire-stage-e-ci-subset` | `reviews/task-30/892-30987-spire-stage-e-ci-subset` | `task-30` | `spire` |
| `review/30988-spire-prepared-xact-reaper` | `reviews/task-30/893-30988-spire-prepared-xact-reaper` | `task-30` | `spire` |
| `review/30989-spire-remote-schema-fingerprint` | `reviews/task-30/894-30989-spire-remote-schema-fingerprint` | `task-30` | `spire` |
| `review/30990-spire-phase12a-final-review` | `reviews/task-30/895-30990-spire-phase12a-final-review` | `task-30` | `spire` |
| `review/30991-spire-remote-candidates-split` | `reviews/task-30/896-30991-spire-remote-candidates-split` | `task-30` | `spire` |
| `review/30992-spire-custom-scan-split` | `reviews/task-30/897-30992-spire-custom-scan-split` | `task-30` | `spire` |
| `review/30993-spire-remote-candidates-test-layout` | `reviews/task-30/898-30993-spire-remote-candidates-test-layout` | `task-30` | `spire` |
| `review/30994-spire-test-layout-standardization` | `reviews/task-30/899-30994-spire-test-layout-standardization` | `task-30` | `spire` |
| `review/30995-spire-lib-fixture-body-split` | `reviews/task-30/900-30995-spire-lib-fixture-body-split` | `task-30` | `spire` |
| `review/30996-spire-customscan-explain-contract` | `reviews/task-30/901-30996-spire-customscan-explain-contract` | `task-30` | `spire` |
| `review/30997-spire-coordinator-directory-rename` | `reviews/task-30/902-30997-spire-coordinator-directory-rename` | `task-30` | `spire` |
| `review/30998-spire-customscan-fixture-split` | `reviews/task-30/903-30998-spire-customscan-fixture-split` | `task-30` | `spire` |
| `review/30999-spire-remote-search-fixture-split` | `reviews/task-30/904-30999-spire-remote-search-fixture-split` | `task-30` | `spire` |
| `review/31000-spire-insert-fixture-split` | `reviews/task-30/905-31000-spire-insert-fixture-split` | `task-30` | `spire` |
| `review/31001-spire-dml-frontdoor-fixture-split` | `reviews/task-30/906-31001-spire-dml-frontdoor-fixture-split` | `task-30` | `spire` |
| `review/31002-spire-placement-fixture-split` | `reviews/task-30/907-31002-spire-placement-fixture-split` | `task-30` | `spire` |
| `review/31003-spire-scan-fixture-split` | `reviews/task-30/908-31003-spire-scan-fixture-split` | `task-30` | `spire` |
| `review/31004-spire-dml-frontdoor-primitive-fixture-split` | `reviews/task-30/909-31004-spire-dml-frontdoor-primitive-fixture-split` | `task-30` | `spire` |
| `review/31005-spire-dml-frontdoor-select-fixture-split` | `reviews/task-30/910-31005-spire-dml-frontdoor-select-fixture-split` | `task-30` | `spire` |
| `review/31006-spire-dml-frontdoor-coordinator-fixture-split` | `reviews/task-30/911-31006-spire-dml-frontdoor-coordinator-fixture-split` | `task-30` | `spire` |
| `review/31007-spire-diagnostics-fixture-split` | `reviews/task-30/912-31007-spire-diagnostics-fixture-split` | `task-30` | `spire` |
| `review/31008-spire-build-fixture-split` | `reviews/task-30/913-31008-spire-build-fixture-split` | `task-30` | `spire` |
| `review/31009-spire-vacuum-fixture-split` | `reviews/task-30/914-31009-spire-vacuum-fixture-split` | `task-30` | `spire` |
| `review/31010-spire-diagnostics-sanity-fixture-split` | `reviews/task-30/915-31010-spire-diagnostics-sanity-fixture-split` | `task-30` | `spire` |
| `review/31011-spire-diagnostics-active-fixture-split` | `reviews/task-30/916-31011-spire-diagnostics-active-fixture-split` | `task-30` | `spire` |
| `review/31012-spire-diagnostics-final-fixture-split` | `reviews/task-30/917-31012-spire-diagnostics-final-fixture-split` | `task-30` | `spire` |
| `review/31013-spire-diagnostics-leaf-fixture-split` | `reviews/task-30/918-31013-spire-diagnostics-leaf-fixture-split` | `task-30` | `spire` |
| `review/31014-spire-build-populated-fixture-split` | `reviews/task-30/919-31014-spire-build-populated-fixture-split` | `task-30` | `spire` |
| `review/31015-spire-build-multistore-fixture-split` | `reviews/task-30/920-31015-spire-build-multistore-fixture-split` | `task-30` | `spire` |
| `review/31016-spire-build-final-fixture-split` | `reviews/task-30/921-31016-spire-build-final-fixture-split` | `task-30` | `spire` |
| `review/31017-spire-phase12b-audit-decisions` | `reviews/task-30/922-31017-spire-phase12b-audit-decisions` | `task-30` | `spire` |
| `review/31018-spire-insert-schema-drift-fixture-split` | `reviews/task-30/923-31018-spire-insert-schema-drift-fixture-split` | `task-30` | `spire` |
| `review/31019-spire-post-build-insert-fixture-split` | `reviews/task-30/924-31019-spire-post-build-insert-fixture-split` | `task-30` | `spire` |
| `review/31020-spire-insert-delta-fixture-split` | `reviews/task-30/925-31020-spire-insert-delta-fixture-split` | `task-30` | `spire` |
| `review/31021-spire-source-identity-insert-fixture-split` | `reviews/task-30/926-31021-spire-source-identity-insert-fixture-split` | `task-30` | `spire` |
| `review/31022-spire-vacuum-final-fixture-split` | `reviews/task-30/927-31022-spire-vacuum-final-fixture-split` | `task-30` | `spire` |
| `review/31023-spire-placement-final-fixture-split` | `reviews/task-30/928-31023-spire-placement-final-fixture-split` | `task-30` | `spire` |
| `review/31024-spire-scan-final-fixture-split` | `reviews/task-30/929-31024-spire-scan-final-fixture-split` | `task-30` | `spire` |
| `review/31025-spire-cost-planner-fixture-split` | `reviews/task-30/930-31025-spire-cost-planner-fixture-split` | `task-30` | `spire` |
| `review/31026-spire-remote-search-final-fixture-split` | `reviews/task-30/931-31026-spire-remote-search-final-fixture-split` | `task-30` | `spire` |
| `review/31027-spire-diagnostics-storage-roundtrip-fixture-split` | `reviews/task-30/932-31027-spire-diagnostics-storage-roundtrip-fixture-split` | `task-30` | `spire` |
| `review/31028-spire-customscan-empty-remote-result` | `reviews/task-30/933-31028-spire-customscan-empty-remote-result` | `task-30` | `spire` |
| `review/31029-spire-fixture-name-spot-check` | `reviews/task-30/934-31029-spire-fixture-name-spot-check` | `task-30` | `spire` |
| `review/31030-spire-non-test-unwrap-expect-audit` | `reviews/task-30/935-31030-spire-non-test-unwrap-expect-audit` | `task-30` | `spire` |
| `review/31031-spire-unsafe-boundary-audit` | `reviews/task-30/936-31031-spire-unsafe-boundary-audit` | `task-30` | `spire` |
| `review/31032-spire-remote-search-test-split` | `reviews/task-30/937-31032-spire-remote-search-test-split` | `task-30` | `spire` |
| `review/31033-spire-customscan-lifecycle-helpers` | `reviews/task-30/938-31033-spire-customscan-lifecycle-helpers` | `task-30` | `spire` |
| `review/31034-spire-customscan-begin-state` | `reviews/task-30/939-31034-spire-customscan-begin-state` | `task-30` | `spire` |
| `review/31035-spire-customscan-read-cancel` | `reviews/task-30/940-31035-spire-customscan-read-cancel` | `task-30` | `spire` |
| `review/31036-spire-phase12b-final-verification` | `reviews/task-30/941-31036-spire-phase12b-final-verification` | `task-30` | `spire` |
| `review/31050-spire-phase12b-midphase-audit` | `reviews/task-30/942-31050-spire-phase12b-midphase-audit` | `task-30` | `spire` |
| `review/31051-spire-mod-test-split` | `reviews/task-30/943-31051-spire-mod-test-split` | `task-30` | `spire` |
| `review/31060-spire-phase12b-final-review` | `reviews/task-30/944-31060-spire-phase12b-final-review` | `task-30` | `spire` |
| `review/31070-spire-phase12c-coverage-audit` | `reviews/task-30/945-31070-spire-phase12c-coverage-audit` | `task-30` | `spire` |
| `review/31080-spire-phase12c-batch1-feedback` | `reviews/task-30/946-31080-spire-phase12c-batch1-feedback` | `task-30` | `spire` |
| `review/31090-spire-phase12c-batch2-feedback` | `reviews/task-30/947-31090-spire-phase12c-batch2-feedback` | `task-30` | `spire` |
| `review/31100-spire-phase12c-batch3-feedback` | `reviews/task-30/948-31100-spire-phase12c-batch3-feedback` | `task-30` | `spire` |
| `review/31110-spire-phase12c-batch4-feedback` | `reviews/task-30/949-31110-spire-phase12c-batch4-feedback` | `task-30` | `spire` |
| `review/31120-spire-phase12c-batch5-feedback` | `reviews/task-30/950-31120-spire-phase12c-batch5-feedback` | `task-30` | `spire` |
| `review/31130-spire-phase12c-final-review` | `reviews/task-30/951-31130-spire-phase12c-final-review` | `task-30` | `spire` |
| `review/31140-spire-spec-backfill` | `reviews/task-30/952-31140-spire-spec-backfill` | `task-30` | `spire` |
| `review/31145-c1-spire-custom-scan-unsafe-boundaries` | `reviews/task-30/953-31145-c1-spire-custom-scan-unsafe-boundaries` | `task-30` | `spire` |
| `review/31146-c1-spire-relation-object-write-boundary` | `reviews/task-30/954-31146-c1-spire-relation-object-write-boundary` | `task-30` | `spire` |
| `review/31147-c1-spire-scan-heap-relation-guard` | `reviews/task-30/955-31147-c1-spire-scan-heap-relation-guard` | `task-30` | `spire` |
| `review/30162-task31-m5-environment-setup` | `reviews/task-31/001-30162-task31-m5-environment-setup` | `task-31` | `task-token` |
| `review/30163-task31-m5-ivf-smoke` | `reviews/task-31/002-30163-task31-m5-ivf-smoke` | `task-31` | `task-token` |
| `review/30164-task31-ecaz-agent-session-docs` | `reviews/task-31/003-30164-task31-ecaz-agent-session-docs` | `task-31` | `task-token` |
| `review/30165-task31-m5-ivf-optimization-plan` | `reviews/task-31/004-30165-task31-m5-ivf-optimization-plan` | `task-31` | `task-token` |
| `review/30169-task31-m5-pqg8-10k-load-baseline` | `reviews/task-31/005-30169-task31-m5-pqg8-10k-load-baseline` | `task-31` | `task-token` |
| `review/30170-task31-m5-pqg8-25k-load-baseline` | `reviews/task-31/006-30170-task31-m5-pqg8-25k-load-baseline` | `task-31` | `task-token` |
| `review/30171-task31-m5-pqg8-50k-load-baseline` | `reviews/task-31/007-30171-task31-m5-pqg8-50k-load-baseline` | `task-31` | `task-token` |
| `review/30172-task31-m5-pqg8-100k-load-baseline` | `reviews/task-31/008-30172-task31-m5-pqg8-100k-load-baseline` | `task-31` | `task-token` |
| `review/30173-task31-m5-pqg8-100k-n128-w500-baseline` | `reviews/task-31/009-30173-task31-m5-pqg8-100k-n128-w500-baseline` | `task-31` | `task-token` |
| `review/30174-task31-m5-100k-n128-nprobe-sweep` | `reviews/task-31/010-30174-task31-m5-100k-n128-nprobe-sweep` | `task-31` | `task-token` |
| `review/30175-task31-m5-100k-n128-rerank-sweep` | `reviews/task-31/011-30175-task31-m5-100k-n128-rerank-sweep` | `task-31` | `task-token` |
| `review/30176-task31-m5-100k-candidate-repeatability` | `reviews/task-31/012-30176-task31-m5-100k-candidate-repeatability` | `task-31` | `task-token` |
| `review/30178-task31-suite-runner-dry-run` | `reviews/task-31/013-30178-task31-suite-runner-dry-run` | `task-31` | `task-token` |
| `review/30179-task31-suite-runner-execution` | `reviews/task-31/014-30179-task31-suite-runner-execution` | `task-31` | `task-token` |
| `review/30180-task31-suite-runner-auto-mode` | `reviews/task-31/015-30180-task31-suite-runner-auto-mode` | `task-31` | `task-token` |
| `review/30181-task31-suite-thresholds` | `reviews/task-31/016-30181-task31-suite-thresholds` | `task-31` | `task-token` |
| `review/30182-task31-suite-filtered-thresholds-resume` | `reviews/task-31/017-30182-task31-suite-filtered-thresholds-resume` | `task-31` | `task-token` |
| `review/30183-task31-suite-candidate-run` | `reviews/task-31/018-30183-task31-suite-candidate-run` | `task-31` | `task-token` |
| `review/30184-task31-suite-balanced-run` | `reviews/task-31/019-30184-task31-suite-balanced-run` | `task-31` | `task-token` |
| `review/30185-task31-suite-candidate-decision` | `reviews/task-31/020-30185-task31-suite-candidate-decision` | `task-31` | `task-token` |
| `review/30186-task31-ivf-score-ranked-probe-order` | `reviews/task-31/021-30186-task31-ivf-score-ranked-probe-order` | `task-31` | `task-token` |
| `review/30187-task31-suite-quality-score-ranked` | `reviews/task-31/022-30187-task31-suite-quality-score-ranked` | `task-31` | `task-token` |
| `review/30188-task31-suite-balanced-score-ranked` | `reviews/task-31/023-30188-task31-suite-balanced-score-ranked` | `task-31` | `task-token` |
| `review/30189-task31-score-ranked-probe-order-decision` | `reviews/task-31/024-30189-task31-score-ranked-probe-order-decision` | `task-31` | `task-token` |
| `review/30190-task31-ivf-heap-ordered-rerank-fetch` | `reviews/task-31/025-30190-task31-ivf-heap-ordered-rerank-fetch` | `task-31` | `task-token` |
| `review/30191-task31-suite-quality-heap-ordered-rerank` | `reviews/task-31/026-30191-task31-suite-quality-heap-ordered-rerank` | `task-31` | `task-token` |
| `review/30192-task31-suite-balanced-heap-ordered-rerank` | `reviews/task-31/027-30192-task31-suite-balanced-heap-ordered-rerank` | `task-31` | `task-token` |
| `review/30193-task31-heap-ordered-rerank-decision` | `reviews/task-31/028-30193-task31-heap-ordered-rerank-decision` | `task-31` | `task-token` |
| `review/30194-task31-ivf-rerank-state-cache` | `reviews/task-31/029-30194-task31-ivf-rerank-state-cache` | `task-31` | `task-token` |
| `review/30195-task31-suite-quality-rerank-state-cache` | `reviews/task-31/030-30195-task31-suite-quality-rerank-state-cache` | `task-31` | `task-token` |
| `review/30196-task31-suite-balanced-rerank-state-cache` | `reviews/task-31/031-30196-task31-suite-balanced-rerank-state-cache` | `task-31` | `task-token` |
| `review/30197-task31-rerank-state-cache-decision` | `reviews/task-31/032-30197-task31-rerank-state-cache-decision` | `task-31` | `task-token` |
| `review/30198-task31-ivf-rerank-loop-cleanup` | `reviews/task-31/033-30198-task31-ivf-rerank-loop-cleanup` | `task-31` | `task-token` |
| `review/30199-task31-suite-quality-rerank-loop-cleanup` | `reviews/task-31/034-30199-task31-suite-quality-rerank-loop-cleanup` | `task-31` | `task-token` |
| `review/30200-task31-current-candidate-decision` | `reviews/task-31/035-30200-task31-current-candidate-decision` | `task-31` | `task-token` |
| `review/30201-task31-m5-quality-neon-rerank` | `reviews/task-31/036-30201-task31-m5-quality-neon-rerank` | `task-31` | `task-token` |
| `review/30202-task31-m5-quality-ecvector-rerank-decode` | `reviews/task-31/037-30202-task31-m5-quality-ecvector-rerank-decode` | `task-31` | `task-token` |
| `review/30203-task31-current-m5-candidate-decision` | `reviews/task-31/038-30203-task31-current-m5-candidate-decision` | `task-31` | `task-token` |
| `review/30210-task32-m5-diskann-final-cross-engine-refresh` | `reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh` | `task-32` | `task-token` |
| `review/30034-task34-comprehensive-hardening` | `reviews/task-34/001-30034-task34-comprehensive-hardening` | `task-34` | `task-token` |
| `review/31145-task36-38-hardening-validation` | `reviews/task-36/001-31145-task36-38-hardening-validation` | `task-36` | `task-token` |
| `review/9042-task42-on-disk-layout-contracts` | `reviews/task-42/001-9042-task42-on-disk-layout-contracts` | `task-42` | `task-token` |
| `review/9043-task42-ivf-diskann-layout-contracts` | `reviews/task-42/002-9043-task42-ivf-diskann-layout-contracts` | `task-42` | `task-token` |
| `review/9044-task42-spire-layout-contracts` | `reviews/task-42/003-9044-task42-spire-layout-contracts` | `task-42` | `task-token` |
| `review/9045-task42-metadata-fixtures` | `reviews/task-42/004-9045-task42-metadata-fixtures` | `task-42` | `task-token` |
| `review/9046-task42-tuple-fixtures` | `reviews/task-42/005-9046-task42-tuple-fixtures` | `task-42` | `task-token` |
| `review/9047-task42-ivf-fixtures` | `reviews/task-42/006-9047-task42-ivf-fixtures` | `task-42` | `task-token` |
| `review/9048-task42-spire-metadata-fixtures` | `reviews/task-42/007-9048-task42-spire-metadata-fixtures` | `task-42` | `task-token` |
| `review/9049-task42-spire-partition-fixtures` | `reviews/task-42/008-9049-task42-spire-partition-fixtures` | `task-42` | `task-token` |
| `review/9050-task42-spire-v2-chain-fixtures` | `reviews/task-42/009-9050-task42-spire-v2-chain-fixtures` | `task-42` | `task-token` |
| `review/9051-task42-hnsw-hot-fixtures` | `reviews/task-42/010-9051-task42-hnsw-hot-fixtures` | `task-42` | `task-token` |
| `review/9052-task42-diskann-overflow-fixture` | `reviews/task-42/011-9052-task42-diskann-overflow-fixture` | `task-42` | `task-token` |
| `review/9053-task42-upgrade-matrix-smoke` | `reviews/task-42/012-9053-task42-upgrade-matrix-smoke` | `task-42` | `task-token` |
| `review/9054-task42-ci-fixture-lanes` | `reviews/task-42/013-9054-task42-ci-fixture-lanes` | `task-42` | `task-token` |
| `review/9055-task42-qemu-endian-lane` | `reviews/task-42/014-9055-task42-qemu-endian-lane` | `task-42` | `task-token` |
| `review/9056-task42-qemu-cross-build-fix` | `reviews/task-42/015-9056-task42-qemu-cross-build-fix` | `task-42` | `task-token` |
| `review/9057-task42-pg-upgrade-smoke` | `reviews/task-42/016-9057-task42-pg-upgrade-smoke` | `task-42` | `task-token` |
| `review/9058-task42-wal-format-policy` | `reviews/task-42/017-9058-task42-wal-format-policy` | `task-42` | `task-token` |
| `review/9059-task42-completion-audit` | `reviews/task-42/018-9059-task42-completion-audit` | `task-42` | `task-token` |
| `review/31150-c1-task49-hardening-ci-governance` | `reviews/task-49/001-31150-c1-task49-hardening-ci-governance` | `task-49` | `task-token` |
| `review/02-tail-page-reuse-and-rollover` | `reviews/task-archive-early-hnsw/000-02-tail-page-reuse-and-rollover` | `task-archive-early-hnsw` | `archive` |
| `review/03-duplicate-coalescing-and-capacity` | `reviews/task-archive-early-hnsw/001-03-duplicate-coalescing-and-capacity` | `task-archive-early-hnsw` | `archive` |
| `review/12-tail-page-rollover-followup` | `reviews/task-archive-early-hnsw/002-12-tail-page-rollover-followup` | `task-archive-early-hnsw` | `archive` |
| `review/19-build-detoast-copy-detection` | `reviews/task-archive-early-hnsw/003-19-build-detoast-copy-detection` | `task-archive-early-hnsw` | `archive` |
| `review/21-page-offset-checked-conversion` | `reviews/task-archive-early-hnsw/004-21-page-offset-checked-conversion` | `task-archive-early-hnsw` | `archive` |
| `review/22-pin-hnsw-rs-version` | `reviews/task-archive-early-hnsw/005-22-pin-hnsw-rs-version` | `task-archive-early-hnsw` | `archive` |
| `review/24-relation-options-cache` | `reviews/task-archive-early-hnsw/006-24-relation-options-cache` | `task-archive-early-hnsw` | `archive` |
| `review/25-zero-allocation-code-scoring` | `reviews/task-archive-early-hnsw/007-25-zero-allocation-code-scoring` | `task-archive-early-hnsw` | `archive` |
| `review/30-plan-and-spec-backfill` | `reviews/task-archive-early-hnsw/008-30-plan-and-spec-backfill` | `task-archive-early-hnsw` | `archive` |
| `review/32-am-options-module-split` | `reviews/task-archive-early-hnsw/009-32-am-options-module-split` | `task-archive-early-hnsw` | `archive` |
| `review/33-am-routine-module-split` | `reviews/task-archive-early-hnsw/010-33-am-routine-module-split` | `task-archive-early-hnsw` | `archive` |
| `review/34-am-build-entrypoints-module-split` | `reviews/task-archive-early-hnsw/011-34-am-build-entrypoints-module-split` | `task-archive-early-hnsw` | `archive` |
| `review/37-am-build-state-type-ownership` | `reviews/task-archive-early-hnsw/012-37-am-build-state-type-ownership` | `task-archive-early-hnsw` | `archive` |
| `review/40-ci-workflow-hardening` | `reviews/task-archive-early-hnsw/013-40-ci-workflow-hardening` | `task-archive-early-hnsw` | `archive` |
| `review/75-shared-am-helper-boundary` | `reviews/task-archive-early-hnsw/014-75-shared-am-helper-boundary` | `task-archive-early-hnsw` | `archive` |
| `review/85-stale-scheduler-node-cleanup` | `reviews/task-archive-early-hnsw/015-85-stale-scheduler-node-cleanup` | `task-archive-early-hnsw` | `archive` |
| `review/86-remove-silent-top-up-reseed` | `reviews/task-archive-early-hnsw/016-86-remove-silent-top-up-reseed` | `task-archive-early-hnsw` | `archive` |
| `review/88-scheduler-node-first-consume` | `reviews/task-archive-early-hnsw/017-88-scheduler-node-first-consume` | `task-archive-early-hnsw` | `archive` |
| `review/116-current-result-debug-boundary` | `reviews/task-archive-early-hnsw/018-116-current-result-debug-boundary` | `task-archive-early-hnsw` | `archive` |
| `review/130-linear-helper-without-pending-drain` | `reviews/task-archive-early-hnsw/019-130-linear-helper-without-pending-drain` | `task-archive-early-hnsw` | `archive` |
| `review/131-explicit-linear-result-materialization` | `reviews/task-archive-early-hnsw/020-131-explicit-linear-result-materialization` | `task-archive-early-hnsw` | `archive` |
| `review/132-unified-staged-result-materialization` | `reviews/task-archive-early-hnsw/021-132-unified-staged-result-materialization` | `task-archive-early-hnsw` | `archive` |
| `review/138-linear-selection-before-materialization` | `reviews/task-archive-early-hnsw/022-138-linear-selection-before-materialization` | `task-archive-early-hnsw` | `archive` |
| `review/140-phase-aware-staged-result-selection` | `reviews/task-archive-early-hnsw/023-140-phase-aware-staged-result-selection` | `task-archive-early-hnsw` | `archive` |
| `review/141-result-state-owned-bookkeeping` | `reviews/task-archive-early-hnsw/024-141-result-state-owned-bookkeeping` | `task-archive-early-hnsw` | `archive` |
| `review/142-direct-staged-selection-in-tuple-production` | `reviews/task-archive-early-hnsw/025-142-direct-staged-selection-in-tuple-production` | `task-archive-early-hnsw` | `archive` |
| `review/144-explicit-phase-dispatch-for-staged-selection` | `reviews/task-archive-early-hnsw/026-144-explicit-phase-dispatch-for-staged-selection` | `task-archive-early-hnsw` | `archive` |
| `review/147-exhaustion-owns-result-state-clearing` | `reviews/task-archive-early-hnsw/027-147-exhaustion-owns-result-state-clearing` | `task-archive-early-hnsw` | `archive` |
| `review/149-opus-initial-review-batch` | `reviews/task-archive-early-hnsw/028-149-opus-initial-review-batch` | `task-archive-early-hnsw` | `archive` |
| `review/150-opus-pass2-review-batch` | `reviews/task-archive-early-hnsw/029-150-opus-pass2-review-batch` | `task-archive-early-hnsw` | `archive` |
| `review/151-spec-plan-evaluation` | `reviews/task-archive-early-hnsw/030-151-spec-plan-evaluation` | `task-archive-early-hnsw` | `archive` |
| `review/152-sonnet-pass3-review-batch` | `reviews/task-archive-early-hnsw/031-152-sonnet-pass3-review-batch` | `task-archive-early-hnsw` | `archive` |
| `review/153-plan-arch-evaluation` | `reviews/task-archive-early-hnsw/032-153-plan-arch-evaluation` | `task-archive-early-hnsw` | `archive` |
| `review/156-main-branch-batch-review` | `reviews/task-archive-early-hnsw/033-156-main-branch-batch-review` | `task-archive-early-hnsw` | `archive` |
| `review/157-cross-branch-alignment` | `reviews/task-archive-early-hnsw/034-157-cross-branch-alignment` | `task-archive-early-hnsw` | `archive` |
| `review/184-phase-aware-result-state-readers` | `reviews/task-archive-early-hnsw/035-184-phase-aware-result-state-readers` | `task-archive-early-hnsw` | `archive` |
| `review/198-a4-visited-set-leakage-across-phases` | `reviews/task-archive-early-hnsw/036-198-a4-visited-set-leakage-across-phases` | `task-archive-early-hnsw` | `archive` |
| `review/202-a4-1536-tail-truncation` | `reviews/task-archive-early-hnsw/037-202-a4-1536-tail-truncation` | `task-archive-early-hnsw` | `archive` |
| `review/206-a4-10k-operating-point-triage` | `reviews/task-archive-early-hnsw/038-206-a4-10k-operating-point-triage` | `task-archive-early-hnsw` | `archive` |
| `review/210-a4-fixture-backed-10k-gate` | `reviews/task-archive-early-hnsw/039-210-a4-fixture-backed-10k-gate` | `task-archive-early-hnsw` | `archive` |
| `review/211-a4-upper-hierarchy-oracle-k` | `reviews/task-archive-early-hnsw/040-211-a4-upper-hierarchy-oracle-k` | `task-archive-early-hnsw` | `archive` |
| `review/212-a4-build-hierarchy-collapse-audit` | `reviews/task-archive-early-hnsw/041-212-a4-build-hierarchy-collapse-audit` | `task-archive-early-hnsw` | `archive` |
| `review/213-a4-layer-localization-after-hierarchy-fix` | `reviews/task-archive-early-hnsw/042-213-a4-layer-localization-after-hierarchy-fix` | `task-archive-early-hnsw` | `archive` |
| `review/216-a4-reference-curve-vs-gate` | `reviews/task-archive-early-hnsw/043-216-a4-reference-curve-vs-gate` | `task-archive-early-hnsw` | `archive` |
| `review/217-a4-synthetic-vs-nfr-dataset-gap` | `reviews/task-archive-early-hnsw/044-217-a4-synthetic-vs-nfr-dataset-gap` | `task-archive-early-hnsw` | `archive` |
| `review/223-a4-real-10k-pass-and-loader-m-values` | `reviews/task-archive-early-hnsw/045-223-a4-real-10k-pass-and-loader-m-values` | `task-archive-early-hnsw` | `archive` |
| `review/224-a4-real-50k-directional-summary` | `reviews/task-archive-early-hnsw/046-224-a4-real-50k-directional-summary` | `task-archive-early-hnsw` | `archive` |
| `review/225-a4-cheaper-external-gate` | `reviews/task-archive-early-hnsw/047-225-a4-cheaper-external-gate` | `task-archive-early-hnsw` | `archive` |
| `review/226-a4-closeout` | `reviews/task-archive-early-hnsw/048-226-a4-closeout` | `task-archive-early-hnsw` | `archive` |
| `review/226-a4-real-50k-200-query-gate` | `reviews/task-archive-early-hnsw/049-226-a4-real-50k-200-query-gate` | `task-archive-early-hnsw` | `archive` |
| `review/256-c1-greedy-upper-layer-seeding` | `reviews/task-archive-early-hnsw/050-256-c1-greedy-upper-layer-seeding` | `task-archive-early-hnsw` | `archive` |
| `review/259-c1-executor-vs-am-startup-split` | `reviews/task-archive-early-hnsw/051-259-c1-executor-vs-am-startup-split` | `task-archive-early-hnsw` | `archive` |
| `review/260-c1-am-startup-boundary-reconciliation` | `reviews/task-archive-early-hnsw/052-260-c1-am-startup-boundary-reconciliation` | `task-archive-early-hnsw` | `archive` |
| `review/261-c1-warm-cache-verified-surface` | `reviews/task-archive-early-hnsw/053-261-c1-warm-cache-verified-surface` | `task-archive-early-hnsw` | `archive` |
| `review/264-c1-warm-steady-state-optimization-survey` | `reviews/task-archive-early-hnsw/054-264-c1-warm-steady-state-optimization-survey` | `task-archive-early-hnsw` | `archive` |
| `review/265-c1-disable-unused-query-prep` | `reviews/task-archive-early-hnsw/055-265-c1-disable-unused-query-prep` | `task-archive-early-hnsw` | `archive` |
| `review/267-c1-plain-query-timing-mode` | `reviews/task-archive-early-hnsw/056-267-c1-plain-query-timing-mode` | `task-archive-early-hnsw` | `archive` |
| `review/268-c1-cached-plan-query-timing` | `reviews/task-archive-early-hnsw/057-268-c1-cached-plan-query-timing` | `task-archive-early-hnsw` | `archive` |
| `review/272-c1-co-located-adjacency-batch-read` | `reviews/task-archive-early-hnsw/058-272-c1-co-located-adjacency-batch-read` | `task-archive-early-hnsw` | `archive` |
| `review/273-c1-negative-server-timing-rejection` | `reviews/task-archive-early-hnsw/059-273-c1-negative-server-timing-rejection` | `task-archive-early-hnsw` | `archive` |
| `review/274-c1-adr029-rank-correlation-study` | `reviews/task-archive-early-hnsw/060-274-c1-adr029-rank-correlation-study` | `task-archive-early-hnsw` | `archive` |
| `review/275-c1-adr029-source-expansion-survivor-gate` | `reviews/task-archive-early-hnsw/061-275-c1-adr029-source-expansion-survivor-gate` | `task-archive-early-hnsw` | `archive` |
| `review/276-c1-inline-heaptid-element-decode` | `reviews/task-archive-early-hnsw/062-276-c1-inline-heaptid-element-decode` | `task-archive-early-hnsw` | `archive` |
| `review/277-c1-successor-scratch-buffer-reuse` | `reviews/task-archive-early-hnsw/063-277-c1-successor-scratch-buffer-reuse` | `task-archive-early-hnsw` | `archive` |
| `review/279-c1-adr031-sign-binary-study` | `reviews/task-archive-early-hnsw/064-279-c1-adr031-sign-binary-study` | `task-archive-early-hnsw` | `archive` |
| `review/281-c1-adr031-cached-binary-prefilter-runtime` | `reviews/task-archive-early-hnsw/065-281-c1-adr031-cached-binary-prefilter-runtime` | `task-archive-early-hnsw` | `archive` |
| `review/282-c1-adr031-real-50k-scale-validation` | `reviews/task-archive-early-hnsw/066-282-c1-adr031-real-50k-scale-validation` | `task-archive-early-hnsw` | `archive` |
| `review/285-c1-adr031-persisted-binary-sidecar-feasibility` | `reviews/task-archive-early-hnsw/067-285-c1-adr031-persisted-binary-sidecar-feasibility` | `task-archive-early-hnsw` | `archive` |
| `review/286-c1-adr031-sidecar-ab-compare` | `reviews/task-archive-early-hnsw/068-286-c1-adr031-sidecar-ab-compare` | `task-archive-early-hnsw` | `archive` |
| `review/289-c1-adr031-on-off-ab` | `reviews/task-archive-early-hnsw/069-289-c1-adr031-on-off-ab` | `task-archive-early-hnsw` | `archive` |
| `review/293-c1-adr032-fused-node-cache` | `reviews/task-archive-early-hnsw/070-293-c1-adr032-fused-node-cache` | `task-archive-early-hnsw` | `archive` |
| `review/415-c1-standalone-cargo-test-pg-backend-stubs` | `reviews/task-archive-early-hnsw/071-415-c1-standalone-cargo-test-pg-backend-stubs` | `task-archive-early-hnsw` | `archive` |
| `review/446-c1-native-hnsw-build-path` | `reviews/task-archive-early-hnsw/072-446-c1-native-hnsw-build-path` | `task-archive-early-hnsw` | `archive` |
| `review/449-c1-native-build-harness-fix-and-hnsw-rs-removal` | `reviews/task-archive-early-hnsw/073-449-c1-native-build-harness-fix-and-hnsw-rs-removal` | `task-archive-early-hnsw` | `archive` |
| `review/450-c1-native-build-stability-tightening` | `reviews/task-archive-early-hnsw/074-450-c1-native-build-stability-tightening` | `task-archive-early-hnsw` | `archive` |
| `review/451-c1-native-build-heuristic-tests` | `reviews/task-archive-early-hnsw/075-451-c1-native-build-heuristic-tests` | `task-archive-early-hnsw` | `archive` |
| `review/453-c1-native-build-helper-coverage` | `reviews/task-archive-early-hnsw/076-453-c1-native-build-helper-coverage` | `task-archive-early-hnsw` | `archive` |
| `review/454-c1-source-gate-fixture-reuse` | `reviews/task-archive-early-hnsw/077-454-c1-source-gate-fixture-reuse` | `task-archive-early-hnsw` | `archive` |
| `review/457-c1-native-build-seed-copy-cleanup` | `reviews/task-archive-early-hnsw/078-457-c1-native-build-seed-copy-cleanup` | `task-archive-early-hnsw` | `archive` |
| `review/461-c1-ec-am-surface-rename` | `reviews/task-archive-early-hnsw/079-461-c1-ec-am-surface-rename` | `task-archive-early-hnsw` | `archive` |
| `review/463-c1-ecaz-rename-alignment` | `reviews/task-archive-early-hnsw/080-463-c1-ecaz-rename-alignment` | `task-archive-early-hnsw` | `archive` |
| `review/464-c1-task-numbering-alignment` | `reviews/task-archive-early-hnsw/081-464-c1-task-numbering-alignment` | `task-archive-early-hnsw` | `archive` |
| `review/465-c1-native-build-doc-alignment` | `reviews/task-archive-early-hnsw/082-465-c1-native-build-doc-alignment` | `task-archive-early-hnsw` | `archive` |
| `review/471-c1-ecaz-cli-script-removal` | `reviews/task-archive-early-hnsw/083-471-c1-ecaz-cli-script-removal` | `task-archive-early-hnsw` | `archive` |
| `review/472-c1-ecaz-cli-dev-tooling-consolidation` | `reviews/task-archive-early-hnsw/084-472-c1-ecaz-cli-dev-tooling-consolidation` | `task-archive-early-hnsw` | `archive` |
| `review/623-c1-ecaz-cli-generic-dev-sql` | `reviews/task-archive-early-hnsw/085-623-c1-ecaz-cli-generic-dev-sql` | `task-archive-early-hnsw` | `archive` |
| `review/633-c1-native-build-level-precompute` | `reviews/task-archive-early-hnsw/086-633-c1-native-build-level-precompute` | `task-archive-early-hnsw` | `archive` |
| `review/666-c1-phase3-real50k-summary` | `reviews/task-archive-early-hnsw/087-666-c1-phase3-real50k-summary` | `task-archive-early-hnsw` | `archive` |
| `review/10048-deleted-entry-point-staleness` | `reviews/task-archive-early-hnsw/088-10048-deleted-entry-point-staleness` | `task-archive-early-hnsw` | `archive` |
| `review/10049-datapage-chain-linear-lookup` | `reviews/task-archive-early-hnsw/089-10049-datapage-chain-linear-lookup` | `task-archive-early-hnsw` | `archive` |
| `review/10058-portable-manifest-source-parquet` | `reviews/task-archive-early-hnsw/090-10058-portable-manifest-source-parquet` | `task-archive-early-hnsw` | `archive` |
| `review/11014-adr045-page-layout-discipline` | `reviews/task-archive-early-hnsw/091-11014-adr045-page-layout-discipline` | `task-archive-early-hnsw` | `archive` |
| `review/11016-phase5b-slim-tuple` | `reviews/task-archive-early-hnsw/092-11016-phase5b-slim-tuple` | `task-archive-early-hnsw` | `archive` |
| `review/11017-phase5c1-persist-sequencer` | `reviews/task-archive-early-hnsw/093-11017-phase5c1-persist-sequencer` | `task-archive-early-hnsw` | `archive` |
| `review/11018-phase5c2-build-orchestrator` | `reviews/task-archive-early-hnsw/094-11018-phase5c2-build-orchestrator` | `task-archive-early-hnsw` | `archive` |
| `review/11024-adr046-review-prep` | `reviews/task-archive-early-hnsw/095-11024-adr046-review-prep` | `task-archive-early-hnsw` | `archive` |
| `review/11025-adr047-review-prep` | `reviews/task-archive-early-hnsw/096-11025-adr047-review-prep` | `task-archive-early-hnsw` | `archive` |
| `review/11026-visited-state-reuse` | `reviews/task-archive-early-hnsw/097-11026-visited-state-reuse` | `task-archive-early-hnsw` | `archive` |
| `review/11027-reader-live-tid-iteration` | `reviews/task-archive-early-hnsw/098-11027-reader-live-tid-iteration` | `task-archive-early-hnsw` | `archive` |
| `review/11030-phase5c3-pgrx-ambuild-wiring` | `reviews/task-archive-early-hnsw/099-11030-phase5c3-pgrx-ambuild-wiring` | `task-archive-early-hnsw` | `archive` |
| `review/11035-phase7c-exact-duplicate-probe-boundary` | `reviews/task-archive-early-hnsw/100-11035-phase7c-exact-duplicate-probe-boundary` | `task-archive-early-hnsw` | `archive` |
| `review/11037-phase7e-forward-link-append` | `reviews/task-archive-early-hnsw/101-11037-phase7e-forward-link-append` | `task-archive-early-hnsw` | `archive` |
| `review/11038-phase7f-free-capacity-backlinks` | `reviews/task-archive-early-hnsw/102-11038-phase7f-free-capacity-backlinks` | `task-archive-early-hnsw` | `archive` |
| `review/11040-phase7h-full-backlink-rewrite-replan` | `reviews/task-archive-early-hnsw/103-11040-phase7h-full-backlink-rewrite-replan` | `task-archive-early-hnsw` | `archive` |
| `review/11047-phase9-review-followup-docs` | `reviews/task-archive-early-hnsw/104-11047-phase9-review-followup-docs` | `task-archive-early-hnsw` | `archive` |
| `review/30154-task-doc-status-cleanup` | `reviews/task-archive-early-hnsw/105-30154-task-doc-status-cleanup` | `task-archive-early-hnsw` | `archive` |
| `review/30156-spec-current-state-refresh` | `reviews/task-archive-early-hnsw/106-30156-spec-current-state-refresh` | `task-archive-early-hnsw` | `archive` |
| `review/30157-ecaz-cli-docs-spec` | `reviews/task-archive-early-hnsw/107-30157-ecaz-cli-docs-spec` | `task-archive-early-hnsw` | `archive` |
| `review/30161-m5-ann-optimization-task-split` | `reviews/task-archive-early-hnsw/108-30161-m5-ann-optimization-task-split` | `task-archive-early-hnsw` | `archive` |
| `review/30655-readme-build-setup` | `reviews/task-archive-early-hnsw/109-30655-readme-build-setup` | `task-archive-early-hnsw` | `archive` |
| `review/30656-readme-performance-summary` | `reviews/task-archive-early-hnsw/110-30656-readme-performance-summary` | `task-archive-early-hnsw` | `archive` |
| `review/30657-mit-license` | `reviews/task-archive-early-hnsw/111-30657-mit-license` | `task-archive-early-hnsw` | `archive` |
| `review/30759-ecaz-cli-repo-root-discovery` | `reviews/task-archive-early-hnsw/112-30759-ecaz-cli-repo-root-discovery` | `task-archive-early-hnsw` | `archive` |
| `review/30767-standalone-pg-stub-invariant` | `reviews/task-archive-early-hnsw/113-30767-standalone-pg-stub-invariant` | `task-archive-early-hnsw` | `archive` |
| `review/31140-c1-unsafe-quality-burndown-scaffold` | `reviews/task-archive-early-hnsw/114-31140-c1-unsafe-quality-burndown-scaffold` | `task-archive-early-hnsw` | `archive` |
| `review/31141-c1-unsafe-burndown-one-entry-production` | `reviews/task-archive-early-hnsw/115-31141-c1-unsafe-burndown-one-entry-production` | `task-archive-early-hnsw` | `archive` |
| `review/31142-c1-am-routine-unsafe-boundaries` | `reviews/task-archive-early-hnsw/116-31142-c1-am-routine-unsafe-boundaries` | `task-archive-early-hnsw` | `archive` |
| `review/31142-standards-coverage-hardening-spec` | `reviews/task-archive-early-hnsw/117-31142-standards-coverage-hardening-spec` | `task-archive-early-hnsw` | `archive` |
| `review/31143-standards-compliance-claim-fixes` | `reviews/task-archive-early-hnsw/118-31143-standards-compliance-claim-fixes` | `task-archive-early-hnsw` | `archive` |
| `review/31144-readme-rust-safety-link` | `reviews/task-archive-early-hnsw/119-31144-readme-rust-safety-link` | `task-archive-early-hnsw` | `archive` |
| `review/31146-nfr016-adr070-spec-review` | `reviews/task-archive-early-hnsw/120-31146-nfr016-adr070-spec-review` | `task-archive-early-hnsw` | `archive` |
| `review/round-30303-30336-feedback.md` | `reviews/task-archive-early-hnsw/121-round-30303-30336-feedback/request.md` | `task-archive-early-hnsw` | `archive` |

## Deferred Paths

- `benchmarks/10064-external-recall-harness-truth-cache` (skip-benchmark)
- `benchmarks/10066-chunked-corpus-prepare-load` (skip-benchmark)
- `benchmarks/11050-task17-recall-doc-diskann-profile` (skip-benchmark)
- `benchmarks/11055-task17-profile-default-sweep` (skip-benchmark)
- `benchmarks/11058-task17-bench-am-preflight` (skip-benchmark)
- `benchmarks/11059-task17-corpus-list-profiles` (skip-benchmark)
- `benchmarks/11061-task17-corpus-inspect-profiles` (skip-benchmark)
- `benchmarks/11063-task17-storage-profiles` (skip-benchmark)
- `benchmarks/11064-task17-bench-sweep-labels` (skip-benchmark)
- `benchmarks/11071-task17-ecaz-fetch-real-corpus` (skip-benchmark)
- `benchmarks/11073-task17-diskann-real-10k-recall` (skip-benchmark)
- `benchmarks/11077-task17-bench-force-ann-path` (skip-benchmark)
- `benchmarks/11078-task17-diskann-real-recall-recovery` (skip-benchmark)
- `benchmarks/11081-task17-diskann-vacuum-recall` (skip-benchmark)
- `benchmarks/11097-task29-diskann-heap-frontier-latency` (skip-benchmark)
- `benchmarks/11098-task29-diskann-early-stop-latency` (skip-benchmark)
- `benchmarks/11101-task29c-diskann-build-phase-profile` (skip-benchmark)
- `benchmarks/11102-task29c-vamana-core-profile` (skip-benchmark)
- `benchmarks/11104-task29c-prune-active-mask-profile` (skip-benchmark)
- `benchmarks/11105-task29-release-latency-refresh` (skip-benchmark)
- `benchmarks/11107-task29d-l64-scan-profile` (skip-benchmark)
- `benchmarks/194-graph-scan-recall-gate` (skip-benchmark)
- `benchmarks/200-a4-recall-gate-rerun` (skip-benchmark)
- `benchmarks/218-a4-real-corpus-recall-lane` (skip-benchmark)
- `benchmarks/219-a4-real-corpus-loader-smoke` (skip-benchmark)
- `benchmarks/220-a4-real-corpus-metric-contract-followup` (skip-benchmark)
- `benchmarks/221-a4-real-corpus-subset-manifest-contract` (skip-benchmark)
- `benchmarks/222-a4-real-corpus-fetch-and-schema-alignment` (skip-benchmark)
- `benchmarks/223-a4-recall-investigation-harness` (skip-benchmark)
- `benchmarks/224-a4-ann-benchmarks-anchor` (skip-benchmark)
- `benchmarks/225-a4-nfr-001-latency-real-corpus` (skip-benchmark)
- `benchmarks/226-a4-recall-smoke-copy-batching` (skip-benchmark)
- `benchmarks/244-c1-real-corpus-latency-hardening` (skip-benchmark)
- `benchmarks/245-c1-real-corpus-latency-10k-run` (skip-benchmark)
- `benchmarks/246-c1-latency-launcher-plan-verification` (skip-benchmark)
- `benchmarks/247-c1-real-corpus-latency-10k-verified-run` (skip-benchmark)
- `benchmarks/283-c1-adr031-real-corpus-recall-validation` (skip-benchmark)
- `benchmarks/284-c1-adr031-high-ef-runtime-recall-parity` (skip-benchmark)
- `benchmarks/30015-task28-ivf-recall-smoke` (skip-benchmark)
- `benchmarks/30035-task28-ivf-cli-bench-profile` (skip-benchmark)
- `benchmarks/30070-task28-ivf-borrowed-scan-recall` (skip-benchmark)
- `benchmarks/30081-task28-ivf-rabitq-profile` (skip-benchmark)
- `benchmarks/30082-task28-ivf-pqfastscan-profile` (skip-benchmark)
- `benchmarks/30112-task28-ivf-latency-memory-hwm` (skip-benchmark)
- `benchmarks/30113-task28-ivf-a9-100k-latency-memory` (skip-benchmark)
- `benchmarks/30117-task28-ivf-pqfastscan-bound-recall100` (skip-benchmark)
- `benchmarks/30132-task28-ivf-990k-lower-nprobe-latency` (skip-benchmark)
- `benchmarks/30133-task28-ivf-990k-balanced-recall100` (skip-benchmark)
- `benchmarks/30147-task28-ivf-recall-truth-cache` (skip-benchmark)
- `benchmarks/30148-task28-ivf-recall-truth-cache-smoke` (skip-benchmark)
- `benchmarks/30149-task28-ivf-990k-w250-recall-cache` (skip-benchmark)
- `benchmarks/30166-task31-m5-real-corpus-preflight` (skip-benchmark)
- `benchmarks/30167-task31-m5-real-corpus-staging` (skip-benchmark)
- `benchmarks/30168-task31-corpus-prepare-profiles` (skip-benchmark)
- `benchmarks/30177-task31-benchmark-suite-runner-adr` (skip-benchmark)
- `benchmarks/30528-spire-cli-benchmark-profile` (skip-benchmark)
- `benchmarks/30530-spire-phase1-recall-latency-gate` (skip-benchmark)
- `benchmarks/30533-spire-local-placement-benchmark` (skip-benchmark)
- `benchmarks/30545-benchmark-profile-surface` (skip-benchmark)
- `benchmarks/30548-spire-boundary-recall-study` (skip-benchmark)
- `benchmarks/30622-spire-suite-explain-profile` (skip-benchmark)
- `benchmarks/30690-spire-pipeline-benchmark-counters` (skip-benchmark)
- `benchmarks/30924-spire-2pc-latency-bulk-load-docs` (skip-benchmark)
- `benchmarks/30964-spire-tuple-transport-bench-knob` (skip-benchmark)
- `benchmarks/30975-spire-tuple-transport-measurement` (skip-benchmark)
- `benchmarks/31141-benchmark-reporting-standard` (skip-benchmark)
- `review/31151-c1-task41-sql-diagnostic-index-guard` (skip-task41)
- `review/31152-c1-task41-more-sql-diagnostic-index-guards` (skip-task41)
- `review/31153-c1-task41-remote-manifest-index-guards` (skip-task41)
- `review/31154-c1-task41-remote-manifest-catalog-guards` (skip-task41)
- `review/31155-c1-task41-remote-manifest-libpq-guards` (skip-task41)
- `review/31156-c1-task41-remote-manifest-executor-guards` (skip-task41)
- `review/31157-c1-task41-remote-manifest-result-guards` (skip-task41)
- `review/31158-c1-task41-scan-diagnostic-relation-guards` (skip-task41)
- `review/31159-c1-task41-coordinator-relation-guards` (skip-task41)
- `review/31160-c1-task41-dml-forwarding-relation-guards` (skip-task41)
- `review/31161-c1-task41-unsafe-surface-strategy` (skip-task41)
- `review/31162-c1-task41-spire-diagnostic-relation-guards` (skip-task41)
- `review/31163-c1-task41-spire-executor-relation-guards` (skip-task41)
- `review/31164-c1-task41-spire-production-diagnostic-guards` (skip-task41)
- `review/31165-c1-task41-spire-pipeline-relation-guards` (skip-task41)
- `review/31166-c1-task41-spire-tuple-payload-relation-guard` (skip-task41)
- `review/31167-c1-task41-spire-search-tuple-payload-guards` (skip-task41)
- `review/31168-c1-task41-spire-remote-search-summary-guards` (skip-task41)
- `review/31169-c1-task41-spire-index-snapshot-guards` (skip-task41)
- `review/31170-c1-task41-spire-maintenance-relation-guards` (skip-task41)
- `review/31171-c1-task41-hnsw-ivf-relation-guards` (skip-task41)
- `review/31172-c1-task41-spire-dml-frontdoor-catalog-guards` (skip-task41)
- `review/31173-c1-task41-spire-coordinator-debug-index-guards` (skip-task41)
- `review/31174-c1-task41-spire-relation-store-open-guards` (skip-task41)
- `review/31175-c1-task41-spire-custom-scan-planner-guards` (skip-task41)
- `review/31176-c1-task41-spire-vacuum-debug-relation-guard` (skip-task41)
- `review/31177-c1-task41-spire-scan-heap-slot-guard` (skip-task41)
- `review/31178-c1-task41-spire-dml-frontdoor-heap-guard` (skip-task41)
- `review/31179-c1-task41-hnsw-shared-debug-index-guard` (skip-task41)
- `review/31180-c1-task41-shared-relation-guard-consolidation` (skip-task41)
- `review/31181-c1-task41-hnsw-scan-debug-index-guards` (skip-task41)
- `review/31182-c1-task41-hnsw-debug-heap-scan-resource-guards` (skip-task41)
- `review/31183-c1-task41-hnsw-graph-debug-index-guards` (skip-task41)
- `review/31184-c1-task41-hnsw-shared-relation-guard-consolidation` (skip-task41)
- `review/31185-c1-task41-spire-vacuum-relation-guard-consolidation` (skip-task41)
- `review/31186-c1-task41-spire-custom-scan-relation-guard-consolidation` (skip-task41)
- `review/31187-c1-task41-spire-maintenance-heap-relation-guard-consolidation` (skip-task41)
- `review/31188-c1-task41-planner-cost-relation-guard-consolidation` (skip-task41)
- `review/31189-c1-task41-shared-spire-cost-relation-guard-consolidation` (skip-task41)
- `review/31190-c1-task41-ivf-debug-duplicate-relation-guard` (skip-task41)
- `review/31191-c1-task41-generic-relation-guard` (skip-task41)
- `review/31192-c1-task41-spire-snapshot-relation-guard` (skip-task41)
- `review/31193-c1-task41-hnsw-build-heap-relation-guards` (skip-task41)
- `review/31194-c1-task41-hnsw-scan-source-heap-relation-guard` (skip-task41)
- `review/31195-c1-task41-shared-snapshot-slot-guards` (skip-task41)
- `review/31196-c1-task41-spire-scan-slot-guard` (skip-task41)
- `review/31197-c1-task41-hnsw-debug-snapshot-slot-guards` (skip-task41)
- `review/31198-c1-task41-shared-index-scan-guard` (skip-task41)
- `review/31199-c1-task41-hnsw-oracle-debug-index-relation-guards` (skip-task41)
- `review/31200-c1-task41-hnsw-scan-state-debug-relation-guards` (skip-task41)
- `review/31201-c1-task41-hnsw-frontier-debug-relation-guards` (skip-task41)
- `review/31202-c1-task41-hnsw-scan-debug-tail-relation-guards` (skip-task41)
- `review/31203-c1-task41-spire-maintenance-slot-guard` (skip-task41)
- `review/31204-c1-task41-spire-remote-heap-resolution-guards` (skip-task41)
- `review/31205-c1-task41-ivf-debug-index-relation-guards` (skip-task41)
- `review/31206-c1-task41-ivf-vacuum-debug-index-guards` (skip-task41)
- `review/31207-c1-task41-custom-scan-test-slot-guards` (skip-task41)
- `review/31208-c1-task41-spire-custom-scan-dml-output-guard` (skip-task41)
- `review/31209-c1-task41-spire-scan-heap-relation-guard` (skip-task41)
- `review/31210-c1-task41-diskann-insert-slot-guards` (skip-task41)
- `review/31211-c1-task41-diskann-scan-rerank-slot-guard` (skip-task41)
- `review/31212-c1-task41-diskann-backlink-slot-guards` (skip-task41)
- `review/31213-c1-task41-diskann-vacuum-refill-fixture-guards` (skip-task41)
- `review/31214-c1-task41-diskann-test-helper-relation-guards` (skip-task41)
- `review/31215-c1-task41-diskann-materialized-chain-helper-reuse` (skip-task41)
- `review/31216-c1-task41-diskann-vacuum-heap-relation-guard` (skip-task41)
- `review/31217-c1-task41-diskann-scan-state-raii` (skip-task41)
- `review/31218-c1-task41-diskann-dead-slot-helper-removal` (skip-task41)
- `review/31219-c1-task41-ivf-debug-heap-scan-guards` (skip-task41)
- `review/31220-c1-task41-ivf-heap-rerank-raii-state` (skip-task41)
- `review/31221-c1-task41-hnsw-heap-rerank-raii-state` (skip-task41)
- `review/31222-c1-task41-hnsw-insert-source-slot-guard` (skip-task41)
- `review/31223-c1-task41-hnsw-parallel-build-worker-relation-guards` (skip-task41)
- `review/31224-c1-task41-hnsw-source-build-scan-guards` (skip-task41)
- `review/31225-c1-task41-hnsw-vacuum-resource-guards` (skip-task41)
- `review/31226-c1-task41-ivf-debug-heap-scan-guards` (skip-task41)
- `review/31227-c1-task41-diskann-scan-snapshot-guard` (skip-task41)
- `benchmarks/354-c1-adr030-v2-verified-grouped-runtime-remeasurement` (skip-benchmark)
- `benchmarks/357-c1-adr030-v2-grouped-exact-cost-profile` (skip-benchmark)
- `benchmarks/364-c1-adr030-v2-pgvector-sql-latency-harness` (skip-benchmark)
- `benchmarks/372-c1-adr030-v2-heap-rerank-cost-profile` (skip-benchmark)
- `benchmarks/396-c1-adr030-v2-pqfastscan-profile-debug-surface-rename` (skip-benchmark)
- `benchmarks/398-c1-adr030-v2-real-corpus-storage-format-harness` (skip-benchmark)
- `benchmarks/399-c1-adr030-v2-external-recall-smoke-storage-formats` (skip-benchmark)
- `benchmarks/423-c1-task16-turboquant-m16-baseline-stage-profile` (skip-benchmark)
- `benchmarks/426-c1-task16-turboquant-quantized-default-measurement` (skip-benchmark)
- `benchmarks/429-c1-task16-turboquant-v3-serious-lane-measurement` (skip-benchmark)
- `benchmarks/430-c1-task16-turboquant-v3-source-raw-rerank-measurement` (skip-benchmark)
- `benchmarks/432-c1-task16-persisted-rerank-source-measurement` (skip-benchmark)
- `benchmarks/440-c1-task16-turboquant-persisted-source-supported-measurement` (skip-benchmark)
- `benchmarks/441-c1-task16-turboquant-inline-raw-source-layout-measurement` (skip-benchmark)
- `benchmarks/446-c1-task16-ecvector-surface-head-to-head-measurement` (skip-benchmark)
- `benchmarks/448-c1-native-build-real-corpus-gate` (skip-benchmark)
- `benchmarks/459-c1-native-build-cached-baseline-measurements` (skip-benchmark)
- `benchmarks/58-benchmark-coverage-and-data-quality` (skip-benchmark)
- `benchmarks/59-consumed-candidate-refill-and-benchmark-baseline` (skip-benchmark)
- `benchmarks/620-c1-parallel-index-build-pg18-measurement` (skip-benchmark)
- `benchmarks/622-c1-parallel-index-build-phase-measurement` (skip-benchmark)
- `benchmarks/624-c1-native-build-sparse-query-score-cache-measurement` (skip-benchmark)
- `benchmarks/625-c1-native-build-layer-search-scratch-measurement` (skip-benchmark)
- `benchmarks/626-c1-parallel-index-build-50k-scale-measurement` (skip-benchmark)
- `benchmarks/627-c1-build-state-dedup-index-measurement` (skip-benchmark)
- `benchmarks/628-c1-native-graph-scratch-cache-measurement` (skip-benchmark)
- `benchmarks/629-c1-native-decoded-score-workspace-measurement` (skip-benchmark)
- `benchmarks/630-c1-native-source-score-workspace-measurement` (skip-benchmark)
- `benchmarks/631-c1-native-neighbor-flatten-measurement` (skip-benchmark)
- `benchmarks/636-c1-concurrent-dsm-code-corpus` (skip-benchmark)
- `benchmarks/648-c1-parallel-concurrent-dsm-50k-measurement` (skip-benchmark)
- `benchmarks/650-c1-parallel-concurrent-dsm-recall-validation` (skip-benchmark)
- `benchmarks/651-c1-parallel-concurrent-dsm-50k-recall-validation` (skip-benchmark)
- `benchmarks/652-c1-parallel-concurrent-dsm-tuned-recall-validation` (skip-benchmark)
- `benchmarks/654-c1-concurrent-dsm-striped-tuned-recall-validation` (skip-benchmark)
- `benchmarks/655-c1-concurrent-dsm-striped-50k-tuned-recall-validation` (skip-benchmark)
- `benchmarks/656-c1-concurrent-dsm-striped-real-50k-recall-validation` (skip-benchmark)
- `benchmarks/661-c1-concurrent-dsm-scratch-buffer-measurement` (skip-benchmark)
- `benchmarks/673-c1-ivf-diskann-bench-pivot` (skip-benchmark)
- `benchmarks/705-c1-spire-recall-fixtures` (skip-benchmark)
- `benchmarks/739-c1-spire-recall-tracker-reconciliation` (skip-benchmark)
- `benchmarks/765-c1-spire-phase13c-aws-readiness` (skip-benchmark)
- `benchmarks/767-c1-spire-phase13-aws-preflight` (skip-benchmark)
- `benchmarks/768-c1-spire-phase13-aws-verification` (skip-benchmark)
- `benchmarks/769-c1-real-corpus-prefix-rename` (skip-benchmark)
- `review/README.md` (doc)
- `review/REVIEWER.md` (doc)
- `review/cloud-10k-baselines` (skip-benchmark)
- `review/cloud-10k-graviton-preopt-baselines` (skip-benchmark)
- `review/cloud-10k-real-baselines` (skip-benchmark)
- `review/cloud-scaling-multi-am` (skip-benchmark)

## Post-Migration Bucket Corrections

After the first mechanical pass, the constructed archive bucket was narrowed by moving obvious historical packets to real task buckets.

- `reviews/task-archive-early-hnsw/000-02-tail-page-reuse-and-rollover` -> `reviews/task-04/001-02-tail-page-reuse-and-rollover`
- `reviews/task-archive-early-hnsw/001-03-duplicate-coalescing-and-capacity` -> `reviews/task-06/018-03-duplicate-coalescing-and-capacity`
- `reviews/task-archive-early-hnsw/002-12-tail-page-rollover-followup` -> `reviews/task-04/002-12-tail-page-rollover-followup`
- `reviews/task-archive-early-hnsw/003-19-build-detoast-copy-detection` -> `reviews/task-02/001-19-build-detoast-copy-detection`
- `reviews/task-archive-early-hnsw/004-21-page-offset-checked-conversion` -> `reviews/task-04/003-21-page-offset-checked-conversion`
- `reviews/task-archive-early-hnsw/005-22-pin-hnsw-rs-version` -> `reviews/task-archive-cross-cutting/00-22-pin-hnsw-rs-version`
- `reviews/task-archive-early-hnsw/006-24-relation-options-cache` -> `reviews/task-03/001-24-relation-options-cache`
- `reviews/task-archive-early-hnsw/007-25-zero-allocation-code-scoring` -> `reviews/task-16/131-25-zero-allocation-code-scoring`
- `reviews/task-archive-early-hnsw/008-30-plan-and-spec-backfill` -> `reviews/task-archive-cross-cutting/001-30-plan-and-spec-backfill`
- `reviews/task-archive-early-hnsw/009-32-am-options-module-split` -> `reviews/task-05/182-32-am-options-module-split`
- `reviews/task-archive-early-hnsw/010-33-am-routine-module-split` -> `reviews/task-05/183-33-am-routine-module-split`
- `reviews/task-archive-early-hnsw/011-34-am-build-entrypoints-module-split` -> `reviews/task-05/184-34-am-build-entrypoints-module-split`
- `reviews/task-archive-early-hnsw/012-37-am-build-state-type-ownership` -> `reviews/task-05/185-37-am-build-state-type-ownership`
- `reviews/task-archive-early-hnsw/013-40-ci-workflow-hardening` -> `reviews/task-09/001-40-ci-workflow-hardening`
- `reviews/task-archive-early-hnsw/014-75-shared-am-helper-boundary` -> `reviews/task-05/186-75-shared-am-helper-boundary`
- `reviews/task-archive-early-hnsw/015-85-stale-scheduler-node-cleanup` -> `reviews/task-05/187-85-stale-scheduler-node-cleanup`
- `reviews/task-archive-early-hnsw/016-86-remove-silent-top-up-reseed` -> `reviews/task-archive-cross-cutting/02-86-remove-silent-top-up-reseed`
- `reviews/task-archive-early-hnsw/017-88-scheduler-node-first-consume` -> `reviews/task-05/188-88-scheduler-node-first-consume`
- `reviews/task-archive-early-hnsw/018-116-current-result-debug-boundary` -> `reviews/task-05/189-116-current-result-debug-boundary`
- `reviews/task-archive-early-hnsw/019-130-linear-helper-without-pending-drain` -> `reviews/task-05/190-130-linear-helper-without-pending-drain`
- `reviews/task-archive-early-hnsw/020-131-explicit-linear-result-materialization` -> `reviews/task-05/191-131-explicit-linear-result-materialization`
- `reviews/task-archive-early-hnsw/021-132-unified-staged-result-materialization` -> `reviews/task-05/192-132-unified-staged-result-materialization`
- `reviews/task-archive-early-hnsw/022-138-linear-selection-before-materialization` -> `reviews/task-05/193-138-linear-selection-before-materialization`
- `reviews/task-archive-early-hnsw/023-140-phase-aware-staged-result-selection` -> `reviews/task-05/194-140-phase-aware-staged-result-selection`
- `reviews/task-archive-early-hnsw/024-141-result-state-owned-bookkeeping` -> `reviews/task-05/195-141-result-state-owned-bookkeeping`
- `reviews/task-archive-early-hnsw/025-142-direct-staged-selection-in-tuple-production` -> `reviews/task-05/196-142-direct-staged-selection-in-tuple-production`
- `reviews/task-archive-early-hnsw/026-144-explicit-phase-dispatch-for-staged-selection` -> `reviews/task-05/197-144-explicit-phase-dispatch-for-staged-selection`
- `reviews/task-archive-early-hnsw/027-147-exhaustion-owns-result-state-clearing` -> `reviews/task-05/198-147-exhaustion-owns-result-state-clearing`
- `reviews/task-archive-early-hnsw/028-149-opus-initial-review-batch` -> `reviews/task-archive-cross-cutting/002-149-opus-initial-review-batch`
- `reviews/task-archive-early-hnsw/029-150-opus-pass2-review-batch` -> `reviews/task-archive-cross-cutting/003-150-opus-pass2-review-batch`
- `reviews/task-archive-early-hnsw/030-151-spec-plan-evaluation` -> `reviews/task-archive-cross-cutting/004-151-spec-plan-evaluation`
- `reviews/task-archive-early-hnsw/031-152-sonnet-pass3-review-batch` -> `reviews/task-archive-cross-cutting/005-152-sonnet-pass3-review-batch`
- `reviews/task-archive-early-hnsw/032-153-plan-arch-evaluation` -> `reviews/task-archive-cross-cutting/006-153-plan-arch-evaluation`
- `reviews/task-archive-early-hnsw/033-156-main-branch-batch-review` -> `reviews/task-archive-cross-cutting/007-156-main-branch-batch-review`
- `reviews/task-archive-early-hnsw/034-157-cross-branch-alignment` -> `reviews/task-archive-cross-cutting/008-157-cross-branch-alignment`
- `reviews/task-archive-early-hnsw/035-184-phase-aware-result-state-readers` -> `reviews/task-05/199-184-phase-aware-result-state-readers`
- `reviews/task-archive-early-hnsw/036-198-a4-visited-set-leakage-across-phases` -> `reviews/task-05/200-198-a4-visited-set-leakage-across-phases`
- `reviews/task-archive-early-hnsw/037-202-a4-1536-tail-truncation` -> `reviews/task-16/132-202-a4-1536-tail-truncation`
- `reviews/task-archive-early-hnsw/038-206-a4-10k-operating-point-triage` -> `reviews/task-05/201-206-a4-10k-operating-point-triage`
- `reviews/task-archive-early-hnsw/039-210-a4-fixture-backed-10k-gate` -> `reviews/task-05/202-210-a4-fixture-backed-10k-gate`
- `reviews/task-archive-early-hnsw/040-211-a4-upper-hierarchy-oracle-k` -> `reviews/task-05/203-211-a4-upper-hierarchy-oracle-k`
- `reviews/task-archive-early-hnsw/041-212-a4-build-hierarchy-collapse-audit` -> `reviews/task-05/204-212-a4-build-hierarchy-collapse-audit`
- `reviews/task-archive-early-hnsw/042-213-a4-layer-localization-after-hierarchy-fix` -> `reviews/task-05/205-213-a4-layer-localization-after-hierarchy-fix`
- `reviews/task-archive-early-hnsw/043-216-a4-reference-curve-vs-gate` -> `reviews/task-05/206-216-a4-reference-curve-vs-gate`
- `reviews/task-archive-early-hnsw/044-217-a4-synthetic-vs-nfr-dataset-gap` -> `reviews/task-05/207-217-a4-synthetic-vs-nfr-dataset-gap`
- `reviews/task-archive-early-hnsw/045-223-a4-real-10k-pass-and-loader-m-values` -> `reviews/task-05/208-223-a4-real-10k-pass-and-loader-m-values`
- `reviews/task-archive-early-hnsw/046-224-a4-real-50k-directional-summary` -> `reviews/task-05/209-224-a4-real-50k-directional-summary`
- `reviews/task-archive-early-hnsw/047-225-a4-cheaper-external-gate` -> `reviews/task-05/210-225-a4-cheaper-external-gate`
- `reviews/task-archive-early-hnsw/048-226-a4-closeout` -> `reviews/task-05/211-226-a4-closeout`
- `reviews/task-archive-early-hnsw/049-226-a4-real-50k-200-query-gate` -> `reviews/task-05/212-226-a4-real-50k-200-query-gate`
- `reviews/task-archive-early-hnsw/050-256-c1-greedy-upper-layer-seeding` -> `reviews/task-archive-cross-cutting/10-256-c1-greedy-upper-layer-seeding`
- `reviews/task-archive-early-hnsw/051-259-c1-executor-vs-am-startup-split` -> `reviews/task-05/213-259-c1-executor-vs-am-startup-split`
- `reviews/task-archive-early-hnsw/052-260-c1-am-startup-boundary-reconciliation` -> `reviews/task-05/214-260-c1-am-startup-boundary-reconciliation`
- `reviews/task-archive-early-hnsw/053-261-c1-warm-cache-verified-surface` -> `reviews/task-05/215-261-c1-warm-cache-verified-surface`
- `reviews/task-archive-early-hnsw/054-264-c1-warm-steady-state-optimization-survey` -> `reviews/task-archive-cross-cutting/11-264-c1-warm-steady-state-optimization-survey`
- `reviews/task-archive-early-hnsw/055-265-c1-disable-unused-query-prep` -> `reviews/task-archive-cross-cutting/12-265-c1-disable-unused-query-prep`
- `reviews/task-archive-early-hnsw/056-267-c1-plain-query-timing-mode` -> `reviews/task-archive-cross-cutting/13-267-c1-plain-query-timing-mode`
- `reviews/task-archive-early-hnsw/057-268-c1-cached-plan-query-timing` -> `reviews/task-archive-cross-cutting/14-268-c1-cached-plan-query-timing`
- `reviews/task-archive-early-hnsw/058-272-c1-co-located-adjacency-batch-read` -> `reviews/task-05/216-272-c1-co-located-adjacency-batch-read`
- `reviews/task-archive-early-hnsw/059-273-c1-negative-server-timing-rejection` -> `reviews/task-archive-cross-cutting/15-273-c1-negative-server-timing-rejection`
- `reviews/task-archive-early-hnsw/060-274-c1-adr029-rank-correlation-study` -> `reviews/task-16/133-274-c1-adr029-rank-correlation-study`
- `reviews/task-archive-early-hnsw/061-275-c1-adr029-source-expansion-survivor-gate` -> `reviews/task-16/134-275-c1-adr029-source-expansion-survivor-gate`
- `reviews/task-archive-early-hnsw/062-276-c1-inline-heaptid-element-decode` -> `reviews/task-archive-cross-cutting/16-276-c1-inline-heaptid-element-decode`
- `reviews/task-archive-early-hnsw/063-277-c1-successor-scratch-buffer-reuse` -> `reviews/task-05/217-277-c1-successor-scratch-buffer-reuse`
- `reviews/task-archive-early-hnsw/064-279-c1-adr031-sign-binary-study` -> `reviews/task-16/135-279-c1-adr031-sign-binary-study`
- `reviews/task-archive-early-hnsw/065-281-c1-adr031-cached-binary-prefilter-runtime` -> `reviews/task-16/136-281-c1-adr031-cached-binary-prefilter-runtime`
- `reviews/task-archive-early-hnsw/066-282-c1-adr031-real-50k-scale-validation` -> `reviews/task-16/137-282-c1-adr031-real-50k-scale-validation`
- `reviews/task-archive-early-hnsw/067-285-c1-adr031-persisted-binary-sidecar-feasibility` -> `reviews/task-16/138-285-c1-adr031-persisted-binary-sidecar-feasibility`
- `reviews/task-archive-early-hnsw/068-286-c1-adr031-sidecar-ab-compare` -> `reviews/task-16/139-286-c1-adr031-sidecar-ab-compare`
- `reviews/task-archive-early-hnsw/069-289-c1-adr031-on-off-ab` -> `reviews/task-16/140-289-c1-adr031-on-off-ab`
- `reviews/task-archive-early-hnsw/070-293-c1-adr032-fused-node-cache` -> `reviews/task-16/141-293-c1-adr032-fused-node-cache`
- `reviews/task-archive-early-hnsw/071-415-c1-standalone-cargo-test-pg-backend-stubs` -> `reviews/task-archive-cross-cutting/009-415-c1-standalone-cargo-test-pg-backend-stubs`
- `reviews/task-archive-early-hnsw/072-446-c1-native-hnsw-build-path` -> `reviews/task-archive-cross-cutting/18-446-c1-native-hnsw-build-path`
- `reviews/task-archive-early-hnsw/073-449-c1-native-build-harness-fix-and-hnsw-rs-removal` -> `reviews/task-10065/002-449-c1-native-build-harness-fix-and-hnsw-rs-removal`
- `reviews/task-archive-early-hnsw/074-450-c1-native-build-stability-tightening` -> `reviews/task-10065/003-450-c1-native-build-stability-tightening`
- `reviews/task-archive-early-hnsw/075-451-c1-native-build-heuristic-tests` -> `reviews/task-10065/004-451-c1-native-build-heuristic-tests`
- `reviews/task-archive-early-hnsw/076-453-c1-native-build-helper-coverage` -> `reviews/task-10065/005-453-c1-native-build-helper-coverage`
- `reviews/task-archive-early-hnsw/077-454-c1-source-gate-fixture-reuse` -> `reviews/task-10065/006-454-c1-source-gate-fixture-reuse`
- `reviews/task-archive-early-hnsw/078-457-c1-native-build-seed-copy-cleanup` -> `reviews/task-10065/007-457-c1-native-build-seed-copy-cleanup`
- `reviews/task-archive-early-hnsw/079-461-c1-ec-am-surface-rename` -> `reviews/task-archive-cross-cutting/19-461-c1-ec-am-surface-rename`
- `reviews/task-archive-early-hnsw/080-463-c1-ecaz-rename-alignment` -> `reviews/task-archive-cross-cutting/010-463-c1-ecaz-rename-alignment`
- `reviews/task-archive-early-hnsw/081-464-c1-task-numbering-alignment` -> `reviews/task-archive-cross-cutting/011-464-c1-task-numbering-alignment`
- `reviews/task-archive-early-hnsw/082-465-c1-native-build-doc-alignment` -> `reviews/task-10065/008-465-c1-native-build-doc-alignment`
- `reviews/task-archive-early-hnsw/083-471-c1-ecaz-cli-script-removal` -> `reviews/task-49/002-471-c1-ecaz-cli-script-removal`
- `reviews/task-archive-early-hnsw/084-472-c1-ecaz-cli-dev-tooling-consolidation` -> `reviews/task-49/003-472-c1-ecaz-cli-dev-tooling-consolidation`
- `reviews/task-archive-early-hnsw/085-623-c1-ecaz-cli-generic-dev-sql` -> `reviews/task-49/004-623-c1-ecaz-cli-generic-dev-sql`
- `reviews/task-archive-early-hnsw/086-633-c1-native-build-level-precompute` -> `reviews/task-10065/009-633-c1-native-build-level-precompute`
- `reviews/task-archive-early-hnsw/087-666-c1-phase3-real50k-summary` -> `reviews/task-05/218-666-c1-phase3-real50k-summary`
- `reviews/task-archive-early-hnsw/088-10048-deleted-entry-point-staleness` -> `reviews/task-archive-cross-cutting/22-10048-deleted-entry-point-staleness`
- `reviews/task-archive-early-hnsw/089-10049-datapage-chain-linear-lookup` -> `reviews/task-05/219-10049-datapage-chain-linear-lookup`
- `reviews/task-archive-early-hnsw/090-10058-portable-manifest-source-parquet` -> `reviews/task-archive-cross-cutting/23-10058-portable-manifest-source-parquet`
- `reviews/task-archive-early-hnsw/091-11014-adr045-page-layout-discipline` -> `reviews/task-17/033-11014-adr045-page-layout-discipline`
- `reviews/task-archive-early-hnsw/092-11016-phase5b-slim-tuple` -> `reviews/task-17/034-11016-phase5b-slim-tuple`
- `reviews/task-archive-early-hnsw/093-11017-phase5c1-persist-sequencer` -> `reviews/task-17/035-11017-phase5c1-persist-sequencer`
- `reviews/task-archive-early-hnsw/094-11018-phase5c2-build-orchestrator` -> `reviews/task-17/036-11018-phase5c2-build-orchestrator`
- `reviews/task-archive-early-hnsw/095-11024-adr046-review-prep` -> `reviews/task-17/037-11024-adr046-review-prep`
- `reviews/task-archive-early-hnsw/096-11025-adr047-review-prep` -> `reviews/task-17/038-11025-adr047-review-prep`
- `reviews/task-archive-early-hnsw/097-11026-visited-state-reuse` -> `reviews/task-17/039-11026-visited-state-reuse`
- `reviews/task-archive-early-hnsw/098-11027-reader-live-tid-iteration` -> `reviews/task-17/040-11027-reader-live-tid-iteration`
- `reviews/task-archive-early-hnsw/099-11030-phase5c3-pgrx-ambuild-wiring` -> `reviews/task-17/041-11030-phase5c3-pgrx-ambuild-wiring`
- `reviews/task-archive-early-hnsw/100-11035-phase7c-exact-duplicate-probe-boundary` -> `reviews/task-17/042-11035-phase7c-exact-duplicate-probe-boundary`
- `reviews/task-archive-early-hnsw/101-11037-phase7e-forward-link-append` -> `reviews/task-17/043-11037-phase7e-forward-link-append`
- `reviews/task-archive-early-hnsw/102-11038-phase7f-free-capacity-backlinks` -> `reviews/task-17/044-11038-phase7f-free-capacity-backlinks`
- `reviews/task-archive-early-hnsw/103-11040-phase7h-full-backlink-rewrite-replan` -> `reviews/task-17/045-11040-phase7h-full-backlink-rewrite-replan`
- `reviews/task-archive-early-hnsw/104-11047-phase9-review-followup-docs` -> `reviews/task-17/046-11047-phase9-review-followup-docs`
- `reviews/task-archive-early-hnsw/105-30154-task-doc-status-cleanup` -> `reviews/task-29/023-30154-task-doc-status-cleanup`
- `reviews/task-archive-early-hnsw/106-30156-spec-current-state-refresh` -> `reviews/task-archive-cross-cutting/012-30156-spec-current-state-refresh`
- `reviews/task-archive-early-hnsw/107-30157-ecaz-cli-docs-spec` -> `reviews/task-49/005-30157-ecaz-cli-docs-spec`
- `reviews/task-archive-early-hnsw/108-30161-m5-ann-optimization-task-split` -> `reviews/task-31/039-30161-m5-ann-optimization-task-split`
- `reviews/task-archive-early-hnsw/109-30655-readme-build-setup` -> `reviews/task-49/006-30655-readme-build-setup`
- `reviews/task-archive-early-hnsw/110-30656-readme-performance-summary` -> `reviews/task-49/007-30656-readme-performance-summary`
- `reviews/task-archive-early-hnsw/111-30657-mit-license` -> `reviews/task-49/008-30657-mit-license`
- `reviews/task-archive-early-hnsw/112-30759-ecaz-cli-repo-root-discovery` -> `reviews/task-49/009-30759-ecaz-cli-repo-root-discovery`
- `reviews/task-archive-early-hnsw/113-30767-standalone-pg-stub-invariant` -> `reviews/task-archive-cross-cutting/25-30767-standalone-pg-stub-invariant`
- `reviews/task-archive-early-hnsw/114-31140-c1-unsafe-quality-burndown-scaffold` -> `reviews/task-35/001-31140-c1-unsafe-quality-burndown-scaffold`
- `reviews/task-archive-early-hnsw/115-31141-c1-unsafe-burndown-one-entry-production` -> `reviews/task-35/002-31141-c1-unsafe-burndown-one-entry-production`
- `reviews/task-archive-early-hnsw/116-31142-c1-am-routine-unsafe-boundaries` -> `reviews/task-35/003-31142-c1-am-routine-unsafe-boundaries`
- `reviews/task-archive-early-hnsw/117-31142-standards-coverage-hardening-spec` -> `reviews/task-49/010-31142-standards-coverage-hardening-spec`
- `reviews/task-archive-early-hnsw/118-31143-standards-compliance-claim-fixes` -> `reviews/task-49/011-31143-standards-compliance-claim-fixes`
- `reviews/task-archive-early-hnsw/119-31144-readme-rust-safety-link` -> `reviews/task-49/012-31144-readme-rust-safety-link`
- `reviews/task-archive-early-hnsw/120-31146-nfr016-adr070-spec-review` -> `reviews/task-49/013-31146-nfr016-adr070-spec-review`
- `reviews/task-archive-early-hnsw/121-round-30303-30336-feedback` -> `reviews/task-archive-cross-cutting/26-round-30303-30336-feedback`

## Manual Bucket Corrections

- `reviews/task-archive-cross-cutting/00-22-pin-hnsw-rs-version` -> `reviews/task-09/002-22-pin-hnsw-rs-version`
- `reviews/task-archive-cross-cutting/02-86-remove-silent-top-up-reseed` -> `reviews/task-05/220-86-remove-silent-top-up-reseed`
- `reviews/task-archive-cross-cutting/10-256-c1-greedy-upper-layer-seeding` -> `reviews/task-05/221-256-c1-greedy-upper-layer-seeding`
- `reviews/task-archive-cross-cutting/11-264-c1-warm-steady-state-optimization-survey` -> `reviews/task-05/222-264-c1-warm-steady-state-optimization-survey`
- `reviews/task-archive-cross-cutting/12-265-c1-disable-unused-query-prep` -> `reviews/task-05/223-265-c1-disable-unused-query-prep`
- `reviews/task-archive-cross-cutting/13-267-c1-plain-query-timing-mode` -> `reviews/task-11/053-267-c1-plain-query-timing-mode`
- `reviews/task-archive-cross-cutting/14-268-c1-cached-plan-query-timing` -> `reviews/task-11/054-268-c1-cached-plan-query-timing`
- `reviews/task-archive-cross-cutting/15-273-c1-negative-server-timing-rejection` -> `reviews/task-11/055-273-c1-negative-server-timing-rejection`
- `reviews/task-archive-cross-cutting/16-276-c1-inline-heaptid-element-decode` -> `reviews/task-05/224-276-c1-inline-heaptid-element-decode`
- `reviews/task-archive-cross-cutting/18-446-c1-native-hnsw-build-path` -> `reviews/task-10065/010-446-c1-native-hnsw-build-path`
- `reviews/task-archive-cross-cutting/19-461-c1-ec-am-surface-rename` -> `reviews/task-17/047-461-c1-ec-am-surface-rename`
- `reviews/task-archive-cross-cutting/22-10048-deleted-entry-point-staleness` -> `reviews/task-06/019-10048-deleted-entry-point-staleness`
- `reviews/task-archive-cross-cutting/23-10058-portable-manifest-source-parquet` -> `reviews/task-12/001-10058-portable-manifest-source-parquet`
- `reviews/task-archive-cross-cutting/25-30767-standalone-pg-stub-invariant` -> `reviews/task-09/003-30767-standalone-pg-stub-invariant`
- `reviews/task-archive-cross-cutting/26-round-30303-30336-feedback` -> `reviews/task-30/956-round-30303-30336-feedback`

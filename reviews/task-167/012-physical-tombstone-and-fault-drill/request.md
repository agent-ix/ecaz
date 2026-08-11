# Task 167 checkpoint: routed physical tombstone and TC-043 fault hook

This checkpoint adds the maintenance and fault-injection pieces needed by the
physical-generation DML path:

- a coordinator-side physical source map keyed by `(index_oid, source_tid)`
  and stable `vec_id`;
- distributed-control `ambulkdelete` enumeration and owner routing;
- an idempotent owner tombstone endpoint and transport;
- physical `debug_fail_insert` injection after graph/owner append and before
  backlink publication;
- replacement of the legacy fold-only multinode mid-insert drill with an
  isolated one-owner physical-generation drill that checks source-row and
  published-record counts after the abort.

Validation:

- `cargo check --no-default-features --features pg18` — passed at
  `57476bc2b`.
- `cargo check -p ecaz` — passed with the existing unrelated
  `LoadedDistributedPlacementConfig.path` dead-code warning.
- focused pgrx command was run, but no trustworthy integration result is
  claimed because the shared artifact-lock behavior suppresses the normal test
  summary in this workspace.

This remains a checkpoint, not closeout. Initial-generation source-map
population, concurrent physical insert/query drill, TC-043 execution,
FR-083-AC-4 parity, throughput A/B, and 10k/50k/100k suite evidence remain.

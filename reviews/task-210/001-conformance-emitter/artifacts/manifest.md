# Task 210 P0 — coordinator-resident storage emitter manifest

Measurement-only packet. It changes no index behaviour; it makes the
coordinator's own resident index state visible to the NFR-021 conformance gate,
which previously could not see it.

## Provenance

- Task bucket/packet: `reviews/task-210/001-conformance-emitter/`
- Runner (CLI) head: `f71fbcc90` — `feat(bench): itemise coordinator-resident
  index state for NFR-021`
- Extension under measurement: `0057a35c0461a8947612aab6b56d089eb67fa051`,
  PG18 release build with `distann-head-attribution-benchmark`. The extension
  was deliberately **not** rebuilt for this run: P0 changes only the CLI
  emitter, so the measured extension is the same release the Task 205 evidence
  used, and the P1 default-path change (`e5047081a`) is **not** in it.
- Suite config: `artifacts/task210-p0-coordinator-storage.json`
  SHA-256 `87ffc74d2eeaa28c186266fa66df252648e14e59c441a7179d0e7332c4d8ac49`
- Structured results: `artifacts/run/results.jsonl` (284 rows)
  SHA-256 `39a0d0500ee957e5097544bcbbfdf7b720df5a53c7000a0e6a877bac6c7db3de`
- Suite manifest: `artifacts/run/suite-manifest.json`; run log
  `artifacts/suite-run.log`
- Fixture: 3 owner nodes, ports 42210–42232, shipped-default arm
  (`traversal_replica=false`), BW=4, H=100, L=32, degree 32, head cap 4096,
  k=10, 200 queries, 50 iterations, 10 warmups.
- Run directories were `~/.ecaz/clusters/task210-p0-{10k,50k,100k}` — outside
  the repository and outside `CARGO_TARGET_DIR` — and are removed after this
  capture. Clusters are not evidence; the cited rows are.

## Inputs

- Staged corpora: `/home/peter/dev/ecaz/data/staged-current`
- Prefixes `ec_real_10k`, `ec_real_50k`, `ec_real_100k`; corpus and query
  SHA-256 values are recorded on every row in `results.jsonl`.

## What the run shows

Before this change the coordinator's `physical_benchmark_storage_node` row was
emitted with `graph_bytes=0 directory_bytes=0 control_bytes=0` and carried only
the optional traversal replica, so the head — the one unsharded structure in
the system — was invisible to the gate that exists to catch unsharded state.

Coordinator row, all three scales (`artifacts/run/*/distann-multinode-summary.log`):

| scale | head_sample_bytes | head_graph_bytes | coordinator_resident_unsharded_bytes | total_resident_bytes |
|---|---:|---:|---:|---:|
| 10k | 25,280,512 | 514,100 | 25,794,612 | 25,794,612 |
| 50k | 25,280,512 | 533,721 | 25,814,233 | 25,814,233 |
| 100k | 25,280,512 | 614,095 | 25,894,607 | 25,894,607 |

The head sample is constant in N — 4,096 full-precision f32 landmarks — which
is exactly why the removed NFR-021 constant-`C` exemption was load-bearing in
the wrong direction: the structure was small, so the requirement blessed it.

Derived conformance row (`physical_benchmark_nfr_021_conformance`, id
`task210-p0-owner`):

```
actual_admissibility=conforming
evidence_complete=true            scales=10k,50k,100k
max_non_owned_records=0           max_orphan_vectors=0
max_unsharded_derived_bytes=0     normalized_bytes_per_owned_record_growth_max=1.094675
coordinator_resident_unsharded_bytes=25894607
outstanding_distribution_gap=ec_distann_generation_head_graph:614095:task-210-P2,
                             ec_distann_generation_head_sample:25280512:task-210-P2
```

The owner arm's own state is conforming — its shards are disjoint and its
normalized growth is 1.09 across a 10× corpus — so lanes that did not introduce
the head gap are not halted by it. The gap itself is reported by relation name,
byte count, and owning phase on every conformance row, and disappears when Task
210 P2 shards the head. A coordinator-resident relation that is **not** on the
known-gap list is a hard violation rather than a reported gap
(`distann_nfr_021_fails_on_a_coordinator_resident_relation_that_is_not_a_known_gap`).

## Validation

- `cargo test -p ecaz-cli commands::bench::suite::tests::distann_` — 29 passed,
  0 failed (24 before this task's slices).
- `cargo clippy -p ecaz-cli --no-deps` — exit 0; only pre-existing warnings.
- Focused new coverage: known-gap reporting, unknown-relation hard violation,
  and gap-clears-when-sharded.

## Re-run

```text
ecaz bench suite run \
  --config reviews/task-210/001-conformance-emitter/artifacts/task210-p0-coordinator-storage.json \
  --artifact-dir reviews/task-210/001-conformance-emitter/artifacts/run \
  --results-output reviews/task-210/001-conformance-emitter/artifacts/run/results.jsonl \
  --manifest-output reviews/task-210/001-conformance-emitter/artifacts/run/suite-manifest.json \
  --log-file reviews/task-210/001-conformance-emitter/artifacts/suite-run.log
```

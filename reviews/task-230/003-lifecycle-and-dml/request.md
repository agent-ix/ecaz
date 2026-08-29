---
task: 230
packet: 003-lifecycle-and-dml
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 10
---

# Task 230 packet 003 — hot/cold runtime fault and lifecycle evidence

Review the suite-driven runtime evidence at `ef0134501`. There is no code change
after the accepted seq-09 checkpoint `deb245711`; `ef0134501` preregistered the
checked-in suite config before any runtime result was read.

## Seq-10 runtime evidence

- The four-owner hot/cold read matrix passed all 25 cells: five production RPCs
  crossed with remote statement timeout, local query cancellation, local
  statement timeout, remote-backend termination, and connection reset/owner
  restart. Every cell drained remote work and completed a clean retry.
- The three-owner hot/cold write/lifecycle matrix passed 23 scenarios and
  emitted 110 records. Its remote mutation, prepare, commit/rollback,
  acknowledgment-loss, partial-commit, missing-intent, owner-death, prepared-
  slot, publication, retirement, reclaim, and operator-stop cases all passed.
- All 12 layout-aware write snapshots reported `hot_cold=true` and
  `cold_pair_balanced=true`; hot and cold tuple totals matched after each
  recovery disposition.
- Each of the three owners successfully materialized a retained `Retired`
  predecessor with attnums 1 and 3, `tuple_payload_missing=false`, two valid
  cumulative offsets, and nonempty hot-plus-cold payload bytes before reclaim.
- Both fixtures unanimously admitted the release extension built at
  `ef0134501` with `pg18,pg-test,distann-head-attribution-benchmark`. The suite
  removed both external cluster directories after durable artifact capture.
- Packet 003's remote retry/intent, restart/owner-failure,
  publication/recovery, and retained-generation runtime obligations are all
  represented. Request review-close of Packet 003.

## Seq-10 validation and provenance

- Checked-in `ecaz bench suite` config:
  `artifacts/task230-packet003-runtime-suite.json`.
- Dry-run: exit 0; exactly the preregistered read and write steps expanded.
- Real run: both suite steps `succeeded`, exit 0; `suite-results.jsonl` contains
  71 result rows (55 drill outcomes, 14 topology rows, two release preflights).
- No production code changed after seq-09, so the seq-09 format/root-clippy/
  CLI-clippy/test receipts remain the code gates for this runtime checkpoint.

## Seq-09 accepted scope

Reviewer seq-09 closed both static carry-ins as DONE. The accepted scope below
remains for packet history.

## Seq-09 payload-offset contract and CLI lint gate

- Resolves seq-08's payload-offset carry-in against the production encoder and
  packet-002 pgrx tests: `ec_distann_materialize_physical_row_payloads` returns
  one cumulative end offset per requested attribute, not a terminal N+1
  offset. The retained-generation harness now requires exactly two offsets for
  `ARRAY[1,3]`.
- Adds the reviewer-requested `cargo clippy -p ecaz-cli --all-targets` receipt.
  It exits 0 with the existing warning baseline and has no warning at the
  seq-09 change. The strict root PG18 gate still reproduces only its five known
  findings.

## Seq-09 validation

- `cargo fmt --all -- --check`: exit 0.
- Six focused `ecaz-cli` Task 230 tests pass.
- Root PG18 clippy: exit 101 only for the five pre-existing findings.
- CLI all-target clippy: exit 0 with the pre-existing warning baseline; no
  Task 230 seq-09 finding.

## Seq-08 accepted scope

Reviewer seq-08 closed the lifecycle and fault-matrix harness as DONE. The
accepted scope below remains for packet history.

## Seq-08 lifecycle and fault-matrix harness

- Adds typed `read_rpc_fault_matrix` and `write_lifecycle_fault_matrix` fields
  to `ecaz bench suite`, including the CLI-equivalent node/debug/mutual-
  exclusion validation, command expansion, and packet-local expected logs.
- Extends the Task 235 lifecycle status reader to count `cold_tier_relid`.
  Hot/cold Ready/Published/Retired partial and final states therefore require
  four live generation relations, while row-heap remains three.
- Makes every write-fault snapshot layout-aware: hot identity is read from
  compact `a_2`, presence of the cold tier is mandatory for this arm, and
  total hot/cold tuple counts must remain equal before, during, and after 2PC
  recovery. Result lines emit the totals and `cold_pair_balanced=true`.
- Adds a retained-predecessor production materialization on every owner after
  successor publication and predecessor retirement but before reclaim. It
  requests attnums 1 and 3 together, forcing hot plus cold reconstruction, and
  asserts one complete row, one cumulative end offset per requested attribute
  (two offsets for attnums 1 and 3), nonempty payload bytes, `Retired`
  admission, and the hot/cold catalog shape.
- Leaves the Task 234 read-RPC matrix itself unchanged; running it on the
  hot/cold fixture exercises all five production RPCs across cancellation,
  remote-backend death, owner connection reset/restart, and clean retry.

## Seq-08 validation

- `cargo check -p ecaz-cli`: exit 0; only the pre-existing dead-code warning.
- `cargo fmt --all -- --check`: exit 0.
- Six focused ecaz-cli tests pass, including typed fault-matrix expansion.
- Mandatory all-target PG18 clippy reproduces only the same five pre-existing
  findings; no finding is in the seq-08 changes.

## Seq-07 accepted scope

Reviewer seq-07 closed the projection-mirror fix as DONE. The accepted scope
below remains for packet history.

## Seq-07 projection-mirror fix

- States and emits one rule for physical attribution:
  `physical_projection_rule=mirrors_end_to_end_projection`. Traversal I/O is
  already measured by the primary id-only end-to-end arm; the direct-relation
  attribution query measures only the selected result projection.
- Applies that rule to all six shapes in both layouts: id-only/hot-scalar read
  `id` / `a_1`; exact-vector reads `embedding` / `a_4`; cold-only reads only
  `payload_note` / cold `a_5`; mixed reads `id, payload_note` / hot `a_1` plus
  cold `a_5`; select-all reads the full row / both tiers.
- Cold-only now issues no hot-relation query at all. The row-heap control also
  drops `embedding` from cold-only, so the A/B measures the §6 cold-scalar
  prediction rather than a 6 KiB vector plus payload.
- Adds a table-driven unit test that pins both the row-heap and hot/cold
  mappings for all six shapes, preventing the mixed projection rule from
  recurring.

## Seq-07 validation

- `cargo fmt --all -- --check`: exit 0.
- Five focused ecaz-cli tests pass, including exact all-six projection mapping.
- Mandatory all-target PG18 clippy reproduces only the same five pre-existing
  findings; no finding is in the seq-07 changes.

## Seq-06 reviewed scope

Reviewer seq-06 accepted both new shapes but returned NOT DONE because the
pre-existing cold-only physical mapping also read the hot exact vector. The
seq-07 scope above addresses that sole blocking finding.

## Seq-06 complete six-shape scope

- Adds typed hot-scalar and exact-vector shapes to both the multinode CLI and
  `ecaz bench suite`, resolving the four-measured-versus-six-predicted gap
  before any packet-004 threshold is frozen or result is read.
- The hot-scalar arm projects the preregistered additional hot scalar (`id`,
  attnum 1) directly from `a_1`; it remains separately labelled from the
  primary id-only arm so both frozen predictions are independently reported.
- The exact-vector arm projects `embedding` / `a_4` directly, measuring the
  materialization side of the PLAIN-inline mechanism rather than only the
  traversal-side evidence carried by id-only.
- Hot-scalar and exact-vector are hot-only and therefore do not require the
  external TOAST fixture. Cold-only, mixed, and select-all retain the strict
  fixture requirement.

## Seq-06 validation

- `cargo fmt --all -- --check`: exit 0.
- Four focused ecaz-cli tests pass, including typed hot-scalar and exact-vector
  expansion without the TOAST fixture.
- Mandatory all-target PG18 clippy reproduces only the same five pre-existing
  findings; no finding is in the seq-06 changes.

## Seq-05 accepted scope

Reviewer seq-05 closed row-tier I/O attribution as DONE. The accepted scope
below remains for packet history.

## Seq-05 row-tier I/O attribution scope

- Adds typed `task230_io_query_shape` values for id-only, cold-only, mixed, and
  select-all arms plus an explicit iteration count to both the suite schema and
  multinode CLI.
- Keeps one query shape per fresh fixture and rejects reuse, so cumulative I/O
  and cache state cannot bleed between shapes.
- Runs the selected end-to-end ANN projection for elapsed/materialized output,
  then runs the matching physical hot/row/cold projection independently on
  every owner.
- Takes each owner's pre/post `pg_statio_all_tables` snapshots and explicit
  `pg_stat_force_next_flush()` in the same backend session that performed the
  attributed relation reads.
- Emits heap, TOAST heap, and TOAST index read/hit deltas per relation, plus an
  aggregate shared-buffer hit ratio; counter resets and relation-identity drift
  fail closed.
- Adds a Task 230-only external, uncompressed TOAST fixture so cold/mixed/all
  shapes do not inherit the unrelated materialization-correctness variant
  matrix.

## Seq-05 validation

- `cargo fmt --all -- --check`: exit 0.
- Four focused ecaz-cli tests pass, including exact six-counter subtraction and
  fail-closed counter reset.
- Mandatory all-target PG18 clippy reproduces only the same five pre-existing
  findings; no finding is in the seq-05 changes.

## Seq-04 accepted scope

Reviewer seq-04 closed the multinode/suite selection and topology harness as
DONE. The accepted scope below remains for packet history.

## Seq-04 multinode and suite harness scope

- Adds explicit `--hot-cold-row-tier` and canonical
  `--hot-payload-attnums` options to `ecaz dev distann-multicluster`, including
  the frozen 1..=1536 dimension bound and fail-closed Task 229 sidecar
  exclusion.
- Extends the typed `ecaz bench suite` step schema and command expansion with
  the same options; no packet-local sweeper or raw-argument escape hatch is
  needed for the full-scale matrix.
- Reads all three cold-tier topology columns, rejects missing/incomplete pairs,
  and includes cold heap bytes in per-owner and aggregate generation storage.
- Attests hot/cold reloptions when reusing a fixture so a row-heap control
  cannot be silently reused as the candidate or vice versa.
- Closes reviewer seq-03's topology interpretation note: receipt digests are
  initial-content signals only before post-Ready DML; afterward graph
  current/tombstone state plus successful vec-id/schema-checked locator
  reconstruction is authoritative.

## Seq-04 validation

- `cargo fmt --all -- --check`: exit 0.
- Three focused ecaz-cli tests pass for canonical hot attnums, complete-pair
  topology validation, and typed suite expansion.
- Mandatory all-target PG18 clippy reproduces only the same five pre-existing
  findings; no finding is in the seq-04 changes.

## Seq-03 accepted scope

Reviewer seq-03 closed retained history and local destructive lifecycle as
DONE. The accepted scope below remains for packet history.

## Seq-03 retained-history and destructive-lifecycle scope

- Documents that the topology orphan columns are raw physical-history counts:
  valid predecessor tuples retained for snapshot-pinned readers are included
  and are storage/churn attribution, not corruption by themselves.
- Extends the hot/cold DML callback to prove a healthy same-identity replacement
  reports exactly one retained hot tuple and one retained cold tuple.
- Proves `DROP INDEX` removes hot, cold, graph, and directory relations plus the
  generation catalog row.
- Proves `REINDEX` removes all four old generation relations and catalog state
  and mints a fresh logical index UUID.
- Proves an aborted `REINDEX` transaction restores all four relations, the cold
  catalog binding, and the original logical index UUID.

## Seq-03 validation

- `cargo fmt --all -- --check`: exit 0.
- Focused PG18 lifecycle group: two passed, zero failed.
- Mandatory all-target PG18 clippy: only the same five pre-existing findings;
  no finding in the seq-03 production documentation or lifecycle callback.

## Seq-02 accepted scope

Reviewer seq-02 closed the topology checkpoint as DONE. The accepted scope
below remains for packet history.

## Seq-02 topology scope

- Replaces the production topology diagnostic's hard-coded Graph V1 decode
  with descriptor-version dispatch, admitting hot/cold Graph V2 records.
- Opens and schema-validates the cold relation under the same inspection lock
  set as hot/graph/directory, reconstructs the frozen logical row from both
  compact tuples, and recomputes the unchanged logical row-tier digest.
- Preserves every existing topology column and appends explicit optional
  `cold_tier_row_count`, `cold_tier_orphan_row_count`, and `cold_tier_bytes`
  columns to both build-id and fingerprint endpoints. Legacy/sidecar
  generations report NULL for all three.
- Adds a published PG18 hot/cold topology callback that proves V2 admission,
  one hot plus one cold row, zero orphans in both tiers, byte accounting for
  both heaps, and logical digest equality with the frozen manifest.

## Seq-02 validation

- `cargo fmt --all -- --check`: exit 0.
- Focused PG18 topology callback: one passed, zero failed.
- Mandatory all-target PG18 clippy: only the same five pre-existing findings;
  no finding in `handoff.rs` or the new topology test.

## Seq-01 accepted scope

Reviewer seq-01 closed the DML/reclaim checkpoint as DONE. The accepted scope
below remains for packet history.

### DML and reclaim implementation

- Physical inserts now map logical source attributes to the compact tier
  ordinals frozen in descriptor V4, append cold then hot, and publish a Graph
  V2 record only after both authoritative tuples exist in the same owner
  transaction.
- Forwarded owner payloads continue to use the full logical source schema on
  the wire. The owner decodes them into a source-heap slot, validates that
  schema against the retained descriptor, and only then partitions the row for
  storage. This keeps compact relation schemas out of the handoff contract.
- Candidate vector and stable-identity reads use the compact hot physical
  ordinals. Backlink read/modify/write, replacement, and tombstone paths
  dispatch through the retained graph-record version, preserving the V2 cold
  locator.
- Same-identity replacement appends a new cold/hot pair and graph version while
  retaining predecessor tuples. Delete changes only the current graph
  tombstone and leaves both locators and both tier heaps untouched.
- Adds PG18 coverage for exact hot/cold locator linkage, insert, replacement,
  graph-only tombstone, injected-failure rollback, retirement, reclaim
  rollback, idempotent reclaim, and dropping the cold tier with the generation.

### Seq-01 validation

- `cargo fmt --all -- --check`: exit 0.
- `cargo pgrx test pg18 test_distann_hot_cold_ --no-default-features --features
  'pg18 pg_test'`: six focused PG18 callbacks pass, including both new tests
  and the four prior format/read-path hot/cold tests.
- The mandatory all-target PG18 clippy gate reports only the same five
  pre-existing failures recorded throughout packet 002; it reports no new
  production-DML or new-test lint.

## Packet status

This requests approval of the runtime harness but is not packet-003 closure.
Still owed after approval are the actual remote retry/intent and fault run,
publication/recovery and retained-generation reads, and restart/owner-failure
reads carried from packet 002.

## Review request

Please verify the suite typing/validation, four-relation lifecycle accounting,
hot/cold DML parity checks, and retained predecessor materialization. Leave
feedback under this packet's `feedback/` directory.

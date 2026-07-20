# Task 179 packet 002 artifacts manifest

- **Head SHA:** `57a45cca1afba081563b33f253dc9d89a4826f08`
- **Task bucket / packet:** `reviews/task-179/002-format-and-control`
- **Branch:** `task-179-ec-distann-physical-shards`
- **Timestamp:** `2026-07-10T14:41:15-07:00`
- **Lane:** Task 179 format/control implementation, NFR-016 format discipline,
  and focused live PG18 control-index behavior
- **Fixture:** canonical v4/v5 metadata plus fourteen DistANN physical/wire
  golden fixtures and the repository upgrade matrix
- **Storage format:** metadata v4 compatibility, distributed control metadata
  v5, physical graph record v1, generation/build/handoff/source/receipt formats
  v1, epoch manifest v2, and fingerprint v1
- **Rerank mode / corpus:** not applicable; no benchmark corpus or performance
  measurement is used
- **Isolated one-index-per-table or shared-table surface:** live PG18 tests use
  isolated tables/indexes; format tests decode immutable fixture files

## Commands

```text
cargo test --lib --no-default-features --features pg18 distann
cargo test --no-default-features --features pg18 --test on_disk_fixtures --test size_of_assertions --test upgrade_matrix
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
cargo pgrx test pg18 test_distann_control
```

Every command was captured with `script -q -e -c` so the artifact includes the
actual command output and `COMMAND_EXIT_CODE`.

## Artifacts

- `distann-unit-tests.log` — filtered DistANN library/unit/pg-test coverage.
- `format-integration-tests.log` — independent fixture decoders, layout
  assertions, and upgrade-matrix checks.
- `clippy-pg18.log` — strict all-target PG18 clippy.
- `pgrx-control-pg18.log` — focused live PG18 distributed-control behavior.

Packet 001 separately owns the refreshed specification artifacts:

- `reviews/task-179/001-physical-hash-shard-spec-and-plan/artifacts/quire-validation.log`
- `reviews/task-179/001-physical-hash-shard-spec-and-plan/artifacts/traceability-audit.log`

## Key result lines cited by `request.md`

- DistANN unit result: `139 passed; 0 failed; 1 ignored`.
- On-disk fixtures: `65 passed; 0 failed`.
- Layout assertions: `13 passed; 0 failed`.
- Upgrade matrix: `2 passed; 0 failed`.
- Clippy: `Finished dev profile` and command exit code 0.
- Focused live PG18 control tests: `3 passed; 0 failed`.
- Every captured command: `COMMAND_EXIT_CODE="0"`.

## Provenance notes

- The implementation and finding-correction checkpoints were committed before
  these artifacts were captured; every log therefore exercises head
  `57a45cca1afba081563b33f253dc9d89a4826f08`.
- The canonical fixture bytes at `57a45cca1afba081563b33f253dc9d89a4826f08`
  are this packet's freeze point. The earlier `04c1ce70b` snapshot/build-spec/
  handoff-batch bytes were superseded by the reviewed u32→u64 FullTransactionId
  correction and are not an accepted format baseline. A later finding-response
  checkpoint further hardens UUID validity and owns its refreshed fixture logs.
- The ignored unit test is an opt-in golden-fixture emitter, not skipped
  correctness coverage.
- `distann-unit-tests.log` includes an inner `pg18 pg_test` build because the
  pgrx test harness self-invokes while discovering SQL entities; it is not a
  second command or an undocumented feature matrix. For the three logs whose
  recorder did not echo argv, the exact commands are the authoritative command
  block above and each `COMMAND_EXIT_CODE` is captured in the log.
- This is a correctness/format review packet, not a benchmark packet. It has no
  suite config or `results.jsonl`, and it claims no latency, recall, load, or
  storage measurement.
- Physical generation storage and multi-node topology do not exist at this
  checkpoint, so no physical/disjoint topology evidence is claimed.

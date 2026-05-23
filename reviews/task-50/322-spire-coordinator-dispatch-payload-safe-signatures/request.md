# Task 50 Review Request: SPIRE Coordinator Dispatch Payload Safe Signatures

## Summary

This slice converts SPIRE coordinator insert dispatch planning and the remote DML payload helpers from raw-relation helpers to `SpireLiveIndexRelation` helpers.

Code commit: `eb530bd890626c8a587df23c3b1d3b7b4742d577`

## What Changed

- Converted `coordinator_insert_dispatch_plan_row` from `unsafe fn(pg_sys::Relation, ...)` to a safe helper accepting `SpireLiveIndexRelation`.
- Converted remote insert/update/delete/select payload helpers in `write_payload.rs` to accept and propagate `SpireLiveIndexRelation`.
- Updated SQL wrappers in `src/lib.rs` to use `with_spire_live_index_relation!` for these coordinator DML paths.
- Removed the now-unused `with_live_index_relation_safe!` macro.
- Updated internal tests that call the AM helpers directly to construct `SpireLiveIndexRelation` under their validated relation guards.

## Completion Audit Note

This packet does not close Task 50. The current audit still finds `1953` unsafe line hits under `src/`, so packet 030 Wave 5 closeout is not satisfied.

## Review Focus

- Please verify the coordinator dispatch and payload helpers no longer accept raw `pg_sys::Relation`.
- Please check that SQL entry points keep the relation guard live while constructing and passing `SpireLiveIndexRelation`.
- Please check the direct internal test calls keep their typed relation scoped under the opened guard.

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- No-match audit for removed coordinator dispatch unsafe signature, raw dispatch call sites, `with_live_index_relation_safe!`, and the old raw dispatch SQL wrapper.
- `make UNSAFE_LEDGER=reviews/task-50/322-spire-coordinator-dispatch-payload-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/322-spire-coordinator-dispatch-payload-safe-signatures unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/322-spire-coordinator-dispatch-payload-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

## Counts

- Unsafe line count: `1953` (down from packet 321 `1958`)
- Unsafe ledger rows: `1364`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/no-unsafe-coordinator-dispatch-payload-signatures.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`

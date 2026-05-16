---
topic: spire-prepared-xact-gid-runbook
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
stage: phase-12.4
status: open
---

# Review Request: SPIRE Prepared Xact GID Runbook

## Scope

Phase 12.4 2PC recovery checkpoint for code commit `44016967`
(`Stabilize SPIRE prepared transaction GIDs`). Reviewer feedback commit
`99e41159` landed on top while artifacts were being recorded; it adds feedback
files only and does not change this code slice.

This packet addresses the 30830 P2 prepared-GID recovery finding and the Phase
12.4 runbook row:

- removes the volatile coordinator backend pid from SPIRE prepared transaction
  GIDs;
- keeps the historical `ec_spire_insert_` prefix for compatibility with the
  current INSERT and DELETE prepare paths;
- makes the stable identity
  `ec_spire_insert_<index_oid>_<node_id>_<served_epoch>_<top_xid>`;
- adds focused PG test assertions that prepared GIDs have the stable four-field
  identity and no pid segment;
- fixes the remote INSERT prepare fixture to create the remote SPIRE index that
  its descriptor already names;
- documents the recovery flow in ADR-069 and `docs/SPIRE_DIAGNOSTICS.md`;
- marks the Phase 12.4 GID and prepared-transaction runbook items complete.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/lib.rs`
- `spec/adr/ADR-069-spire-distributed-write-path-scope.md`
- `docs/SPIRE_DIAGNOSTICS.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`

## Validation

- `git diff --check 95f7487e..44016967`
  - artifact: `artifacts/git-diff-check.log`
- `cargo fmt --check`
  - artifact: `artifacts/cargo-fmt-check.log`
- `cargo test insert_remote_prepare --lib --no-default-features --features pg18`
  - artifact: `artifacts/cargo-test-insert-remote-prepare-lib.log`
- `cargo pgrx test pg18 test_ec_spire_prepare_coordinator_insert_tuple_payload_sql`
  - artifact: `artifacts/cargo-pgrx-test-coordinator-insert-tuple-payload.log`
- `cargo pgrx test pg18 test_ec_spire_prepare_coordinator_delete_tuple_payload_sql`
  - artifact: `artifacts/cargo-pgrx-test-coordinator-delete-tuple-payload.log`

## Review Focus

- Confirm dropping backend pid from the GID is sufficient for the 30830 P2
  recovery concern without widening the current compatibility surface.
- Confirm the runbook decision rules are conservative enough, especially the
  instruction to avoid remote-only bulk resolution when the affected key or
  coordinator outcome is unknown.
- Confirm the fixture change is appropriate: the existing test registered a
  remote index name, and this packet now creates that remote index explicitly.

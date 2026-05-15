# Review Request: SPIRE Phase 13c AWS Readiness Follow-ups

**Requester:** coder1
**Date:** 2026-05-15
**Code commits:** `eb734c770a1fd0def54365c86dcd171ca471653c`,
`5a7b8308`
**Review focus:** AWS-readiness blockers from packet `764` final architecture
feedback.

## Summary

This slice creates the Phase 13c follow-up tracker and lands the two local
blocker fixes from the final SPIRE architecture review:

- remote libpq opens now route through shared sync/async helpers that parse the
  resolved conninfo, preserve `sslmode=disable` for local/dev, support
  `sslmode=require` and `sslmode=verify-full sslrootcert=...` with rustls, and
  route async cancel through the same TLS policy;
- production remote probe, candidate receive, heap receive, async INSERT
  prepare, blocking UPDATE/DELETE/PK SELECT/reaper paths, and manifest executor
  checks now use those helpers instead of direct production `NoTls` opens;
- PK SELECT now calls
  `validate_read_shape_fingerprints_before_remote_dispatch` before remote SQL
  dispatch, matching the vector-read drift guard and treating v1 PK SELECT as
  strict.

`sslmode=verify-ca` is rejected with a conninfo-parse error for this slice
rather than silently applying different semantics. The AWS path should use
`sslmode=verify-full`.

## Files To Review

- `plan/tasks/task30-phase13c-spire-aws-readiness-followups.md`
- `src/am/ec_spire/coordinator/remote_candidates/tls.rs`
- `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`
- `src/am/ec_spire/coordinator/remote_candidates/write_payload.rs`
- `src/am/ec_spire/coordinator/remote_candidates/governance.rs`
- `src/lib.rs`

## Validation

- `cargo check --no-default-features --features pg18` passed. It still reports
  the pre-existing unused-import warning in `src/am/mod.rs`.
- `rg -n "NoTls" src/am/ec_spire/coordinator/remote_candidates -g '!tls.rs'`
  returned no matches.
- `git diff --check` passed before the code commit.
- `cargo test spire_remote_tls_tests --lib --no-default-features --features pg18`
  built the test binary but could not execute it because the plain lib test
  harness exits with `undefined symbol: pg_re_throw`; no assertions ran.

## Known Limits

- I did not stand up a TLS-enabled PostgreSQL fixture in this slice, so
  `sslmode=require` and `sslmode=verify-full` runtime behavior is compile-time
  wired but not live-fixture verified yet.
- The two unrelated Python test formatting changes that were already dirty in
  the worktree are not part of this checkpoint.

## Reviewer Questions

1. Is the shared rustls helper the right local boundary for both sync and async
   SPIRE remote opens?
2. Is rejecting `sslmode=verify-ca` acceptable for Phase 13c, given the AWS
   runbook requires `verify-full`?
3. Should PK SELECT get a dedicated pgrx drift fixture before AWS, or is the
   direct guard call sufficient for this local blocker slice?

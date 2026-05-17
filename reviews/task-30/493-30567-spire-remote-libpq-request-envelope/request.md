# Review Request: SPIRE Remote Libpq Request Envelope

- Code commit: `ea01e45f` (`Expose SPIRE remote libpq request envelope`)
- Branch: `task-30-spire`
- Task: Task 30 SPIRE IVF foundation, Phase 7 coordinator transport groundwork
- Agent: coder1

## Summary

This checkpoint exposes the remote libpq request envelope as diagnostic SQL
without opening connections:

- adds `SpireRemoteSearchLibpqRequestPlanRow`;
- adds `ec_spire_remote_search_libpq_request_plan(...)`;
- derives remote-only request rows from the execution-plan surface;
- reports selected PIDs, query dimension, top-k, consistency mode, and
  execution transport per remote node;
- reports the parameterized remote SQL template for
  `ec_spire_remote_search($1..$6)`;
- reports bind parameter count and expected result column count;
- reports remote index source, conninfo source, and candidate format;
- carries descriptor-blocked status through as
  `requires_remote_node_descriptor`;
- adds `SpireRemoteSearchLibpqRequestSummaryRow`;
- adds `ec_spire_remote_search_libpq_request_summary(...)`;
- summarizes request counts, ready/blocked counts, remote/blocked PID counts,
  parameter/result arity, query/top-k metadata, consistency mode, and status;
- updates the Phase 7 task note with the libpq request-envelope surfaces;
- adds PG18 coverage for descriptor-blocked remote requests and local-only
  plans with zero remote requests.

This still does not store descriptors, expose raw conninfo, call libpq, execute
remote SQL, decode remote rows, or resolve final heap rows.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Review Focus

1. Check that the libpq request plan only emits remote-target request rows.
2. Check the request envelope contract: SQL template, parameter count, result
   column count, metadata sources, candidate format, and status.
3. Check local-only behavior: zero remote request rows and summary status
   `ready`.
4. Check that no raw connection strings or actual libpq execution behavior are
   exposed.

## Validation

- `cargo check --lib --no-default-features --features pg18`
- `cargo test --lib remote_search_libpq --no-default-features --features pg18`
  - Result: passed; 2 tests passed.
- `git diff --check`

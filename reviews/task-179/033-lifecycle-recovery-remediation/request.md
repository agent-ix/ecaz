# Review request: lifecycle recovery remediation

Please review code commit `822949d6d16135d784c57ac57d010f38474f8f88`
against the cross-cutting findings in Task 179 packets 006, 019, 020, 025,
029, and 031.

## What changed

- Every mutating handoff/build/publication/retirement/abandonment endpoint now
  rejects non-READ-COMMITTED transactions before input work, locks, catalog
  access, or RPC with `EC_TRANSACTION_ISOLATION`.
- Resolved participant conninfo is removed from parse/connect diagnostics.
- Exact scan tokens are tracked by transaction/subtransaction and released by
  abort callbacks when PostgreSQL ERROR/cancellation skips Rust `Drop`; normal
  executor shutdown still releases eagerly. Failed eager release and an
  unconfirmed fence unlock now warn rather than becoming silent.
- `ec_distann_cancel_epoch_publish` adds an audited, commit-boundary-checked
  `Pending -> Cancelled` transition. It verifies the exact predecessor tuple is
  still active (or both are absent for the first epoch), records
  `session_user`/reason/time, terminalizes the registration, clears the build
  gate, retains the durable decision/fingerprint, and makes decide/recovery fail
  closed thereafter.
- Publish/registration schema constraints and FR-082/Task-179 text now include
  the terminal Cancelled state. Both SECURITY DEFINER event-trigger paths pin
  `pg_temp` last.
- Retire recovery now rejects same-transaction decision consumption using the
  decision row's `xmin`, matching publish recovery.
- Physical implementation endpoints and both SQL wrappers are SECURITY DEFINER
  with pinned search paths and no PUBLIC execute privilege, including the
  legacy caller-function materialization endpoint.
- Physical scan open retries exactly once on the resolve/pin/revalidate
  `EC_EPOCH_MISMATCH` race.
- The real multicluster remote-owner proof now requires an
  `EcDistannDistributedScan` EXPLAIN plan and compares the returned nearest
  source identity, rather than accepting a locally satisfiable EXISTS.
- The physical benchmark's `physical_ms` timer now includes `build_epoch`
  rather than stopping after begin-build.

## Validation

See `artifacts/manifest.md` and its packet-local logs.

- Focused PG18 pgrx lifecycle test: pass, 1/1.
- PG18 clippy with `-D warnings`: pass.
- `ecaz-cli` check: pass (one pre-existing dead-field warning).

## Review questions

1. Does transaction/subtransaction token ownership close packet-020 P1-1 for
   pooled backends after ERROR/cancel?
2. Is the isolation guard early and broad enough to close packet-025 P1-1?
3. Does audited Pending cancellation close the logical-index availability
   wedge while preserving the durable fingerprint/ordinary-abort safety rule?
4. Do the privilege changes close the raw implementation-function bypass?

## Explicitly still open

This is not Task 179 closeout. In particular:

- a separate explicit recovery path must reclaim reachable Ready or
  Published-but-never-active participant generations named by a Cancelled
  decision while preserving an audit/tombstone; cancellation currently
  unwedges future builds but deliberately leaves those durable generations;
- the changed real-multicluster plan assertion needs a fixture rerun;
- remote RPC timeout/interrupt behavior, physical hot-path caching/cost
  decomposition, persisted-head sensitivity/cap evidence, and final accepted
  performance evidence remain open.

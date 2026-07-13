# Review request: preserve transport state across cancellation

## Scope

Please review implementation commit `f5720977e` as the remediation for packet
036 P1-1.

The remote transport previously called PostgreSQL's interrupt checker from
inside `runtime.block_on`, while `DISTANN_TRANSPORT_STATE` was held through a
`RefCell::borrow_mut`. A query-cancel ERROR can longjmp past Rust destructors,
leaving that mutable borrow permanently held and making the next transport call
panic.

This checkpoint:

- removes PostgreSQL interrupt checks from the async await helper;
- performs the pre- and post-call interrupt checks outside the complete
  thread-local `RefCell` borrow scope; and
- adds a live PG18 regression that establishes a pooled connection, has a
  second backend issue `pg_cancel_backend` during a remote `pg_sleep`, catches
  the cancellation in an internal subtransaction, and immediately reuses the
  same pooled transport from the same backend.

The remote query and connection deadlines remain unchanged. Packet 036's
serial-fanout P2 and await-boundary cancellation-latency P3 remain outside this
narrow remediation.

## Validation

See `artifacts/manifest.md` and its packet-local logs. At the exact
implementation head:

- PG18 warnings-denied clippy passes;
- the cancel-mid-await then same-backend-reuse regression passes; and
- the existing pooled-session statement-timeout regression still passes.

## Requested decision

Please confirm that PostgreSQL ERROR can no longer cross a live transport-state
`RefMut`, and close packet 036 P1-1 if the live cancellation/reuse proof is
sufficient.

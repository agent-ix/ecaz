# Review request: prompt remote transport cancellation

## Scope

Please review implementation commit `a94e5e9be` as the remediation for packet
036 P3 and the remaining mid-connect half of its cancellation-test request.

The current-thread Tokio runtime previously observed PostgreSQL interrupts
only before and after `block_on`. A Ctrl-C or `pg_cancel_backend` arriving while
the runtime was parked therefore waited for the remote statement timeout or
the client fallback deadline before releasing coordinator locks.

This checkpoint:

- races every connection/query await against a 5 ms poll of PostgreSQL's
  backend-local `InterruptPending` and `QueryCancelPending` flags;
- observes those flags without invoking `CHECK_FOR_INTERRUPTS` while the
  transport-state `RefCell` borrow is live;
- makes a best-effort libpq cancel-token delivery bounded to 100 ms for an
  in-flight remote query;
- records the local interrupt, returns normally through the async runtime, and
  clears all pooled clients/driver tasks before leaving the Rust guard scope;
- invokes PostgreSQL's interrupt checker only after the `RefMut` has dropped,
  preserving packet 040's backend-poisoning fix; and
- aborts a pooled connection's driver task whenever the connection is dropped,
  so interrupted protocol state cannot be reused.

Connect establishment uses the same interrupt race without a cancel token;
dropping the connect future closes that attempt before the outer PostgreSQL
cancel is raised.

## Validation

See `artifacts/manifest.md`. At the exact implementation SHA:

- strict PG18 clippy passes;
- all eight focused transport unit tests pass;
- the existing remote statement-timeout test passes;
- a 5-second remote sleep under a 10-second remote budget is cancelled after a
  second backend calls `pg_cancel_backend`, completes in under one second, and
  is followed by successful same-backend transport reuse; and
- a deliberately blackholed TCP/PostgreSQL handshake under a 10-second connect
  budget is cancelled the same way, completes in under one second, and is
  followed by a successful real connection from the same backend.

## Benchmark status

The 5 ms poll participates in every foreground remote await. Although the
operation is a cheap backend-flag read plus a timer future, this packet does not
claim it is latency-free from static/live evidence. Packet 050's direct-reader
candidate is the immutable pre-change 10k/50k/100k baseline; a following
same-config canonical suite packet will measure recall, warmed latency,
storage, topology, and the same-data control before performance closeout.

## Requested decision

Please confirm that cancellation now unwinds promptly without permitting a
PostgreSQL longjmp across a live Rust transport-state borrow, and close packet
036 P3 if the mid-await/mid-connect proofs plus the following A/B are
sufficient. This packet does not close Task 179 or unrelated fault-window and
Task 172 gates.

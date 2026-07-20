# Review request: post-closeout safety and resource hardening

## Status

This packet responds to packet 060's outside verification. Task 179 remains
done, as that feedback explicitly decided. Code checkpoint `7bb215458` fixes
P2-1 through P2-4; this request records the required P2-10 disposition and the
remaining follow-on routing. No merge to `main` is requested or performed.

## Fixed now

### P2-1: transaction fence callback borrow

`remove_xact_fence_reference` now removes the entry inside the thread-local
`RefCell` borrow and performs `LockRelease` plus shared-registry cleanup only
after the `RefMut` has been dropped. A PostgreSQL ERROR during either external
operation can no longer poison the backend-local borrow state.

### P2-2 and P2-3: prompt, portable transport interrupts

The async transport poll now:

- treats `ProcDiePending` as a prompt stop condition in addition to ordinary
  query cancellation;
- reads pgrx's bound PostgreSQL globals directly with volatile loads;
- removes the glibc-specific `dlsym(NULL, ...)` lookup and its repeated
  per-poll symbol resolution; and
- supplies inert definitions only in the standalone Rust-test C stubs, while
  the production extension continues to bind PostgreSQL's real globals.

The pure predicate test covers idle, incomplete cancel, complete cancel, and
backend-termination flag combinations. The live PG18 cancellation test proves
`pg_cancel_backend` remains prompt and the pooled transport reconnects/reuses
successfully afterward.

### P2-4: CustomScan Rust-state cleanup on ERROR

The CustomScan exec state now embeds a `MemoryContextCallback` registered on
the executor's per-query context before BeginCustomScan can allocate owned
payload/search state. The callback runs `drop_in_place` on normal context reset
and ERROR/abort cleanup. `EndCustomScan` eagerly drains the potentially large
Rust resources but no longer manually `pfree`s PostgreSQL-owned state; the
eventual callback safely drops the emptied wrapper exactly once.

The unpublished-control regression now executes the CustomScan failure in an
internal subtransaction and asserts that rollback invokes exactly one cleanup
callback.

## P2-10 evidence disposition

The historical packet-031/032 topology artifacts do not contain NFR-018's
later `control_index_bytes` and heap-vs-TOAST columns. That gap is acknowledged
and is not reconstructed or papered over here. No packet-059 acceptance number
depends on a fabricated split, and the outside review explicitly left Task 179
done. Any future Task 172 topology/storage recapture must emit the complete
current accounting schema; packets 031/032 remain historical evidence with
this limitation.

## Remaining P2 routing

- P2-5 is waived for Task 179's deprecated, non-gate legacy multinode lane. If
  that lane remains supported, its directory-cache fix belongs with its next
  maintenance change rather than the physical closeout.
- P2-6, including the BW/H shape A/B and owner-side fixed-cost reductions,
  belongs to Task 172's still-open performance program.
- P2-7, P2-8, and P2-9 are accepted as the first hardening/refactor/testing
  slice for Task 167 before physical DML adds more lifecycle transitions.
- The P3 inventory is acknowledged with no action in this packet, matching the
  reviewer's requested posture.

## Validation

All commands ran at `7bb215458823908397cb76ee2b4630da8d989d65`:

- interrupt predicate unit test: 1 passed, 0 failed;
- PG18 unpublished-control CustomScan rollback: 1 passed, 0 failed, with the
  exactly-once cleanup assertion;
- PG18 live cancellation and pooled-session reuse: 1 passed, 0 failed; and
- PG18 production-library clippy: pass with `-D warnings`.

These changes alter cancellation and cleanup behavior only. They do not change
distance scoring, graph traversal, result ordering, storage layout, or the
accepted 10k/50k/100k physical benchmark matrix.

## Requested verification

Please verify the P2-1 through P2-4 fixes and the P2-10 disposition, and close
packet 060's requested feedback acknowledgement. This packet remains open for
outside feedback.

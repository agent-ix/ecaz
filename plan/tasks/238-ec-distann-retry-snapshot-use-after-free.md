# Task 238: ec_distann Retry Snapshot Use-After-Free

Status: **fix (`010a0accc`) and regression test landed on `task222-recovery`;
before/after proof in `reviews/task-238/001-retry-snapshot-uaf/`; main landing
outstanding** (2026-08-24). Priority: **P0 correctness — a live crash on
`origin/main`.**

Origin: found by the Task 222 coder while root-causing eight SIGSEGV runs in
`reviews/task-222/002-contract-and-correctness/artifacts/gdb-backtrace.log`.
Not Task 222 scope — the payload mask was never implicated.

## Why

`src/am/ec_distann/generation_read.rs:6237` on `origin/main`:

    let retry_snapshot = Some(retry_snapshot);
    if let Some(snapshot) = retry_snapshot.as_ref() {
        self.snapshot = snapshot.as_ptr();
    }

`lookup_graph_nodes_with_reopened_intent_retry` returns a
`RegisteredSnapshotGuard` as its fourth tuple element precisely so the caller
keeps the snapshot registered. The caller binds it to a **local**, copies the
raw pointer into `self.snapshot`, and drops the guard at the end of the
method. `self` outlives that scope, so `self.snapshot` dangles and every later
hop round runs visibility checks through freed memory.

Evidence: SIGSEGV in `HeapTupleSatisfiesMVCC` (`heapam_visibility.c:1080`)
reached via `lookup_graph_nodes` → `expand_nodes_masked`, with
`0x7f7f7f7f7f7f7f7f` (PostgreSQL `CLOBBER_FREED_MEMORY`) visible up the stack.

Introduced by `79afb0d82` "Reopen guards for every owner retry path"
(2026-08-15) and present on `origin/main` since.

## Blast radius

The retry path is entered when the initial graph lookup returns
`OwnedRecordMissing` — a node the traversal expects is not visible under the
current snapshot.

**Corrected 2026-08-24 by the packet 001 runs.** The original wording here said
this requires "a read racing a concurrent owner write or build." It does not.
The three-owner PG18 fixture reaches the path and segfaults **without any
external concurrency** (`artifacts/pg18-forced-retry-without-fix.log`, crash at
the Task 222 cached-plan `EXECUTE`), so ordinary multi-owner reads can take it
whenever an owned record is not yet visible to the scan's snapshot.

- Benchmark exposure still needs an explicit statement rather than an
  assumption. The published Task 222 matrices ran with the fix present
  (`c9f79be4a` postdates `010a0accc`), so those numbers are not in question.
- The failure is a backend crash, or — if the freed snapshot still holds
  plausible bytes — **wrong visibility answers** rather than a crash. The
  wrong-answer case must still be assessed explicitly.

## Goal

Land the fix on `main` independently of any benchmark gate, with a regression
test that fails without it.

## Scope

1. **Fix** (done, `010a0accc`): hold the guard in a
   `retry_snapshot: Option<RegisteredSnapshotGuard>` field on
   `GenerationExpander` so the guard's lifetime matches the pointer's use;
   `retry_snapshot: None` at the four construction sites.
2. **Regression test** (outstanding): `ec_distann.debug_force_frontier_retry`
   (`options.rs:722`, consumed at `generation_read.rs:398`) already forces the
   retry path once and **no test currently sets it**. Add a PG18 test that
   drives a multi-hop expansion through the forced retry and then continues
   traversal, so the dangling snapshot is dereferenced. It must fail on
   `origin/main` and pass with the fix.
3. **Landing** (outstanding): cherry-pick onto a branch off `origin/main` and
   open a PR. `generation_read.rs` on this branch differs from main by exactly
   this commit, so the cherry-pick is conflict-free.
4. **Disclosure**: state in the packet whether any recent distann run could
   have taken the retry path, and whether wrong-visibility results are
   possible as well as crashes.

## Non-goals

- Waiting on Task 222's 100k A/B, or any other benchmark gate. This is a
  correctness fix on main and does not require benchmark evidence.
- Broadening into a general audit of RAII guard lifetimes (worth doing;
  file separately if wanted).

## Acceptance

1. A regression test fails on `origin/main` and passes with the fix.
2. The fix is merged to `main` via its own PR, not as a passenger on the
   materialization follow-ups branch.
3. The blast-radius statement (crash vs wrong visibility, benchmark exposure)
   is recorded in the review packet.

## Required review packets

1. `reviews/task-238/001-retry-snapshot-uaf/`

## References

- `reviews/task-222/002-contract-and-correctness/artifacts/gdb-backtrace.log`
- `reviews/task-222/002-contract-and-correctness/feedback/2026-08-24-01-reviewer.md`
- Introducing commit `79afb0d82`; fix `010a0accc`

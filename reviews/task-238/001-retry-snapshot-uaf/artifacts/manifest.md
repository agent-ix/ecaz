# Task 238 packet 001 artifact manifest

- Head SHA at capture: `a3898ff43` (test added on top; committed as the
  following commit on `task222-recovery`)
- Fix under test: `010a0accc` "Retain refreshed graph snapshots across hop
  rounds" (`src/am/ec_distann/generation_read.rs`)
- Task bucket: `reviews/task-238/`
- Packet: `reviews/task-238/001-retry-snapshot-uaf/`
- Timestamp: `2026-08-24T12:11-07:00`
- Lane / fixture: PG18 focused three-owner physical handoff
  (`test_distann_payload_projection_contract`), mixed local and
  loopback-remote owners
- Isolation: correctness test only. No benchmark surface, no `ecaz bench suite`
  result claimed. Both runs executed in a dedicated detached worktree
  (`.worktrees/task238-uaf-test`, since removed) so the authoring branch's
  working tree was never modified. The host had no other benchmark or cluster
  process running during either run — verified before starting, because a
  Task 227 latency suite had been active earlier the same day.

## Artifacts

### `pg18-forced-retry-with-fix.log`

Command:

`cargo pgrx test pg18 test_distann_payload_projection_contract --no-default-features --features pg18`

Tree: `a3898ff43` plus the new regression block, fix present.

Key result: `running 1 test`;
`test tests::pg_test_distann_payload_projection_contract ... ok`;
`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2578 filtered out;
finished in 77.74s`.

### `pg18-forced-retry-without-fix.log`

Command: identical, with only `010a0accc` reverted —
`git checkout 010a0accc^ -- src/am/ec_distann/generation_read.rs`
(2 insertions, 9 deletions; no other commit on this branch touches that file,
so the revert is exactly the fix and nothing else).

Key result: `test tests::pg_test_distann_payload_projection_contract ...
FAILED`; `client backend (PID 1744184) was terminated by signal 11:
Segmentation fault`; `test result: FAILED. 0 passed; 1 failed`.

`DETAIL: Failed process was running: EXECUTE task222_cached_projection
(ARRAY[30.0, 2.0, 0.0, 1.0]::real[])`

## What these two runs prove, and what they do not

**Proven:** the fix is necessary. The same test, on the same tree, with only
the nine-line guard-lifetime change reverted, segfaults the backend; with the
change present it passes. That is the before/after pair the task's acceptance
criterion 1 asks for.

**Proven:** the regression is now covered by the committed suite. Any future
reintroduction of the dangling-guard pattern fails
`test_distann_payload_projection_contract`.

**Not proven — stated so it is not inferred:** that the newly added
forced-retry block is what catches it. The crash in the without-fix run occurs
at `ec_distann_physical_lifecycle.rs:3772`, the pre-existing Task 222
cached-plan `EXECUTE` case, roughly 300 lines *before* the new block at
`:4066` ever runs. The coder's existing Task 222 coverage already reproduces
this use-after-free naturally, without setting
`ec_distann.debug_force_frontier_retry`.

The new block is therefore belt-and-braces, and is worth keeping for one
reason: the natural reproduction depends on the fixture incidentally reaching
a not-yet-visible owned record. That is an emergent property of the fixture,
not an asserted one, and it can disappear silently if the fixture changes.
`SET ec_distann.debug_force_frontier_retry = on` makes the path deterministic
and intentional, and the block asserts both that the forced retry returns rows
identical to the known-good baseline and that traversal continues afterwards
against a live snapshot.

## Reviewer-introduced regression, caught and fixed (2026-08-24)

The regression block landed in `f019d0fd5` runs inside
`run_distann_three_owner_physical_handoff`, which **two** `pg_test` entry
points call. Only `test_distann_payload_projection_contract` was run before
committing. The sibling `test_distann_three_owner_physical_handoff` then
failed, and the three runs below establish attribution unambiguously:

| tree | result | artifact |
| --- | --- | --- |
| `a3898ff43` (before the block) | ok, 1 passed, 243.43s | `pg18-sibling-handoff-baseline-a3898ff43.log` |
| `f019d0fd5` (block as first committed) | FAILED | `pg18-sibling-handoff-broken-f019d0fd5.log` |
| with the sink fix | ok, 1 passed, 263.36s | `pg18-sibling-handoff-fixed.log` |

Cause: forcing a retry also drives the Task 167 retry-attribution `INSERT`,
and this branch predates main's guard making `ec_distann_retry_attribution`
optional. The projection-contract path creates that relation as a fixture
hack; the sibling path does not, so the block borrowed a fixture it did not
own and hit
`relation "ec_distann_retry_attribution" does not exist` (SQLSTATE 42P01).

Fix: the block now creates the sink itself with `CREATE UNLOGGED TABLE IF NOT
EXISTS`, so it is self-contained in both callers and a no-op where the
relation already exists. Landed as a follow-up commit rather than an amend,
because `f019d0fd5` was already pushed to a shared branch.

**Standing issue this exposes:** the missing production guard has now been
worked around twice in this file — once by the coder, once here. The durable
fix is rebasing the branch on `main` rather than a third fixture hack. Recorded
in `reviews/task-222/004-full-scale-decision/feedback/2026-08-24-01-reviewer.md`.

## Correction to the blast-radius note in the task file

`plan/tasks/238-...md` says the retry path is entered when "a read races a
concurrent owner write or build." These runs show it is reached in the
three-owner fixture **without any external concurrency** — a not-yet-visible
owned record is enough. The exposure is therefore wider than that wording
implies: ordinary multi-owner reads can take the path. The task file has been
updated accordingly.

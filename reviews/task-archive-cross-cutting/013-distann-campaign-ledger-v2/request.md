---
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 01
---

# DistANN follow-up campaign ledger — clean current-main integration

## Seq01 correction response

Commit `e86d7813c` closes both seq01 blockers without changing import scope:

1. Task 238's header and README row now state unambiguously that the equivalent
   lifetime fix is on main, while forced-retry test commit `3b8b872d6` is only
   authored against exact main on the campaign stack and is **not** on main.
   DoD item 2 and outside closeout remain open.
2. The sole dead Task 222 feedback citation was removed from Task 238; the
   packet-local/backtrace reference remains.

The correction commit message also discloses seq01's non-blocking scope note:
this integration intentionally fills main's missing, accurate, singular roadmap
rows for the already-accepted Tasks 227 and 236. Please perform the bounded
seq02 rereview authorized by
`feedback/2026-08-26-01-reviewer.md`.

This packet requests review of the clean campaign-ledger integration at
`8d63e3d2a842a257fcff20cdc79c34f005132b24` on branch
`integrate/distann-campaign-ledger-v2`.

## Why this integration exists

The original campaign stack was never merged to main and now conflicts with
the independently landed Tasks 222, 226, 227, 236, and 239. The campaign plan
requires a clean documentation import from current main rather than merging
that old stack wholesale. Current main is
`2adf3d543b7e08b031563dc4ee15e6204d9c0038` (Task 239 PR #88).

## Scope

Commit `40d3b7ef1` imports only the missing canonical task definitions for Tasks
223--235 and 237--238, their planning packets, the matching task-index rows,
and the corresponding roadmap program/candidate entries. It also restores the
already-reviewed Task 238 packet-001 evidence carried by that docs integration.
Conflict resolution preserves main's accepted Tasks 222, 226, 227, 236, and
239 verbatim and orders Tasks 237/238 before 239.

Commit `8d63e3d2a` selectively imports the already-reviewed Task 223/224
disposition packets from their accepted campaign branches and synchronizes the
task header, task index, and roadmap:

- Task 223 is complete / outside-reviewed ACCEPT / STOP on the accepted
  0.514999 ms, 4.439647% zero-cost ceiling.
- Task 224 is complete / packet-003 review-closed ACCEPT / STOP; MAT-25 is
  retired and MAT-26 is unmeasured with a void candidate axis and no finalist.
  Its feature-only/default-off diagnostic implementation remains on the
  reviewed campaign stack and is explicitly not productionized by this import.
  Task 239 has since resolved the carried 12/10 divergence.
- Task 225 remains independently conditional.
- Task 229 is now `ready`: Tasks 222--224 and 239 have all satisfied its four
  entry conditions. Its matched-position/counterbalanced A/B rule is retained.
- Tasks 230--233 remain mandatory and ordered regardless of Task 229's result.

## Explicit exclusions

- No `src/**`, `crates/**`, SQL, spec, suite-config, or runtime code is imported.
- The old campaign branch is not merged.
- Task 224's feature-only diagnostic code is not ported to production main.
- Open Task 234/235 implementation stacks are not represented as landed; their
  canonical rows remain NOT DONE/ready according to the current-main campaign
  state.
- Task 238 remains closeout-incomplete; this integration restores its task and
  evidence but does not self-accept it.

## Validation

- `git diff origin/main...8d63e3d2a --name-only` contains no path under
  `src/`, `crates/`, `sql/`, or `spec/`.
- Every task in 223--235, 237--239 has exactly one task definition, one task
  index row, and one roadmap program row.
- No corpus TSV, truth cache, tunnel, SSM/polling tree, or other banned runtime
  artifact is introduced.
- The Task 223/224 packet bytes are imported from their reviewed branches
  without normalization. Seven historical files retain their pre-existing
  trailing blank line, so `git diff --check` reports those byte-preservation
  warnings; changing them would invalidate committed packet hashes.
- Tests were not run because this is documentation/evidence integration only.

Please verify the exact task set, absence of runtime changes, preservation of
main's five accepted statuses, Task 223/224 evidence provenance, Task 229 entry
condition closure, and that the resulting canonical ledger is suitable to land
before Task 229 implementation begins.

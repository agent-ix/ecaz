# Review Request: Task 17 Status Doc Refresh

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `plan/tasks/17-diskann-access-method.md`
- `plan/design/diskann-scan-pgrx.md`

## What this packet is

This is a doc-only cleanup packet after Phase 9.

No `src/am/ec_diskann/*` code changed. The goal is only to bring the
task-17 planning surfaces back into sync with the current branch state:
the callback buildout is now landed through planner activation, but the
task doc still read like Phase 9 was pending and the scan design doc
still called ADR-046 / ADR-047 "PROPOSED".

## Why this slice

After packet 11045, the code state and the planning docs had diverged in
three visible ways:

1. `plan/tasks/17-diskann-access-method.md` still said task 17 was "in
   progress" with Phase 5C-3 / 6B prep outstanding.
2. The same task doc still described the end state as planner-gated /
   explicit-opt-in-only, which was true before Phase 9 but is no longer
   true now that `ec_diskann_amcostestimate` is live.
3. `plan/design/diskann-scan-pgrx.md` still listed ADR-046 and ADR-047
   as `PROPOSED` even though both are already accepted and reflected by
   the landed callback code.

This packet fixes those stale statements without widening into new code
or new runtime behavior.

## What changed

### `plan/tasks/17-diskann-access-method.md`

Refreshed the top-level task status to say what the branch now actually
contains:

- callback buildout phases 1–9 are landed on `adr034-diskann-rebased`
- remaining work is review / merge / higher-level signoff, not another
  missing AM callback
- the 2026-04-19 prep items are now marked resolved
- the top summary now lists the landed build / scan / insert / vacuum /
  planner slices explicitly

Also corrected two stale planning assumptions:

- the example SQL now uses the real AM/opclass names:
  `USING ec_diskann (embedding ecvector_diskann_ip_ops)`
- the end-state text now says Postgres may naturally choose
  `ec_diskann` on sufficiently large ordered queries, while still
  clarifying that there is no "default AM" flip

Phase 9's section now points at packet 11045 and marks the cost callback
plus planner-gate lift as landed, while leaving FR-023 / FR-024 as PG18
follow-up work rather than pretending those are done.

Finally, the old native-build conflict-surface note is now labeled
historical/resolved so future readers do not treat a 2026-04-19 branch
coordination rule as a current blocker.

### `plan/design/diskann-scan-pgrx.md`

Only one change:

- ADR-046 and ADR-047 references now say `ACCEPTED` instead of
  `PROPOSED`

## Boundary after this packet

The docs should now match the current branch reality:

- task 17 callback buildout is materially landed on the branch
- planner activation is no longer described as future work
- the scan design doc no longer points at obsolete ADR status

This packet does **not** claim final merge or final release signoff.
It only removes stale planning language.

## Verification

Doc-only packet. No code changed, so I did not rerun the full Rust / pg17
verification stack for this slice.

Validation performed:

- reviewed the staged diff for scope
- confirmed only the two plan/design markdown files changed

## Reviewer notes

- This is intentionally a doc sync, not a retroactive rewrite of the
  whole historical phase checklist.
- The task doc now includes an explicit "historical planning note" so
  the remaining lower sections can stay as the original plan structure
  without pretending to be the live status dashboard.
- The packet does not change any runtime claims beyond what packets
  11029 through 11045 already landed and verified.

## Not doing in this packet

- Any `src/am/ec_diskann/*` code changes
- Any test/lint baseline cleanup
- Any project-wide status rewrite outside the DiskANN planning surfaces

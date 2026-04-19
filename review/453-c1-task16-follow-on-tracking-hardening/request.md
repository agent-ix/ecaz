# Review Request: C1 Task16 Follow-On Tracking Hardening

Current head at execution: `8e2d1ee`

## Context

Reviewer feedback on packet `452` accepted the task-16 storage-policy deferral
but raised three follow-up hygiene concerns:

1. the `EXTENDED` / `MAIN` build-time collapse should survive as an explicit
   tracked bug/follow-on, not just a note in packet text
2. ADR-044 should say exactly what reopens the deferred matrix instead of
   "after native build" in the abstract
3. the lever-4 `ef_search` matrix should live in a standalone follow-on task
   so it does not disappear when task 16 merges

This slice closes those tracking gaps without changing the task-16 merge
decision itself.

## What changed

Updated:

- `spec/adr/ADR-044-ecvector-rerank-source-location-and-storage-policy.md`
- `plan/tasks/16-turboquant-iteration.md`
- `plan/tasks/README.md`
- added `plan/tasks/17-post-native-build-storage-and-lever4-followons.md`

### 1. ADR-044 now has explicit reopen criteria

The ADR no longer says only "revisit after ADR-042". It now says reopen the
matrix only after:

1. ADR-042 native HNSW build lands
2. a fresh 50k `EXTERNAL` TurboQuant build establishes a stable native-build
   baseline
3. `EXTENDED`, `MAIN`, and `PLAIN` either:
   - build within about `±20%` of that baseline, or
   - any remaining gap is explained by profiling rather than mystery

Only then rerun the deferred q200/storage-policy cells and use them for a
product default choice.

### 2. The build-collapse bug now has a durable home

New task:

- `plan/tasks/17-post-native-build-storage-and-lever4-followons.md`

It explicitly carries forward the old-builder abnormality as a work item:

- reproduce the task-16 `EXTENDED` / `MAIN` build collapse on native build
- if it disappears, close it as old-builder-specific
- if it persists, profile and fix it before trusting storage-policy readouts

So the `1292.15s` `EXTENDED` build and `24:27+` `MAIN` build no longer live
only in packet prose.

### 3. Lever-4 `ef_search` matrix now survives task-16 merge

Task 17 also owns:

- the lever-4 `full_lut` `ef_search` matrix at `64 / 128 / 256`
- the persisted-default decision that depends on that matrix

Task 16's deferred-follow-on section now points directly at task 17 as the
primary tracking file, so the work cannot be orphaned when task 16 merges.

### 4. Task-16 plan is now consistent with repo truth

While touching the follow-on tracking, two stale checklist items were cleaned
up:

- ADR-043 ratification is marked closed because ADR-043 is already `ACCEPTED`
- the stale `ecqvector` doc/error-text blocker is marked closed because grep
  now shows that old name only in historical notes

## Why this matters

The branch was already merge-ready from the task-16 point of view. This slice
does not reopen that. It makes the deferred work harder to lose:

- the reopen criteria are concrete
- the build-collapse bug has a home
- the lever-4 matrix has a standalone task file

## Validation

Docs / planning only.

No Rust or SQL code changed in this slice, so the cargo / pgrx / clippy
checkpoint trio was not rerun.

## Review focus

1. Do the new ADR-044 reopen criteria give enough structure to avoid "deferred
   forever" drift?
2. Is task `17-post-native-build-storage-and-lever4-followons.md` the right
   place to carry both:
   - the old-builder `EXTENDED` / `MAIN` build-collapse bug
   - the lever-4 `ef_search` matrix
3. Is task 16's deferred-follow-on pointer to task 17 sufficient to keep these
   items from being orphaned at merge?

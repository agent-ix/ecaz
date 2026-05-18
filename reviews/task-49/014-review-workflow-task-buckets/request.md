# Review Request: Task-Scoped Review Bucket Migration

Scope:
- `AGENTS.md`
- `CLAUDE.md`
- `reviews/`
- legacy `review/` pointer docs
- path references in task/design/spec/docs files

What changed:
- Moved completed/non-deferred review packets from flat `review/` into
  task-scoped buckets under `reviews/task-{id}/`.
- Added task-local sortable packet prefixes so each bucket orders as
  `001`, `002`, `003` or wider where needed.
- Kept deferred Task 41 and active benchmark/measurement packets in legacy
  `review/` for later focused migration.
- Updated workflow docs so review requests, feedback, validation logs,
  benchmark logs, and artifacts all live under the owning task's review packet.
- Added a Task 29e task file to anchor the existing `reviews/task-29e/` bucket.
- Recorded the migration and correction mapping in `reviews/MIGRATION.md`.

Validation performed:
- Explicit `taskNN` packet-name tokens match their `reviews/task-{id}` bucket:
  `0` mismatches.
- Every non-archive `reviews/task-*` bucket has a matching task definition:
  `0` missing task files.
- Packet directory ordering prefixes are present and lexically/numerically
  contiguous from `001`: `0` ordering problems.
- No Task 41 packet moved into `reviews/`: `0` Task 41 directories under
  `reviews/`.
- Legacy `review/` contains only deferred material: `77` Task 41 entries,
  `115` benchmark-like entries, and `2` pointer docs.
- Stale references to migrated legacy packet paths outside legacy `review/`:
  `0`.

Review focus:
- Whether the task bucket mapping is correct enough for the non-deferred
  migrated packets.
- Whether `AGENTS.md` and `CLAUDE.md` describe the new local-review/log/artifact
  contract clearly enough for future agents.
- Whether the remaining `task-archive-cross-cutting` entries are acceptable as
  broad historical packets rather than forced into an implementation task.

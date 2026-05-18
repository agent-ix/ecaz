# Task-Scoped Reviews

Review packets are organized by task. The task is the unit of isolation for
requests, feedback, validation logs, benchmark logs, and artifacts.

## Layout

```text
reviews/
    task-42/
      001-short-topic/
      request.md
      artifacts/
        manifest.md
        ...
      feedback/
        2026-05-17-01-reviewer.md
```

- `task-{id}` matches the canonical task definition in `plan/tasks/`.
- Subtasks keep their suffix, for example `task-29a`.
- Historical work that predates the current task taxonomy may use explicit
  archive task buckets such as `task-archive-cross-cutting`.
- Packet directories are task-local and sortable: `001-`, `002-`, `003-`, and
  so on. Use at least three digits; widen only if a bucket exceeds 999 packets.

## Packet Contents

- `request.md` is the review request and summary.
- `feedback/` contains durable reviewer/coder feedback files.
- `artifacts/` contains all durable evidence for review and testing:
  - test logs
  - benchmark logs
  - corpus/load logs
  - raw measurement output
  - generated SQL fixtures
  - JSON/JSONL result files
  - screenshots and audit outputs

Measurement packets must include `artifacts/manifest.md` with head SHA, task
bucket, packet path, command, timestamp, lane/fixture details, and key result
lines cited by `request.md`.

## Migration Notes

`reviews/MIGRATION.md` records the first flat-to-task migration. Legacy
`review/` remains only as a temporary holding area for deferred Task 41
packets. Do not add new packets to `review/`.

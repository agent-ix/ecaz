# Claude Agent Workflow

Follow `AGENTS.md` for the repository workflow, review-packet rules, checkpoint
rules, and local safety rules.

## Task File Lookup

- Canonical task definitions live under `plan/tasks/`, not under `review/`.
- Use `plan/tasks/README.md` as the task index. Numbered primary tasks use the
  `NN-slug.md` filename pattern.
- If a requested task is not present in the current checkout, do not infer from
  similarly numbered review packets. Refresh or inspect `origin/main` first,
  for example `git fetch origin main` and
  `git ls-tree --name-only origin/main:plan/tasks`.
- Current hardening follow-up tasks are `35` through `49` in `plan/tasks/`.
  Task 42 is `plan/tasks/42-on-disk-format-invariants.md`.

## Local Operator CLI

- Prefer `ecaz-cli` for local PostgreSQL/pgrx setup, SQL checks, corpus
  generation/load/list/inspect, and benchmark/storage commands when that
  surface exists.
- In sandboxed agent sessions, invoke the installed binary by absolute path,
  currently `/Users/peter/.cargo/bin/ecaz`, so one approval rule can cover the
  operator surface consistently.
- Route PG18 socket work through `ecaz` commands such as `ecaz dev sql`,
  `ecaz corpus ...`, and `ecaz bench ...` instead of direct `psql`, wrapper
  scripts, or one-off shell plumbing.
- Use packet-local logging flags (`--log-file` or command-specific
  `--log-output`) for review artifacts.
- If a repeated setup or benchmark operation is missing from `ecaz-cli`, add a
  narrow CLI command or option instead of working around the sandbox with ad hoc
  commands.

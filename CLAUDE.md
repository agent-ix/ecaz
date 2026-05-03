# Claude Agent Workflow

Follow `AGENTS.md` for the repository workflow, review-packet rules, checkpoint
rules, and local safety rules.

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

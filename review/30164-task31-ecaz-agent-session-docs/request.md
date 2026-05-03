# Task 31 Ecaz Agent Session Docs

Reviewer: please review this docs-only setup checkpoint.

## Scope

This checkpoint documents the local operator workflow that came out of the Task
31 M5 smoke setup. The goal is to reduce repeated sandbox/approval friction by
making `ecaz-cli` the explicit boundary for local PG18/pgrx and benchmark
operations.

Docs checkpoint commit: `cc4b1bd9ca3ad044cb96d30e46f7e9500469c7f6`

Files changed:

- `AGENTS.md`
- `CLAUDE.md`
- `crates/ecaz-cli/README.md`

## What Changed

- `AGENTS.md` now tells agents to prefer `ecaz-cli` for PG18/pgrx setup, SQL
  checks, corpus generation/load/list/inspect, and benchmark/storage commands.
- `CLAUDE.md` was added with the same local operator guidance and a pointer back
  to `AGENTS.md`.
- `crates/ecaz-cli/README.md` now documents that sandboxed agent sessions should
  prefer the absolute installed path, currently `/Users/peter/.cargo/bin/ecaz`,
  so one approval rule can cover the operator surface.
- The README also documents using `ecaz dev sql --log-output` for packet-local
  SQL logs and says missing repeated setup/benchmark operations should become
  narrow `ecaz` commands/options instead of ad hoc shell workarounds.

## Validation

No tests were run. This is a documentation-only checkpoint with no code or
runtime behavior change.

## Follow-Up

Use the documented `/Users/peter/.cargo/bin/ecaz` path for the first real Task
31 IVF baseline packet unless the local shell environment is updated to put
Cargo's bin directory on `PATH`.

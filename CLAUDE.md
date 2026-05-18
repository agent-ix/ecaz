# Agent Workflow

This repository uses a task-scoped review-packet workflow in addition to
normal code changes. Two roles operate against it: **coder** (implements work,
requests review) and **reviewer** (reads checkpoints, leaves feedback). The
task is the unit of isolation: review requests, feedback, validation logs,
benchmark logs, and artifacts all live under that task's review bucket.

See `reviews/README.md` for full structure and conventions.

---

## Common Rules

### Task-Scoped Review Buckets

Canonical task definitions live under `plan/tasks/`, not under `review/` or
`reviews/`. Review packets live under `reviews/` in matching task buckets:

    reviews/task-42/
      001-short-topic/
        request.md
        artifacts/
          manifest.md
          ...
        feedback/
          2026-05-17-01-reviewer.md

- Bucket names are `task-{task-id}` where `{task-id}` matches the task file
  identity, for example `plan/tasks/42-on-disk-format-invariants.md` maps to
  `reviews/task-42/`.
- Subtasks keep their suffix: `29a` maps to `reviews/task-29a/`.
- Historical work that predates the current task taxonomy may use explicit
  archive buckets such as `reviews/task-archive-cross-cutting/`.
- Do not create top-level packet directories under `review/` or `reviews/`.
  New packets must be inside the owning task bucket.

### Packet Ordering

Packet directories inside a task bucket must sort in chronological order.

- Prefix every packet directory with the next task-local ordinal:
  `001-`, `002-`, `003-`, and so on.
- Use at least three digits. If a bucket ever grows beyond 999 packets,
  widen the prefix for that bucket without changing the descriptive slug.
- Keep the descriptive packet slug after the ordinal; do not use global random
  number ranges for new work.

### Task File Lookup

- Use `plan/tasks/README.md` as the task index. Numbered primary tasks use the
  `NN-slug.md` filename pattern.
- Review packet numbers or ordinals are not task numbers. Do not infer a task
  from a similarly numbered review packet.
- If a requested task is not present in the current checkout, refresh or inspect
  `origin/main` before declaring it missing, for example:
  `git fetch origin main` and
  `git ls-tree --name-only origin/main:plan/tasks`.
- Current hardening follow-up tasks are `35` through `49` in `plan/tasks/`.
  Task 42 is `plan/tasks/42-on-disk-format-invariants.md`.

### Feedback Files

- Feedback always lands as a file under the packet's `feedback/` directory:
  `reviews/task-{id}/{ordinal-topic}/feedback/{YYYY-MM-DD}-{seq}-{agent}.md`.
  Chat output alone is invisible to the coder inbox loop.
- Frontmatter is required: `agent`, `role`, `model`, `date`, `seq`.
- Any agent can leave feedback on any topic.

### Review, Test, Benchmark, and Artifact Logs

- Any output intended to support a review must be packet-local under
  `reviews/task-{id}/{ordinal-topic}/artifacts/`.
- This includes test logs, benchmark logs, corpus/load logs, raw measurement
  output, generated SQL fixtures, JSON/JSONL result files, screenshots, and
  one-off audit outputs.
- Do not cite local-only `tmp/` paths, terminal scrollback, or files outside the
  packet as durable review evidence.
- Measurement packets must include `artifacts/manifest.md` as the packet-local
  source of truth for artifact metadata.
- `manifest.md` should record, for each artifact:
  - head SHA
  - task bucket and packet path
  - lane / fixture / storage format / rerank mode where applicable
  - command used
  - timestamp
  - whether the run used isolated one-index-per-table or shared-table surfaces
  - the key result lines that `request.md` cites
- `request.md` should summarize the result and point at the packet-local
  artifact files.

### Legacy `review/` Holding Area

`review/` is now a temporary legacy holding area only. It currently contains
deferred Task 41 packets only. Do not add new packets there.

### Benchmark Data Packets

Pure benchmark/measurement packets (no code change under review, just
measurement evidence) live under top-level `benchmarks/<topic>/`, with
`manifest.md` at the packet root and raw logs under `artifacts/`.
Code-review packets that happen to include benchmark evidence stay under
`reviews/task-{id}/{ordinal}-<topic>/` with their own
`artifacts/manifest.md`, and SHOULD cite the owning `benchmarks/<topic>/`
packet by path when one exists. See
`spec/non-functional/NFR-007-benchmark-provenance.md` for the normative
storage rule.

### Push and Visibility

- Push committed checkpoints, packet updates, and feedback files to the remote
  immediately after committing. **Anything that exists only locally — including
  chat output — is invisible to other agents.**
- When committing on a feature branch, push to **that branch**. If working
  across multiple branches, commit and push to each separately.
- After pushing, verify the push succeeded before moving on.

### Local Safety Rules

- Do not revert unrelated local changes.
- Preserve the current on-disk layout unless a very small change is clearly
  justified.
- Do not use `/tmp`-based hacks or alternate scratch homes to work around
  approval, sandbox, or environment constraints; use the normal repo and user
  tool layouts instead.
- Add ADRs for design decisions that need durable rationale.
- Never run destructive git operations (reset, rebase, drop commits) without
  reading the affected commits and getting explicit confirmation from the user
  first.

### Local Operator CLI

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
  `--log-output`) targeting the packet's `artifacts/` directory for review,
  test, and benchmark evidence.
- If a repeated setup or benchmark operation is missing from `ecaz-cli`, add a
  narrow CLI command or option instead of working around the sandbox with ad hoc
  commands.

---

## Coder Workflow

### Trigger

Invoked to implement, continue, or close out a task on the current branch.

### Inbox: Process Feedback Before New Work

- At the start of a turn, scan the owning task bucket under `reviews/` for new
  feedback files you have not processed.
- Also scan legacy `review/` only when working on a deferred Task 41 packet
  that has not been migrated yet.
- For benchmark/measurement work, scan `benchmarks/<topic>/` for the latest
  packet manifests in the same lane.
- If new feedback is present for a topic you own, process it before starting
  new implementation work.
- Do not close review requests yourself. Leave requests open until an outside
  reviewer has responded.
- Do not re-triage closed review topics unless an outside reviewer reopens them.

### Checkpoint Rules

- Work in narrow, testable slices.
- Do not run tests by default. Run tests only when a change is risky enough that
  static review is not sufficient, when PostgreSQL callback behavior must be
  verified, or when the user explicitly asks for tests.
- The primary validation target is PG18. When tests are necessary, prefer the
  narrowest PG18-focused command that covers the touched behavior, for example:
  - focused `cargo test ...`
  - focused or full `cargo pgrx test pg18`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- PG17 is optional compatibility coverage. Do not run PG17 tests unless the
  user explicitly requests PG17 validation or the change is specifically
  PG17-facing.
- Commit each reviewed code checkpoint. If tests are skipped under this policy,
  state that clearly in the commit/review context.

### Output

- A code commit that lands the slice.
- A matching review request under
  `reviews/task-{id}/{next-ordinal}-{topic}/request.md`, committed separately
  from the code change.
- Any review, test, benchmark, or measurement logs stored under that packet's
  `artifacts/` directory.
- Both commits pushed to the branch per the Common push rule.

---

## Reviewer Workflow

See `reviews/REVIEWER.md` for full reviewer trigger, scope, and output rules.

Reviewer quick rules:

- Read the requested packet under `reviews/task-{id}/`, including
  `request.md`, packet-local artifacts, and existing feedback.
- If no packet is named, review the relevant packets in the owning task bucket
  that lack current reviewer feedback.
- Write findings to
  `reviews/task-{id}/{ordinal-topic}/feedback/{YYYY-MM-DD}-{seq}-reviewer.md`.
- Put any review, test, benchmark, or measurement logs cited by feedback under
  that same packet's `artifacts/` directory.

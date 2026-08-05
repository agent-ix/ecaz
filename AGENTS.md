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
- Commit and push every review request (`request.md`) and feedback file immediately after creating it; uncommitted files are invisible to the other role.

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

### Never Commit: Corpus Data, Operational Logs, and Polling Cruft

A review packet is decision-grade evidence, not a capture of everything the run
emitted. A packet should be tens of files, not hundreds. The following are
**banned from commits** and are gitignored (see root `.gitignore`); committing
them bloats packets and git history with regenerable or throwaway data:

- **Corpus / query / ground-truth data** (`*.tsv`, `*.tsv.gz` under `reviews/`
  or `benchmarks/`). Regenerable via `ecaz corpus`. Record the corpus prefix,
  scale, and SHA in `manifest.md` instead of committing the data. The single
  largest object in this repo's history is a committed corpus `.tsv` — never
  add more.
- **SSM / tunnel / polling exhaust**: `tunnel-state/`, `tunnel-*.log`,
  `*.tunneled.json`, `diagnostic-while-*/` status snapshots, and
  `pg-readonly-status.log`. These are session/operational state, not evidence.
- **Raw SSM RunShellScript output trees** (`**/awsrunShellScript/`
  stdout/stderr dumps, often several MB each). Keep only the cited result
  lines, copied into a small packet-local log or quoted in `manifest.md`.
- **Poll snapshots**: `ssm-command-invocation.latest.json`,
  `list-command-invocations*.json`. Keep the single
  `ssm-command-invocation.final.json` when the manifest cites its command id /
  status.
- **Regenerable caches**: `truth-cache/` recall ground-truth.

What **does** belong in a packet: `manifest.md`, `request.md`, `feedback/*.md`,
the `ecaz bench suite` config, `suite-manifest*.json` + `suite-results*.jsonl`,
and the specific recall / latency / storage / load / inspect result logs that
`request.md` cites. If a packet is accumulating hundreds of files, you are
committing exhaust — stop and prune before requesting review. `.gitignore`
prevents new commits of this cruft, but already-tracked copies must be removed
with an explicit `git rm` pass.

### Where Runtime Output Goes: `--run-dir`, `target/`, and Worktrees

On 2026-07-27 this dev host filled a 1TB disk to 100%. Roughly 450G was Rust
build output duplicated across 23 checkouts, and roughly 200G was **bench
PostgreSQL clusters written into `target/`** — multi-GB PGDATA trees that read
as compiled output to anyone auditing disk usage, and that `cargo clean` does
not touch. They survived every previous cleanup because of where they were put.

**`target/` is for Cargo build output and nothing else.** Never write PGDATA,
corpus data, truth caches, or run artifacts there.

#### `--run-dir` on multinode fixtures

The multinode fixtures (`ecaz dev spire-multicluster`,
`ecaz dev distann-multicluster`) default their run directory to
`$ECAZ_CLUSTER_ROOT` (default `~/.ecaz/clusters/`), which is outside the repo
and outside `target/`. The default is resolved by `default_cluster_root()` in
`crates/ecaz-cli/src/commands/dev/support.rs`.

- **`spire-multicluster`: omit `--run-dir` and pass `--run-id`.** The run id is
  already part of the default path, so distinct arms get distinct directories
  without naming one. An explicit `--run-dir` is the exception here.
- **`distann-multicluster`: has no `--run-id`,** and its default is a single
  fixed `distann-local-multinode` directory, so concurrent or per-arm runs do
  need `--run-dir`. Point it **under `$ECAZ_CLUSTER_ROOT`**
  (e.g. `~/.ecaz/clusters/distann-<arm>`), never inside the repo.
- **Never point `--run-dir` inside the repo**, and never at `target/...`.
  `--run-dir target/task188-bw8-100k` is exactly the pattern that filled the
  disk.
- Any other `--run-dir` target — e.g. staging clusters on a different volume —
  needs a genuine reason **stated in the packet `manifest.md`**.
- **Clusters are not review evidence.** Cite the result logs under the packet's
  `artifacts/`, not the cluster directory. Remove the run directory once the
  cited results are captured; a fixture left resident is many GB per arm.

#### Build output and `CARGO_TARGET_DIR`

- Local dev hosts may point every worktree at one shared `CARGO_TARGET_DIR`
  (this host uses `~/.cargo-target`). Cargo namespaces artifacts by package and
  hash, so sharing is safe, and it collapses the per-worktree dependency graph.
- **Do not assume built artifacts are at `<repo>/target/`.** In Rust, resolve
  through `cargo_target_dir()` in `crates/ecaz-cli/src/commands/dev/support.rs`.
  In shell, use `${CARGO_TARGET_DIR:-target}`.
- Do not commit an absolute `build.target-dir` to the repo's `.cargo/config.toml`
  — it is machine-specific and would break other hosts and CI.

#### Worktrees

- A worktree per task is fine; a worktree per task **forever** is not. Reap it
  once the task's branch merges.
- Use `ecaz dev worktree-prune` to see what is reclaimable. It reports only
  unless given `--apply`, and refuses to remove worktrees that are unmerged,
  recently active, or hold uncommitted changes.
- The checkout is shared with other agents. Never remove, reset, or switch a
  worktree or branch you do not own.

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

Promoted current benchmark state lives under `benchmarks/current/<lane>/`.
Those lanes are intentionally mutable indexes for the accepted current result
on each host class (`m5-local`, `intel-local`, `aws-intel`, `aws-graviton`).
Do not use a current lane as the only evidence source: its `manifest.md` must
cite the immutable source packet, head SHA, standard suite config, and raw
artifacts used for promotion.

### Task Closeout Requires 10/50/100k Benchmark Evidence

**A task that changes quantizer, index, scan, rerank, posting, or storage
behavior MUST NOT be closed, promoted, or merged-as-done on static code review,
unit/pgrx tests, or predicted wins alone.** Closeout requires **A/B benchmark
evidence at 10k / 50k / 100k minimum** (recall + latency + storage) for the
**relevant quant/index** affected by the change, produced via `ecaz bench suite`
and stored in the owning packet.

- **Always test and measure; assumptions must be confirmed by facts.** Predictions
  in this codebase frequently fall flat — "this will be faster / this is
  recall-neutral / this win is marginal" is not a finding until a benchmark proves
  it. A recall-safety unit test proves *correctness*, not the *latency/recall
  effect*.
- **A/B per change, at each gate.** Measure the effect of each change in isolation
  (gate on/off, before/after commit, or plain-vs-variant index) so it is clear
  which change moved the bar and which did not. Do **not** stack several changes
  and bench only the aggregate — that destroys per-change attribution.
- **Minimum matrix:** the relevant access method/quantizer × **10k / 50k / 100k**
  (the staged real-corpus scales) × `recall` + `latency` + `storage`. Add the
  variant axis the change introduces (e.g. rerank_format, quant_bits, residual
  on/off, prune on/off). 1m is encouraged when 50k/100k show promise.
- **"Bench deferred to a host" is not a closeout.** The task stays open until the
  evidence lands. Local development hosts that have `ecaz` built + PG18 running +
  the staged corpora (e.g. the Intel desktop) ARE bench hosts — check for the
  binary and `data/staged-current/` before ever claiming env-blocked.
- Evidence storage + provenance follow
  `spec/non-functional/NFR-007-benchmark-provenance.md`; no fabricated numbers,
  every cited result traces to a `results.jsonl` artifact.

### Distributed Measurement: Multi-Node Arms Only

**Any decision about distributed behavior MUST be measured on a multi-node
configuration. A single-node or single-instance arm is NEVER acceptable as the
basis for a decision about a distributed algorithm.**

ec_distann — and any other distributed access method — ships as a distributed
system. A single node is not distributed and never can be. Measuring one and
concluding about the other is a category error, not an approximation.

- **The one permitted use of a single-instance arm:** a labeled baseline that
  quantifies how much overhead distribution adds. Report it, then set it aside.
- **Forbidden uses:** gate, threshold, promotion control, Pareto reference,
  headline result, or the organizing frame of a task, packet, or recommendation.
- **Label node count at every number.** Every latency/recall/storage figure in a
  `request.md`, `manifest.md`, summary table, or report to the operator must
  state its arm's node count. Never place a single-instance number in the same
  table as multi-node numbers without labeling both.
- **Never switch arm classes mid-analysis without saying so.** If earlier tables
  were multi-node and a later comparison introduces a single-instance arm, say so
  at the point of use, not in a footnote.
- **Matched arms or no comparison.** Compared arms must match on beam width, hop
  rounds, head search width / seed count, `top_k`, sweep, corpus, and iteration
  count. The `distann-local-multinode` fixture derives its single control's BW/H
  from the *step* defaults rather than per-variant values
  (`distann_multicluster.rs` pushes the single arm with top-level values and the
  physical arms with per-variant ones), so that row is config-matched to at most
  one variant per run — check before citing it.
- **Loopback is not a network.** `distann-local-multinode` runs every node on one
  host. It is valid for measuring software overhead and invalid for any claim
  about network cost or real multi-host behavior; say which you are measuring.

Precedent, and why this is non-negotiable: Tasks 198/199 promoted a coordinator
traversal replica holding every owner's graph records and full-precision vectors
on one node. It became the program's latency control, so forward work was
measured against a single-node index. Task 190 was closed INVALID, Task 201 was
superseded, and Task 210 (P0) exists to restore sharding. This rule exists so
that class of error cannot recur.

### Benchmark Runner: `ecaz bench suite` Only

**All benchmark matrices, sweeps, and multi-step measurement runs MUST be
driven by `ecaz bench suite` (FR-038) with a JSON `SuiteConfig` checked
into the owning packet.** Do not write new bash sweepers, per-packet
`run-matrix.sh`, or one-off shell glue around `ecaz corpus load` /
`ecaz bench {recall,latency,storage}`.

- The canonical runner lives in `crates/ecaz-cli/src/commands/bench/suite.rs`
  and supports dry-run, resume, audit, status, report, thresholds, and a
  structured `suite-manifest.json` + `results.jsonl`.
- Reusable standard configs live under `crates/ecaz-cli/suites/`, with current
  lane configs under `crates/ecaz-cli/suites/current/`. Prefer running those
  configs with `--artifact-dir` instead of copying task-local suite JSON.
- If `ecaz bench suite` is missing a step type, profile, or option you
  need, extend the suite runner in `ecaz-cli` instead of forking the
  workflow into a script. Land that extension as its own commit before
  using it in a packet.

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

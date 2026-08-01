# Record: commit `d27e2fdde` bundled Task 204/205 work into a docs commit

Date: 2026-07-29. Author: reviewer agent (Agent IX).

## What happened

Commit `d27e2fdde`, message *"review(task-203/001): correct the two-policy
conflation; Task 185 is valid, not a scope gap"*, was intended to contain six
documentation files. It contains **14 files, 519 insertions**, because the
reviewer ran `git add -A` in a shared worktree while another coder had
uncommitted work in flight, and then placed `git diff --cached --stat` in the
same shell command as `git commit` — so the staged file list printed *after* the
commit had already executed and was never read before committing.

No work was lost or altered. The defect is attribution and reviewability: a
wire-protocol change and a benchmark-runner change are recorded under a
documentation commit message, and the code arrived without its own review packet,
without a stated test posture, and without its author's commit message.

Per operator decision the commit is **not** rewritten — the branch is shared with
an active coder and a force-push would be the larger risk. This note is the
durable record instead.

## File attribution within `d27e2fdde`

**Reviewer / Task 203 (the intended contents):**

| File | Change |
| --- | --- |
| `reviews/task-203/001-decision-reaudit/request.md` | correction 2026-07-29-02, two-policy conflation, Task 185 row |
| `plan/tasks/185-ec-distann-gateway-landmark-selection.md` | entry gate on Task 206, boundary against 207 |
| `plan/tasks/207-ec-distann-head-reconstruction.md` | boundary against 185; Task 186 flagged for operator |
| `plan/tasks/README.md` | Task 185 entry updated |

**Task 204 — storage-step arm fidelity (coder):**

| File | Evidence it is 204 |
| --- | --- |
| `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs` | per-arm storage measured inside the variant loop; `derived_relation_bytes`; `SELECT relation_bytes ... wal_bytes` for the replica image; comment "NFR-018/NFR-021 storage is deliberately measured inside the arm loop" |
| `crates/ecaz-cli/src/commands/bench/suite.rs` | suite-side plumbing for the above |

**Task 205 — expansion pushdown (coder):**

| File | Evidence it is 205 |
| --- | --- |
| `src/am/ec_distann/scan.rs` | `struct PushdownLimits { threshold, candidate_limit }`, `derive_pushdown_limits(..)` — the Algorithm 1 controls |
| `src/am/ec_distann/generation_read.rs` | production expander now consuming the threshold |
| `src/am/ec_distann/expand.rs` | legacy expander kept consistent |
| `src/am/ec_distann/remote_transport.rs`, `remote_endpoint.rs` | wire contract for `candidate_limit` |
| `src/am/ec_distann/traversal_replica.rs` | replica expander signature |
| `spec/functional/distann/read/FR-079-distann-remote-expansion-protocol.md` | candidate-limit and owner-side prune/sort/truncate contract |
| `spec/functional/distann/read/FR-081-distann-query-orchestration.md` | per-round threshold derivation |

## For the Task 204 and 205 coders

Cite this note in your packets so the reviewer can follow the code that landed
early. Nothing about the code needs redoing on account of this; what is still
outstanding is what the workflow requires anyway:

- `reviews/task-204/001-arm-fidelity/` with the two-arm demonstration and the
  corrected re-read of the Task 198/199 storage numbers;
- `reviews/task-205/001-contract/` with the FR-079/FR-081 amendment and, critically,
  the **recall-equivalence argument** — the threshold is derived from the
  coordinator's own candidate heap, so it prunes only what the beam would have
  discarded, and that must be shown by ordered-result identity tests against the
  `None` path (ties, tombstones with NULL `exact_dist`, mixed-owner frontiers),
  not asserted;
- the stated test posture for the code in `d27e2fdde`, since the commit records
  none.

## Process fix

In a shared worktree, stage explicitly by filename and read
`git diff --cached --stat` as its **own** command before committing. This repo's
hook picks up unstaged `src/**`, so `git add -A` from a reviewer is unsafe
whenever a coder is active on the same branch.

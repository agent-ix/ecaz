---
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 01
---

# Task 239 packet 003 — exact-main harness port and semantic proof preregistration

Packet 002 review-closed NOT DONE after its only run failed before semantics:
the later Task 224 CLI asserted 40 stage rows against exact-main's 37-row
extension. Reviewer seq03 prohibited a packet-002 rerun and authorized this
single-variable port: exact main plus only the Task 239 harness correction,
followed by code/config/dry review before any live action.

No extension install, PostgreSQL fixture, or live suite is authorized by this
request. Review must explicitly authorize a new one-shot run.

## Frozen checkpoints

- Baseline: exact current main
  `41392c011106cb040095fd6004c4d5c0f136f1a0`.
- Ported harness code: `21c013079723decfecb6880f40d099af5b37d627`.
  The cherry-pick applies every Task 239 semantic/emission hunk and deletes only
  the Task 224 provenance prefix absent on main, retaining main's original
  corpus/query/extension suffix.
- Main-compatible suite config:
  `0adea669ba2dfaacb7c20a81f41af827280e2a48`.
- Frozen validation and proposed build/run checkpoint:
  `4ab2aa9a90f14b045298ac9fe408f9a4b586bf3c`; the only code change after the
  port is a unit test pinning the incremental-emission suppression predicate.
- Suite config:
  `crates/ecaz-cli/suites/task239-main-port-semantics-10k.json`, SHA-256
  `53e13d779e2452a4282f8a076c17eb082396df615efd3e45393d2054257a4532`.

Relative to exact main, the frozen checkpoint changes no `src/**` extension
file. Under `src/**` and `crates/**`, it changes only the Task 239 CLI harness
and the checked-in suite config: 217 insertions / 40 deletions across two files.
The extension and runner therefore share main's 37-row stage schema, and the
only executable delta against packet 001's runner is the Task 239 harness fix.

## Ported correction

1. Every shared-session semantic arm unconditionally sets its batch size.
2. Attribution builds read `current_setting` immediately after each arm's
   settings; seven core rows record control/candidate values, and the mixed and
   outage rows record the candidate value. Missing GUCs fail closed.
3. Each semantic row is emitted immediately with full provenance. Successful
   rows are suppressed from later stdout replay but remain exactly once in the
   summary; a unit test distinguishes these prefixes from the Task 167 skip row
   and ordinary benchmark rows.
4. A failing core row is emitted with `pass=false`, identity digests, effective
   batch settings, work counts, and its bound before returning the error.

The independent recall/latency children retain their existing behavior: eager
0 is explicit; lazy 10 uses the production default `-1`, which resolves to 10
in a fresh child session. This is disclosed inference, not semantic-arm
attestation, and has no bounded-read decision weight.

## Preregistered one-shot surface

After explicit outside authorization only, build and install both the release
extension and release CLI from the clean detached frozen checkpoint
`4ab2aa9a9...`. The extension uses
`pg18,distann-head-attribution-benchmark`; runner and extension SHA must be
identical.

Run the checked-in suite once, without `--continue-on-error`, resume,
selected-step execution, replacement, or config edits. It creates a fresh
three-node, one-index-per-table 10k fixture on ports 44070--44072 at
`/home/peter/.ecaz/clusters/task239-main-port-semantics-10k`. The surface is
the packet-001 attribution lane: staged `ec_real_10k`, query SHA
`a2c191bb...04ae8`, persisted head 32/32, RaBitQ, BW4/H100/L32, eager-0
control, lazy-10 candidate, 200 recall queries, stage counters, and all nine
semantic scenarios. Fault and concurrency drills are skipped. Main has no
routed-drill skip option, so the packet-001 routed DELETE+VACUUM drill remains
enabled. One-iteration timing/storage are diagnostic only.

## Fixed decision gate

The run passes only if the suite exits zero and all of the following hold:

1. Suite manifest runner SHA and unanimous extension SHA both equal
   `4ab2aa9a90f14b045298ac9fe408f9a4b586bf3c`; extension profile is release,
   features are exactly `distann-head-attribution-benchmark,pg18`, and no
   `pg-test` appears. Config hash is exactly
   `53e13d779e2452a4282f8a076c17eb082396df615efd3e45393d2054257a4532`.
2. No Task 167 `candidate_default_quality_gate_failed` skip appears. The main
   log and summary each contain exactly one row for all nine scenarios:
   `fewer_than_window`, `exactly_one_window`, `more_than_window`,
   `reject_first_window`, `reject_multiple_windows`, `null_payload`,
   `toasted_projection_qual`, `mixed_local_remote`, and
   `post_first_batch_remote_failure`. No
   `physical_materialization_correctness` row reports `pass=false`. The
   expected `physical_benchmark_insert_throughput_ab ... pass=false
   reason=single_control_skipped` row does not fail this gate.
3. Every core row reports `control_batch_size=0 candidate_batch_size=10`, exact
   eager/candidate identity, zero duplicates, and its existing bound. Mixed and
   outage rows each report `candidate_batch_size=10` and their scenario-specific
   pass fields.
4. `fewer_than_window`: rows 5, remote requested 6, local consumed 2, payload
   reads 8, bound 10, duplicate 0, digest
   `08efa609...d2017dfc` in both arms.
5. `exactly_one_window`: rows 10, remote requested 6, local consumed 4, payload
   reads 10, bound 10, duplicate 0, digest
   `df979e2d...6cfc77d` in both arms.
6. `mixed_local_remote` returns ten rows with positive local and remote
   consumption summing to ten and zero duplicates. The post-first-batch outage
   returns its first ten-row batch, records positive remote requests, fails
   closed afterward, and records zero duplicates.
7. Both recall children complete at 0.9990 over 200 queries / 2,000 trials, and
   their predictions are byte-identical to each other and packet 001 SHA-256
   `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`.

If every gate passes, classify the result **HARNESS REGRESSION CORRECTED;
EXACT-MAIN LAZY-10 SEMANTIC PATH RESTORED TO 10/10** and request outside
semantic closeout. If any gate or setup step fails, preserve the sole run,
attempt no continuation or replacement, and return to review without changing
the bound.

On a semantic failure, completed/failing rows live in the main multinode log;
the compact summary and `results.jsonl` may not exist. That main log is the
authoritative failure-path evidence.

## Validation

- `cargo fmt --all -- --check`: pass.
- Focused `cargo test -p ecaz-cli materialization_`: 7/7 pass.
- Exact `4ab2aa9a9` release CLI build: pass; one existing unread-field warning.
- Suite audit: pass, one step.
- Dry-run: runner `4ab2aa9a9...`, exact config hash, one selected dry step;
  expanded command contains stage counters, both variants, semantic
  correctness, recall, and no unsupported routed-drill skip.
- Packet-003 live run directory remains absent.

See `artifacts/manifest.md` for commands and final artifact hashes.

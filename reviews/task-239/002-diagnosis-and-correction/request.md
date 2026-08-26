---
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 01
---

# Task 239 packet 002 — semantic harness diagnosis and correction

Packet 001 review-closed the exact-main result: the semantic candidate's 12/10
counter shape is eager-path behavior, not a production lazy-10 regression.
Reviewer seq03 then isolated the first causal harness change. Task 191 at
`7883cfcf8` and Task 198 at `2ff72b3e4` unconditionally set each semantic arm's
batch size and recorded the accepted 6 remote + 4 local = 10 reads. Task 199
commit `241579dfb` changed the helper to omit the `SET` for batch size 10. The
same coordinator session consequently retained the eager control's 0 for the
nominal lazy-10 candidate.

This packet restores the known-good harness contract. It changes no extension
scan, rerank, posting, payload, storage, or other production runtime behavior.
No live install or suite is authorized until an outside reviewer accepts the
code, exact-code dry run, and fixed decision rules below.

## Correction

Code checkpoint `8d8c181b889c8e0b5fb154b639cbfb9bd2ce34a9` changes only
`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`:

1. `materialization_variant_settings_sql` always emits
   `SET ec_distann.benchmark_materialization_batch_size = <variant>`. A shared
   semantic session can no longer inherit the prior arm's value.
2. Immediately after applying each arm's settings and before executing its
   semantic query, the harness reads
   `current_setting('ec_distann.benchmark_materialization_batch_size', true)`.
   The seven core rows record both `control_batch_size` and
   `candidate_batch_size`; mixed-owner and post-first-batch rows record the
   candidate value. Missing attribution GUCs fail closed.
3. Every semantic scenario row is flushed with full corpus/query/build
   provenance as it completes. Successful callers suppress later stdout replay
   but retain the rows once in the summary. A later scenario failure can no
   longer erase preceding passing counters.
4. A failing core scenario emits `pass=false` with digests, effective batch
   settings, counts, and bound before returning its error.

The separate benchmark-child helper intentionally still omits an explicit
batch-size GUC for value 10; those children use independent sessions and the
omission selects the production default `-1`. Packet 002 corrects only the
shared-session semantic helper.

Config checkpoint `d03997c7aef2ff217d0535b47d0b8af765b8500f` adds
`crates/ecaz-cli/suites/task239-corrected-semantics-10k.json`, SHA-256
`bd74199c5fc26d7dffc6b72582915529cbd1c7453ec4ff8fdaad82d7605e6f21`.

## Preregistered corrected matrix

Build and install the release extension from the clean detached packet-001
baseline checkout at exact current main `41392c011...` with features
`pg18,distann-head-attribution-benchmark`. Run the suite with the corrected
release CLI from the separate clean detached checkout at exact `d03997c7a...`.
This is the same extension/CLI split packet 001 used and isolates the CLI-only
harness correction from the six intervening extension-source commits on the
Task 239 branch. The one checked-in suite step creates a fresh three-node,
one-index-per-table 10k fixture at ports 44070--44072 and run directory
`/home/peter/.ecaz/clusters/task239-corrected-semantics-10k`.

The step uses the packet-001 search and corpus surface: staged `ec_real_10k`,
query SHA `a2c191bb...04ae8`, persisted head 32/32, RaBitQ neighbors, BW4,
H100, L32, native/default sender and payload SQL, eager-0 control and
production lazy-10 candidate. It runs recall and the complete nine-scenario
matrix. Fault, concurrency, single-control, and routed DELETE+VACUUM drills are
skipped; the latter is now a recognized CLI option at this checkpoint.

One-iteration latency/storage are diagnostic context only. No timing value or
between-arm delta can pass or fail this packet.

## Fixed decision gate

No `--continue-on-error`, resume, selected-step execution, or replacement run
is allowed.

If a semantic scenario fails, the rows completed before the failure and the
failing `pass=false` row are durable in the step's main
`distann-local-multinode.log`; the compact summary and `results.jsonl` are not
written on that failure path. The main log is therefore the authoritative
failure-path evidence.

The suite passes only if all of the following hold:

1. Preflight is unanimous extension release SHA `41392c011...`, exact features
   `distann-head-attribution-benchmark,pg18`, and no `pg-test`; the live suite
   manifest records runner SHA `d03997c7a...` and the exact config hash. The
   operator must inspect the emitted extension fields; exact SHA/features are
   not pinned by the preflight implementation itself.
2. No Task 167 `candidate_default_quality_gate_failed` skip appears. The main
   log and summary each contain exactly one row for all nine scenarios:
   `fewer_than_window`, `exactly_one_window`, `more_than_window`,
   `reject_first_window`, `reject_multiple_windows`, `null_payload`,
   `toasted_projection_qual`, `mixed_local_remote`, and
   `post_first_batch_remote_failure`. No
   `physical_materialization_correctness` row reports `pass=false`. The
   expected `physical_benchmark_insert_throughput_ab ... pass=false
   reason=single_control_skipped` row comes from the preregistered
   `skip_single_control` setting and does not fail this gate.
3. Every one of the seven eager/candidate rows reports
   `control_batch_size=0 candidate_batch_size=10`, exact result identity, zero
   duplicate requests, and its existing bound. The two additional rows each
   report `candidate_batch_size=10` and their scenario-specific pass fields.
4. `fewer_than_window` reproduces the accepted Task 191/198 shape:
   rows 5, remote requested 6, local consumed 2, payload reads 8, bound 10,
   duplicate 0, digest `08efa609...d2017dfc` in both arms.
5. `exactly_one_window` reproduces the accepted shape: rows 10, remote
   requested 6, local consumed 4, payload reads 10, bound 10, duplicate 0,
   digest `df979e2d...6cfc77d` in both arms.
6. `mixed_local_remote` returns ten rows with positive local and remote
   consumption summing to ten and zero duplicates. The post-first-batch outage
   returns its first ten-row batch, records a positive remote request count,
   fails closed afterward, and records zero duplicates.
7. Both recall children complete at 0.9990 over 200 queries / 2,000 trials and
   their prediction files are byte-identical to each other and packet 001's
   SHA-256
   `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`.

If every gate passes, classify packet 002 as **HARNESS REGRESSION CORRECTED;
PRODUCTION LAZY-10 SEMANTIC PATH RESTORED TO 10/10** and advance to packet 003
for outside semantic closeout. If any gate fails, preserve the original run,
attempt no replacement, and return to review without changing the bound.

Because this is an operator-harness-only correction, the repository's
10k/50k/100k production-behavior A/B closeout rule is not triggered.

## Validation

- New focused regression: 1/1 pass.
- Focused ecaz-cli materialization group: 6/6 pass.
- `cargo fmt --all -- --check`: pass.
- Exact `d03997c7a` release CLI build: pass; only the existing unread
  `LoadedDistributedPlacementConfig.path` warning.
- Exact-code suite audit: pass.
- Exact-code dry run: runner SHA `d03997c7a...`, corrected config hash exact,
  one step with status `dry-run`; expanded command includes stage counters,
  both semantic variants, and the recognized routed-drill skip.

See `artifacts/manifest.md` for commands and artifact hashes.

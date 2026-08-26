---
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 01
---

# Task 239 packet 001 — exact-current-main reproduction preregistration

Task 224 packet 003 returned the correct and byte-identical ten rows for the
native `exactly_one_window` comparison, but its attribution build reported
eight remotely requested plus four executor-local rows: 12 payload reads for a
strict bound of 10. Seq01 review established that the harness's nominal
lazy-10 candidate inherits the eager control's batch-size GUC, so this is an
eager-path counter observation, not yet a production lazy-10 finding. Accepted
Task 191 and Task 198 evidence on the same staged 10k corpus/query recorded 6
remote + 4 local = 10. This packet preregisters the smallest exact-current-main
reproduction before any bound, runtime, or fixture-methodology change.

No live suite, extension installation, or cluster creation is authorized by
this request. The two live lanes remain prohibited until an outside reviewer
explicitly authorizes both the ordered run and the decision rules below.

## Frozen revisions and inputs

- Exact remote `main`: `41392c011106cb040095fd6004c4d5c0f136f1a0`,
  verified again through the GitHub ref API on 2026-08-26.
- Detached clean build/run checkout:
  `/home/peter/dev/ecaz/.worktrees/task239-main-run-build`.
- Corrected config checkpoint: `44c9eac00`; the configs are read by absolute
  path from that worktree, but the runner is invoked with the detached
  exact-main checkout as CWD so `runner_git_commit` remains the exact main SHA.
- Production config:
  `crates/ecaz-cli/suites/task239-current-main-production-10k.json`, SHA-256
  `3ddd441a401feee50b03fd89d5bc1b10cf7c77f6d6f14c260bd85af2f16fcf3f`.
- Attribution config:
  `crates/ecaz-cli/suites/task239-current-main-attribution-10k.json`, SHA-256
  `24b261617eed9a940391dd6ddab433c1ab888d1258a12c67a428cb4495c26292`.
- Corpus prefix `ec_real_10k`; staged manifest SHA-256
  `cb3c68a3090ab4ff767f4e36448e5d90a95ae6416b50265a991d96184d00a561`;
  query SHA-256
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`.

Immediately before the first installation, the operator must compare the
remote `main` ref with `41392c011...`. If it has moved, neither lane may run:
the packet and detached checkout must be amended and rereviewed.

The exact-main preflight enforces a nonempty, unanimous cross-node SHA/profile/
feature tuple, release profile, and absence of `pg-test`; it does **not** pin
the expected SHA or feature list. The operator must therefore read the emitted
`release_profile_preflight` line before accepting either lane and require exact
SHA `41392c011...` plus features `pg18` for lane 1 or
`distann-head-attribution-benchmark,pg18` for lane 2. Any mismatch invalidates
that lane. Lane 1's required seven rows with `attribution_available=false` are
an additional backstop against accidentally installing the attribution build.

## Ordered two-lane run

### 1. Featureless production lane

Build/install the extension from the detached checkout with release `pg18`
only, with no debug or attribution feature. Use the already-built exact-main
release CLI and run the production suite on a fresh three-node, one-index-per-
table fixture at ports 44050--44052. Its run directory is
`/home/peter/.ecaz/clusters/task239-current-main-production-10k`, outside the
repository, and must not exist at start.

This lane is the production-path gate. In a featureless release,
`ec_distann.benchmark_materialization_batch_size` is absent and
`materialization_batch_size()` always returns the production lazy-10 policy.
The two named variants and their two recall arms are therefore the **same
production configuration under two labels**, not an eager/lazy-10 A/B. Task
199's normal-release 50k packet records the same feature-isolation behavior at
`reviews/task-199/003-release-matrix-and-decision/artifacts/repeat-50k/normal-release-ab-50k/distann-multinode-summary.log`.

The lane must emit exactly the seven core
`physical_materialization_correctness` rows:
`fewer_than_window`, `exactly_one_window`, `more_than_window`,
`reject_first_window`, `reject_multiple_windows`, `null_payload`, and
`toasted_projection_qual`. Every row must report eager/candidate result
identity and `attribution_available=false`, and the lane must also emit
`physical_materialization_feature_isolation ... normal_release=true
attribution_hooks_absent=true semantic_scenarios=7`. Recall must complete for
both labels, while being interpreted as two repetitions of the same production
configuration. Zero attribution counters have no decision weight because the
functions are intentionally absent in the normal release.

A zero step exit is insufficient. If the Task 167 candidate-default quality
gate emits `physical_benchmark_materialization_correctness ... pass=skipped
reason=candidate_default_quality_gate_failed`, or if the seven-row set is
missing or duplicated, stop and classify the lane as invalid even if the
process exits zero.

Exact main's routed DELETE+VACUUM drill cannot be disabled and runs after the
semantic capture. If exact provenance, the complete seven-row set, feature-
isolation line, and both recall labels are already captured, a later routed-
drill failure is recorded as **PRODUCTION GATE PASSED; POST-GATE DRILL
FAILED**. It does not erase the captured gate or prohibit lane 2 after cluster
cleanup, and it does not authorize a replacement lane-1 run.

If the production lane fails any provenance, semantic, or recall requirement,
stop immediately. Do not install the attribution build or run lane 2. The
packet is inconclusive and returns to review with the original failure.

After capture, stop and remove the production cluster before installing the
attribution extension.

### 2. Native-sender attribution lane

Only after lane 1 passes, install exact-main release features
`pg18,distann-head-attribution-benchmark`. Run the attribution suite on a
separate fresh three-node, one-index-per-table fixture at ports 44060--44062
and run directory
`/home/peter/.ecaz/clusters/task239-current-main-attribution-10k`.

Exact main predates Task 224's owner-payload-shape, locality-profiler, fast-
sender, and routed-drill skip options. Those four unknown config keys have been
removed rather than relying on serde's silent discard. Main has only the
native sender/payload SQL; the attribution feature supplies the counters that
make the bounded-read signal observable.

The semantic harness does **not** restore the production lazy-10 GUC after its
eager control. It first sets
`ec_distann.benchmark_materialization_batch_size=0`; the candidate's configured
value 10 emits no `SET`, so it inherits 0 on the same session. The attribution
matrix therefore compares eager against eager and its counters describe the
**eager path**, not production lazy-10. Packet 001 deliberately preserves this
exact-main harness behavior; fixing it would dissolve the reproduction
provenance. A harness correction plus direct production-lazy-10 observation is
a named packet 002 item.

The complete nine-scenario matrix is required: the seven core scenarios plus
`mixed_local_remote` and `post_first_batch_remote_failure`, exactly once each.
Every completed row must retain exact result identity, zero duplicate remote
requests, and the existing unqualified bound. Both named recall labels must
complete, but those recall rows are not used to establish the semantic
harness's leaked candidate GUC. The fixture is independent of lane 1, so
cross-lane generation identity is not claimed unless the emitted fingerprints
prove it.

The exact-main routed DELETE+VACUUM drill cannot be disabled and runs after the
semantic matrix. If it fails after a complete semantic matrix, preserve and
classify the already-captured semantic evidence under the rules below; report
the drill failure separately and do not attempt a replacement run. A drill
failure before a complete matrix, or a zero-exit Task 167 quality-gate skip,
makes the lane invalid.

## Fixed decision rules

No `--continue-on-error`, resume, selected-step run, or post-failure replacement
run is allowed in packet 001.

1. If lane 1 passes and lane 2 fails exactly in `exactly_one_window` with
   correct/identical ten rows, `remote_requested=8`, `local_consumed=4`,
   `payload_reads=12`, bound 10, and `duplicate_requested=0`, classify the
   observation as **REPRODUCED — EAGER-PATH COUNTER SHAPE ON EXACT CURRENT
   MAIN**. It is explicitly not evidence of a production lazy-10 regression.
   Proceed to packet 002 to correct/observe the candidate GUC, isolate the true
   production lazy-10 counter shape, and perform targeted call-path diagnosis.
   Do not widen the bound.
2. If lane 2 instead completes all nine scenarios with 10/10 bounded reads for
   `exactly_one_window`, classify it as **NOT REPRODUCED ON EXACT MAIN'S EAGER
   HARNESS PATH**, not as task closeout. The pre-registered next comparison is
   the Task 224 delta that produced the observation: exact checkpoint
   `b834b7fb3715b8fea27d78bbf577c2b47b55d220` / PR #87's integration delta,
   including its +730-line `generation_read.rs` materialization-path change.
   Packet 002 tests that delta directly before attributing the difference to
   fresh-fixture or owner-placement variation.
3. Any other lane-2 failure or counter shape is **INCONCLUSIVE** and returns to
   review with the original artifacts; no post-hoc rerun or rule amendment is
   permitted.

For rules 1 and 2, a routed DELETE+VACUUM failure after the full semantic set
does not erase the classification; the semantic rows precede that drill. A
missing/duplicate scenario set or Task 167 `pass=skipped` line is instead an
invalid lane regardless of process exit.

The one-iteration latency and storage fields are context only and have no
decision weight. Packet 001 changes no runtime behavior, so the 10k/50k/100k
A/B closeout matrix is not triggered. If packet 002 changes scan, rerank,
posting, payload, or storage behavior, Task 239 packet 004 becomes mandatory.

## Validation already complete

- Both corrected checked-in suite configs pass `ecaz bench suite audit` using
  the exact-main release binary, not the Task 239 branch binary.
- Exact-main release `ecaz-cli` built successfully from the detached clean
  checkout.
- Exact-main dry runs expand the intended commands and write manifests whose
  `runner_git_commit` is exactly `41392c011...`; both steps have status
  `dry-run`.
- The production command contains no stage-counter or full-metrics switch. The
  attribution command explicitly enables stage counters. Neither exact-main
  expansion contains the four post-main Task 224 switches, and both expansions
  retain the routed DELETE+VACUUM drill because exact main has no skip option.

## Response to reviewer seq01

1. The attribution candidate is now explicitly classified as eager due to the
   leaked control GUC; packet 001 does not fix the exact-main harness, and rule
   1 cannot label its counters as a production lazy-10 regression.
2. Lane 1 is now a repeated production-path/feature-isolation gate, not an
   eager/lazy-10 A/B; the Task 199 precedent is cited.
3. All four main-unknown config keys are removed. The request no longer claims
   payload-shape or profiler pinning, and it preserves semantic classification
   if the unavoidable post-matrix routed drill fails.
4. Provenance text now distinguishes enforced unanimity/release/no-`pg-test`
   checks from the operator's exact-SHA/features inspection.
5. A clean-main result directs packet 002 to test the Task 224/PR #87 delta
   directly, and both lanes now fail on the zero-exit Task 167 skip condition.

See `artifacts/manifest.md` for commands, hashes, and the packet-local evidence
inventory.

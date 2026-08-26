---
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 01
---

# Task 239 packet 001 — exact-current-main reproduction preregistration

Task 224 packet 003 returned the correct and byte-identical ten rows for the
native `exactly_one_window` eager/lazy-10 comparison, but its attribution build
reported eight remotely requested plus four executor-local rows: 12 payload
reads for a strict bound of 10. Accepted Task 191 and Task 198 evidence on the
same staged 10k corpus/query recorded 6 remote + 4 local = 10. This packet
preregisters the smallest exact-current-main reproduction before any bound,
runtime, or fixture-methodology change.

No live suite, extension installation, or cluster creation is authorized by
this request. The two live lanes remain prohibited until an outside reviewer
explicitly authorizes both the ordered run and the decision rules below.

## Frozen revisions and inputs

- Exact remote `main`: `41392c011106cb040095fd6004c4d5c0f136f1a0`,
  verified again through the GitHub ref API on 2026-08-26.
- Detached clean build/run checkout:
  `/home/peter/dev/ecaz/.worktrees/task239-main-run-build`.
- Task/config branch checkpoint: `81e26f8d0`; the configs are read by absolute
  path from that worktree, but the runner is invoked with the detached
  exact-main checkout as CWD so `runner_git_commit` remains the exact main SHA.
- Production config:
  `crates/ecaz-cli/suites/task239-current-main-production-10k.json`, SHA-256
  `6778beb12f920a413fdd6cca99736616c5b3acbcbc6f0000b57572678ce6f110`.
- Attribution config:
  `crates/ecaz-cli/suites/task239-current-main-attribution-10k.json`, SHA-256
  `2e1b2523bf27877d650bcee362464d0e4dbe22e297a567fd5a2437845a28c3cd`.
- Corpus prefix `ec_real_10k`; staged manifest SHA-256
  `cb3c68a3090ab4ff767f4e36448e5d90a95ae6416b50265a991d96184d00a561`;
  query SHA-256
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`.

Immediately before the first installation, the operator must compare the
remote `main` ref with `41392c011...`. If it has moved, neither lane may run:
the packet and detached checkout must be amended and rereviewed. Both live
suites must attest exact SHA `41392c011...`, release profile, and their expected
feature set on every node; mismatch fails closed.

## Ordered two-lane run

### 1. Featureless production lane

Build/install the extension from the detached checkout with release `pg18`
only, with no debug or attribution feature. Use the already-built exact-main
release CLI and run the production suite on a fresh three-node, one-index-per-
table fixture at ports 44050--44052. Its run directory is
`/home/peter/.ecaz/clusters/task239-current-main-production-10k`, outside the
repository, and must not exist at start.

This lane is the production-path gate. It must exit zero and emit exactly the
seven core `physical_materialization_correctness` rows:
`fewer_than_window`, `exactly_one_window`, `more_than_window`,
`reject_first_window`, `reject_multiple_windows`, `null_payload`, and
`toasted_projection_qual`. Every row must report eager/candidate result
identity and `attribution_available=false`. It must also complete recall for
both eager and lazy-10 variants. Its zero attribution counters have no
decision weight because the functions are intentionally absent in the normal
release.

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

The lane keeps `owner_fast_real_array_send=false`,
`skip_owner_locality_profile=true`, vector-bearing payload shape, and the
production eager/lazy-10 pair. Thus the sender and payload SQL remain native;
the feature supplies only the counters needed to make the bounded-read signal
observable.

The complete nine-scenario matrix is required: the seven core scenarios plus
`mixed_local_remote` and `post_first_batch_remote_failure`, exactly once each.
Every completed row must retain exact result identity, zero duplicate remote
requests, and the existing unqualified bound. Both eager and lazy-10 recall
must complete. The fixture is independent of lane 1, so cross-lane generation
identity is not claimed unless the emitted fingerprints prove it.

## Fixed decision rules

No `--continue-on-error`, resume, selected-step run, or post-failure replacement
run is allowed in packet 001.

1. If lane 1 passes and lane 2 fails exactly in `exactly_one_window` with
   correct/identical ten rows, `remote_requested=8`, `local_consumed=4`,
   `payload_reads=12`, bound 10, and `duplicate_requested=0`, classify the
   observation as **REPRODUCED ON EXACT CURRENT MAIN**. Proceed to packet 002
   for frozen-placement repetition and targeted call-path/bisect diagnosis.
   Do not widen the bound.
2. If lane 2 instead completes all nine scenarios with 10/10 bounded reads for
   `exactly_one_window`, classify it as **NOT REPRODUCED ON ONE FRESH
   FIXTURE**, not as task closeout. Packet 002 repeats the frozen generation and
   placement before any disposition.
3. Any other lane-2 failure or counter shape is **INCONCLUSIVE** and returns to
   review with the original artifacts; no post-hoc rerun or rule amendment is
   permitted.

The one-iteration latency and storage fields are context only and have no
decision weight. Packet 001 changes no runtime behavior, so the 10k/50k/100k
A/B closeout matrix is not triggered. If packet 002 changes scan, rerank,
posting, payload, or storage behavior, Task 239 packet 004 becomes mandatory.

## Validation already complete

- Both checked-in suite configs pass `ecaz bench suite audit`.
- Exact-main release `ecaz-cli` built successfully from the detached clean
  checkout.
- Exact-main dry runs expand the intended commands and write manifests whose
  `runner_git_commit` is exactly `41392c011...`; both steps have status
  `dry-run`.
- The production command contains no stage-counter, full-metrics, owner-shape,
  locality-profile, or fast-sender switch. The attribution command explicitly
  enables stage counters while leaving the native sender and locality profiler
  disabled.

See `artifacts/manifest.md` for commands, hashes, and the packet-local evidence
inventory.

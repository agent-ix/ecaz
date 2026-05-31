# Task 77: SPIRE Candidate Materialization Optimization

Status: complete (2026-05-31, no-slice closeout `reviews/task-77/002-phase1-no-slice-closeout/`)
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (direct follow-up to Tasks 75/76)

## Why

Tasks 73 through 76 established the SPIRE query-time problem at the
high-recall 100k point:

- Task 73 showed SPIRE can reach recall@10 `0.9975-1.0000`, but only by
  paying a large latency cost versus IVF.
- Task 74's Intel profiler showed visible SPIRE self-time is dominated by
  quantized scoring, not by a small routing-orchestration hotspot.
- Task 75's corrected diagnostic rerun showed the high-recall SPIRE point scans
  `15,506,227` leaf candidates over 200 queries while only `5,000` rows survive
  to heap rerank and `2,000` are returned.
- Task 76 showed no safe default change: 100k SPIRE high-recall latency remains
  much slower than IVF at comparable recall, and the local 1M fixture was not
  available to justify an adaptive or quality-preset policy.

The next useful SPIRE optimization work is therefore not "raise the default"
or "tweak routing until recall drops." It is to reduce the cost of producing,
scoring, retaining, and materializing candidates at the same recall floor.

## Outcome

Task 77 landed the SQL-visible attribution hook and the Intel-local Phase 1
measurement packet. The high-recall 100k funnel shows approximate quantized
candidate scoring accounts for roughly `82-83%` of the measured local
candidate-path time, while row materialization plus heap append is only about
`5-6%`. That leaves no bounded SPIRE-local candidate materialization slice with
a defensible path to the task's `>=10%` p50 win gate.

The task therefore closes by the allowed no-slice branch. Follow-up Task 78
owns the scoring-kernel/storage-format work needed to reduce the dominant cost
without changing SPIRE recursion semantics or defaults.

## Non-Goals

- Do not change SPIRE on-disk format or partition-object schema in this task.
- Do not change recursion semantics or the Task 30 route-selection contract.
- Do not change SPIRE defaults or add quality presets; Task 76 closed that
  decision as no-change for the current evidence.
- Do not optimize the shared quantized scoring kernel as the first slice. If a
  fixed-candidate microbench proves the kernel itself is the bottleneck, file
  or link a codec/kernel task rather than hiding that work here.
- Do not use AWS for exploratory loops. Start local Intel, keep AWS shut down
  until a local slice passes its gate and needs representative confirmation.

## Phase 1 - Boundable Candidate Cost Research

Land one measurement packet before changing scan behavior. Required contents:

- Reuse the Task 75 funnel surface at the 100k high-recall points:
  `tg64/nprobe64`, `tg96/nprobe96`, and `tg128/nprobe128`, all with
  `boundary_replica_count=0`.
- Add or capture enough diagnostics to split candidate cost into:
  - leaf route/object reads,
  - candidate row decode/materialization,
  - approximate quantized scoring,
  - heap-retained candidate maintenance,
  - final rerank/top-k handoff.
- For each candidate class, report count, elapsed time where measurable, and
  bytes touched or rows decoded where available.
- Test at least one fixed-candidate microbench or replay mode so a proposed
  slice can distinguish "fewer candidates" from "cheaper candidates."
- Rank candidate P0 slices by expected p50 win and correctness risk.

Phase 1 closes when the packet chooses exactly one of:

- a bounded P0 slice with a defensible recall-preservation argument,
- "candidate/materialization cost is not locally reducible without storage
  format or scoring-kernel work," with evidence,
- a new task split for scoring-kernel or storage-format work.

## Phase 2 - P0 Slices

P0 slices land one at a time. Candidate directions:

1. **Score-bound early termination proof and prototype** - use observed heap
   bounds and approximate score margins to skip scoring work only when exact
   top-k preservation can be argued and measured.
2. **Cheaper discarded-candidate path** - reduce allocation, decode, or
   materialization for candidates that will not survive to heap rerank.
3. **Leaf-local candidate batching** - batch scoring and retention so per-row
   overhead falls without changing selected leaves or recursion semantics.
4. **Safe pre-scoring routing predicate** - only if Phase 1 identifies a
   routing-level signal that rejects leaves before scoring while preserving the
   Task 73/75 recall floor.
5. **Candidate replay/fixed-surface optimizer** - if the best immediate win is
   a reusable benchmark harness needed to evaluate the above slices honestly.

Each slice must:

- preserve Task 73/75 100k high-recall recall@10 within `0.5 pp`;
- preserve 10k recall behavior from Task 76;
- show at least `10%` p50 latency improvement at the matched-recall 100k point,
  or be shelved with a recorded reason;
- include candidate-funnel before/after evidence from the same suite shape;
- include PG18 clippy validation;
- include Intel-local perf or stage-profile evidence if claiming a scan-side
  latency win.

## AWS Confirmation Gate

Do not run AWS during Phase 1 or failed local slices. Run AWS only after a local
slice passes the 100k matched-recall gate and has a reviewer-ready packet.

AWS confirmation must:

- use the same suite shape as the accepted local slice;
- capture recall, latency p50/p95/p99, candidate funnel rows, and relevant
  stage-profile data;
- record cloud status before and after the run;
- shut down or pause profiles immediately after the evidence is captured.

## Exit Criteria

- Phase 1 measurement packet landed with a ranked optimization decision.
- Every Phase 2 P0 slice either landed with measured local win and recall
  preservation, or is shelved with packet-local evidence.
- If a slice lands, AWS confirmation is captured or explicitly deferred with a
  reviewer-accepted reason.
- No new `unsafe { ... }` blocks.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  clean.
- Closeout packet flips `plan/tasks/77-spire-candidate-materialization-optimization.md`
  status to `complete`.

## Coordination

- **Task 75** provides the candidate-funnel diagnostic baseline and no-slice
  routing decision.
- **Task 76** keeps defaults unchanged until candidate/materialization cost is
  reduced or broader evidence changes the policy.
- **Task 74** provides the profiler baseline that scan-side claims must compare
  against.
- **Task 30 Phase 13d** covers distributed production-read efficiency and
  observability; this task covers local high-recall candidate cost first.
- **Task 72** remains independent build-time work.

## Stop Conditions

- Stop a slice if it changes recursion semantics or route ownership rules.
- Stop a slice if it improves p50 by trading away the Task 73/75 recall floor.
- Stop local exploration if Phase 1 shows the remaining gap belongs primarily
  to shared quantized scoring kernels or a storage-format redesign; file that
  task instead of forcing it into SPIRE routing code.

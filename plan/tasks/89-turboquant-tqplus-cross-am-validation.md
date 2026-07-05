# Task 89: TurboQuant TQ+ Cross-AM and Cross-Corpus Validation

Status: proposed (2026-06-07)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (TQ+ validation follow-up to Task 86 slim closeout)

## Why

Task 86 packet 011 measured TurboQuant TQ+ on IVF against
real DBPedia 10k/50k/100k fixtures and showed a clean win:
recall@10 improved +0.75–2.15 pp at every measured cell,
latency improved at every cell, storage near-neutral
(+0.2–1.6 B/row from amortized 12 KiB per-index calibration
overhead).

That's a strong result on a narrow surface. The surface was:
**one AM (IVF), one corpus (DBPedia), one quantizer lane
(no-QJL 4-bit), three scales (10k / 50k / 100k), one seed.**

TQ+ shipped under Task 86 as a new on-disk format tag
(`StorageFormat::TurboQuantTqPlus = 4`) with operator-visible
reloption naming (`storage_format = 'turboquant_tqplus'`).
That is an architecturally significant commit:

- A new on-disk format is durable and hard to revert once
  operators start writing it into CREATE INDEX DDL.
- The format choice (separate tag vs. flag-on-TQ) is
  acknowledged in packet 011's format plan as **tactical**
  — chosen for smaller blast radius and convenient reuse of
  `IvfPqCodebookTuple` storage, not as a principled long-term
  design.

Per the Task 86 final closeout discussion (2026-06-07),
TQ+'s production rollout was deferred from Task 86's
production scope pending broader validation. Task 86
narrowed to ship only the SPIRE TurboQuant LUT routing
(safe cross-AM consistency fix). The TQ+ code is preserved
in git history (commits `e0ae9fe7d`, `c7e85e8ac`,
`16f1e6104`, and successors) as the starting point for
re-landing once this task's validation completes.

This task expands the validation surface to give TQ+ a
defendable production-rollout decision.

## Goal

Produce reviewer-approvable evidence answering one of three
questions:

1. **"Promote TQ+ to production as shipped"** — measured
   wins across multiple AMs + at least one second corpus
   + acceptable streaming-insert drift behavior; separate
   format tag design is the right long-term shape.
2. **"Re-design TQ+ before promotion"** — measurement
   reveals enough surface area to justify either
   consolidating TQ + TQ+ into a single format with a
   calibration flag, or different storage shape than the
   `IvfPqCodebookTuple` chain reuse.
3. **"Defer TQ+"** — measurement reveals a workload class
   where TQ+ regresses materially (recall, latency, or
   storage), and the win on IVF/DBPedia isn't broadly
   representative; file Stop Condition and shelve.

## Why this is hard

- **Cross-AM port**: TQ+ currently lives only in IVF. Porting
  to SPIRE/HNSW/DiskANN requires understanding each AM's
  scoring path and calibration-loading lifecycle. SPIRE and
  IVF share posting-list shapes; HNSW and DiskANN are graph-
  traversal AMs and may need different calibration-load
  timing.
- **Cross-corpus measurement** requires a non-DBPedia
  embedding distribution. Calibration is fit from a sample
  at index build; whether it generalises across embedder
  families (OpenAI / Cohere / image / multilingual) is the
  load-bearing question.
- **Streaming-insert drift** isn't currently measured by any
  existing test. The calibration is fit once at index build;
  if the inserted-row distribution drifts from the training
  sample, recall may degrade. Need to define a drift-detection
  threshold and a realistic insert-proportion stress test.
- **The format-design question** (separate tag vs. flag-on-TQ
  vs. consolidation) is architecturally meaningful and should
  not be answered ad hoc per-AM. Resolve it in Phase 1 ADR
  before further per-AM ports.
- **Backward-compat constraint**: any format consolidation
  must preserve the ability to read TQ-tag-1 indexes and the
  TQ+-tag-4 indexes from packet 011's measurement runs (the
  measurement runs may persist as reference fixtures).

## Scope

### In scope

1. **Format design ADR** — decide separate tag vs. flag-on-TQ
   vs. other consolidation shape. Reviewer-approved before
   Phase 2 starts.
2. **Cross-AM ports**: port TQ+ to SPIRE, HNSW, DiskANN. Per
   the format design ADR, the porting work may either reuse
   the IVF format tag or land a consolidated format.
3. **Cross-corpus measurement** on at least one non-DBPedia
   embedding distribution. Reasonable candidates:
   text-embedding-3-large, Cohere, multilingual-e5, an image
   embedder (whichever is most accessible).
4. **Streaming-insert drift test**: build small (10 k), insert
   10 % / 25 % / 50 % more rows post-build, measure recall
   delta against full-rebuild baseline.
5. **Per-AM real-corpus suite measurement** for each ported
   AM, following Task 86 packet 011's shape (recall@10 +
   p50/p95/p99 + storage at all measured cells).
6. **Final closeout decision** — one of the three Goal
   outcomes, documented with the evidence trail.

### Out of scope

- TurboQuant 2-bit / 8-bit lanes (separate follow-up).
- QJL-enabled lanes (separate follow-up).
- Cross-quantizer comparison (TQ+ vs RaBitQ vs PQ etc.).
- Candidate-batching kernel optimisations (Task 87).
- Streaming ANN result iteration (Task 88).

## Phase 1 — Format Design ADR

Land an ADR before any code port. The ADR must:

- Compare separate-format-tag (current Task 86 packet 011
  shape) vs. flag-on-TQ (calibration metadata byte on tag 1)
  vs. any other consolidation alternative.
- Address backwards-compat: how do existing TQ-tag-1 indexes
  read, and how do existing TQ+-tag-4 indexes (from packet
  011 measurement) read?
- Address operator-visible naming: what `storage_format`
  reloption values exist after this task? Should
  `turboquant_tqplus` survive, become a deprecated alias, or
  rename?
- Address the `IvfPqCodebookTuple` reuse — is the calibration
  storage the right shape long-term, or should it be its own
  page primitive?
- Pre-commit a measurement methodology so per-AM packets
  cite the same suite shape.

ADR must be reviewer-approved before Phase 2 starts.

## Phase 2 — SPIRE TQ+ Port

- Port TQ+ to SPIRE following Phase 1 ADR design.
- Real-corpus suite on DBPedia 10 k / 50 k / 100 k
  (matching Task 86 packet 008 + packet 011 surface).
- Per-AM validation gate: recall byte-equal or improved at
  every cell; latency improves at every cell; storage
  documented and within ADR-specified bounds.

## Phase 3 — HNSW TQ+ Port

- Port TQ+ to HNSW.
- Same real-corpus suite shape.
- Same per-AM validation gate.
- Document any HNSW-specific calibration-load timing concerns
  (HNSW greedy search has different scoring-load patterns
  than IVF posting-list scan).

## Phase 4 — DiskANN TQ+ Port

- Port TQ+ to DiskANN.
- Same real-corpus suite shape.
- Same per-AM validation gate.
- Document any DiskANN-specific concerns (per-page scoring;
  TQ adapter mapping per Task 86 packet 006's surfaced gap).

## Phase 5 — Cross-Corpus Measurement

- Pick at least one non-DBPedia embedding distribution.
- Run all 4 AMs (IVF + SPIRE + HNSW + DiskANN) against the
  second corpus at one fixture scale (10 k or 50 k is
  sufficient; 100 k is bonus).
- Per-cell validation: TQ+ wins (or ties within 0.5 pp
  recall) against TQ on the second corpus.
- If a corpus shows regression: **block promotion on that
  corpus class**; document the regression characteristics
  in the closeout.

## Phase 6 — Streaming-Insert Drift Test

- Build an index at small scale (10 k) with TQ+.
- Insert 10 % / 25 % / 50 % more rows post-build.
- Measure recall@10 delta against a full-rebuild baseline
  at the post-insert size.
- Define drift acceptance threshold in Phase 1 ADR (proposal:
  recall delta ≤ 0.5 pp at 25 % insert; ≤ 1 pp at 50 %).
- If drift exceeds threshold: document the workload class
  where TQ+ shouldn't be used, or propose calibration-refit
  triggers.

## Phase 7 — Closeout Decision

Reviewer-approved closeout packet citing one of:

- **Promote**: per-AM evidence shows TQ+ wins on all 4 AMs +
  second corpus + acceptable drift behavior. ADR-resolved
  format design. Operator guidance documented. Status flips
  to `complete (promoted)`.
- **Re-design**: measurement reveals consolidation is
  warranted; file the consolidation work as Task 89.1 or
  similar; status flips to `complete (re-design)`.
- **Defer**: measurement reveals a workload class where TQ+
  regresses; document the boundary, archive the IVF
  measurement as a reference for the bounded surface where
  it does win, status flips to `complete (deferred)`.

## Validation gate (per AM, per cell)

1. **Recall@10**: byte-equal or improved within 0.5 pp at
   every measured cell vs. pre-TQ+ baseline.
2. **Latency p50 / p95 / p99**: improves or within noise
   (±5 %) at every measured cell.
3. **Storage**: per-index metadata overhead documented;
   per-row delta amortizes to near-zero at production scale.
4. **All existing pg_test surfaces pass** for the AM under
   port.
5. **Suite-driven per FR-038**: `ecaz bench suite` with
   checked-in `suite.json`, baseline source install vs.
   change source install, both columns committed.
6. **Determinism**: deterministic calibration sample for
   fixed seed; per-AM golden test asserting bit-identical
   index pages across rebuilds at the same seed.

## Acceptance criteria

1. Phase 1 ADR reviewer-approved.
2. Per-AM evidence packets for SPIRE, HNSW, DiskANN.
3. Cross-corpus evidence packet.
4. Streaming-insert drift evidence packet.
5. Closeout packet naming one of the three Goal outcomes.
6. Per memory `feedback_no_premature_task_close`: closeout
   only flips status when all gates above are documented as
   pass / scoped-defer / explicit-stop.

### Per-AM completion is non-negotiable

The validation surface includes IVF (from Task 86 packet 011
— count as already validated), plus SPIRE + HNSW + DiskANN
in this task. **All four AMs must be tested.** A partial
result (e.g. "TQ+ works on SPIRE but we didn't port to
HNSW") doesn't satisfy the goal — that's a Promote-with-
caveats outcome that should still document the un-tested
AM's status.

If an AM port reveals a structural blocker (e.g. calibration
metadata can't fit cleanly into the AM's storage layout),
that's evidence for the Re-design or Defer outcome, not
permission to skip the AM.

## Coordination

- **Depends on Task 86 (slim)** being merged so the SPIRE
  LUT routing baseline exists in main.
- **Task 87** (candidate batching) is independent. Task 87
  Phase 1 design must treat TQ+ as a quant type the
  abstraction must accommodate even though TQ+ isn't yet
  promoted; the contract's quant-agnostic shape doesn't
  depend on Task 89's outcome.
- **Task 88** (streaming ANN) is fully independent.
- **pgvectorscale + TurboVec references** unchanged from
  Task 86.
- **The reverted Task 86 TQ+ commits** (`e0ae9fe7d`,
  `c7e85e8ac`, `16f1e6104`, `54e1383f8`, `2817e3b39`,
  `55e492899`, `73de41981`, `d58ff8716`, `74f1a3bf2`) are
  the natural starting point — cherry-pick them onto the
  Task 89 branch and iterate from there.

## Stop conditions

- Per AM, if TQ+ recall regresses materially at any measured
  cell, block the per-AM gate and triage.
- If cross-corpus measurement reveals systematic regression
  on a non-DBPedia distribution, escalate to the Defer or
  Re-design outcome.
- If calibration drift exceeds the Phase 1 ADR threshold at
  realistic insert proportions, document the workload class
  and defer or document operator guidance.
- If Phase 1 ADR can't resolve the format-design question to
  reviewer satisfaction, escalate to project owner before
  any per-AM port starts.

## References

- Task 86 (predecessor, slim): `plan/tasks/86-turboquant-turbovec-improvements.md`
- Task 86 packet 011 (original IVF TQ+ measurement, preserved
  in git history): `reviews/task-86/011-ivf-tqplus-real-spread/`
- Task 86 packet 011 format plan: `reviews/task-86/011-ivf-tqplus-real-spread/artifacts/tqplus-format-plan.md`
- Task 86 packet 010 closeout audit: `reviews/task-86/010-closeout-audit/`
- Task 86 packet 016 final-audit: `reviews/task-86/016-final-audit/`
  (includes reviewer feedback on the original closeout)
- Reverted Task 86 commits (TQ+ work, preserved in git
  history): `e0ae9fe7d`, `c7e85e8ac`, `16f1e6104`, etc.
- pgvectorscale reference: `/Users/peter/dev_bak/pgvectorscale/`

## Estimated size

Medium-large. 2-4 weeks for one coder including ADR, 3 AM
ports, cross-corpus measurement, streaming-insert drift
test, and closeout. The graph-AM ports (HNSW + DiskANN) are
the hardest part because TQ+'s calibration-load lifecycle
needs to fit into greedy-search patterns rather than
posting-list scans.

# Task 89: TurboQuant TQ+ Cross-AM and Cross-Corpus Validation

Status: proposed (2026-06-07); Phase 1 direction selected by
ADR-081 (2026-06-25)
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

TQ+ briefly shipped under Task 86 as a new on-disk format tag
(`StorageFormat::TurboQuantTqPlus = 4`) with operator-visible
reloption naming (`storage_format = 'turboquant_tqplus'`).
That shape is not the Phase 1 starting point:

- A new on-disk format is durable and hard to revert once
  operators start writing it into CREATE INDEX DDL.
- IVF now uses storage-format tag `4` for `coarse_rerank`, so
  the old tag assignment cannot simply be resurrected.
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

ADR-081 selects the next build direction: re-land TQ+ first as
an **IVF-only experimental TurboQuant calibration profile** under
`storage_format = 'turboquant'`, without a public
`turboquant_tqplus` storage format. The first gate must evaluate
more than the no-QJL 4-bit lane; no-QJL was selected for speed,
but Task 89 is a reevaluation of the TurboQuant quality/speed
trade-off if calibration works.

This task expands the validation surface in stages. IVF evidence
comes first; cross-AM production rollout comes only after the IVF
format, mode, and drift gates pass.

## Goal

Produce reviewer-approvable evidence answering one of four
questions:

1. **"Keep TQ+ experimental after IVF"** — IVF evidence is
   promising but limited to a narrow TurboQuant mode, corpus, or
   insert profile. Do not port broadly yet.
2. **"Re-design TQ+ before promotion"** — measurement
   reveals enough surface area to justify either
   consolidating TQ + TQ+ into a single format with a
   calibration flag, or different storage shape than the
   `IvfPqCodebookTuple` chain reuse.
3. **"Defer TQ+"** — measurement reveals a workload class
   where TQ+ regresses materially (recall, latency, or
   storage), and the win on IVF/DBPedia isn't broadly
   representative; file Stop Condition and shelve.
4. **"Promote to cross-AM production validation"** — IVF
   evidence is strong across the measured TurboQuant modes,
   storage overhead is bounded, and insert drift is acceptable.
   Only this outcome starts SPIRE/HNSW/DiskANN ports.

## Why this is hard

- **Mode coverage**: TQ+ must be evaluated beyond no-QJL
  4-bit. The QJL/gamma-aware TurboQuant path and existing IVF
  TurboQuant scoring modes may change the quality/speed trade-off.
- **Cross-AM port**: if IVF passes, porting to
  SPIRE/HNSW/DiskANN requires understanding each AM's scoring
  path and calibration-loading lifecycle. SPIRE and IVF share
  posting-list shapes; HNSW and DiskANN are graph-traversal AMs
  and may need different calibration-load timing.
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
- **The format-design question** (separate public format vs.
  TurboQuant option vs. consolidation) is architecturally
  meaningful. ADR-081 chooses an experimental IVF option first,
  but deliberately defers the final public DDL shape until after
  IVF mode and drift evidence.
- **Backward-compat constraint**: any format consolidation
  must preserve the ability to read TQ-tag-1 indexes and the
  TQ+-tag-4 indexes from packet 011's measurement runs (the
  measurement runs may persist as reference fixtures).

## Scope

### In scope

1. **ADR-081 experimental profile** — TQ+ starts as an IVF-only
   experimental TurboQuant calibration profile, not a public
   `storage_format`.
2. **IVF TQ+ experimental build**: re-land the calibration fit,
   encode, metadata persistence, query preparation, and scoring
   hooks under the experimental profile.
3. **TurboQuant mode coverage**: measure no-QJL 4-bit and the
   reachable QJL/gamma-aware IVF TurboQuant path. Existing
   non-default IVF TurboQuant scoring modes are in scope when the
   harness can reach them without adding new bit-mode surfaces.
4. **Streaming-insert drift test**: build small (10 k), insert
   10 % / 25 % / 50 % more rows post-build, measure recall
   delta against full-rebuild baseline.
5. **Public format recommendation**: after IVF evidence, decide
   whether TQ+ should remain experimental, become a TurboQuant
   option, become a separate public storage format, or defer.
6. **Cross-corpus measurement** on at least one non-DBPedia
   embedding distribution. Reasonable candidates:
   text-embedding-3-large, Cohere, multilingual-e5, an image
   embedder (whichever is most accessible).
7. **Cross-AM ports**: SPIRE, HNSW, and DiskANN only after the
   IVF promotion gate passes.
8. **Final closeout decision** — one of the Goal
   outcomes, documented with the evidence trail.

### Out of scope

- New TurboQuant bit-mode surfaces that are not already reachable
  through existing IVF build/scan paths.
- Cross-quantizer comparison (TQ+ vs RaBitQ vs PQ etc.).
- Candidate-batching kernel optimisations (Task 87).
- Streaming ANN result iteration (Task 88).

## Phase 1 — ADR-081 Experimental Profile

ADR-081 selects the Phase 1 build shape:

- `storage_format = 'turboquant'` remains the family selection.
- TQ+ is selected by an experimental/internal calibration option,
  tentatively `turboquant_calibration = 'tqplus_experimental'`.
- The option is IVF-only.
- No public `storage_format = 'turboquant_tqplus'` lands in this
  phase.
- The implementation must reject unknown TQ+ calibration metadata
  on unsupported builds, following ADR-070's compatibility
  discipline.

## Phase 2 — IVF Experimental TQ+ Build

- Re-land the Task 86 IVF TQ+ calibration code as the starting
  point, but adapt it to current IVF metadata where tag `4` now
  means `coarse_rerank`.
- Fit calibration from deterministic IVF training vectors in
  rotated TurboQuant space.
- Encode vectors in calibrated space using existing TurboQuant
  packed-code layouts.
- Persist calibration metadata separately from packed per-vector
  code bytes.
- Store candidate renormalization without widening the no-QJL 4-bit
  payload; QJL/gamma-aware TQ+ uses the existing posting gamma field
  for residual gamma and appends a 4-byte renormalization scalar to
  the experimental posting payload.
- Prepare queries with inverse calibration and bias handling.
- Route scoring through existing IVF TurboQuant scorers wherever
  possible.
- Add deterministic rebuild tests for fixed seed/training sample.

## Phase 3 — IVF Mode Matrix

- Real-corpus suite on DBPedia 10 k / 50 k / 100 k.
- A/B against uncalibrated TurboQuant for each measured mode.
- Required cells:
  - no-QJL 4-bit;
  - reachable QJL/gamma-aware IVF TurboQuant fixture.
- Additional cells: any existing IVF TurboQuant exact-score mode
  reachable without adding new bit-mode surfaces.
- Report recall@10, p50/p95/p99 latency, query-preparation time
  where available, storage, calibration metadata bytes, and
  per-vector scalar bytes if present.

## Phase 4 — IVF Streaming-Insert Drift

- Build an IVF index at small scale (10 k) with TQ+.
- Insert 10 % / 25 % / 50 % more rows post-build.
- Measure recall@10 delta against a full-rebuild baseline at
  the post-insert size.
- Initial acceptance threshold: recall delta ≤ 0.5 pp at 25 %
  insert and ≤ 1 pp at 50 % insert. Tighten or relax only with
  packet-backed rationale.

## Phase 5 — Cross-Corpus Measurement

- Pick at least one non-DBPedia embedding distribution.
- Run IVF against the second corpus at one fixture scale (10 k
  or 50 k is sufficient; 100 k is bonus) before public format
  promotion.
- Per-cell validation: TQ+ wins (or ties within 0.5 pp
  recall) against TQ on the second corpus.
- If a corpus shows regression: **block promotion on that
  corpus class**; document the regression characteristics
  in the closeout.

## Phase 6 — Public Shape Gate

Reviewer-approved gate packet decides one of:

- keep TQ+ IVF-only and experimental;
- promote a public TurboQuant calibration option;
- introduce a separate public `turboquant_tqplus` storage format
  with a fresh tag/version plan;
- redesign metadata/storage before public exposure;
- defer TQ+.

No SPIRE/HNSW/DiskANN port starts before this gate unless the
project owner explicitly asks for a speculative port.

## Phase 7 — Cross-AM Ports

- Port TQ+ to SPIRE, HNSW, and DiskANN only if Phase 6 promotes
  cross-AM validation.
- Each port gets the same real-corpus suite shape appropriate to
  its AM.
- Document AM-specific calibration-load concerns, especially
  graph traversal timing for HNSW/DiskANN.

## Phase 8 — Closeout Decision

Reviewer-approved closeout packet citing one of:

- **Promote**: evidence shows TQ+ wins on the promoted surface,
  second corpus, and acceptable drift behavior. ADR-resolved
  public format design and operator guidance documented. Status
  flips to `complete (promoted)`.
- **Keep experimental**: IVF evidence is promising but too narrow
  for public exposure or cross-AM work. Status flips to `complete
  (experimental)`.
- **Re-design**: measurement reveals consolidation is
  warranted; file the consolidation work as Task 89.1 or
  similar; status flips to `complete (re-design)`.
- **Defer**: measurement reveals a workload class where TQ+
  regresses; document the boundary, archive the IVF
  measurement as a reference for the bounded surface where
  it does win, status flips to `complete (deferred)`.

## Validation gate (per measured mode, per cell)

1. **Recall@10**: byte-equal or improved within 0.5 pp at
   every measured cell vs. pre-TQ+ baseline.
2. **Latency p50 / p95 / p99**: improves or within noise
   (±5 %) at every measured cell.
3. **Storage**: per-index metadata overhead documented;
   per-row delta amortizes to near-zero at production scale.
4. **All existing pg_test surfaces pass** for IVF under the
   experimental profile; later AM ports inherit the same rule.
5. **Suite-driven per FR-038**: `ecaz bench suite` with
   checked-in `suite.json`, baseline source install vs.
   change source install, both columns committed.
6. **Determinism**: deterministic calibration sample for
   fixed seed; golden test asserting bit-identical
   index pages across rebuilds at the same seed.

## Acceptance criteria

1. ADR-081 recorded and reviewed before public format promotion.
2. IVF experimental TQ+ implementation behind a non-public
   calibration option.
3. IVF mode-matrix evidence covering no-QJL 4-bit and reachable
   QJL/gamma-aware TurboQuant.
4. Cross-corpus evidence packet before public format promotion.
5. Streaming-insert drift evidence packet.
6. Public-shape gate packet.
7. Closeout packet naming one of the Goal outcomes.
8. Per memory `feedback_no_premature_task_close`: closeout
   only flips status when all gates above are documented as
   pass / scoped-defer / explicit-stop.

### Cross-AM completion is a promotion gate

The first build target is IVF. SPIRE, HNSW, and DiskANN are not
Phase 1 prerequisites. They become non-negotiable only for a
claim that TQ+ is a cross-AM production format.

If an AM port reveals a structural blocker (e.g. calibration
metadata can't fit cleanly into the AM's storage layout),
that's evidence for the Re-design or Defer outcome, not
permission to skip the AM.

## Coordination

- **Depends on Task 86 (slim)** being merged so the SPIRE
  LUT routing baseline exists in main.
- **ADR-081** controls the Phase 1 experimental profile.
- **Task 87** (candidate batching) is independent. Task 87
  must treat TQ+ as calibrated TurboQuant rather than a
  separate quantizer family unless Phase 6 chooses a separate
  public storage format.
- **Task 88** (streaming ANN) is fully independent.
- **pgvectorscale + TurboVec references** unchanged from
  Task 86.
- **The reverted Task 86 TQ+ commits** (`e0ae9fe7d`,
  `c7e85e8ac`, `16f1e6104`, `54e1383f8`, `2817e3b39`,
  `55e492899`, `73de41981`, `d58ff8716`, `74f1a3bf2`) are
  the natural starting point — cherry-pick them onto the
  Task 89 branch and iterate from there.

## Stop conditions

- If TQ+ recall regresses materially at any measured IVF mode
  cell, block the promotion gate and triage.
- If cross-corpus measurement reveals systematic regression
  on a non-DBPedia distribution, escalate to the Defer or
  Re-design outcome.
- If calibration drift exceeds the Phase 1 ADR threshold at
  realistic insert proportions, document the workload class
  and defer or document operator guidance.
- If Phase 6 cannot resolve the public format-design question to
  reviewer satisfaction, escalate to project owner before any
  cross-AM port starts.

## References

- Task 86 (predecessor, slim): `plan/tasks/86-turboquant-turbovec-improvements.md`
- Task 86 packet 011 (original IVF TQ+ measurement, preserved
  in git history): `reviews/task-86/011-ivf-tqplus-real-spread/`
- Task 86 packet 011 format plan: `reviews/task-86/011-ivf-tqplus-real-spread/artifacts/tqplus-format-plan.md`
- Task 86 packet 010 closeout audit: `reviews/task-86/010-closeout-audit/`
- Task 86 packet 016 final-audit: `reviews/task-86/016-final-audit/`
  (includes reviewer feedback on the original closeout)
- ADR-081: `spec/adr/ADR-081-tqplus-experimental-calibration-profile.md`
- Reverted Task 86 commits (TQ+ work, preserved in git
  history): `e0ae9fe7d`, `c7e85e8ac`, `16f1e6104`, etc.
- pgvectorscale reference: `/Users/peter/dev_bak/pgvectorscale/`

## Estimated size

Medium for the IVF experimental gate; medium-large if Phase 6
promotes cross-AM validation. The first slice is IVF build,
mode-matrix measurement, cross-corpus check, and insert-drift
evidence. SPIRE/HNSW/DiskANN ports are separate follow-on
phases after the public-shape gate.

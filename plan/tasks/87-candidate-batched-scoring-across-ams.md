# Task 87: Candidate-Batched Scoring Across Access Methods

Status: proposed (2026-06-07)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (kernel-level optimization unlock; follows Task 86)

## Why

Task 86 (TurboQuant improvements) investigation surfaced that
TurboVec-style scoring kernels — 32-vector blocked u8 nibble
LUTs, multi-query lanes, fused top-k — don't transfer to our
AMs because **all four currently score candidates inline,
per-vector, with no batching abstraction**. The kernel ideas
require a contiguous candidate buffer to amortize block setup
and exploit SIMD width.

Reference: pgvectorscale's `StreamingDiskANN` already maintains
a `resort_buffer` (in
`pgvectorscale/src/access_method/scan.rs`) which is
functionally a small candidate batch. The pattern is well-
established in production vector search (FAISS IVF+PQ,
pgvectorscale, reference HNSW implementations).

Per Task 86 closeout (`reviews/task-86/010-closeout-audit/`),
the "32-vector blocked slabs" follow-up was design-rejected
under the current scoring shape because "AM transfer
complexity; flat contiguous slabs do not map directly onto
graph traversal or page-bounded scans." That rejection is
true under inline scoring. **Candidate batching is the
prerequisite that makes block kernels applicable to our AMs.**

Without batching, the scoring kernel is typically 30–70 % of
query wall time (varies by AM and nprobe). With a batched
SIMD block kernel (FAISS-class), the scoring step is
typically 2–5× faster. End-to-end query speedup is therefore
plausibly 1.3–3× — meaningfully larger than Task 86's
shipped 1.04× SPIRE LUT routing.

## Why this is hard

- Multi-AM refactor: touches scoring sites in HNSW, DiskANN,
  IVF, SPIRE — four different traversal patterns.
- HNSW's greedy search is iterative; you can batch within a
  frontier-evaluation step but not across steps (next step
  depends on the score result).
- DiskANN's per-page scan has a natural batch size but
  variable list length.
- IVF + SPIRE posting lists are variable-length — batch
  handles the head; needs scalar tail handling.
- Block kernels typically use SIMD intrinsics — safety bar
  for `unsafe` is real per `feedback_dont_defer_safety_fixes`.
- The abstraction must compose with Task 88 (streaming ANN
  result iteration) without forcing a redesign.

## Goal

Cut per-AM query latency on the scoring share by ≥ 2× on a
real-corpus suite at recall-preserving fixtures. End-to-end
query latency improvement depends on each AM's scoring share
of total query time; measure both per-AM scoring delta and
per-AM end-to-end delta.

## Scope

### In scope

1. Shared `CandidateBatch` abstraction (`src/quant/` or
   `src/am/common/`). **The abstraction itself must be
   quantizer-agnostic** — the batch surface holds opaque
   `(node_id, code_ptr, gamma?)` tuples; only the kernel
   that consumes the batch is quantizer-specific.
2. Per-AM scoring-site refactor in **all four AMs**:
   HNSW, DiskANN, IVF, SPIRE. **No AM may be skipped.**
3. Route the **TurboQuant no-QJL 4-bit** scoring path
   through the abstraction in each AM. This is the path
   Task 86 validated end-to-end on SPIRE; it has the
   broadest existing test coverage and the clearest kernel
   shape.
4. Optionally one new 32-block u8 nibble LUT kernel
   (TurboVec-style); whether to land it in Task 87 vs a
   follow-up depends on the Phase 1 design call.
5. Per-AM real-corpus measurement with baseline + change
   cells (recall@10 + p50/p95/p99 latency + storage).
6. Compose-with-streaming contract documented so Task 88's
   resort-buffer can sit on top of the same abstraction.

### Out of scope

- Streaming ANN result iteration semantics (Task 88).
- TurboQuant TQ+ calibration (separate follow-up).
- On-disk format changes.
- New SIMD intrinsics beyond what existing kernels use,
  unless the Phase 1 design call lands the 32-block kernel
  and that requires new `target_feature` paths (in which
  case the safety bar applies fully).
- **Block kernels for quantizer variants other than TQ
  no-QJL 4-bit** (see "Quantizer scope" below).

### Quantizer scope

The `CandidateBatch` abstraction is quantizer-agnostic by
design — adding a new quant type later must not require an
abstraction redesign. But **kernel work in Task 87 is scoped
to TurboQuant no-QJL 4-bit only.** Other quant types stay on
their current inline scoring paths until follow-up tasks
land their block kernels.

Quant type roadmap (rough ROI ranking from Task 86 analysis;
ordering of follow-ups is a separate prioritisation call):

| Quant type | Task 87? | Notes |
|---|---|---|
| TurboQuant no-QJL 4-bit | **YES** | Task 87 in-scope; what Task 86 already validated on SPIRE |
| TurboQuant no-QJL 2-bit | follow-up | Higher kernel-density potential than 4-bit |
| TurboQuant QJL (any bits) | follow-up | Residual-sign path is more complex |
| RaBitQ | follow-up | Already block-oriented in the literature |
| Binary fingerprint | follow-up | Hamming-distance kernels vectorise trivially |
| PQ | follow-up | Classic FAISS block-kernel territory |
| f32 raw | **never** | Already SIMD-friendly per-vector; batching adds copy overhead with no kernel win |

Phase 1 design must verify the abstraction can host all of
the "follow-up" entries above without redesign — if the
contract bakes in TurboQuant-only assumptions, the design is
wrong and must be revised before Phase 2 starts. Specifically,
the batch's per-candidate metadata must accommodate quant
types with different per-vector side data (e.g. binary has
none; QJL TQ carries gamma + residual signs; RaBitQ carries
rotation metadata).

## Phase 1 — Design Packet

Land one design packet before any AM-level refactor:

- Pick the `CandidateBatch` abstraction shape (struct fields,
  fill/flush contract, lifetime of the underlying code-ptr
  refs, batch-size policy).
- **Verify the abstraction is quantizer-agnostic** by walking
  each follow-up quant type from the Quantizer Scope table
  (TQ 2-bit, TQ QJL, RaBitQ, binary, PQ) and confirming the
  contract can host their per-candidate metadata shapes
  without redesign. If a quant type forces a contract change,
  fix it here, not in a future task.
- Specify how each AM's existing scoring site maps to the
  abstraction (per-AM contract).
- Cross-reference against pgvectorscale's `resort_buffer`
  pattern; document where they meet vs differ.
- Decide whether the TurboQuant no-QJL 4-bit dim-LUT kernel
  alone is enough for Task 87, or whether a new 32-block u8
  nibble LUT kernel also lands now, based on the expected
  per-AM scoring-share win.
- Document the streaming-ANN compose contract so Task 88
  doesn't force a redesign.
- Pre-commit a measurement methodology: which real fixtures,
  which nprobe / `ef` settings, how the scoring share is
  isolated from total query time.

## Phase 2–5 — Per-AM Integration Slices

**One slice per AM**, landed in this order (smallest blast
radius first, then increasing complexity):

### Phase 2: SPIRE integration

- SPIRE's **TurboQuant no-QJL 4-bit** scoring site uses
  `CandidateBatch`. Other SPIRE quant paths (e.g. QJL
  variants) stay on inline scoring.
- Real-corpus suite: real10k / 50k / 100k DBPedia spread,
  baseline-source-install vs change-source-install (Task 86
  packet 008 shape), recall@10 + p50/p95/p99 + storage.
- **Per-AM validation gate**: SPIRE TQ no-QJL 4-bit recall
  byte-equal at every cell; scoring-share latency ≥ 2×
  faster on the batched path; end-to-end latency improves
  at every cell; non-batched SPIRE quant paths show no
  regression.

### Phase 3: IVF integration

- IVF's **TurboQuant no-QJL 4-bit** scoring site uses
  `CandidateBatch`. Other IVF quant paths stay on inline
  scoring.
- Same real-corpus suite shape.
- **Per-AM validation gate**: IVF TQ no-QJL 4-bit recall
  byte-equal at every cell; scoring-share latency ≥ 2×
  faster on the batched path; end-to-end latency improves
  at every cell; non-batched IVF quant paths show no
  regression.

### Phase 4: DiskANN integration

- DiskANN's **TurboQuant no-QJL 4-bit** scoring site uses
  `CandidateBatch` (per-page batches). Other DiskANN quant
  paths (currently grouped-PQ / RaBitQ per Task 86 packet
  006) stay on inline scoring.
- Same real-corpus suite shape.
- **Per-AM validation gate**: DiskANN TQ no-QJL 4-bit
  recall byte-equal at every cell; scoring-share latency
  ≥ 2× faster on the batched path;
  end-to-end latency improves at every cell.

### Phase 5: HNSW integration

- HNSW's **TurboQuant no-QJL 4-bit** scoring site uses
  `CandidateBatch` (per-frontier batches; batching within
  each greedy-search step). Other HNSW quant paths (QJL
  variants, RaBitQ if present) stay on inline scoring.
- Same real-corpus suite shape.
- **Per-AM validation gate**: HNSW TQ no-QJL 4-bit recall
  byte-equal at every cell; scoring-share latency improves
  on the batched path (the per-frontier batch may not hit
  2× because batches are smaller — document the achieved
  factor); end-to-end latency improves at every cell;
  non-batched HNSW quant paths show no regression.

## Phase 6 — Closeout

- All four AM slices reviewer-approved.
- Aggregate measurement comparison (4-AM × 3-corpus matrix).
- Closeout packet citing per-AM evidence.
- Status flip to `complete` referencing the closeout packet.

## Validation gate (per AM, every cell)

1. **Recall@10 byte-equal** at every fixture × nprobe cell vs
   pre-refactor baseline.
2. **Scoring-share latency** (isolated from traversal share)
   measurably faster.
3. **End-to-end p50/p95/p99 latency** improves at every cell.
4. **Storage unchanged** (no format change in scope).
5. **All existing pg_test surfaces pass** for the AM under
   slice.
6. **No new `unsafe` outside the existing SIMD kernel
   boundary** unless Phase 1 design lands the 32-block
   kernel — in which case full safety-doc + anti-pattern B
   compliance.
7. **Suite-driven per FR-038** — `ecaz bench suite` with
   checked-in `suite.json`, baseline source install vs
   change source install columns, both committed in the
   packet.

## Acceptance criteria

1. `CandidateBatch` abstraction lives in a shared module.
2. **All four AMs (HNSW, DiskANN, IVF, SPIRE) route through
   it on their scoring path.** Not "the first two and we'll
   see." Not "three of four with the fourth deferred."
3. Per-AM real-corpus suite evidence ships in each slice's
   packet.
4. All existing pg_test surfaces pass across all four AMs.
5. Closeout packet cites per-AM evidence + aggregate matrix.
6. `plan/tasks/87-…md` status flips to `complete` only
   referencing the closeout packet.

### Per-AM completion is non-negotiable

The task is **not** complete until all four AM slices have
shipped + been reviewer-approved + met the per-AM validation
gate. A partial close (e.g. "shipped SPIRE + IVF, deferring
HNSW + DiskANN") requires:

- An explicit Stop Condition packet naming the per-AM
  blocker (e.g. "HNSW per-frontier batch is too small to
  amortize overhead; measured scoring-share win is < 5 %").
- Reviewer acceptance of the Stop Condition.
- A follow-up task explicitly scoped for the deferred AM(s).

Single-AM ship does **not** satisfy Task 87. The whole point
is cross-AM applicability of the abstraction; shipping it
on one AM and walking away is a Task 86-class kernel-routing
fix, not Task 87.

## Coordination

- **Depends on Task 86** (TurboQuant improvements) closeout
  being merged.
- **Task 88** (streaming ANN) sits on top of this
  abstraction. Sequence Task 87 first; Task 88 reuses the
  `CandidateBatch` shape inside its resort_buffer.
- **TurboQuant TQ+** (Task 86 packet 002 follow-up) is a
  separate effort but will benefit from this abstraction —
  the calibration prototype already produces `Prepared…Query`
  shapes that batch naturally.
- **pgvectorscale** is a read-only reference: clone at
  `/Users/peter/dev_bak/pgvectorscale/`; key files
  `access_method/scan.rs` (resort_buffer pattern) and
  `access_method/graph/mod.rs` (streaming iteration).

## Coder workflow notes

- Phase 1 design packet must land before any AM slice
  starts. Reviewer approves Phase 1 explicitly.
- Each Phase 2–5 slice is its own packet with its own
  real-corpus measurement. No "batched landing" of multiple
  AMs in one packet.
- Per memory `feedback_no_premature_task_close`, the
  reviewer drives to 100 % of acceptance per AM; off-ramps
  require Stop Condition packets.
- Per memory `feedback_dont_defer_safety_fixes`, every new
  unsafe block in any block kernel ships with a `# Safety`
  doc.
- Per memory `feedback_anti_pattern_b_unbounded_lifetime`,
  the abstraction's lifetime contract must not use safe
  `fn(*mut T) -> &'a T` patterns; typed view wrappers expose
  operations.
- Per memory `feedback_coder_push_smoke_checks`, push after
  every slice; run smoke checks (focused `cargo test` +
  `cargo check pg18`) between slices.

## Stop conditions

- Per AM, if measured scoring-share speedup is < 5 % after
  the refactor (sized to whatever the abstraction overhead
  costs), back the AM out and document. A no-op refactor
  must not land in tree.
- If per-AM recall regresses on any cell, BLOCK that AM's
  slice and triage before retrying.
- If the abstraction shape can't compose with Task 88's
  streaming requirement (discovered late), pause Task 87
  and revisit Phase 1.

## References

- Task 86: `plan/tasks/86-turboquant-turbovec-improvements.md`
  (predecessor; surfaced the kernel-vs-access-pattern
  mismatch)
- Task 86 closeout: `reviews/task-86/010-closeout-audit/`
  (32-block kernel deferred for "AM transfer complexity")
- Task 86 packet 001 transferability matrix (per-AM block-
  kernel fit ranking)
- pgvectorscale resort_buffer pattern:
  `/Users/peter/dev_bak/pgvectorscale/pgvectorscale/src/access_method/scan.rs`
- FR-038 (benchmark provenance): every suite checked-in JSON
- ADR-075 (Task 65b stepping stone framing) — similar
  staged-rollout pattern

## Estimated size

Large. 4–6 weeks for one coder including Phase 1 design,
4 AM slices each with their own real-corpus measurement, and
closeout. The HNSW + DiskANN slices are the harder ones
because their batch sizes are smaller and the abstraction
must work for both flat-batch and per-step-batch shapes.

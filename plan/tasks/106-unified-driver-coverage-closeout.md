# Task 106: Unified-Driver Coverage Closeout (Targeted Gap Pass)

Status: complete (2026-06-14, closeout). Evidence: `reviews/task-106/`
packets 001–004; reviewer judged closeable in
`reviews/task-106/004-aws-targeted-bench/feedback/2026-06-14-01-reviewer.md`
(all four §9 gaps + multi-bit per-ISA routing recorded closed/decided in
ADR-077 §9 + matrix §6a/§9; recall parity 90/90 both AWS lanes; AC1–AC4
met). Original proposal (2026-06-12; operator-confirmed during the Task 105
sweep — "we're not going to fix now we just need to note them so we
can do smaller targeted pass after this" → "let's get those oversights
into a new task")
Owner: coder (to be assigned). One coder.
Priority: 2 (after Task 105 closes; before the safety/cleanup/release
track treats the kernel surface as final)

## Why

The Task 105 full-scale sweep sharpened four cells where the
ADR-077 §1 claim — any quant × any index through the one
CandidateBatch/width-cascade surface — does not hold. They are noted
in ADR-077 §9 and the aggregate matrix §6a; this task is the smaller
targeted pass that closes or formally documents each.

## Scope (one slice per gap, independently land-able)

1. **SPIRE × RaBitQ migration** (`src/am/ec_spire/quantizer/mod.rs`):
   `score_payload_ip` scores per-candidate via
   `estimate_ip_scalar_only`; the chunked-max path uses the legacy
   `rabitq.rs` batch estimator; `ec_spire.candidate_batch_scoring` is
   inert on this lane (~0% on/off at every scale, no counters, all
   hosts). Migrate to the rabitq32 driver (`score_rabitq_bits1_batch_for`
   shape), gaining counters, the width cascade, and the scalar
   per-payload path. Family contract: rabitq32 tolerance lane
   (ADR-076; the legacy estimator is the bit-equal-by-construction
   reference where order is preserved). Evidence: suite cells on the
   t105 SPIRE rabitq fixtures (counter rows expected to appear), recall
   parity per contract, e2e A/B.
2. **HNSW × grouped-PQ engagement decision**: traversal scores one
   search-code at a time (`score_grouped_search_code_result`); the
   batch override is parity-test-only (Task 94 documented; M5 + sweep
   confirmed zero engagement). Decide: add a natural traversal batch
   boundary (measure first whether the width distribution justifies
   it — reuse the Task 98 Phase A histogram method) or document as a
   permanent structural exclusion in ADR-077. A measured "skip" is an
   acceptable outcome.
3. **IVF × TQ-QJL engagement diagnosis**: real batch counters at
   Task 97's 512/4096-row fixtures (incl. `isa=sve2` on G4) but zero
   at 10k × 1024-dim profile shapes on every host, while small e2e
   on/off deltas persist. Find the gating condition (posting width,
   format/bits gate in `use_scratch_soa_batch_decode_for_format`,
   rerank config, nlists), then fix-or-document. The diagnosis comes
   first; no fix without it.
4. **SPIRE × pq_fastscan product gap** (owner decision required):
   reloption parses but `encode_assignment_payload` requires a
   persisted grouped-PQ model no fixture flow provides — the index
   cannot be built on any host (Task 104 finding). Options: wire the
   model persistence so the surface is real (then it inherits the
   grouped-PQ kernel family), or mark the reloption rejected-at-parse
   and record a permanent exclusion. Either way the matrix cell stops
   saying "product gap".

### Out of scope

- New kernels, quants, or ISA work (the families are final; this is
  routing/registration/engagement only).
- Re-running the Task 105 full-scale sweep (its evidence stands; new
  cells here get their own packet-local suite evidence on the
  existing snapshot fixtures).
- The bits=2/4/8 RaBitQ kernel question (Task 93 scope decision
  stands).

## Acceptance criteria

1. Each of the four gaps lands as: migrated/fixed with the
   established kernel-evidence gates, or a documented decision with
   measurements — no cell left saying "unknown".
2. ADR-077 §9 and the aggregate matrix §6a updated to closed/decided
   for all four.
3. Recall parity per family contract at every touched cell; counter
   attribution truthful (new counter rows must appear for slice 1).
4. No regression on untouched cells (focused suite A/B on the
   affected fixtures only — staged, quiet-host protocol).

## References

- ADR-077 §9 (canonical gap wording); aggregate matrix §6a
- `reviews/task-105/` packets (sweep evidence + protocols)
- Task 94 task file (HNSW grouped-PQ out-of-scope rationale)
- Task 97 packets (IVF QJL counter evidence at small fixtures)
- Task 104 packet 008 (SPIRE pqfs product-gap finding)

## Estimated size

Small-medium: slice 1 is the only real implementation; 2–4 are
measure/diagnose/decide passes. One coder, a few days including
review rounds.

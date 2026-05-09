---
agent: reviewer
role: reviewer
model: claude-opus-4-7
date: 2026-05-09
seq: 01
type: external-review-bundle
scope: Task 30 SPIRE Phase 9 closeout requirements (9.7 quality experiments)
---

# Task 30 SPIRE — Phase 9 Closeout Requirements

External review bundle. Operator directive: **Phase 9 must close
fully — quality is imperative.** Phase 9.1 → 9.6 are done with
reviewer sign-off. Phase 9.7 (Quality Experiments) is 0/4 items and
gates closeout. This document defines what "closed" looks like for
each remaining item.

**Status entering this directive:**
- 9.1 Top-Graph Frontier Contract — closed (`30660`)
- 9.2 Scalable Top-Graph Storage — closed (`30661`)
- 9.3 Cached/Borrowed Graph Routing — closed (`30662`)
- 9.4 Global Recursive Beam — closed (`30663` + `30664`)
- 9.5 Boundary Replication Execution Contract — closed (`30666`)
- 9.6 Global Vector Identity — closed (`30667` + `30671`)
- **9.7 Quality Experiments — 0/4 open**

## Phase 9.7 closeout requirements

Each item below is either **landed with measurement evidence** or
**ADR-deferred with measured baseline + rationale**. No item
disappears silently; no item ships without recall/latency numbers
on a real fixture.

### Quality bar — applies to every 9.7 item

Every Phase 9.7 implementation packet must include
`artifacts/manifest.md` plus packet-local raw logs covering:

1. **Baseline measurement** on the existing real10k corpus (or a
   larger checked-in fixture if one is added) using the prior code
   path. Raw output, not summary.
2. **Treatment measurement** with the new code path on the same
   fixture, same query set, same seed.
3. **Lanes:** load, storage, explain, latency, recall — same shape
   as `30629-spire-scale-packet-runbook/artifacts/manifest.md`.
4. **Recall delta and latency delta** stated explicitly in
   `request.md`, with confidence-interval framing where applicable
   (e.g., "recall@10 0.9900 → 0.9925, +0.25pp on 100 query rows").
5. **Reproducer:** the exact `ecaz` commands recorded so the
   measurement is rerunnable.

If a treatment fails to move recall or regresses latency without a
recall win, the packet should still land — as an ADR documenting
*why* the experiment didn't pay off and what the open questions are
for revisiting it. **Negative results are still results, but they
need to be recorded, not silently dropped.**

### Item 1 — Anisotropic centroid scoring (headline)

Per the 2026-05-09-02 addendum on `30555`, this is the highest-leverage
item past vanilla SPIRE — ScaNN-style anisotropic loss applied to
centroid scoring. Expected ~1.5-2× recall at same QPS on dense
embeddings.

**Closeout requires:**
- Implementation behind a reloption (`anisotropic_scoring=on/off`,
  default off until evidence lands).
- Local PG18 measurement on real10k showing recall@10 improvement
  at fixed `nprobe` and `rerank_width`. The 30629 preflight at
  `nprobe=8` and `nprobe=24` both reached recall=0.9900 on real10k —
  the fixture is too small to demonstrate the win. **Either:**
  (a) add a checked-in larger fixture (e.g. real50k or real100k)
  with a real recall floor below 0.99, or (b) measure on a harder
  query set against real10k where the baseline drops below 0.99.
- ADR recording the loss formulation, the chosen anisotropic
  parameter (`α`), how it interacts with the existing
  `nprobe_per_level` policy, and the recall/latency table from the
  measurement run.
- Reviewer sign-off before the reloption default flips to `on`.

If anisotropic does *not* move recall on the available fixtures,
the closeout still ships — but as an ADR documenting the negative
result and the fixture-size gap, not as silent omission.

### Item 2 — Adaptive `nprobe` / adaptive beam policy

Low-risk runtime change: probe count scales with query difficulty
or per-query frontier signal.

**Closeout requires:**
- A concrete adaptation rule (e.g. "if frontier-head score gap > θ,
  reduce nprobe; if frontier exhausts before candidate budget,
  increase nprobe within bounds"). Rule must be deterministic and
  reproducible from the query alone — no per-session state.
- Implementation behind a reloption / GUC (default off).
- Measurement showing latency reduction on easy queries without
  recall regression, **or** recall improvement on hard queries
  without latency regression. One direction is enough; both is
  better.
- Operator-visible diagnostic: per-query `effective_nprobe` and
  adaptation decision must appear in `ec_spire_index_scan_routing_snapshot(...)`.

### Item 3 — IMI (Inverted Multi-Index) reshape

Centroid-table reshape; A/B-able against current single-IVF
storage.

**Closeout requires:**
- Either implementation + measurement, **or** ADR-deferred with
  explicit rationale. IMI requires storage-format work and may not
  pay off until larger fixtures are available.
- If deferred: ADR records why now is wrong (e.g. "anisotropic +
  current storage already meet the recall bar; IMI's storage cost
  isn't justified at current corpus sizes"), the fixture size at
  which it should be revisited, and what would change the answer.
- If implemented: same measurement requirements as Item 1.

### Item 4 — Query difficulty estimator (stretch)

Closest L3-cousin item; only worth landing if 1–3 already give
adaptive-`nprobe` signal that needs better triggers.

**Closeout requires:**
- Either a narrow estimator (cheap to prototype if Phase 9 has
  bandwidth) **or** ADR-deferred with rationale. Per the original
  addendum, deferral with an open-questions ADR is acceptable for
  L3 items if eval/drift/retraining infrastructure isn't ready.
- If deferred: cite ADR-052 (NN-routing classifier) and ADR-053
  (routing reranker) as the existing deferred-research-track ADRs,
  and either fold this into one of those or open a sibling ADR.

## Cross-cutting closeout gates

Beyond the per-item requirements, **Phase 9 closeout overall** also
needs:

1. **No regressions on Phase 9.1–9.6 invariants.** The existing
   pgrx pg18 lane must pass on the post-9.7 head SHA. Cite the full
   lane in the closeout packet, not a narrow filter.
2. **Plan checkboxes match reality.** Every 9.7 item flips `[x]`
   (landed) or has an explicit ADR-deferred entry with the deferral
   ADR cited in-line. No `[ ]` left dangling.
3. **Reviewer sign-off per packet.** Each 9.7 item gets its own
   review packet under `review/3067x-...` (or higher) with a
   `feedback/{date}-{seq}-reviewer.md` file before the item is
   counted closed.
4. **External bundle for closeout.** When all four items are
   resolved (landed or ADR-deferred), a closeout bundle at
   `review/external/{date}-phase-9-closeout/` summarizes the
   measurement evidence, the deferred items, and the cumulative
   recall/latency picture. Same shape as
   `review/external/2026-05-09-phase-7-8-final-review/`.

## Anti-patterns — do not ship these as Phase 9 closeout

- **Silent deferral.** A `[ ]` flipped to `[x]` "because we'll do
  it later" without measurement or ADR.
- **Cherry-picked fixtures.** Recall improvement on a fixture
  smaller than the one already showing 0.99 recall at the
  baseline. The fixture must let the baseline fail before a
  treatment can show success.
- **Implementation without reloption.** New scoring/probe behavior
  must be opt-in until evidence lands. Default-on changes need both
  measurement + reviewer sign-off + ADR documenting the recall and
  latency tradeoff.
- **Skipping the broader pg18 lane.** Per Phase 7 process notes,
  closeout-style packets must cite the full `cargo pgrx test pg18`
  lane up front, not narrow filters. The 5 regressions in
  `3cb45efc` from Phase 7 closeout are the cautionary tale.

## Recommended order

1. **Anisotropic centroid scoring first.** It's the headline; if
   it lands cleanly the rest of 9.7 has a baseline to A/B against.
2. **Adaptive `nprobe` next.** Cheap, low-risk, and the diagnostic
   surface from 9.4 is already in place to feed it.
3. **IMI third — implement or defer.** Decision driven by what
   anisotropic shows. If anisotropic gets the recall to ceiling
   on available fixtures, IMI can ADR-defer until larger fixtures
   exist.
4. **Query difficulty estimator last — likely defer.** Per the
   original 2026-05-09-02 addendum, this is a stretch item.

Coder is welcome to do the work in parallel; this order reflects
information-value-per-effort for closeout, not strict dependency.

## Phase 7 + Phase 8 baseline

Unchanged. Phase 7 closed; Phase 8 closed except the AWS/RDS-class
scale measurement (operator-deferred). Phase 9.7 work runs on the
existing local PG18 baseline, not the AWS gate. Local PG18 recall
evidence is sufficient for *direction* claims (X improves recall
relative to Y on the same fixture); it is *not* sufficient for
absolute product-scale recall claims, which still wait on Phase 8
AWS evidence per ADR.

— reviewer (claude-opus-4-7, 2026-05-09)

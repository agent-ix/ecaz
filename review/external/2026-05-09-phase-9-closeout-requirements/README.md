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

**Operator directive (2026-05-09):**
1. **Quality work happens now, on the local main machine.** AWS
   testing is deferred to a final phase much further out, after
   quality is done. AWS is no longer a near-term gate.
2. **Every Phase 9.7 item gets a recorded local baseline** on the
   main machine, regardless of whether the treatment lands or
   defers. The baselines are the durable reference future
   experiments will compare against.
3. **Items that can land cleanly locally must land.** No silent
   skipping.
4. **Items blocked by available-fixture limits get baseline +
   ADR-deferred-treatment.** The treatment waits for a larger
   local fixture or a different query construction, *not* for
   AWS.

The classification per item is in each section below. Each item
has:
- a **baseline requirement** (always required, regardless of
  treatment disposition);
- a **treatment disposition** (land now, or ADR-defer with
  conditions for revisit).

No item disappears silently; ADR-deferred items still need a
written ADR recording the rationale and the conditions that would
revisit the decision.

## Baseline benchmark requirement (applies to all 9.7 items)

Before any 9.7 treatment work — and as a Phase 9 closeout
artifact — record a **canonical local baseline** on the main
machine covering every fixture and lane combination future
experiments will compare against. This is the durable reference;
treatments add A/B columns to it, they do not replace it.

Required:

1. **Per-fixture baseline runs.** For each checked-in corpus
   (currently real10k; add real50k / real100k if available),
   record load + storage + explain + latency + recall lanes using
   the existing pre-9.7 code path. Same shape as the 30629
   preflight manifest.
2. **Per-knob baseline sweeps.** Within each fixture, record
   baseline at the canonical `nprobe` sweep (e.g. 8, 16, 24, 32),
   `rerank_width` (e.g. 0, 25, 50), and any other knob the 9.7
   treatments will vary.
3. **Single baseline packet:** open
   `30676-spire-phase9-quality-baseline` (or next free number) to
   collect these artifacts under
   `artifacts/manifest.md`. Head SHA pinned to the pre-9.7
   checkpoint.
4. **Re-run reproducer.** The exact `ecaz` commands recorded so
   the baseline is rerunnable on the same machine after any
   environment refresh.
5. **Baseline summary in `request.md`** listing the canonical
   recall/latency table at each (fixture, nprobe, rerank_width)
   point. This is the table 9.7 treatment packets will diff
   against.

The baseline packet is a Phase 9 closeout artifact, not a
treatment. It lands once, then 9.7 treatment packets cite it.

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

**Classification: blocked on harder local fixture or harder query
construction.** Local recall@10 on real10k saturates at 0.99 at
both `nprobe=8` and `nprobe=24`, so a recall-improvement
treatment can't be demonstrated against the current baseline. Per
2026-05-09-02 addendum on `30555`, this is the highest-leverage
item past vanilla SPIRE — expected ~1.5–2× recall at same QPS on
dense embeddings.

**Required regardless of treatment disposition:**
- Baseline measurement on real10k (already in the canonical
  baseline packet).
- Baseline on any larger checked-in fixture if added.
- Baseline on a "hard query" subset against real10k if one is
  constructed (queries that fall below 0.95 recall on the
  baseline). This may be cheap to build by selecting query
  vectors with low max-cosine to any corpus row.

**Treatment path A (land now, preferred):**
- Construct a hard-query subset against real10k where baseline
  recall@10 drops below ~0.95, **or** add a checked-in real50k
  fixture, then:
- Implementation behind reloption (`anisotropic_scoring=on/off`,
  default off until evidence lands).
- Treatment measurement showing recall@10 improvement at fixed
  `nprobe` and `rerank_width` against the baseline.
- ADR recording loss formulation, chosen `α`, interaction with
  `nprobe_per_level`, and the recall/latency A/B table.
- Reviewer sign-off before the reloption default flips to `on`.

**Treatment path B (defer treatment, baseline only):**
- ADR recording: implementation deferred until a fixture/query
  set exists locally where baseline recall drops measurably; the
  current saturation gap on real10k; the expected recall delta
  per ScaNN literature; and the conditions under which the
  implementation should land (specifically: when a hard-query
  subset or a larger local fixture exposes the baseline ceiling).
- Plan checkbox flipped `[x]` with the deferral ADR cited inline.
- The canonical baseline packet records the saturation evidence
  so future-you can find the deferral rationale quickly.

### Item 2 — Adaptive `nprobe` / adaptive beam policy

**Classification: must-land locally.** Low-risk runtime change;
adaptation rule is per-query and demonstrable on real10k (easy vs
hard queries within the same fixture). No fixture-size dependency.

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

**Classification: blocked on larger local fixture (defer
treatment, record baseline).** IMI is a storage-format A/B that
mainly pays off at larger fixture sizes; on real10k the existing
single-IVF storage is unlikely to demonstrate a meaningful
storage-cost or recall delta.

**Required regardless of disposition:**
- Baseline storage-format measurement on real10k (already in
  canonical baseline packet, single-IVF).

**Treatment path A (defer, preferred until larger local fixture
exists):**
- ADR-deferred with rationale: storage-format A/B at real10k
  scale doesn't change the answer; decision waits for a larger
  local fixture. Plan checkbox `[x]` with deferral ADR cited.

**Treatment path B (land):**
- Implementation behind reloption (`storage_format=imi/single_ivf`,
  default unchanged). Treatment measurement against baseline.
  ADR recording the storage/recall A/B table.

### Item 4 — Query difficulty estimator (stretch)

**Classification: research-track / defer.** Already framed as
deferrable in 2026-05-09-02. Only worth landing if Item 2
adaptive-`nprobe` signals need better triggers.

**Required regardless of disposition:**
- No baseline-specific work; Item 2's adaptive-nprobe diagnostics
  serve as the input signal for any future estimator design.

**Treatment requires:**
- Either a narrow estimator (cheap to prototype if Item 2 leaves a
  visible gap) **or** ADR-deferred with rationale. Per the original
  addendum, deferral with an open-questions ADR is the expected
  shape.
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

1. **Canonical baseline packet first.** Land
   `30676-spire-phase9-quality-baseline` (or next free number)
   with full per-fixture, per-knob baseline measurements on the
   main machine. Everything else cites this.
2. **Adaptive `nprobe` next.** Cheap, low-risk,
   demonstrable on real10k. Item 2's diagnostic surface from 9.4
   is already in place. Lands as a real treatment.
3. **Hard-query subset construction or larger local fixture.**
   If achievable, this unlocks Item 1 (anisotropic) for real
   treatment landing. If not, go to step 4.
4. **Item 1 disposition.** Either land treatment (if step 3
   gave a fixture/query set with measurable baseline failure) or
   ADR-defer treatment with the saturation evidence cited.
5. **Item 3 IMI disposition.** Almost certainly ADR-defer until
   step 3 exists; the decision rule is the same as Item 1.
6. **Item 4 estimator disposition.** Defer (research track).

Coder may do these in parallel where independent; the order
reflects information-value-per-effort for closeout, not strict
dependency.

## Phase 7 + Phase 8 + AWS gate

- **Phase 7:** closed. Unchanged.
- **Phase 8 (excluding AWS scale measurement):** closed.
  Unchanged.
- **AWS/RDS-class scale measurement:** per 2026-05-09 operator
  directive, deferred to a final phase much further down — after
  quality work (Phase 9 + 10 + any later quality slices) is done.
  Phase 8's scale-packet checkbox stays `[ ]` and the runbook
  scaffold (`30629`) waits.

Local PG18 recall evidence on the main machine is the active
reference for all 9.7 work. Direction claims (X improves recall
relative to Y on the same fixture) require local A/B evidence;
absolute product-scale recall claims wait on the eventual AWS
phase per ADR.

— reviewer (claude-opus-4-7, 2026-05-09)

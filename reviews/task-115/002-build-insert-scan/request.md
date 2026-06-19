# Task 115 / 002 — Phases 2+3: gated residual build/insert + scan integration

Branch: `task-115-ivf-rabitq-residual-quantization`
Code commits:
- `7208d8226` — Phase 2: gated build + insert encoding + metadata flag + reloption
- `00f2655c6` — Phase 3: scan integration + 113 recall-safety + diagnostic
- `15ba3e0f5` — reviewer carry-forward: residual build==insert equivalence test

Builds on packet 001 (Phase 1, APPROVED). Phases 4–5 are env-blocked; ready-to-run
configs ship in this packet's artifacts (see below). 115 is NOT closed — promotion
is bench-gated.

## What landed

### Phase 2 — gated build + insert encoding (default stays plain)

- **Gate:** new `rabitq_residual` reloption (int 0/1, default 0 = plain;
  rejected unless `quantizer='rabitq'`). Flows into `EcIvfOptions.rabitq_residual`
  and is persisted in **metadata byte 35** (`MetadataPage.rabitq_residual`). Plain
  indexes (and any index that wrote 0 there) decode as plain — the recall-safe
  default. Clean two-mode current-build switch, not old-version compat.
- **Encoder:** `IvfQuantizer` carries `rabitq_residual`;
  `encode_source_residual(source, centroid)` (RaBitQ only) returns the same-length
  payload as plain (`encode_code_residual` from packet 001). Plain `encode_source`
  rejects a residual quantizer so a centroid is never silently dropped.
- **Build:** postings are residual-re-encoded against `model.centroids[list_id]`
  inside the per-list loop — the point where the list's centroid is in hand —
  mirroring the existing deferred-PQ re-encode pattern.
- **Insert:** residual re-encode after centroid assignment in
  `insert_into_trained_index` (the centroid is only known post-assignment);
  metadata→options reconstruction carries the flag.

### Phase 3 — scan integration

- Scan resolves the quantizer in residual mode from metadata. The per-list exact
  `⟨q, c⟩` (from the already-loaded centroid scores, indexed by list_id) is carried
  per posting through the SoA and dense scratch and **added to the RaBitQ residual
  estimate** at every scoring site (row direct, row SoA batch, dense block direct,
  dense coalesced batch, scalar fallback) to recover the full `⟨q, o⟩`. The offset
  is `0.0` in plain mode → plain-mode scores stay byte-identical.
- **heap-f32 rerank is untouched** (it rescpres the exact source vector,
  independent of the posting code). Confirmed byte-identical to plain (test below).
- Diagnostic: `debug_ec_ivf_rabitq_residual` exposes whether the index stores
  residual payloads.

### Task 113 coordination — recall-safety (the crux)

113's posting-prune Cauchy-Schwarz cutoff `||o||·||q||/|o_dot|` is a sound upper
bound on the estimate of `⟨q, o⟩` for **plain** payloads. Under residual encoding
the quantized estimate is `⟨q, o − c⟩`, so the sound full-score upper bound is
`⟨q, c⟩ + ||r||·||q||/|r_dot|` — the cutoff would need a per-list `−⟨q, c⟩` shift
the per-payload scoring sites do not carry. Rather than apply the plain-derived
bound to a residual estimate (a recall bug), **residual mode runs the posting scan
UNPRUNED** (`bound_pruning_active = uses_score_bound_pruning() && !rabitq_residual()`).
The shifted-cutoff is the documented follow-up lever; recall is preserved by
construction. Plain mode is unchanged (113 prune still default-on, still byte-safe).

## Reviewer 115/001 carry-forward items — all addressed

1. **Centroid consistency (build = insert = scan):** build/insert encode against
   the same frozen `model.centroids[list_id]`; scan adds `⟨q,c⟩` from the same
   centroids (`load_centroid_scores`). Centroid training untouched.
   `test_ec_ivf_rabitq_residual_build_equals_insert` pins that a row scores
   identically whether encoded at build or via INSERT.
2. **heap-f32 rerank untouched:** `test_ec_ivf_rabitq_residual_heap_f32_rerank_
   matches_plain` proves byte-identical exact top-k vs plain.
3. **Mode gating + flag:** clean two-mode reloption + metadata byte 35.
4. **Residual-mode pruned==unpruned:** `test_ec_ivf_rabitq_residual_posting_bound_
   prune_equals_unpruned` — byte-identical outputs AND zero pruned-by-bound in both
   GUC states (guards the plain-derived bound from ever firing on residual payloads).

## Tested green (pg18, this box)

`artifacts/phase23-residual-pg18-tests.log` (5 pgrx tests):
- `..._coexists_with_plain` — both buildable/scannable; flag distinguishes them.
- `..._heap_f32_rerank_matches_plain` — residual exact top-k == plain.
- `..._posting_bound_prune_equals_unpruned` — residual prune A/B byte-identical, 0 pruned.
- `..._build_equals_insert` — build-encoded twin == insert-encoded duplicate score.
- `..._insert_after_build` — residual insert surfaces the row.

`artifacts/phase23-unit-tests.log`:
- 146 ec_ivf unit tests (incl. `metadata_roundtrips_rabitq_residual_flag`).
- 4 packet-001 residual scalar reference tests (re-confirmed).

`artifacts/cargo-clippy.log`: `--all-targets ... -D warnings` clean.

Regression: plain `test_ec_ivf_posting_bound_prune_equals_unpruned` re-run green
(plain mode unchanged).

Pre-existing failures noted by the task brief (`..._empty_index_build...`,
ec_spire placement) were not touched and are out of scope.

## Phases 4/5 — deferred bench configs (env-blocked, NFR-007)

This box has no `ecaz` binary / no staged corpora, so no numbers are produced.
Ready-to-run configs in `artifacts/`:
- `task-115-residual-recall-per-probe.intel-local.json` — PLAIN vs RESIDUAL
  recall-per-probe, same corpus/nlists/q-set, standard nprobe sweep
  `[8,16,24,32,48,64]`, rerank=heap_f32, reporting recall@10/NDCG@10 + index size +
  build time + counters. Plain vs residual is a **build-time reloption**, so the
  A/B is two indexes (stated in the config `comment`).
- `task-115-residual-matched-recall-latency.intel-local.json` — Phase 5
  matched-recall latency confirmation (placeholder nprobe sweeps to be filled from
  Phase-4 results; run only if recall improves).

**Promotion is bench-gated:** the recall-per-probe win is the entire point of
Task 115 and can only be shown on the Intel bench desktop. Default stays plain
RaBitQ until that evidence lands (Non-Goal: no default change without it).

## Artifacts

- `artifacts/manifest.md`
- `artifacts/phase23-residual-pg18-tests.log`
- `artifacts/phase23-unit-tests.log`
- `artifacts/cargo-clippy.log`
- `artifacts/task-115-residual-recall-per-probe.intel-local.json`
- `artifacts/task-115-residual-matched-recall-latency.intel-local.json`

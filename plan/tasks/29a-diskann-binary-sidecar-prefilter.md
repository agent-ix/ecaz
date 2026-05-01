# Task 29a: DiskANN Binary-Sidecar Prefilter

Status: landed on `main` as part of Task 29
Owner: coder1 / runtime-index track
Backstory: `review/11095-task29-diskann-pgvectorscale-comparison/`
(`feedback.md` for the review, `prefilter-detail.md` for the deep
quant breakdown)

## Goal

Close the DiskANN persisted-scan recall gap from ~0.93 to a target of
≥ 0.97 on the local real-10k corpus by replacing the grouped-PQ4 scan
prefilter with the already-persisted `binary_words` sidecar
(SRHT-rotated sign bits, Hamming popcount distance). Confirm with
`review/11091/11094` repro and aggregate sweep before considering
follow-on cleanup.

This was the required Task 29 recall fix when opened. The binary-sidecar
prefilter landed and the follow-on latency cleanup happened in the same Task 29
lane; grouped-PQ deletion remains out of scope because grouped-PQ is shared
infrastructure and the DiskANN GUC-controlled fallback path.

## Why this work

The probe series in `11087..11094` converged on a sharp diagnosis:

- The build is already optimal — `ambuild.rs:292-294` uses exact f32
  source IP as the build distance. `11090` confirmed the in-memory
  Vamana over the same source vectors gets recall@10 = 0.9995.
- The persisted scan prefilter (grouped-PQ4 at 48 B/node) is the
  ceiling. `11094` confirmed the missing exact IDs `9717` and `7782`
  for query `10001` never enter the candidate frontier under PQ4
  scoring — no rerank-budget tuning can recover them.
- The `tuple.binary_words` sidecar (192 B/node, sign of SRHT-rotated
  coords) is already populated by `ambuild.rs:259-265` for every
  index built with `has_binary_sidecar = true` (the default for any
  dim with `persisted_sidecar_word_count > 0`, which covers 1536 d).
  The scan path has never read it.

The fix is a wire-up, not a new algorithm.

## Size, speed, and quality vs pgvectorscale SBQ

This is the "is it worth it over SBQ" question — short answer: roughly
equivalent in size/speed, with a small quality wash that breaks
favorably for ec_diskann's specific constraints (unit-norm contract,
already-persisted sidecar, no training pass needed).

### Size at 1536 d

| | per-node tuple | metadata pages | per-query state |
|---|---|---|---|
| ec_diskann binary sidecar | 192 B (`binary_words: [u64; 24]`) | seed-derived signs, no I/O | 192 B rotated query bits |
| pgvectorscale SBQ 1-bit | 192 B (1 bit/dim packed) | `mean[1536]` = 6 KiB on metadata chain | 192 B mean-thresholded query bits |

Per-node footprint is identical. Both consume `dim/8` bytes for the
prefilter signal. The ec_diskann path skips the `mean[]` metadata
page (the SRHT signs regenerate from the seed), so total index size
is marginally smaller — ~6 KiB per index, irrelevant.

### Speed

Per-visit score arithmetic is identical: `Σ popcount(query[w] XOR
code[w])` over 24 u64 words. On AVX2 with the unrolled
`distance_xor_optimized` shape from
`pgvectorscale/src/access_method/distance/mod.rs:255-323`, that's
~30 ns per score including loads. Both implementations land at the
same hot-path cost.

Per-query setup differs:

| | setup work | latency contribution |
|---|---|---|
| ec_diskann sidecar | SRHT rotation of 1536-d query + sign-pack | ~15 µs (rotation dominates) |
| pgvectorscale SBQ | read mean[] metadata page + threshold-pack | ~5 µs (metadata read on cold cache) |

ec_diskann is ~10 µs slower per query setup. This is a one-time cost
amortized over hundreds of visits per query, so it's ~3% of a 300 µs
per-query budget at L=200. Not a meaningful difference.

Build-time training cost differs more:

- ec_diskann sidecar: zero. Signs are seed-derived; no training pass.
- pgvectorscale SBQ: Welford online mean over the training sample
  (`SbqQuantizer::add_sample`). One pass over the corpus during
  ambuild.

Skipping the training pass is genuinely useful: it shaves a corpus
scan from build time and removes a degree of freedom from build
parallelism (no shared accumulator).

### Quality

This is where the analysis stops being mechanical. Both encodings
target the same goal — preserve cosine ordering with 1 bit per
dimension — but via different mechanisms.

**pgvectorscale SBQ**: bit i = `v[i] > mean[i]`. The threshold is
*learned* from training samples, so each bit is balanced ~50/50
across the corpus by construction. Adapts to per-coordinate
distribution.

**ec_diskann sidecar**: bit i = `(SRHT(v))[i] > 0`. The rotation
spreads coordinate energy uniformly, so each rotated coordinate is
~zero-mean by construction (under the standard SRHT assumptions).
Then sign on the rotated coord. Data-oblivious. This is literally
Charikar's sim-hash with a structured rotation in place of a random
Gaussian projection — and Charikar's bound `Hamming/dim ≈ θ/π` is
tight for unit-norm vectors.

When the two diverge:

- **SBQ wins** when the corpus has strong axis-aligned structure
  (some coords near-constant; some bimodal at non-zero means).
  Per-coordinate adaptation extracts more information than a
  data-oblivious rotation. Common in older embedding models with
  uneven coordinate magnitudes.
- **Sidecar wins** when the corpus has uniform coordinate
  distributions (which is what learned embeddings *converge to*
  through normalization layers). The SRHT rotation makes the bound
  tight; SBQ's threshold becomes ≈ zero for these corpora and
  loses its adaptive advantage.

For 1536-d learned embeddings (OpenAI, Voyage, Cohere), the
practical recall difference between the two is in the noise floor of
benchmarking. Both should land in the 0.96–0.99 recall@10 range at
L=100 on typical workloads.

The reasons we pick the sidecar path are operational, not
quality-driven:

1. **The bytes are already persisted.** Existing real-10k indexes
   built with `has_binary_sidecar = true` work as-is.
2. **The unit-norm contract is already enforced** (`mod.rs:60-78`).
   Charikar's bound is in scope by design.
3. **No training pass.** Build cost is unchanged.
4. **No metadata page read at scan time.** The signs are
   seed-derived.

If we were greenfielding this and didn't have the sidecar, SBQ would
be the safer default (more workload-tested). But we have it, the
contract supports it, and the wire-up is 50 lines.

### Tied popcount comparison

The user flagged ties as a concern. Concrete numbers for 1536-d
unit-norm vectors:

For a query with true IP `c`, expected Hamming distance to a
candidate with the same IP:
```
E[Hamming] = dim * arccos(c) / π
```
Standard deviation of observed Hamming around that mean (binomial
under random projection):
```
σ ≈ sqrt(dim * p * (1-p))   where p = arccos(c) / π
```

For `dim = 1536`:

| true IP | E[Hamming] | σ | typical "tied window" (±1σ) |
|---|---|---|---|
| 0.95 | 156 bits | 11.6 | 144–167 |
| 0.85 | 271 bits | 14.7 | 256–286 |
| 0.50 | 512 bits | 19.6 | 492–532 |

So for a query whose top-10 true IPs are in [0.85, 0.95], the
candidates land in Hamming buckets across a ~115-bit range with
±15-bit per-vector noise. In a 10k corpus, you typically get 5–15
candidates with Hamming distances within `±σ` of the top-K window.
Concrete implication: a `rerank_budget ≥ 64` covers the noise
window comfortably for top-10 retrieval.

Tie-break mechanism comparison:

- **ec_diskann today** (`scan.rs:89-97`): `ScanCandidate::cmp`
  ties by `block_number` then `offset_number`. Effectively
  arbitrary within a tied popcount bucket — which TID got persisted
  first wins.
- **pgvectorscale**
  (`graph/neighbor_with_distance.rs:74-83`):
  `DistanceWithTieBreak` is more sophisticated *only when distance
  is exactly 0.0*. For non-zero distances it falls back to
  `f32::total_cmp` on the distance value, which has undefined
  ordering between equal floats. So for popcount ties at any
  non-zero Hamming, pgvectorscale is no better than ec_diskann.

**Bottom line on ties**: both implementations have effectively
arbitrary tie-break for popcount-tied non-zero distances. With
`rerank_budget = 64`, the heap rerank handles the noise. If
measurement after the swap shows tie-tail leakage past the budget,
the cheap fix is to add a secondary tie-break via the grouped-PQ
score (still on disk in `tuple.search_code`); the deeper fix would
be to widen the prefilter representation. Neither is needed
speculatively.

### Why prefilter at all (vs pure heap rerank)

The general "is a prefilter worth it" question: yes, by ~10–50× on
visit cost. Without a prefilter, every visited node costs a heap
fetch (random I/O, ~100 µs cold). With a prefilter, only the top
`rerank_budget` candidates pay heap-fetch cost. At L=200,
rerank_budget=64, that's 200 cheap visits + 64 heap fetches vs 200
heap fetches. The prefilter is the difference between an index that
serves vector queries in 100 ms and one that serves them in 1 s.

This is true for both SBQ and the sidecar — they're both prefilters.
The real comparison question is just which prefilter, addressed
above.

## Implementation plan

### Files touched

- `src/am/ec_diskann/scan_state.rs` — add `query_binary_words:
  Vec<u64>` to `DiskannScanOpaque`.
- `src/am/ec_diskann/scan_query.rs` — add
  `pack_query_sign_bits(rotated_query: &[f32], dimensions: usize)
  -> Vec<u64>`. Mirror the index-side derive shape from
  `quant::rabitq::derive_persisted_sidecar_words`.
- `src/am/ec_diskann/routine.rs` — in `ec_diskann_amrescan`
  (~line 613), populate `opaque.query_binary_words` after the SRHT.
  Replace the prefilter closure (~line 664) with a Hamming popcount
  variant gated on `metadata.payload_flags &
  PAYLOAD_FLAG_BINARY_SIDECAR`. Fall back to the existing PQ4
  prefilter when the flag is clear.
- `src/am/ec_diskann/options.rs` — add a session GUC
  `ec_diskann.prefilter_kind` with values `auto` (default; obey
  flag), `binary_sidecar`, `grouped_pq`. Used for A/B measurement
  only; document as a temporary tuning knob.
- `src/am/ec_diskann/scan.rs` — no changes. The prefilter is a
  closure parameter; the scan shell stays distance-agnostic.

### Sketch — query-side packing

```rust
// scan_query.rs
pub(super) fn pack_query_sign_bits(rotated_query: &[f32], dimensions: usize) -> Vec<u64> {
    let word_count = dimensions.div_ceil(64);
    let mut words = vec![0u64; word_count];
    for (i, &value) in rotated_query.iter().take(dimensions).enumerate() {
        if value >= 0.0 {
            words[i / 64] |= 1u64 << (i % 64);
        }
    }
    words
}
```

The threshold (`>= 0.0`) must match the index-side packer
exactly. The current index-side packer is in
`quant::rabitq::derive_persisted_sidecar_words` →
`binary_sign_words_from_packed_no_qjl_4bit`. Confirm bit-for-bit
parity with a unit test that round-trips a vector through both
encode paths and asserts byte equality.

### Sketch — prefilter closure swap

```rust
// routine.rs, in ec_diskann_amrescan, replacing line 664
let use_sidecar = opaque.metadata.payload_flags & PAYLOAD_FLAG_BINARY_SIDECAR != 0
    && options::prefilter_kind_session() != PrefilterKind::GroupedPq;

let results = if use_sidecar {
    let qb: &[u64] = &opaque.query_binary_words;
    scan::vamana_scan_with(
        &reader,
        &mut opaque.visited,
        params,
        |tuple| hamming_xor_popcount(qb, &tuple.binary_words) as f32,
        rerank_closure,
    )
} else {
    scan::vamana_scan_with(
        &reader,
        &mut opaque.visited,
        params,
        |tuple| -grouped_pq_score_f32(&opaque.query_lut, group_count, &tuple.search_code),
        rerank_closure,
    )
};
```

`hamming_xor_popcount` is a transliteration of pgvectorscale's
`distance_xor_optimized`
(`distance/mod.rs:255-323`). For 1536 d → 24 u64 words, a fixed-size
unrolled loop is fine; the SIMD-friendly version is the same shape.

### What stays exactly as is

- The build path. Sidecar is already populated.
- The rerank closure. Still does exact heap IP.
- The greedy descent. Still uses `ScanCandidate` and the existing
  frontier mechanics. (`BinaryHeap` swap is a separate task.)
- Vacuum / insert paths. They already maintain the sidecar.

## Test plan

### Unit

- `pack_query_sign_bits` byte-equality with index-side packer for a
  set of synthetic and corpus-sampled vectors.
- Hamming distance reflexivity, symmetry, and identity-of-equal
  inputs.
- Closure swap: scan with sidecar prefilter on a synthetic 32-node
  graph; assert top-1 matches brute force; assert
  `rerank_budget` cap still bounds rerank calls.

### pgrx integration

- Add `pg_test_ec_diskann_sidecar_prefilter_matches_pq` that loads
  a 200-row real-vector fixture, builds a DiskANN index with
  `has_binary_sidecar = true`, runs the same query under both
  prefilters, and asserts the sidecar variant returns a strict
  superset of the PQ variant's top-10 (modulo tie-break noise) on
  ≥ 90% of queries.
- Reuse the existing real-10k regression harness to assert the
  sidecar variant beats the PQ variant on aggregate recall@10.

### Bench (the actual landing gate)

Use `ecaz-cli` against the existing local PG18 baseline. Three
required measurements:

1. **`11091` SQL-vs-memory compare under sidecar** at
   `list_size = 100`, `rerank_budget = 64`. Query `10001` should
   match exact 10/10. Repeat for the four other sample queries. Log
   to `review/<packet>/artifacts/sql-vs-memory-sidecar.log`.
2. **`11094` grouped-frontier-style probe under sidecar** at
   `list_size = 200`, `rerank_budget = 200`. IDs `9717` and `7782`
   should appear in the top of the simulated frontier (rank ≤ 50).
   Log to `review/<packet>/artifacts/sidecar-frontier.log`.
3. **Aggregate sweep**: 200-query recall@10 / NDCG / mean-latency
   table at `list_size = 64, 100, 200, 400` and
   `rerank_budget = 32, 64, 100, 200`. Log to
   `review/<packet>/artifacts/sidecar-recall-sweep.log`. This is
   the headline result.

## Decision gates

After Step 3 of the bench plan:

- **Recall ≥ 0.97 at `list_size=100, rerank_budget=64`**: land 29a.
  Open Task 29b (latency cleanup: `BinaryHeap` frontier, redundant
  read drop, grouped-PQ deletion).
- **Recall in [0.95, 0.97)**: investigate whether mean-centering
  the sidecar bits closes the remaining gap. This would require a
  build-format change (one new field on metadata page, one
  one-pass training step). Open Task 29a-2 for that.
- **Recall < 0.95**: stop. The Hamming-on-rotated-signs hypothesis
  is wrong for this corpus. Re-investigate before any further
  prefilter changes. Possible causes to check:
  - SRHT seed mismatch between build and scan
  - Sign-bit packing endianness mismatch
  - Unit-norm violation in the corpus

## Risks

1. **Existing indexes built without `has_binary_sidecar`**: not in
   production, no concern. The gated fallback handles correctness
   anyway. Once 29a lands, future builds with the flag clear should
   error rather than silently fall back — flagged as a follow-up
   note for Task 29b.

2. **The unit-norm contract becomes load-bearing for prefilter
   correctness.** Charikar's `Hamming/dim ≈ θ/π` bound assumes
   unit-norm inputs. The validators in `mod.rs:60-78` already
   enforce this with `warn_on_non_unit_source_vector_sample` at
   build time and `warn_on_non_unit_source_vector` at insert time.
   These are warnings, not errors. For Task 29a we should
   *upgrade* the build-time check to an error (already enforced by
   the v0 distance wrapper for unit-norm IP, so this is making
   explicit what was implicit). The size/speed/quality comparison
   above documents that this constraint is the price of the
   sidecar approach over SBQ — if we ever want to support
   non-unit-norm corpora, we'd need to layer the SBQ-style
   mean-centering on top.

3. **Tied popcounts.** The size/speed/quality section above gives
   concrete numbers: typical top-10 candidates land in a ~115-bit
   Hamming range with ~±15-bit noise, so 5–15 candidates per
   "tight" bucket. `rerank_budget = 64` covers this comfortably.
   The aggregate sweep in the test plan exercises four
   `rerank_budget` values; if recall@10 plateaus below the target
   even at `rerank_budget = 200`, ties are a candidate
   contributor. The cheap mitigation is a secondary tie-break via
   the still-persisted `tuple.search_code` (PQ4 score). Do not
   pre-emptively wire this — only if the sweep flags a tie problem.

## Acceptance criteria

- A `review/` packet capturing the three bench measurements above
  with full reproducibility metadata.
- recall@10 ≥ 0.97 at default reloptions on real-10k.
- No latency regression at `list_size = 200` vs the current PQ4
  baseline (expected: equal or slightly faster).
- Either the gated PQ4 fallback removed, or a tracked follow-up
  task to remove it once all production-equivalent indexes have
  been rebuilt (per the no-deprecated-names rule, the fallback
  cannot live in main long-term).
- Task 29b opened with the latency cleanup scope and a pointer to
  the 29a measurements.

## Out of scope (deferred to Task 29b)

- Replacing `Vec<ScanCandidate>` linear-scan frontier with
  `BinaryHeap`.
- Dropping the redundant `read_node` of the picked tuple at
  `scan.rs:290`.
- Removing the grouped-PQ scan path and its codebook chain.
- Decision on whether grouped-PQ becomes a finer second-stage
  filter (only worth investigating if 29b latency numbers show
  heap rerank is meaningful overhead).

# Quant Prefilter — Detailed Breakdown

This is the deep dive on the two prefilter scoring paths. Companion to
`feedback.md` in this packet. Numbers are for 1536-d unit-normalized
embeddings (the ec_diskann real-10k corpus); explicit derivations are
shown so you can substitute different `dim` values.


## 0. The two pieces every prefilter has

Both ec_diskann and pgvectorscale follow the same conceptual shape:

1. **A code stored on each indexed tuple.** This is the per-node
   fingerprint that lives in the page tuple alongside the heap TID and
   the neighbor list. It is read on every visit during greedy descent.
2. **A query-side scoring state** built once at `amrescan`. This is
   the pre-rotated / pre-quantized query plus whatever lookup tables
   are needed to score the persisted code against it.
3. **A `score(code) -> f32` function** that the greedy descent calls
   per node visited.

The cost equation per visit is roughly:

```
visit_cost ≈ page_read + decode + score(code)
            ≈ O(BLCKSZ for first hit, then in cache) + O(tuple_bytes)
              + O(score_arithmetic)
```

For both implementations, the page read dominates wall time on cold
indexes; the score arithmetic dominates on hot indexes. The interesting
axis is *how much information* the code carries about the original
vector relative to its bytes. That information determines recall.


## 1. ec_diskann — grouped-PQ4

### 1.1 Build-time training (`am/common/training.rs`)

`train_grouped_pq4_model(source_vectors, dimensions, seed, group_size,
train_size, kmeans_iters)`:

1. Build an SRHT (subsampled randomized Hadamard) rotation:
   `signs = sign_vector(transform_dim, seed)` — `transform_dim` is 1536
   for the 1536-d corpus (the special-cased tile path keeps it at 1536
   rather than rounding to 2048).
2. Apply the rotation to every training sample. The result is the
   "rotated domain" of length `transform_dim`.
3. Split the rotated 1536-d vector into `group_count = 1536 / 16 = 96`
   groups of `group_size = 16` consecutive dimensions each.
4. For each group independently, run k-means with `K = GROUPED_PQ_CENTROIDS = 16`
   centroids over the training samples' restriction to that group. This
   produces 96 codebooks, each `16 centroids × 16 dims = 256 floats`.

The codebook for group g is `centroids_g ∈ R^{16 × 16}`. Total codebook
storage: `96 × 16 × 16 × 4 B = 96 KiB` (this lives in a chained set of
codebook tuples, indexed off `metadata.grouped_codebook_head`).

### 1.2 Per-node encoding (`derive_grouped_pq4_code`)

Per node, given source vector `v ∈ R^1536`:

1. `r = SRHT(v, signs)` — a length-1536 rotated vector.
2. For each group g, find the nearest centroid:
   `idx_g = argmin_k || r[g*16 .. (g+1)*16] - centroids_g[k] ||²`.
   `idx_g ∈ {0..15}` — fits in 4 bits.
3. Pack the 96 nibbles into 48 bytes via `pack_grouped_pq_nibbles`
   (`code[g/2]` low nibble = even-index, high nibble = odd-index).

Storage shape per node: `search_code: [u8; 48]`.

This means each node's prefilter fingerprint represents its rotated
1536-d vector by a sequence of 96 indices, each pointing to one of 16
trained centroids in a 16-d subspace. Total information content is
`96 × log2(16) = 96 × 4 = 384 bits` — but those bits are *learned*: each
group's 4 bits describe which of 16 centroids that group landed on, not
which side of an arbitrary axis.

### 1.3 Query-side state (`build_grouped_pq_lut_f32`, `routine.rs:596-624`)

Per query at `amrescan`:

1. Rotate the query: `q_r = SRHT(q, signs)` — length 1536.
2. Build a LUT shaped `[group_count × GROUPED_PQ_CENTROIDS]` =
   `96 × 16 = 1536` floats:

   ```
   lut[g*16 + k] = <q_r[g*16 .. (g+1)*16], centroids_g[k]>
   ```

   That is, for each (group g, centroid k) pair, precompute the inner
   product between the query's slice for group g and centroid k of that
   group's codebook.

LUT size: `1536 × 4 B = 6 KiB` per query. Built once per `amrescan`.

### 1.4 Score function (`grouped_pq_score_f32`)

Per node visit:

```
score(code) = Σ_{g=0..95} lut[g*16 + nibble_g(code)]
```

i.e. read 96 nibbles (48 bytes), do 96 indexed loads from the LUT, sum.

Approximate inner product reconstruction:

```
<q, v> ≈ <q_r, r>             (SRHT preserves inner products exactly,
                                up to rotation)
       = Σ_g <q_r[g], r[g]>   (split by group)
       ≈ Σ_g <q_r[g], centroids_g[idx_g]>
       = score(code)          (by definition of the LUT)
```

The `≈` is the source of error: each group replaces the true 16-d slice
`r[g]` with one of 16 centroids `centroids_g[idx_g]`. The reconstruction
error per group is bounded by the k-means quantization error of that
group's training distribution.

### 1.5 What the prefilter is *actually* sensitive to

For the inner product `<q, v>` to be well-preserved, every group's
rotated slice `r[g]` needs to be close to its assigned centroid in a
direction-aware sense. Specifically, the per-group reconstruction error
`e_g = r[g] - centroids_g[idx_g]` enters the score linearly via
`<q_r[g], e_g>`.

Two failure modes:

1. **Cluster collapse**: if many vectors land in the same centroid for
   most groups, the prefilter can't distinguish them. (The 16 centroids
   per 16-d group is generous — k-means usually gets reasonable
   separation here.)
2. **Direction-amplified error**: the query may be near-parallel to the
   error vector for some group, amplifying that group's reconstruction
   noise into the total score. SRHT rotation is supposed to whiten this
   by spreading energy across dimensions, but for adversarial query/
   vector pairs it doesn't fully eliminate it.

Empirically on the real-10k corpus, the recall ceiling at L=200 is
~0.93 (`11088`/`11092`). `11094` showed two specific exact neighbors
(IDs `9717`, `7782`) never enter the L=200 frontier under PQ scoring,
which is exactly mode 2: those nodes have group reconstruction error
that aligns adversarially with the query vector for query `10001`.


## 2. ec_diskann — `binary_words` sidecar (currently dead at scan time)

### 2.1 What's stored

`am/common/training.rs:111` calls `derive_persisted_sidecar_words`
which calls `quantizer.binary_sign_words_from_packed_no_qjl_4bit(code)`
on the rotated, PQ-packed code. The sidecar word count is
`dim.div_ceil(64)` u64s when supported. For 1536 d:

- `sidecar_word_count = 24 u64 = 192 bytes per node`
- Each bit is the sign of one rotated coordinate (positive → 1,
  negative → 0). Effectively `r_i > 0` after the SRHT.

This is exactly the same shape as pgvectorscale's SBQ at 1 bit/dim
(see §3 below), with the difference that the bits are taken on the
SRHT-rotated coordinates rather than directly on the source-vector
coordinates. SRHT preserves inner products, so the rotated bits carry
the same first-order information as taking signs on the unrotated
coordinates would, with the bonus that the rotated distribution is
more uniform (energy spread).

### 2.2 What the matching prefilter would look like

If you wired `binary_words` as the prefilter:

Per query at `amrescan`:
- Rotate the query: `q_r = SRHT(q, signs)` (already happens —
  `routine.rs:613-617` produces `opaque.query_rotated`).
- Quantize to bits: `q_bits[i] = (q_r[i] > 0) ? 1 : 0`, packed into
  `dim.div_ceil(64) = 24` u64s. Reuses
  `quantizer.binary_sign_words_from_packed_no_qjl_4bit` or a direct
  sign-bit pack on the rotated query.

Per node visit:
```
score(code) = popcount_xor(q_bits, tuple.binary_words)   // small = similar
```

This is `Σ_i bit_i(q_r) XOR bit_i(r)` — the Hamming distance between
the sign patterns. As a similarity measure it is a known approximation
to angular distance (and therefore to inner product, when both vectors
are unit-normalized — which the unit-norm validators in
`mod.rs:60-78` already enforce):

```
1 - 2 * Hamming(sign(q_r), sign(r)) / dim ≈ (2/π) * arcsin(<q_hat, v_hat>)
```

i.e. Hamming distance is monotonic in `<q, v>` for unit-norm inputs.
Not perfectly proportional, but monotonic — and that is all a prefilter
needs, because rerank handles the final ordering.

Information content per node: `1536 bits = 192 bytes` of *direct
rotated coordinate signs*, vs PQ4's 384 bits of *learned centroid
indices*. PQ4 has fewer bits but each bit is informationally denser
(picking 1-of-16 in a 16-d space is more information than 1-of-2 in a
1-d space). The empirical question is which preserves IP order *better*
on real workloads — pgvectorscale's published recall numbers and the
0.99-vs-0.93 gap between ec_diskann's in-memory replay and persisted
scan both point at the SBQ-shape being meaningfully better at the
prefilter task than the PQ4-shape at this dim.


## 3. pgvectorscale — SBQ (Scalar Binary Quantization)

### 3.1 Build-time training (`sbq/quantize.rs:115-148`)

`SbqQuantizer::add_sample` runs Welford's online mean+variance over the
training vectors, accumulating `mean[i]` and `m2[i]` per dimension. No
rotation, no k-means.

For 1 bit/dim (the default at ≥ 900 d), only the mean is used. For >1
bit/dim (only allowed at < 900 d), `m2` is used to compute z-scores.

### 3.2 Per-node encoding (`SbqQuantizer::quantize`)

Per node, given source vector `v ∈ R^dim`:

For 1 bit/dim:
```
bit_i(v) = (v[i] > mean[i]) ? 1 : 0
packed into dim/64 u64 words
```

i.e. each bit records whether the source coordinate is above the
training-set mean for that coordinate.

For >1 bit/dim (small-dim only):
```
z_i = (v[i] - mean[i]) / sqrt(m2[i] / count)
index_i = (z_i + 2) / (4 / (bits + 1))    // bin into bits+1 ranges
            in {0, 1, ..., bits}
```
Then `index_i` ones are filled from the LSB end of the `bits` slot for
dimension `i` (thermometer encoding). `bits=2` → 2 bits/dim, three
levels: 00, 01, 11.

For 1536 d at 1 bit/dim: `1536 / 64 = 24` u64s = **192 bytes per node**.

### 3.3 Query-side state (`sbq/mod.rs:139-159`)

Per query at `amrescan`:
```
SbqSearchDistanceMeasure::new(quantizer, query)
  .vec = quantizer.quantize(query.to_index_slice())
```

That is: quantize the query the same way as the index nodes.
For 1 bit/dim: 192 bytes per query, computed once per `amrescan`.

There is no LUT — the query is just the same shape as the stored code,
because the distance is symmetric XOR.

### 3.4 Score function (`distance::distance_xor_optimized`)

Per node visit:
```
score(code) = Σ_w popcount(query_bits[w] ^ code[w])
```

with the loop unrolled per word count for SIMD-friendly codegen
(`distance/mod.rs:255-323`). On x86 with `popcnt`, this is a few cycles
per u64 word — for 1536 d at 24 words, on the order of 30 ns per score
including loads.

Approximate inner product / cosine reconstruction:

For unit-normalized `q`, `v` (or after `preprocess_cosine`),

```
<q_hat, v_hat> ≈ cos(π * Hamming(sign(q), sign(v)) / dim)
                          (Charikar's sim-hash inequality, exact in
                           expectation under random projection)
```

pgvectorscale's SBQ uses the *training-set mean* as the threshold rather
than zero, which sharpens the bit distribution: each bit is balanced
50/50 across the corpus by construction, so two random vectors disagree
on ~50% of bits and the popcount distribution is centered. That is
informationally optimal for binary quantization — every bit carries one
bit of information about the corpus.

### 3.5 What the prefilter is sensitive to

Failure modes:

1. **Coordinates with low variance**: if `mean[i]` is very stable
   across the training set, then almost no node's `v[i]` will be on the
   "rare" side of the mean, so that bit is uninformative. This rarely
   happens for 1536-d learned embeddings (which are usually
   variance-balanced by training).
2. **Tied-popcount frontier**: two nodes at identical Hamming distance
   to the query are tie-broken by `DistanceWithTieBreak` using the
   index pointer. For 192-byte codes at 10k corpus, tied popcounts
   happen but rerank handles it.

In aggregate, SBQ at 1 bit/dim is a worse approximator of any *single*
inner product than grouped-PQ4 in expectation (less compressive but
also less directional information per byte) — but it is a much better
*ranking preserver* on the typical embedding distribution because the
errors are uniformly spread rather than concentrated in adversarial
groups.


## 4. Side-by-side at 1536 d, unit-normalized

| Property | ec_diskann grouped-PQ4 (active) | ec_diskann binary sidecar (latent) | pgvectorscale SBQ 1-bit |
|---|---|---|---|
| Per-node code size | 48 B | 192 B | 192 B |
| Bits of information per node | 384 (96 × 4) | 1536 | 1536 |
| Granularity | 1-of-16 over 16-d slice | 1-of-2 per rotated dim | 1-of-2 per source dim |
| Rotation before quantize? | Yes — SRHT seeded at `seed=42` | Yes — same SRHT | No |
| Mean-centering? | No (centered by k-means initialization) | No (sign of rotated value) | Yes (mean-centered per dim) |
| Codebook? | 96 × 16 × 16 floats = 96 KiB on metadata chain | None | None — just `[mean[i]; dim]` on metadata |
| Query state per `amrescan` | LUT 96 × 16 × 4 B = 6 KiB | 192 B (rotated, sign-packed) | 192 B (mean-thresholded, sign-packed) |
| Score arithmetic | 96 indexed LUT loads + sum | 24 popcount(XOR) | 24 popcount(XOR) |
| Score ≈ what target distance | rotated IP | angular distance | angular distance |
| Symmetric in q/v? | Yes (query also gets LUT'd) | Yes (XOR is symmetric) | Yes (XOR is symmetric) |
| Trained per-corpus? | Yes (k-means) | No | No (only mean is fit, trivially) |

Two takeaways:

- **Information density**. SBQ stores 4× more information per node than
  PQ4 (1536 bits vs 384 bits), and that information is uniformly spread
  across coordinates rather than concentrated on which-of-16 within
  groups. For preserving a global ranking — which is exactly what a
  greedy frontier needs — uniform information density beats peaky
  density.
- **Adversarial concentration**. The PQ4 failure mode is per-group
  reconstruction error aligned with the query, which produces
  catastrophic per-query misses (the `9717`/`7782` symptom). The SBQ
  failure mode is degraded recall in expectation, but no single query
  loses an exact neighbor for arithmetic reasons — only for tie-break
  reasons, which rerank fixes.


## 5. Why the recall numbers come out the way they do

The probe data:

- In-memory Vamana replay over the same 1536-d source vectors with
  exact f32 IP at every step: **recall@10 = 0.9995** (`11090`).
- Persisted ec_diskann at default reloptions
  (`graph_degree=32, build_list_size=100, alpha=1.2`) over the same
  graph with grouped-PQ4 prefilter and exact IP rerank with
  `rerank_budget=64`: **recall@10 ≈ 0.931** (`11088`).
- Persisted ec_diskann with `rerank_budget=200`, `list_size=200`:
  **recall@10 ≈ 0.9845** (`11093`).

The 0.9995 → 0.93 drop is the **prefilter cost** — replacing exact IP
with PQ4 in the greedy traversal. The `11094` probe confirmed this is
not a "rerank window too small" issue: the missing neighbors are not in
the candidate frontier *at all* under PQ4 scoring.

The expected pgvectorscale-style number (SBQ 1-bit on a Vamana built
with exact IP, on this corpus) is somewhere in 0.96–0.99 range based on
their published benchmarks. That is the gap the binary sidecar would
close.


## 6. The wiring change for binary-sidecar prefilter

Concrete sketch of the change at `routine.rs`:

### 6.1 In `amrescan` setup

After `opaque.query_rotated = encode_query_srht(...)` (line 613):

```rust
// Pack the rotated query's sign bits into u64 words. Same shape as
// derive_persisted_sidecar_words on the index side.
opaque.query_binary_words = pack_query_sign_bits(
    &opaque.query_rotated,
    opaque.metadata.dimensions as usize,
);
```

Where `pack_query_sign_bits` walks the rotated f32 array and sets bit i
of word `i / 64` if the rotated coordinate is non-negative. This is a
small new helper (mirrors the index-side packing in
`quant::rabitq::derive_persisted_sidecar_words`).

### 6.2 In the prefilter closure (line 664)

Gate on the sidecar flag and swap the body:

```rust
let prefilter = if (opaque.metadata.payload_flags & PAYLOAD_FLAG_BINARY_SIDECAR) != 0 {
    let qb: &[u64] = &opaque.query_binary_words;
    move |tuple: &VamanaNodeTuple| -> f32 {
        // Hamming distance — small = similar. ScanCandidate orders
        // ascending by score, which matches.
        hamming_xor_popcount(qb, &tuple.binary_words) as f32
    }
} else {
    let qlut: &[f32] = &opaque.query_lut;
    move |tuple: &VamanaNodeTuple| -> f32 {
        -grouped_pq_score_f32(qlut, group_count, &tuple.search_code)
    }
};
```

`hamming_xor_popcount(a, b) = a.iter().zip(b).map(|(x,y)| (x^y).count_ones()).sum::<u32>()`
— the unrolled SIMD-friendly version is in
`pgvectorscale/src/access_method/distance/mod.rs:266-323` and
transliterates straightforwardly.

### 6.3 Rerank stays put

The rerank closure (line 665) does not change. It still fetches the
heap source vector and computes exact `-<q, v>`. The sidecar prefilter
is purely a candidate-frontier scoring change; the final answer is the
same exact IP it always was.

### 6.4 What to measure to confirm

1. Rerun `11091`'s SQL-vs-memory compare. Query `10001` should now
   match exact 10/10 at `list_size ≥ 100`.
2. Rerun `11094`'s grouped-frontier probe with the sidecar prefilter
   variant. Both `9717` and `7782` should appear in the top of the
   simulated frontier.
3. Aggregate sweep across `list_size = 64, 100, 200`. Recall floor
   should move from ~0.93 to ~0.96+.
4. Latency at `list_size = 200`: the score arithmetic is cheaper per
   node (24-word popcount vs 96-nibble LUT lookup), so latency should
   not regress. If anything, slightly faster.


## 7. Side note: PQ4's *real* role

This is not an argument that grouped-PQ4 is bad — it's an argument that
it is the wrong tool for the *traversal-frontier* job at this dimensionality.

Grouped-PQ4 is excellent as a finer second-stage filter:

- 48 B/node is genuinely small enough to be kept in a memory-resident
  cache for an entire 10M-row index.
- The score is a learned IP estimate, more accurate than Hamming for
  unit-norm vectors.
- You can do FastScan-style 32-wide SIMD over packed nibbles, which is
  ~4 ns per code on AVX2.

A two-stage prefilter design — cheap Hamming over `binary_words` to
build the frontier, then PQ4 to refine the top-K of the frontier
before exact rerank — would in theory get the best of both. That is
explicitly outside Task 29 scope; first close the recall gap with the
single-stage Hamming prefilter, then evaluate whether PQ4 as a refiner
buys anything beyond what exact heap rerank already gives.


## 8. References

- Grouped-PQ4 model + LUT: `src/quant/grouped_pq.rs`
- Grouped-PQ4 training: `src/am/common/training.rs:41-105`
- SRHT rotation: `src/quant/rotation.rs`
- Binary sidecar word count + derive:
  `src/quant/rabitq.rs:941-961`,
  `src/am/common/training.rs:107-113`
- ec_diskann scan prefilter call site: `src/am/ec_diskann/routine.rs:664`
- ec_diskann query LUT build: `src/am/ec_diskann/scan_query.rs`,
  `src/am/ec_diskann/routine.rs:596-624`
- pgvectorscale SBQ quantizer: `pgvectorscale/src/access_method/sbq/quantize.rs`
- pgvectorscale SBQ search distance:
  `pgvectorscale/src/access_method/sbq/mod.rs:139-159`
- pgvectorscale XOR popcount: `pgvectorscale/src/access_method/distance/mod.rs:255-323`
- pgvectorscale SBQ default = 1 bit/dim at ≥ 900 d:
  `pgvectorscale/src/access_method/meta_page.rs:312-323`

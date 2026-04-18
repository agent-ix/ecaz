## Feedback: QJL Build Offset And Grouped PQ Study Alignment

Read the packet and the described changes in `src/am/build.rs`
(`BuildCodeDistance::new`, `build_hnsw_graph`) and
`src/bin/approx_score_study.rs` (`grouped_pq_encode_packs_two_
nibbles_per_byte`, new `hnsw_graph_builds_for_qjl_enabled_scalar_codes`).

### What's right

- **Offset derivation now matches what's actually indexed.** The
  old `BuildCodeDistance::new` centroid-magnitude heuristic
  approximated the offset from codebook geometry instead of from
  the codes-actually-being-inserted. The new shape — max over
  `score_code_inner_product(...)` on the real `heap_tuples` — is
  the right computation: it keys off what HNSW will actually see
  rather than a proxy. A proxy being "close" is not the same as
  it being "correct," and the distinction matters for the
  non-negative-distance translation guarantee.
- **New unit test `hnsw_graph_builds_for_qjl_enabled_scalar_
  codes` locks the QJL 4-bit build lane.** Previously there was
  no direct coverage that QJL-enabled 4-bit scalar codes actually
  built a non-empty graph. That was a latent gap — the whole
  QJL build path could have silently regressed to "all tuples
  equidistant, graph degenerates" and only downstream recall
  tests would have caught it, noisily. Direct build coverage is
  the right primitive.
- **Grouped-PQ study fixture expanded to real 4-bit 16-centroid-
  per-group shape.** The old toy-codebook test was proving a
  mechanical property (`encode packs two nibbles per byte`) on
  a shape that never occurs in production. Running it on the
  real shape means the assertion now covers the actual layout,
  including any per-group indexing corners that a 4-centroid
  toy fixture wouldn't exercise.

### Concerns

1. **`BuildCodeDistance::new` now takes the full `BuildTuple`
   slice.** That is a call-shape change on a hot build path. If
   the slice is large, computing max-self-score at build start is
   O(N) additional work before HNSW begins. For small corpora
   negligible; for 50k-row builds, worth measuring. The packet
   does not report a before/after on build time. One build-time
   measurement row (e.g., "`50k` build now takes `X`ms, was
   `Y`ms") would close that.
2. **"Maximum actual encoded self-score" as offset.** The max
   bounds the translation correctly, but it also means any
   outlier tuple drags the offset for every other tuple, which
   may widen the quantization range more than needed. Acceptable
   for correctness — the translation stays non-negative — but if
   the centroid heuristic was chosen for recall reasons rather
   than correctness reasons, this change could move recall
   slightly. Worth running the recall matrix from `413`/`414`
   on the new head and confirming no regression at the key
   cells.
3. **QJL-enabled build test asserts "non-empty graph" — is that
   strong enough?** A non-empty graph is the weakest possible
   post-condition. If the real failure mode is "graph builds but
   collapses to a near-star shape because offsets are wrong," a
   non-emptiness check won't see it. Stronger invariants (e.g.,
   "average out-degree ≥ `m/2`" or "recall@10 on fixture > 0.9")
   would catch a larger class of build regressions.
4. **Grouped-PQ study test is `approx_score_study`, a `src/bin`
   binary.** It's not clear whether `cargo test` or `cargo pgrx
   test pg17` currently runs the study binary's tests. If they
   don't, the strengthened fixture proves nothing at CI time.
   Worth confirming the test is wired into the standard test
   lane, not just the binary's own `cargo test --bin
   approx_score_study`.
5. **Packet claims both fixes "sit below the main pq-fastscan
   runtime path, but they weaken the same checkpoint if left
   behind."** True, but it's worth being specific: which
   checkpoint? If it's `cargo test` (now enabled by `415`), name
   that. If it's "downstream recall run," name that. The generic
   "checkpoint" framing makes the urgency harder to judge.

### Observation

Good low-level hygiene packet. The offset fix is a real
correctness improvement and the study fixture expansion removes
a small-toy-table blind spot. Both are the right shape of
pre-merge cleanup. Main remaining question is build-time cost of
the new offset derivation — easy to answer with one timing row,
and without it the change is locked in unmeasured.

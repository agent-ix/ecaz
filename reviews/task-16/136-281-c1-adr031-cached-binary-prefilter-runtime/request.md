# Review Request: C1 ADR-031 Cached Binary Prefilter Runtime Slice

## Context

Packet `279` showed that ADR-031's sign-derived binary score is promising on the
real `tqhnsw_real_10k` corpus only when candidate binary codes are cached or
stored. On-the-fly binary derivation was slower than the current exact scorer,
so that form should not be integrated into scan execution.

The user chose the cached path first rather than a persisted sidecar format
change. That makes the next runtime slice:

1. prepare a binary query once per scan
2. cache binary codes once per loaded graph element
3. stop exact-scoring every newly loaded graph element eagerly
4. use a conservative binary survivor gate before exact rescoring in the
   layer-0 successor path

## Problem

The current scan cache shape defeats ADR-031 before it starts:

- `CachedGraphElement` no longer stores code bytes after packet `278`
- new live elements are exact-scored eagerly on cache miss and the result is
  inserted into the score cache immediately

That eager exact-score path means a runtime prefilter cannot save work on newly
seen candidates. To make ADR-031 meaningful, exact scoring has to become lazy so
only binary survivors pay the full scorer.

## Implementation

Completed work:

1. add scan-local prepared binary-query support for the no-QJL `1536x4-bit` lane
2. cache sign-derived binary words on `CachedGraphElement`
3. make exact score-cache population lazy instead of mandatory on element load
4. apply a conservative source-local binary prefilter in
   `cached_scan_successor_candidates_for_layer(...)`
5. benchmark the warm verified real-corpus `m=8`, `ef_search=40` cell on a
   release install, including survivor-budget probes at `drop=0`, `drop=2`,
   and `drop=4`

The initial runtime target is intentionally narrow:

- cached only, not persisted
- no format changes
- no beam-level batching yet
- no aggressive survivor budget unless the warm real-corpus surface justifies it

Files changed:

- `src/quant/prod.rs`
- `src/am/scan.rs`

## Outcome

Kept.

Release-verified warm real-corpus results on the current C1 seam:

- `drop=0`: `p50=2.866ms`, `p95=3.614ms`, `p99=3.945ms`, `mean=2.828ms`
- `drop=2`: `p50=2.831ms`, `p95=3.610ms`, `p99=4.213ms`, `mean=2.826ms`
- `drop=4`: `p50=2.865ms`, `p95=3.610ms`, `p99=3.928ms`, `mean=2.824ms`
- `drop=4` confirm: `p50=2.831ms`, `p95=3.599ms`, `p99=3.751ms`, `mean=2.816ms`

Interpretation:

- the cached ADR-031 path is a real warm-latency win on the verified real
  corpus seam
- the dominant win comes from making exact scoring lazy for newly loaded graph
  elements instead of exact-scoring every live element eagerly on cache miss
- once the runtime is on a release install, the source-local survivor budget is
  effectively flat at `drop=0`, `drop=2`, and `drop=4`; the binary rejection
  step is not the main contributor to the win
- the current `drop=4` budget is acceptable to keep because it stayed in the
  same latency band across repeated release runs

The earlier `~18ms` local read from this slice is not the canonical result; it
was taken against a debug-installed extension immediately after `cargo pgrx
test`. The release-verified runs above are the decision surface.

## Decision

Keep the cached ADR-031 runtime slice.

For the current `10K` warm verified seam, this slice now runs well below the
NFR target:

- target: `p50 < 5ms`, `p99 < 15ms`
- observed: `p50 ≈ 2.8ms`, `p99 ≈ 3.8-4.2ms`

The next follow-on question is not whether this cached runtime seam works; it
does. The next question is whether the same advantage survives on the normative
larger real-corpus lane and whether ADR-031 needs a better integration point
than the current source-local filter once the corpus grows.

## Success Criteria

- scan execution can use cached binary candidate codes without deriving them on
  every score
- newly loaded candidates that lose the binary prefilter do not pay the exact
  scorer
- the packet records whether cached ADR-031 filtering is a real warm-latency
  win on the verified real-corpus seam

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_10k --m 8 --ef-search 40 --cache-state warm-after-prime3 --warmup-passes 3 --session-mode per-cell --timing-mode cached-plan --output /tmp/adr031_cached_runtime_m8_ef40_drop4.summary`
- `scripts/bench_sql_latency_verified_scratch.sh --prefix tqhnsw_real_10k --m 8 --ef-search 40 --cache-state warm-after-prime3 --warmup-passes 3 --session-mode per-cell --timing-mode cached-plan --output /tmp/adr031_cached_runtime_m8_ef40_drop4_confirm.summary`

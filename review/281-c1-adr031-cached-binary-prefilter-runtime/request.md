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

## Planned Implementation

This slice is expected to:

1. add scan-local prepared binary-query support for the no-QJL `1536x4-bit` lane
2. cache sign-derived binary words on `CachedGraphElement`
3. make exact score-cache population lazy instead of mandatory on element load
4. apply a conservative source-local binary prefilter in
   `cached_scan_successor_candidates_for_layer(...)`
5. benchmark the warm verified real-corpus `m=8`, `ef_search=40` cell to decide
   whether this runtime seam is a keep or a discard

The initial runtime target is intentionally narrow:

- cached only, not persisted
- no format changes
- no beam-level batching yet
- no aggressive survivor budget unless the warm real-corpus surface justifies it

## Success Criteria

- scan execution can use cached binary candidate codes without deriving them on
  every score
- newly loaded candidates that lose the binary prefilter do not pay the exact
  scorer
- the packet records whether cached ADR-031 filtering is a real warm-latency
  win on the verified real-corpus seam

## Validation Plan

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- verified warm real-corpus run on the current C1 seam

# Review Request: C1 ADR-031 Real 50k Scale Validation

## Context

Packet `281` landed the first ADR-031 runtime slice on `main`:

- scan-local prepared binary query state
- cached binary codes on graph elements
- lazy exact scoring for newly loaded graph elements
- a source-local ADR-031 successor gate on the ordered-scan runtime

That slice produced a strong release-verified warm result on the real `10k`
fixture:

- `tqhnsw_real_10k`
- `m=8`
- `ef_search=40`
- `warm-after-prime3`
- `session-mode=per-cell`
- `timing-mode=cached-plan`
- `mean ≈ 2.82ms`

The user-directed next step is scale validation, not more `10k` polishing.

## Problem

The `10k` seam is encouraging, but `NFR-001` is not a `10k` benchmark. The
important next question is whether the same cached ADR-031 runtime shape stays
fast on the larger real-corpus lane where graph breadth and candidate pressure
are higher.

Before expanding the full matrix, we need the first scale read at the gate
point:

- `tqhnsw_real_50k`
- `m=8`
- `ef_search=40`

## Planned Run

First scale-validation command:

```bash
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_50k \
  --m 8 \
  --ef-search 40 \
  --query-limit 200 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell \
  --timing-mode cached-plan \
  --output /tmp/adr031_real_50k_m8_ef40_q200.summary
```

Why `query-limit 200` first:

- large enough to be more credible than a tiny smoke
- small enough to avoid wasting a long run if the larger lane regresses badly
- directly comparable to the earlier real-`50k` gate pattern used elsewhere in
  the repo

If this looks healthy, the follow-on should widen to the full canonical query
table.

## Interim Result

The first `50k` gate read is now complete.

Fixture preparation:

- initial benchmark attempt failed because the scratch database did not yet have
  `tqhnsw_real_50k_queries`
- loaded the staged real corpus fixture with:

```bash
./scripts/load_real_corpus_scratch.sh \
  --prefix tqhnsw_real_50k \
  --corpus-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_corpus.tsv \
  --queries-file /home/peter/dev/datasets/tqhnsw_real_50k/tqhnsw_real_50k_queries.tsv \
  --m 8
```

Verified warm scale read:

```text
prefix=tqhnsw_real_50k
m=8
ef_search=40
query_limit=200
cache_state=warm-after-prime3
warmup_passes=3
session_mode=per-cell
timing_mode=cached-plan
p50=4.557ms
p95=6.111ms
p99=7.239ms
mean=4.655ms
min=3.012ms
max=7.672ms
server_qps=214.82
wall=5.73s
```

Comparison to packet `281`:

- real `10k`, same `m=8` / `ef_search=40` warm seam: `mean ≈ 2.82ms`
- real `50k`, first scale gate: `mean = 4.655ms`

This is slower than the `10k` lane, but still strong enough to clear the
`NFR-001` latency targets at this first `50k` read:

- `p50 < 5ms`
- `p99 < 15ms`

## Next Step

Keep the ADR-031 cached runtime path and widen this packet to the full
canonical `1000`-query `tqhnsw_real_50k` read before making any more runtime
changes.

## Success Criteria

- the `tqhnsw_real_50k` fixture exists and the verified launcher can run it
- the packet records the first warm ADR-031 latency read on the `50k` lane
- the result is compared directly against the `10k` read from packet `281`
- the packet makes an explicit keep/pivot call for the next ADR-031 step

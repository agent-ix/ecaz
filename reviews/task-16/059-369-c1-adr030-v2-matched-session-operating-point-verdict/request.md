# Review Request: C1 ADR-030 V2 Matched-Session Operating-Point Verdict

## Context

Packet `368` corrected the SQL comparison surface between isolated grouped
tqvector `m=16` and pgvector `m=16` by matching the session shape:

- both sides used `per-cell plain-server`
- both sides used the same `tqhnsw_real_50k_queries_50` query subset

That packet established the corrected SQL means:

- tqvector SQL was faster at every measured `ef_search`
- pgvector still had much higher recall from the earlier direct-runtime packet

The remaining question was not “who is faster?” in isolation. It was:

> what are the actual same-latency and same-recall operating points once the
> corrected SQL timings and the same-query direct recall tables are read
> together?

## Problem

The branch had all the raw numbers, but not the actual decision surface.

Without explicitly crossing:

- packet `368`’s corrected SQL means
- packet `363`’s isolated grouped tqvector direct recall table
- packet `363`’s pgvector direct recall table

it was too easy to say only “tqvector is faster, pgvector is more accurate”
without answering the more useful product question:

> where does each lane actually own the curve?

## Planned Slice

Do one narrow interpretation batch:

1. keep the corrected matched-session SQL means from packet `368`
2. reuse the isolated grouped tqvector `m=16` direct recall table from packet
   `363`
3. reuse the pgvector `m=16` direct recall table from packet `363`
4. compute explicit:
   - same-latency budget comparisons
   - same-recall target comparisons
5. record the operating-point verdict for the branch

## Implementation

No code changes in this packet.

This is a measurement / interpretation-only checkpoint based on already
recorded and already rerun surfaces.

Supporting calculations were derived directly from the packet tables with a
small one-shot Python comparison on the current branch.

## Validation

No new code landed in this packet.

The packet depends on already completed live measurements from:

- packet `363` direct-runtime recall tables
- packet `368` matched-session SQL reruns

## Inputs

### tqvector isolated grouped `m=16`

Direct recall from packet `363` on the same `50`-query subset:

| ef_search | Recall@10 |
|----------:|----------:|
| 40  | `0.9200` |
| 64  | `0.9380` |
| 128 | `0.9400` |
| 320 | `0.9460` |

Corrected matched-session SQL means from packet `368`:

| ef_search | SQL mean ms |
|----------:|------------:|
| 40  | `0.959` |
| 64  | `1.525` |
| 128 | `2.163` |
| 320 | `4.360` |

### pgvector `m=16`

Direct recall from packet `363` on the same `50`-query subset:

| ef_search | Recall@10 |
|----------:|----------:|
| 40  | `0.9860` |
| 64  | `0.9920` |
| 128 | `0.9980` |
| 320 | `0.9980` |

Corrected matched-session SQL means from packet `368`:

| ef_search | SQL mean ms |
|----------:|------------:|
| 40  | `1.641` |
| 64  | `1.775` |
| 128 | `3.101` |
| 320 | `6.443` |

## Measurements

### Same-latency budget read

Using only measured points and asking: “at or below this SQL latency budget,
what is the best measured recall from the other system?”

| Budget owner | Budget | Owner recall | Other-system best measured recall within budget |
|-------------|-------:|-------------:|-----------------------------------------------:|
| tqvector `ef=40`  | `0.959ms` | `0.9200` | pgvector: no measured point this fast |
| tqvector `ef=64`  | `1.525ms` | `0.9380` | pgvector: no measured point this fast |
| tqvector `ef=128` | `2.163ms` | `0.9400` | pgvector `ef=64`: `0.9920 @ 1.775ms` |
| tqvector `ef=320` | `4.360ms` | `0.9460` | pgvector `ef=128`: `0.9980 @ 3.101ms` |

Reading the same budgets from pgvector’s side:

| pgvector point | SQL mean ms | pgvector recall | tqvector best measured recall within budget |
|---------------|------------:|----------------:|--------------------------------------------:|
| `ef=40`  | `1.641` | `0.9860` | tqvector `ef=64`: `0.9380 @ 1.525ms` |
| `ef=64`  | `1.775` | `0.9920` | tqvector `ef=64`: `0.9380 @ 1.525ms` |
| `ef=128` | `3.101` | `0.9980` | tqvector `ef=128`: `0.9400 @ 2.163ms` |
| `ef=320` | `6.443` | `0.9980` | tqvector `ef=320`: `0.9460 @ 4.360ms` |

### Same-recall target read

Using only measured points and asking: “what is the fastest measured SQL point
that reaches this recall target?”

Targets defined by tqvector:

| Recall target | tqvector fastest measured SQL | pgvector fastest measured SQL reaching target |
|--------------:|------------------------------:|---------------------------------------------:|
| `0.9200` | `0.959ms` | `1.641ms` |
| `0.9380` | `1.525ms` | `1.641ms` |
| `0.9400` | `2.163ms` | `1.641ms` |
| `0.9460` | `4.360ms` | `1.641ms` |

Targets defined by pgvector:

| Recall target | pgvector fastest measured SQL | tqvector fastest measured SQL reaching target |
|--------------:|------------------------------:|----------------------------------------------:|
| `0.9860` | `1.641ms` | not reached in the measured tqvector sweep |
| `0.9920` | `1.775ms` | not reached in the measured tqvector sweep |
| `0.9980` | `3.101ms` | not reached in the measured tqvector sweep |

## Interpretation

This packet gives the branch a much sharper decision boundary than packet
`368` alone.

The actual measured operating-point verdict is:

1. tqvector owns the ultra-low-latency corner.
   - below about `1.6ms`, pgvector has no measured point that matches its
     latency
   - that corner is roughly `Recall@10 = 0.92 .. 0.938`
2. pgvector owns the moderate-and-up latency region.
   - by tqvector’s `ef=128` point (`0.9400 @ 2.163ms`), pgvector already has a
     strictly better measured point: `0.9920 @ 1.775ms`
   - by tqvector’s `ef=320` point (`0.9460 @ 4.360ms`), pgvector again has a
     strictly better measured point: `0.9980 @ 3.101ms`
3. tqvector does not reach pgvector’s low-end measured recall floor.
   - even pgvector’s fastest measured point is `0.9860`
   - the measured tqvector sweep tops out at `0.9460`

So the honest summary is:

- tqvector grouped-v2 is a real sub-`1.6ms` / sub-`0.94` recall lane
- pgvector dominates once the latency budget rises beyond that corner
- this is not a smooth “tqvector is always faster but lower quality” curve;
  it is a curve with a narrow latency-first pocket and then a pgvector-dominant
  region above it

## Risk / Follow-up

This packet does not change runtime behavior, but it changes what is worth
doing next.

Immediate implications:

1. if the product target is around `Recall@10 >= 0.94`, the current grouped
   tqvector `m=16` lane is not competitive with pgvector on the measured SQL
   surface
2. if the product target values the lowest-latency corner more than recall,
   tqvector still has a defensible niche
3. future runtime work should be judged against this boundary, not just against
   “faster than pgvector” in the abstract

The next useful batch is no longer generic benchmarking. It is a product-level
choice:

- either push tqvector recall materially higher without losing the sub-`1.6ms`
  corner
- or explicitly frame grouped-v2 as a latency-first mode rather than a general
  pgvector replacement on this corpus

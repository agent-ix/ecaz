# Paired ec_ivf vs vchord comparison

Reviewer 2026-05-22-04 P1 asked for a paired same-host
runner-matched comparison; the earlier extrapolation against the
older comparator packet (different host, no nprobe sweep) was
underestimating the gap. This packet runs both engines on the
same `m8g.2xlarge`, same query IDs, same prewarmed buffer pool,
within minutes of each other.

## Method

- Host: m8g.2xlarge, 32 GB RAM, 100 GB EBS
- Both extensions loaded via `shared_preload_libraries = 'ecaz,vchord'`
- Same query set per scale (queries table replicated from ec_ivf
  format → vchord `vector(1536)` via `translate(source::text, '{}', '[]')::vector`)
- Each scale prewarmed via `pg_prewarm` before sweep
- ec_ivf latency via `ecaz bench latency` (libpq prepared stmts, 500 iter)
- vchord latency via inline PL/pgSQL loop with `clock_timestamp()` deltas (500 iter)
- Ground truth: numpy exhaustive top-10 IP, q=100 per scale
- Both engines' top-10 results compared against the same ground truth

Artifacts:

- `artifacts/paired-all.log` (50k+100k latency + ec_ivf recall, in-memory)
- `artifacts/paired-1m-full.log` (1m latency + ec_ivf recall)
- `artifacts/recall-final.log` (vchord recall at all 3 scales)

## Results

### 50k (k=10, q=100 for recall, 500 iter latency)

| nprobe/probes | ec_ivf p50 | ec_ivf recall | vchord p50 | vchord recall |
| --- | --- | --- | --- | --- |
| 16 | 2.28 | 0.925 | **0.53** | 0.889 |
| 32 | 3.00 | 0.963 | **0.69** | 0.949 |
| 64 | 4.53 | 0.985 | **1.07** | 0.977 |
| 128 | 7.43 | **0.999** | 1.89 | 0.995 |
| 256 | 12.0 | 1.000 | 2.97 | 0.996 |

### 100k

| nprobe/probes | ec_ivf p50 | ec_ivf recall | vchord p50 | vchord recall |
| --- | --- | --- | --- | --- |
| 16 | 7.67 (cold) | 0.852 | **0.54** | 0.840 |
| 32 | 3.52 | 0.920 | **0.71** | 0.901 |
| 64 | 5.49 | 0.961 | **1.30** | 0.951 |
| 128 | 9.65 | 0.985 | **2.44** | 0.979 |
| 256 | 18.1 | **0.996** | 4.85 | 0.995 |

### 1m

| nprobe/probes | ec_ivf p50 | ec_ivf recall | vchord p50 | vchord recall |
| --- | --- | --- | --- | --- |
| 16 | 14.2 (cold) | 0.899 | **3.74** | 0.928 |
| 32 | 11.1 | 0.942 | **3.00** | 0.955 |
| 64 | 18.6 | 0.970 | **5.10** | 0.974 |
| 128 | 33.8 | 0.985 | **9.39** | 0.987 |
| 256 | 66.1 | 0.993 | **18.98** | 0.992 |
| 512 | 143 | — | **54.02** | 0.998 |

## Recall-matched comparison (the apples-to-apples cells)

| Scale | Recall band | ec_ivf cell | vchord cell | ec_ivf gap |
| --- | --- | --- | --- | --- |
| 50k | ~0.98 | nprobe=64 4.53ms @ 0.985 | probes=64 1.07ms @ 0.977 | **4.2× slower** |
| 50k | ~0.995 | nprobe=128 7.43ms @ 0.999 | probes=128 1.89ms @ 0.995 | **3.9× slower** |
| 100k | ~0.98 | nprobe=128 9.65ms @ 0.985 | probes=128 2.44ms @ 0.979 | **4.0× slower** |
| 100k | ~0.995 | nprobe=256 18.1ms @ 0.996 | probes=256 4.85ms @ 0.995 | **3.7× slower** |
| 1m | ~0.985 | nprobe=128 33.8ms @ 0.985 | probes=128 9.39ms @ 0.987 | **3.6× slower** |
| 1m | ~0.993 | nprobe=256 66.1ms @ 0.993 | probes=256 18.98ms @ 0.992 | **3.5× slower** |
| 1m | ~0.998 | (would need nprobe>512) | probes=512 54.02ms @ 0.998 | structural gap |

**ec_ivf is consistently 3-4× slower than vchord at every matched
recall band across all scales we measured.** The ratio is remarkably
stable from 50k to 1m, which says it isn't a corpus-size issue —
it's a per-query cost gap in the design.

## What previously gave the wrong impression

Earlier in this packet (`results.md`) I claimed ec_ivf was 1.4×
behind at 50k and faster than vchord at 1m. That comparison used:

- The older `benchmarks/comparators-50k-100k-1m` packet's vchord
  numbers, which ran on a different host (instance type unclear in
  the comparator manifest) and only measured vchord at its default
  `probes` setting — no sweep.
- For 1m the comparator measured vchord at 90.3 ms p50 @ recall
  0.9995. On *this* host with paired methodology vchord hits
  recall 0.9870 in 9.39 ms p50 and recall 0.9980 in 54.02 ms p50.
  The 90.3 ms figure was a much slower host (or much higher probes)
  than what's representative on m8g.2xlarge.

So the original "we beat vchord at 1m" framing was an artifact of
comparing across hardware classes and across single-point vs swept
configurations. The paired sweep removes both confounds.

## Where vchord wins this comparison

1. **Inline f32 source storage.** vchord's RaBitQ-on-IVF index
   stores the full f32 vector inline alongside the RaBitQ code, so
   its "rerank" reads the same page that scored the candidate. Our
   `rerank='heap_f32'` path issues a separate `table_tuple_fetch_row_version`
   + toast detoast for each rerank candidate, paying a heap I/O +
   detoast for every top-K candidate. At 1m with width=50 that's
   ~50 toast detoasts per query, each ~5 µs = 250 µs of overhead
   we don't get back. Inline source would erase this.
2. **Smaller per-list scan cost** at fixed nprobe. vchord at 1m
   probes=128 visits 128 of 1024 lists in 9.4 ms p50. ec_ivf at
   1m nprobe=128 visits roughly the same number of lists but in
   33.8 ms — 3.6× slower per list. Per-tuple kernel work + list
   directory walk are the candidates.

## What ec_ivf gets right

- **Recall curve matches vchord's** at every operating point. The
  ec_ivf cells reach 0.985-0.996 recall at the same nprobes vchord
  uses. So the issue isn't *correctness* of the IVF + RaBitQ +
  rerank design; it's per-query throughput at fixed work.
- **Storage is competitive**: 1.5 GB vs vchord's 8 GB at 1m
  (because we don't store inline f32). That's a 5× storage
  advantage — but a real one if storage is the constrained axis,
  and the source of the latency gap.

## Recommended next round

1. **Inline f32 source storage on the ec_ivf RaBitQ path.** The
   biggest single lever. Closes the rerank-side cost gap directly.
2. **Per-list scan cost attribution** via EXPLAIN counters at 1m
   probes=128 paired cell — measure postings visited, scored,
   pages read, heap fetches. Identify whether the gap is in
   posting-iter, candidate-volume, or rerank.
3. **Smaller `nlists` sweep at 1m** — both engines could be off
   their per-corpus geometry optimum.

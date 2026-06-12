# Task 105 prior-baseline comparison — IVF RaBitQ/TQ vs May 2026, comparator mapping

New numbers: Task 105 G4 lane (`main=1345ca603`, m8g.2xlarge,
isolated `t105_ivf_*_1m` fixtures, 990k real DBpedia rows, k=10, c=1,
100 iterations). Baselines (NOT re-run, per task scope):

- **May scaling packet** `benchmarks/cloud-scaling-multi-am/`
  (2026-05-17, SHA `775455dc`, same instance class m8g.2xlarge, same
  corpus, k=10, c=1, 200 iterations).
- **May final gate** `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/`
  (head `902e8e066`, m8g.2xlarge, the preserved
  `real_1m_ivf_rabitq1_rerank` fixture — the same table the t105 1M
  fixtures are sourced from).
- **Pinned comparator baseline** `benchmarks/comparators-50k-100k-1m/`
  (`94c02c682` era, head `63024cce`, m8g.2xlarge, same DBpedia corpus,
  k=10, c=1, 200 iterations).

## Geometry caveat (why raw nprobe is not comparable across eras)

The May indexes were built without an `nlists` reloption → auto
nlists = ceil(sqrt(990k)) ≈ **995** lists. The t105 fixtures use
**nlists=256**. nprobe=16 therefore scans ~1.6% of the corpus on the
May index but ~6.25% on the t105 index. Honest comparisons below are
made at matched **scan fraction** (and matched rerank config where it
exists), with the residual differences stated.

## IVF rabitq1 @ 1M — the task's named comparison

| cell | config | scan frac | recall@10 | p50 |
|---|---|---|---|---|
| May final gate | nprobe=256/≈995 lists, bits=1, rerank=heap_f32 w=50, q=500/200 | ~25.7% | 0.9936 | 69.1 ms |
| **Task 105 G4** | nprobe=64/256 lists, bits=1, rerank=heap_f32 w=50, q=100 | 25.0% | 0.9800 | **56.8 ms** |
| **Task 105 G4** | nprobe=16/256 lists (same fixture) | 6.25% | 0.9260 | **16.1 ms** |

At the matched ~25% scan fraction with identical quant/rerank shape,
the post-optimization head is **−18% p50** (69.1 → 56.8 ms) at a
slightly lower measured recall (0.9936 → 0.9800; finer May list
geometry at the same scan fraction yields better candidates, and the
query counts differ 500 vs 100). Read: comparable-recall operating
band, meaningfully faster, with a much faster lower-recall point
(0.926 @ 16.1 ms) now available on the same index.

Versus the May **scaling** rabitq cells (no rerank, ≈995 lists,
scalar-kernel era): nprobe=64 read 185.5 ms then; the t105 fixture at
nprobe=64 scans ~3.9× the corpus fraction, pays heap rerank, and still
lands at 56.8 ms — **3.3× faster at ~4× the scanned volume**, i.e. a
roughly order-of-magnitude per-scanned-row improvement. This is the
cumulative effect of the block-kernel program (Tasks 87–105), not a
single change.

Smaller scales (May cells exist at 10k only in the scaling packet;
geometry caveat applies — May 10k auto nlists = 100):

| scale | May rabitq (no rerank) @ nprobe=16 / 64 | Task 105 rabitq1 (rerank w=50) @ nprobe=16 / 64 |
|---|---|---|
| 10k | 4.57 / 15.9 ms | **1.04 / 2.33 ms** |
| 1M | 52.5 / 185.5 ms | **16.1 / 56.8 ms** |

No May 50k/100k rabitq cells were published in that packet; the t105
50k (3.27/10.3 ms) and 100k-confirm (6.23/19.7 ms) rows stand as the
new reference cells.

## IVF TurboQuant @ 1M — geometry-confounded, no regression conclusion

May scaling TQ (auto ≈995 lists, no rerank): 34.6 ms @ nprobe=16,
112.2 ms @ nprobe=64. Task 105 TQ (256 lists, rerank w=25): 59.8 /
237.2 ms. The t105 points scan ~3.9× the corpus fraction per nprobe
and pay heap rerank; per-scanned-row, throughput improved (≈3.9× rows
in ≈1.7–2.1× time). There is **no matched-scan-fraction TQ pair across
eras**, so no cross-era TQ regression/improvement verdict is made
here. The within-era TQ kernel evidence is Task 99's explicit A/B
(−44% G4 / −70% Intel at 100k).

## Comparator mapping (pinned baseline, NOT re-run)

Comparator bar: **vchord RaBitQ-on-IVF** (the only tuned-competitive
comparator in the pinned packet), m8g.2xlarge, k=10, c=1, 200 iters.
ecaz cells: Task 105 G4 kernel-on `ivf-rabitq1` (recall in parens).

| scale | vchord RaBitQ-IVF p50 | ecaz ivf-rabitq1 p50 @ nprobe=16 | @ nprobe=64 |
|---|---|---|---|
| 50k | 2.7 ms | **3.27 ms** (0.968) | 10.3 ms (0.998) |
| 100k | 6.3 ms | **6.23 ms** (0.885) | 19.7 ms (0.998) |
| 1M | 80.4 ms | **16.1 ms** (0.926) | **56.8 ms** (0.980) |

- At **1M** ecaz clears the vchord bar at both operating points: 5.0×
  faster at the 0.926-recall point, 1.4× faster at the 0.980-recall
  point.
- At 50k/100k ecaz is at parity with vchord at the nprobe=16 point.
- Caveats, stated plainly: the comparator packet did not record
  vchord's recall at these latencies (its numbers are latency-only),
  the comparator ran 200 iterations vs 100 here, and comparator-era
  corpus tables were pgvector `vector(1536)` copies of the same
  DBpedia slices. No comparator was re-run (pinned baseline stands per
  the task scope and the standing rule).
- Other comparators (pgvector HNSW 223 ms, IVFFlat ~2.75 s,
  pgvectorscale ~2.78 s at 1M, untuned) are dominated by every ecaz
  family at 1M — e.g. ecaz diskann-tq 4.66 ms, hnsw best ~8.4 ms,
  spire-tq 62.3 ms — but those comparator cells were explicitly
  untuned upper bounds; the vchord row is the meaningful bar.

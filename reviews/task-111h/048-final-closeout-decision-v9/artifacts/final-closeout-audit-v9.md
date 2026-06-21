# Task 111h Corrected Final Closeout Audit v9

Head SHA: `b088c07536c2e7001ab259efc0b925c33c70471b`

This audit replaces the stale packet 041 closeout decision. Packet 041 feedback
accepted the common persisted-rerank engineering but rejected the decision
because RaBitQ-4 clips, exact-dequant/fidelity levers, TurboQuant fidelity, and
best-config matched-recall comparisons were missing. Packets 043-047 now cover
those missing gates.

## Evidence Sources

| Packet | Evidence |
| --- | --- |
| `reviews/task-111h/043-exact-dequant-score-mode/` | Implements `rerank_exact_dequant=1`, persists score mode in metadata v9, updates fixtures and upgrade matrix, and validates targeted PG18 tests/checks. |
| `reviews/task-111h/044-corrected-compact-10k-v9/` | Corrected 10k/w64 suite: source f32, index f16, RQ4 clips 2/3/4 estimator+exact, RQ8 clips 2/3/4 estimator+exact, TQ default+exact. 65/65 steps succeeded. |
| `reviews/task-111h/045-corrected-compact-50k-v9/` | Corrected 50k/w64 suite with the same 16-cell matrix. 65/65 steps succeeded. |
| `reviews/task-111h/046-corrected-compact-100k-v9/` | Corrected 100k/w64 suite with the same 16-cell matrix. 65/65 steps succeeded. |
| `reviews/task-111h/047-corrected-compact-1m-locked-v9/` | Locked 1M/w64 best-config suite: source f32, index f16, RQ4 estimator c3, RQ8 estimator c4, RQ8 exact c4, TQ default, TQ exact. 44/44 steps succeeded. |

All benchmark evidence above is packet-local and driven by `ecaz bench suite`
with checked-in configs, suite manifests, result JSONL, generated reports,
storage logs, and per-step recall/latency/load logs.

## Reopened Follow-Up Audit

| Reopened requirement | Status | Evidence |
| --- | --- | --- |
| RaBitQ-4 clip sweep `{2,3,4}` before abandon/iterate/promote | Complete | Packets 044, 045, and 046 each include RQ4 estimator and exact-dequant cells at clips 2, 3, and 4. Best RQ4 is clip 3, but it does not reach recall@10 >= 0.97 at 50k, 100k, or 1M. |
| RaBitQ-8 matched-recall comparison vs index f16 at best clip | Complete | Packets 044-046 identify clip 4 as best. Packet 047 locks RQ8 c4 at 1M. The threshold table below compares against f16 at 0.97 and 0.99. |
| Exact-dequant scoring or equivalent fidelity lever | Complete | Packet 043 implements exact-dequant. Packets 044-047 measure exact-dequant for RQ4/RQ8/TQ where required. |
| TurboQuant fidelity lever coverage | Complete | Packets 044-047 include TurboQuant exact-dequant cells. Exact-dequant does not improve TQ recall in any corrected sweep. |
| Matched-recall vs index f16 at recall 0.97 and 0.99 | Complete | Threshold table below, backed by summaries and `results.jsonl` in packets 044-047. |
| Corrected 10k/50k/100k before final 1M | Complete | Packets 044, 045, and 046 were completed and pushed before packet 047. |

## Corrected Threshold Comparison

Best compact configs used here:

- RaBitQ-4: estimator clip 3.
- RaBitQ-8: estimator clip 4 unless exact-dequant is explicitly called out.
- TurboQuant: default and exact are both measured; neither improves recall over
  the other in the corrected sweeps.

| Scale | Format | First nprobe >=0.97 | First nprobe >=0.99 | Best recall@10 |
| --- | --- | ---: | ---: | ---: |
| 10k | index f16 | 8 | 16 | 0.9990 |
| 10k | RQ4 c3 | 8 | not hit | 0.9835 |
| 10k | RQ8 c4 | 8 | 16 | 0.9990 |
| 10k | TQ default/exact | 8 | not hit | 0.9815 |
| 50k | index f16 | 64 | 128 | 0.9985 |
| 50k | RQ4 c3 | not hit | not hit | 0.9605 |
| 50k | RQ8 c4 | 64 | 128 | 0.9930 |
| 50k | TQ default/exact | not hit | not hit | 0.9590 |
| 100k | index f16 | 64 | 128 | 0.9975 |
| 100k | RQ4 c3 | not hit | not hit | 0.9530 |
| 100k | RQ8 c4 | 128 | 200 | 0.9915 estimator / 0.9920 exact |
| 100k | TQ default/exact | not hit | not hit | 0.9565 |
| 1M | index f16 | 64 | not hit | 0.9880 |
| 1M | RQ4 c3 | not hit | not hit | 0.9400 |
| 1M | RQ8 c4 | 64 | not hit | 0.9840 |
| 1M | TQ default/exact | not hit | not hit | 0.9490 default / 0.9480 exact |

## Locked 1M Decision Data

The final-scale packet deliberately used the best corrected compact configs
from 10k/50k/100k instead of spending 1M on the full 16-cell matrix.

| Cell | r@10 n64 | mean n64 ms | r@10 n128 | mean n128 ms | r@10 n200 | mean n200 ms | ec_ivf index |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| source f32 | 0.9770 | 18.7 | 0.9860 | 29.9 | 0.9880 | 40.6 | 226.8 MiB |
| index f16 | 0.9770 | 21.4 | 0.9860 | 31.2 | 0.9880 | 43.0 | 3.2 GiB |
| RQ4 estimator c3 | 0.9290 | 18.6 | 0.9380 | 28.5 | 0.9400 | 41.0 | 1.0 GiB |
| RQ8 estimator c4 | 0.9730 | 18.1 | 0.9820 | 30.1 | 0.9840 | 42.3 | 1.8 GiB |
| RQ8 exact c4 | 0.9730 | 17.9 | 0.9820 | 30.2 | 0.9840 | 54.3 | 1.8 GiB |
| TQ default | 0.9400 | 18.1 | 0.9480 | 28.7 | 0.9490 | 41.7 | 1.0 GiB |
| TQ exact | 0.9390 | 18.0 | 0.9470 | 28.5 | 0.9480 | 45.5 | 1.0 GiB |

Immediate 1M facts:

- Source f32 and index f16 have identical recall, but f16 is slower and much
  larger than source f32 in the measured path.
- RQ8 clip 4 is the only compact quantized format that reaches recall@10 >=
  0.97 at 1M/w64, but it remains below source/f16 recall at every measured high
  nprobe and does not hit 0.99.
- RQ8 exact-dequant does not improve recall over the estimator and worsens
  nprobe 200 latency in the locked 1M run.
- RQ4 and TQ do not reach recall@10 >= 0.97 at 1M/w64.

## Final Product Decisions

| Placement / format | Decision | Evidence-backed rationale |
| --- | --- | --- |
| `source/f32` | Promote as default/reference. | It uses the existing heap/source vector, adds no compact payload storage, and is the best high-recall default in corrected 100k/1M evidence. |
| `table/*` | Reserve; not a Task 111h product path. | Packet 034 documents why table-owned storage is not implemented in 111h and reserves it for a future DDL/MVCC design. Query-time conversion is no longer product-facing table placement. |
| `index/f16` | Do not promote; iterate only if f32 source storage is removed/replaced or f16 scoring/layout materially changes. | Corrected 1M: same recall as source f32, slower latency, and 3.2 GiB index vs 226.8 MiB for source f32. Corrected 100k: slower than source at n64 and much larger. |
| `index/rabitq4` | Abandon current 111h high-recall candidate. | After clip 2/3/4 and exact-dequant sweeps, best RQ4 does not hit 0.97 at 50k, 100k, or 1M. |
| `index/rabitq8` | Iterate only; do not promote in 111h. | RQ8 c4 is the only compact quantized candidate that reaches 0.97 at 1M, but it does not reach 0.99, exact-dequant does not help, and it does not beat source/f16 as a default high-recall path. It remains the best compact follow-up candidate for future index-only/cold-IO work. |
| `index/turboquant` | Abandon current 111h high-recall candidate. | TQ default and exact-dequant never hit 0.97 at 50k/100k/1M; exact-dequant does not improve recall. |

## Acceptance Criteria Audit

1. **Placement semantics fixed:** Complete. The task and packets use `source`,
   reserved `table`, and `index`; diagnostic query-time conversion is not the
   product-facing table path.
2. **Common architecture for f16/RQ4/RQ8/TQ:** Complete. Packets 030-032 and
   043 cover the common payload/scorer architecture and exact-dequant mode.
3. **Packed scorer-width index payload layout:** Complete. Packets 030 and 041
   engineering audit accepted the packed group/segment layout; the corrected
   decision now measures the formats on top of it.
4. **Table-owned payload disposition:** Complete. Packet 034 provides the
   evidence-backed reserve/reject decision for table-owned storage in 111h.
5. **PG18 correctness coverage:** Complete. Packets 030, 031, 032, and 043
   provide lifecycle, fixture, update/snapshot, and metadata-version evidence.
6. **Benchmark matrix and artifacts:** Complete for closeout. Original breadth
   matrix is packet-local in packets 024/026/027/028/036/040; the corrected
   compact decision gaps are packet-local in packets 044-047. All corrected
   benchmark packets include checked-in suite configs, manifests, results JSONL,
   storage logs, and per-step logs.
7. **Final decision for every format/placement:** Complete. Decision table
   above covers source, table, f16, RQ4, RQ8, and TQ without leaving a format as
   "not tried."

## Remaining Work Moved Out Of 111h

These are not 111h closeout blockers because the current evidence supports not
promoting compact formats as defaults:

- CLI profile known-reloption hygiene for
  `rabitq_rerank_least_squares`, `rabitq_rerank_clip`, and
  `rerank_exact_dequant`.
- Any future f16 architecture that stores f16 instead of the source f32 vector,
  rather than duplicating source data in the index.
- Any future RQ8 index-only/cold-IO product lane that can justify lower recall
  or recover the 0.99 gap.
- Any real table-owned compact payload DDL/MVCC storage design.

# Task 206 result summary

> Superseded for the k-head axis by `../006-re-review-corrections/`. The
> release rows below requested 128 versus 200 but both used the compiled
> production derivation; they are not a seed-count A/B.

Decision-arm results copied from the packet-local `run/results.jsonl` and
scale summaries. Recall is `recall@k`; latency is mean/p50/p95/p99/max ms.

| scale | k_head | recall | latency | physical generation bytes |
|---|---:|---:|---|---:|
| 10k | 128 | 0.9884 | 194.95; 182.8/183.2/206.7/213.0/213.8 | 242,794,496 |
| 10k | 200 | 0.9884 | 201.99; 181.5/183.0/193.0/205.4/213.8 | 242,794,496 |
| 50k | 128 | 0.9601 | 205.82; 192.3/190.0/205.7/214.7/219.2 | 1,242,734,592 |
| 50k | 200 | 0.9601 | 204.43; 186.5/185.7/201.9/210.8/213.4 | 1,242,734,592 |
| 100k | 128 | 0.9585 | 231.44; 191.6/191.4/204.5/223.8/232.5 | 2,496,659,456 |
| 100k | 200 | 0.9585 | 227.11; 192.3/191.9/206.9/218.0/227.7 | 2,496,659,456 |

The storage row is invariant across the A/B. The physical-generation versus
single-index totals reach about 2.92x at 100k, while the NFR-021 normalized
criterion passes at 1.094707; these are different accounting surfaces.

The requested per-round diagnostic was attempted on the feature-enabled lane:
the suite config and child argv contain `ec_distann.scan_profile_notice=on`,
and the CLI now combines child stdout and stderr before parsing. Nevertheless,
the captured logs contain zero `ec_distann_scan_round` lines at 100k and in the
10k smoke. There is therefore no claimed transport/straggler/byte attribution.

The separate owner-traversal control did complete its 10k arm: membership
recall `0.9727`; physical latency mean/p50/p95/p99/max
`411.00/400.70/504.20/521.50/527.10` ms; single-index control recall
`0.9133` and latency mean/p50/p95/p99/max `46.00/44.00/61.70/65.00/67.60`
ms. Stage counters reported 298.88 requested and returned nodes per scan,
14,667.28 request bytes, and 26,713.76 response bytes. This is diagnostic
owner-oracle evidence, not a replacement for the clean release matrix.

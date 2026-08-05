# Task 212 / 213 fused crown capacity matrix

The valid release run used extension commit `a8b1699528e593b45f55fc25329199714d4627ff` and release profile. The suite configuration is `task212-capacity-suite.json`. Seven arms are represented by their compact summary logs; the two 100k reruns are recorded in `suite-manifest-r2.json` and `results-r2.jsonl` after failed partial cluster initializations were discarded.

Every arm is physical, three-owner, `head_index_cap=4096`, `head_search_width=32`, `head_seed_count=32`, `beam_width=4`, `hop_rounds=100`, RabitQ, and `fused_head_hop=true`. Every arm is labeled `seed_set_change=true`: capacity controls crown coverage, but the crown's code-scored ranking is not the exact-distance `head_sample_exact` ranking.

| Capacity | Scale | Recall | Recall mean ms | Latency mean ms | Storage amplification |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 512 | 10k | 0.9990 | 35.34 | 33.80 | 1.235867 |
| 2048 | 10k | 0.9985 | 33.60 | 32.10 | 1.235467 |
| 4096 | 10k | 0.9985 | 34.16 | 33.30 | 1.235867 |
| 512 | 50k | 0.9555 | 44.40 | 43.50 | 1.332667 |
| 2048 | 50k | 0.9585 | 43.22 | 41.40 | 1.332667 |
| 4096 | 50k | 0.9555 | 42.88 | 42.10 | 1.332667 |
| 512 | 100k | 0.9135 | 43.33 | 41.00 | 1.351147 |
| 2048 | 100k | 0.9300 | 41.00 | 39.50 | 1.351147 |
| 4096 | 100k | 0.9310 | 43.05 | 42.30 | 1.351147 |

Capacity 2048 is selected for the opt-in fused configuration: it is the latency winner at all three scales and remains within 0.001 recall of capacity 4096 at 100k. The shipped defaults remain safe opt-in defaults (`crown_capacity=0`, `fused_head_hop=off`) because all measured fused arms intentionally change the seed set; enabling the measured consumer is explicit rather than silently changing the default recall policy.

All fused arms reported 6400 crown seeds / 200 fused hops / 6400 first-round requested ids on recall and 1600 / 50 / 1600 on latency; fallbacks were zero. The pruning A/B remains a separate no-effect finding: it activated but pruned zero shards and showed no latency improvement.

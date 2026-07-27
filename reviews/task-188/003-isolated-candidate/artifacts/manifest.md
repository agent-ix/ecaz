# Task 188 isolated-candidate manifest

- Head SHA at pre-registration: `c3f52fdd6` (Task 188 attribution checkpoint)
- Task bucket: `reviews/task-188/003-isolated-candidate/`
- Candidate: BW8/H100, exact-scored bounded-head seeds; control BW4/H100
- Required matrix: `ec_real_10k`, `ec_real_50k`, `ec_real_100k`; recall + warm serial latency + storage for both arms
- Lane: PG18 local, three-owner physical DistANN, isolated one-index-per-table surfaces per scale
- Suite config: `task188-bw8-full-scale-suite.json`
- Planned command: `ecaz bench suite audit --config reviews/task-188/003-isolated-candidate/artifacts/task188-bw8-full-scale-suite.json`, then `ecaz bench suite run` with packet-local manifest/results
- Fixture note: the 10k evaluation fixture has 200 queries and no disjoint training slice, so its training policy intentionally uses the disjoint rows 201–400 from the 100k query fixture; evaluation remains `ec_real_10k`.
- Decision status: full-scale confirmation complete; BW8 selected as the sole follow-up research candidate; no production default or persisted-format change is claimed
- Head SHA at measurement: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`
- Historical runner note: every variant row in this packet has
  `materialization_batch_size=0` because the pre-fix parser defaulted an
  omitted variant field to eager-0. These rows are retained as unbatched
  historical evidence and are not the final acceptance measurement.
- Corrected confirmation: `../005-batch10-reconfirmation/`, which explicitly
  sets production `materialization_batch_size=10` and emits paired per-query
  recall wins/losses plus bootstrap intervals.
- Structured suite manifest: `artifacts/run/suite-manifest.json`
- Structured suite results: `artifacts/run/results.jsonl`
- Concise cited result lines: `artifacts/task188-bw8-full-scale-results.log`

## Full-scale result summary

| scale | BW4 control | BW8 candidate | storage / gate |
|---|---|---|---|
| 10k | recall 1.0000; warm mean 34.20 ms; p95 39.30 ms | recall 1.0000; warm mean 34.20 ms; p95 41.10 ms | physical 242,794,496 bytes for both; topology and engagement passed |
| 50k | recall 0.9840; warm mean 40.00 ms; p95 49.40 ms | recall 0.9865; warm mean 47.90 ms; p95 71.20 ms | physical 1,242,750,976 bytes for both; topology and engagement passed |
| 100k | recall 0.9740; warm mean 43.40 ms; p95 52.10 ms | recall 0.9805; warm mean 44.10 ms; p95 61.50 ms | physical 2,496,651,264 bytes for both; topology and engagement passed |

The exact head seed digest is held constant between BW4 and BW8 at each
scale. The candidate improves recall at 50k and 100k without changing the
persisted surface; the 50k tail/mean regression remains an explicit follow-up
acceptance concern.

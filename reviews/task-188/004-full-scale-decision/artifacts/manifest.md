# Task 188 full-scale decision manifest

| artifact | purpose |
|---|---|
| `../003-isolated-candidate/artifacts/task188-bw8-full-scale-results.log` | concise cited A/B result lines at 10k/50k/100k |
| `../003-isolated-candidate/artifacts/run/results.jsonl` | structured `ecaz bench suite` result source |
| `../003-isolated-candidate/artifacts/run/suite-manifest.json` | suite provenance and completion state |
| `../003-isolated-candidate/artifacts/task188-bw8-full-scale-suite.json` | checked-in SuiteConfig used for the matrix |

Measurement head: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`.

All three scales used the PG18 local three-owner physical lane, isolated
one-index-per-table surfaces, exact head scoring with the same seed digest
within each A/B pair, RaBitQ graph payloads, 200 evaluation queries, 2000
recall trials, and 50 warm-latency iterations after 10 warmups. Topology and
remote-owner engagement passed for both arms at every scale.

Decision: BW8 is the only candidate selected for a separate follow-up
research/productionization task. Task 188 changes no production behavior.

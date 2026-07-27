# Task 188 full-scale decision manifest

| artifact | purpose |
|---|---|
| `../003-isolated-candidate/artifacts/task188-bw8-full-scale-results.log` | concise cited A/B result lines at 10k/50k/100k |
| `../003-isolated-candidate/artifacts/run/results.jsonl` | structured `ecaz bench suite` result source |
| `../003-isolated-candidate/artifacts/run/suite-manifest.json` | suite provenance and completion state |
| `../003-isolated-candidate/artifacts/task188-bw8-full-scale-suite.json` | checked-in SuiteConfig used for the matrix |

Historical measurement head: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`.

Final decision evidence is the corrected packet
`../005-batch10-reconfirmation/`; packet 003's omitted variant batch field
made its rows eager-0 and they are not final acceptance evidence.

All three scales used the PG18 local three-owner physical lane, isolated
one-index-per-table surfaces, exact head scoring with the same seed digest
within each A/B pair, RaBitQ graph payloads, 200 evaluation queries, 2000
recall trials, and 50 warm-latency iterations after 10 warmups. Topology and
remote-owner engagement passed for both arms at every scale.

Decision: BW8 is the only candidate accepted for isolated search-budget
research. The corrected batch-10 run has paired per-query outcomes and lower
BW8 warm mean/p95 at every scale, with identical storage and passed topology /
engagement. Task 188 changes no production behavior.

Stage-counter qualification: packet 002's Phase 1 rows were instrumented;
packet 003's historical full-scale rows were not. The corrected packet does
not claim stage counters for the BW8 confirmation.

# Task 188 isolated-candidate manifest

- Head SHA at pre-registration: `c3f52fdd6` (Task 188 attribution checkpoint)
- Task bucket: `reviews/task-188/003-isolated-candidate/`
- Candidate: BW8/H100, exact-scored bounded-head seeds; control BW4/H100
- Required matrix: `ec_real_10k`, `ec_real_50k`, `ec_real_100k`; recall + warm serial latency + storage for both arms
- Lane: PG18 local, three-owner physical DistANN, isolated one-index-per-table surfaces per scale
- Suite config: `task188-bw8-full-scale-suite.json`
- Planned command: `ecaz bench suite audit --config reviews/task-188/003-isolated-candidate/artifacts/task188-bw8-full-scale-suite.json`, then `ecaz bench suite run` with packet-local manifest/results
- Decision status: candidate selected for full-scale confirmation; no production default or persisted-format change is claimed

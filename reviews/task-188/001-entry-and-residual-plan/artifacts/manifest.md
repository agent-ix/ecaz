# Task 188 entry-and-residual-plan manifest

- Head SHA at plan creation: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7` (`origin/main`)
- Task bucket: `reviews/task-188/001-entry-and-residual-plan/`
- Entry evidence: Task 185 packet `004-full-scale-decision`; Task 186 branch packet `001-capacity-control` and `002-hierarchy-screen` (the cited Task 186 branch was unmerged and its hierarchy result was a query-time/arbitrary-representative prototype, not a production routing conclusion)
- Fixture reserved: `ec_real_100k`, query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Planned lane: PG18 local, three-owner physical DistANN, isolated one-index-per-table surface with shared generation within the suite
- Planned scoring: RabitQ stored neighbor codes, exact-scored 16,384 training-landmark head for bounded-head/control arms, BW/H isolated controls
- Suite config: `task188-residual-attribution-100k-suite.json`
- Planned command: `ecaz bench suite audit --config reviews/task-188/001-entry-and-residual-plan/artifacts/task188-residual-attribution-100k-suite.json`, followed by `ecaz bench suite run` with packet-local artifact and manifest/result outputs
- Decision status: entry gate satisfied; no candidate selected or production change claimed

This packet is the pre-registration checkpoint. Measurement provenance and
result artifacts belong to the follow-on search/graph attribution packet.

# Task 186 full-scale-decision manifest

- Decision: prototype-scoped STOP; full 10k/50k/100k candidate matrix not required because no measured candidate passed the 100k promotion gate.
- Source packets: `001-capacity-control`, `002-hierarchy-screen`, and `003-compressed-hierarchy-screen`; entry correction and Task 185 handoff: `005-entry-and-head-design`
- Source head SHA: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`
- Source fixture: `ec_real_100k`; query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Durable evidence: source packet manifests and structured `results.jsonl` files; no new raw measurement was run in this decision packet
- Capacity qualification: 4096/8192/16384 were independent physical generations (not paired per-query A/B); report mechanism coverage first and treat the aggregate recall deltas as noise-sensitive until a paired test is run.
- Hierarchy qualification: the historical arm rebuilds 16,384 region assignments and 256 vectors per query, uses arbitrary lowest-index representatives, and applies 256/16/512 caps. It emitted no representatives-scored, groups-opened, landmarks-scored, seed, remote-request, peak-memory, or spill counters. These are unknown, not zero.
- Compressed arm: NOT SCREENED. No compressed-head implementation or measurement exists in the cited packets; it remains an open alternative requiring a new packet/task.

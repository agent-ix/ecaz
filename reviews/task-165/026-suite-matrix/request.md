# Task 165 — packet 026: ec_distann single-node suite matrix (10k/50k/100k)

Coder review request. Closes the packet-021 **P1 "phase closeout evidence is
below the repo gate"** for the single-node axis: real `ecaz bench suite`
recall + latency + storage for `ec_distann` at 10k / 50k / 100k on staged real
DBpedia, release build, with `suite-manifest.json` + `results.jsonl`.

## Summary

Single-node `ec_distann` (global Vamana, graph_degree=32), release `.so`, sweep =
the registered `default_sweep` `[16,32,64,100,200]`, k=10:

- **recall@10** — 10k: 0.9935→1.0; 50k: 0.915→0.995; 100k: 0.8685→0.9925 across
  the ef-bar sweep.
- **latency p50** — 10k 1.7–10 ms; 50k 2.4–13 ms; 100k 2.5–14 ms (warm).
- **storage** — index 110.6 MiB (10k) / 423.6 MiB (50k) / see log (100k);
  8883.7 B/row index at 50k.
- Build times: 10k 13.8s, 50k 151s, 100k 378s.

## Evidence (`reviews/task-165/026-suite-matrix/`)

- `artifacts/manifest.md` — full matrix, command, provenance, per-scale numbers.
- `artifacts/distann-suite.json` — the bespoke `SuiteConfig` (ec_distann is the
  5th AM, not in the canonical lane config; sweep copied from profiles.rs).
- `artifacts/suite-manifest.json` + `artifacts/results.jsonl` — canonical suite
  artifacts (every cited number traces here, NFR-007).
- `artifacts/{recall,latency,storage,load}-{10k,50k,100k}-distann.log`.

## Provenance notes

- Ran against a fresh DB `distann_t165` because the shared `tqvector_bench`
  carries a stale ecaz extension without the ec_distann AM; creating a fresh DB
  avoided a destructive `DROP EXTENSION CASCADE` on shared data. Leftover
  corpus tables from an initial mis-targeted attempt were dropped from
  `tqvector_bench`.
- Release `.so` (`target/release/libecaz.so`) installed for the run; it is ahead
  of the shared 08:41 build only by this branch's off-by-default debug GUCs +
  FR-082 read sourcing (recall/latency/storage-neutral for single node).

## Status of the packet-021 P1s

- ✅ NFR-020/TC-042 fault taxonomy complete (packet 024, 12/12).
- ✅ FR-082 published-epoch read consumption (packet 025).
- ✅ Single-node ec_distann 10k/50k/100k recall+latency+storage via suite (this).
- ⏳ Multinode distinct_recall as a suite `distann-local-multinode` step — M4 gate
  (already proven byte-identical on the real 3-node fixture; packaging as a suite
  step is the remaining follow-up, prerequisite task-138 + task-146 merged).

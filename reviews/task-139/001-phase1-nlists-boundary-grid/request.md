# Task 139 Packet 001 Closeout: Superseded By Debug-Build Substrate

Status: superseded. Do not review this packet as a completed Pareto decision
and do not cite its latency columns against release baselines.

Reviewer feedback in
`feedback/2026-07-04-01-agent-ix.md` established that the local multi-instance
fixture installed a dev-profile `ecaz.so` for every completed cell. The completed
50k evidence is retained only for recall/scan-shape/failure-mode context. The
remaining Task 139 work is wound down and superseded by Tasks 141-146, with the
honest Pareto rerun owned by Task 146 after the substrate is fixed.

## Preserved Evidence

- `artifacts/manifest.md` records the debug-build taint and the wind-down
  status.
- `artifacts/task139-phase1-50k-suite.json` and
  `artifacts/task139-phase1-100k-suite.json` are the pre-registered suite
  configs.
- 12 completed 50k cells have `bench-suite/results.jsonl`: all nlists
  128/316/512/1024 x boundary_replica_count 0/1/2.
- `artifacts/50k-n2048-b0/` and `artifacts/50k-n2048-b1/` preserve repeated
  production-read failure evidence:
  `remote_candidate_receive_failed` from node_id 2.

## Wind-Down Scope

- No more 50k cells will be launched in this packet.
- The 100k grid was not run.
- Task 139 phases 2-4 are not started.
- Absolute latency values in this packet are invalid for product decisions.
  Recall, scan-profile counters, selected-PID counts, and storage shape may be
  quoted only with the debug-substrate caveat.

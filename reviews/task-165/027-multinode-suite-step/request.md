# Task 165 — packet 027: `distann-local-multinode` suite step + 50k multinode distinct-recall

Coder review request. Closes the last explicit Task 165 Required-Evidence item:
**"50k multinode distinct_recall ≥ single-node − 0.001 via `ecaz bench suite`."**

## What landed

A new `ecaz bench suite` step kind, **`distann-local-multinode`** (sibling of the
existing `spire-local-multinode`), that drives the real N-node
`ecaz dev distann-multicluster local-multinode-pg18` fixture from a `SuiteConfig`
and emits `suite-manifest.json` + the fixture summary as packet-local evidence.
A `--skip-fault-drills` fixture flag runs only the multi-node distinct-recall
gates (the scaled evidence) without the fault matrix (proven scale-independently
at the fixture default size in packet 024).

- Suite runner extension: `crates/ecaz-cli/src/commands/bench/suite.rs`
  (`DistannLocalMultinodeStep`, `expand_distann_local_multinode`, validation).
- Fixture: `--skip-fault-drills` recall-only mode.

## Evidence (`reviews/task-165/027-multinode-suite-step/`)

- `artifacts/distann-multinode-suite.json` — the `SuiteConfig` (nodes=3,
  rows=50000, dim=16, graph_degree=32, queries=100, top_k=10).
- `artifacts/suite-manifest.json` — canonical suite manifest.
- `artifacts/.../distann-multinode-summary.log` — the fixture summary with:
  - `RECALL_RESULT n_queries=… identical=… mismatched_ids=0` (multi == single),
  - `suite_recall_gate single=… multi=… delta=… pass=true` (distinct_recall(multi)
    ≥ distinct_recall(single) − 0.001, via `ecaz bench recall`),
  - `qual_correctness … mismatch=0 pass=true`,
  - `fr082_published_epoch … pass=true`.

## Command

```
ecaz bench suite run \
  --config reviews/task-165/027-multinode-suite-step/artifacts/distann-multinode-suite.json \
  --host /home/peter/.pgrx --port 28818
```

## Task 165 M3 status — all Required Evidence + Acceptance Criteria met

- ✅ AC-1 100% drill pass across the NFR-020 taxonomy (packet 024, 12/12).
- ✅ AC-2 FR-082 all ACs incl. restart + retirement override; published-epoch
  read consumption (packet 025).
- ✅ AC-3 tombstone/delta DML distributed (mid-delete + mid-insert drills, 024).
- ✅ epoch-swap-under-load consistent (packet 025 concurrency drill).
- ✅ single-node 10k/50k/100k recall+latency+storage via suite (packet 026).
- ✅ 50k multinode distinct_recall via `ecaz bench suite` (this packet).

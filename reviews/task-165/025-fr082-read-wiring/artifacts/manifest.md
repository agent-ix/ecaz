# Packet 025 — FR-082 published-epoch read wiring (artifacts manifest)

- head SHA: df9aad958483778dc9474392f323a0b2dc3b42c1
- task bucket / packet: reviews/task-165/025-fr082-read-wiring
- surface: real 3× PG18 fixture (`ecaz dev distann-multicluster`), replicated
  deterministic corpus, installed debug `.so` (adds no SQL; reverted to the
  shared release build after the run)
- fixture params: nodes=3, rows=2000, dim=16, graph_degree=32, queries=50, top_k=10
- timestamp: 2026-07-09T18:30:30Z

## Command

```
ecaz dev distann-multicluster local-multinode-pg18 \
  --nodes 3 --rows 2000 --dim 16 --queries 50 --top-k 10 \
  --artifact-dir reviews/task-165/025-fr082-read-wiring/artifacts \
  --log-file reviews/task-165/025-fr082-read-wiring/artifacts/fixture-run.log
```

## The finding this closes (packet 020/021 P1)

> "FR-082 active epoch publication is still not wired into reads … scans continue
> to use the session GUC epoch … publish writes metadata.active_epoch but reads
> never consume it … epoch-swap-under-load must prove one-epoch-per-scan."

## What changed

Reads now source the scan epoch from the persisted manifest
(`metadata.active_epoch`, Published by default for any built index) rather than
the `ec_distann.epoch` GUC. Touched every fingerprint-producing site so the
coordinator and owners agree on the published epoch:

- `roster.rs`: `placement_directory_for_epoch(epoch)` + `scan_epoch(metadata)`
  (published `active_epoch`, else GUC fallback).
- `routine.rs` scan loop: re-reads metadata per attempt, builds placement +
  fingerprint from `scan_epoch` → one epoch per scan; a concurrent republish is
  an FR-082-AC-2 retriable mismatch (restart under the refreshed epoch).
- `expand` / `materialize` / `apply_record_writes` / `epoch_fingerprint`
  endpoints, CustomScan owner materialization, and the debug expander all compute
  their fingerprint from the published epoch.

## Key result lines (`artifacts/distann-multinode-summary.log`)

- `fr082_published_epoch base_ok=true coord_only_publish_errored=true all_publish_swap_ok=true pass=true`
  - **base_ok** — baseline multi-node scan at the built-in published epoch works.
  - **coord_only_publish_errored** — publishing epoch 2 on the COORDINATOR ONLY
    breaks the scan (fingerprint mismatch). This can only happen if reads consume
    `active_epoch`; the session GUC is never set in this drill. This is the direct
    proof the manifest epoch drives reads.
  - **all_publish_swap_ok** — publishing epoch 2 on EVERY node swaps the epoch and
    the top-k matches the baseline (a real publish changes what queries consume,
    losslessly).
- `concurrency_scan_insert_epochswap pass=true` — now performs COORDINATED
  all-node epoch publishes under scan + insert load. Each scan returns wholly from
  one epoch, or fail-closes on a transient mismatch during the swap window
  (one-epoch-per-scan, never a torn read).
- `RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0` — recall unchanged.
- `recovery … mismatched_ids=0 recovered=true`; all 12 fault drills `pass=true`;
  GATE PASS.

## Validation

- 70 ec_distann unit tests pass (`cargo test --no-default-features --features
  pg18 --lib am::ec_distann::`); clippy clean (extension + CLI).
- Extension `.so` swapped to the debug build for the run, reverted to the shared
  release build after (this change adds no SQL, only read-path epoch sourcing).

## Note / follow-up

The distributed-delete *routing* (coordinator → owner tombstone on a base DELETE)
remains a later M3 wire-up (dml.rs); publish/retire are per-node endpoints the
coordinator drives. This packet wires epoch *consumption* on the read path; the
FR-082 lifecycle write endpoints (publish/retire/force-retire/status) already
landed in packets 014/016.

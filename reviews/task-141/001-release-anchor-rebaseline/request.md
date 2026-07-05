# Task 141 Review Request: Release Anchor Rebaseline

Please review packet 001 for Task 141.

This packet closes the P0 bench-integrity slice:

- `spire-local-multinode` now installs release `ecaz.so` by default, with an explicit debug escape hatch.
- `ecaz bench suite` release guard now covers `spire-pipeline` and every latency-emitting step kind in this path.
- Per-node backend profiles are recorded in suite manifests and stamped into every cited `spire-pipeline` latency/profile row.
- Anchor cells were rerun on the release multinode fixture: 50k n128/b0, 50k n1024/b0, and 100k n1024/b0.
- A matched debug-vs-release 50k n1024/b0 cell quantifies the distortion.
- Task 123 packet 009's 87 ms row is reconciled against the new release anchor and the later 200-query Task 123 packet 017 result.
- Task 139 packet 001 has a taint annotation at `reviews/task-139/001-phase1-nlists-boundary-grid/artifacts/manifest.md`.

## Code Since Early Feedback

Reviewer feedback in `feedback/2026-07-04-01-agent-ix.md` called out the debug-install bypass, missing per-node guard tests, and the socket-discovery fallback. Commit `692626971` addresses those:

- Result rows now carry `backend_build_profile` and `backend_node_profiles`.
- The release guard validation is extracted and unit-tested against a debug backend node.
- `spire-local-multinode` suites fail closed if socket-port discovery cannot verify worker nodes.
- Non-multinode fallback is logged rather than silent.

## Evidence

Packet manifest: `reviews/task-141/001-release-anchor-rebaseline/artifacts/manifest.md`.

Key nested results:

- `artifacts/release-50k-n128-b0-r2/bench-suite/results.jsonl`
- `artifacts/release-50k-n1024-b0-r2/bench-suite/results.jsonl`
- `artifacts/release-100k-n1024-b0-r2/bench-suite/results.jsonl`
- `artifacts/debug-50k-n1024-b0-r2/bench-suite/results.jsonl`

The end-to-end production-read query anchor is:

| Cell | nprobe | build | query p50 | query p95 | recall@10 |
| --- | ---: | --- | ---: | ---: | ---: |
| 50k n128/b0 | 64 | release | 77.131 ms | 86.048 ms | 0.9865 |
| 50k n1024/b0 | 64 | release | 108.127 ms | 116.733 ms | 0.9375 |
| 100k n1024/b0 | 64 | release | 113.852 ms | 119.286 ms | 0.9105 |
| 50k n1024/b0 | 64 | debug | 608.748 ms | 630.598 ms | 0.9375 |

Matched 50k n1024/b0 debug/release distortion:

| nprobe | query p50 ratio | profile total p50 ratio | recall delta |
| ---: | ---: | ---: | ---: |
| 8 | 5.27x | 4.94x | 0.0000 |
| 16 | 5.38x | 5.02x | 0.0000 |
| 32 | 5.49x | 5.35x | 0.0000 |
| 64 | 5.63x | 5.63x | 0.0000 |
| 96 | 5.73x | 5.94x | 0.0000 |

## Validation

Passed:

```text
cargo test -p ecaz-cli release_guard -- --nocapture
cargo test -p ecaz-cli socket_port_discovery_empty_dir_documents_coordinator_fallback -- --nocapture
cargo build -p ecaz-cli
```

`cargo build -p ecaz-cli` emitted one existing warning about `LoadedDistributedPlacementConfig.path`.

## Review Focus

1. Confirm the backend provenance row stamping is enough to make debug escape-hatch rows self-identifying.
2. Confirm the packet uses the end-to-end query rows as anchors and treats production-read profile rows as attribution only.
3. Confirm the 87 ms reconciliation is sufficient to stop citing Task 123 packet 009 as comparable debug-grid evidence.
4. Confirm the Task 139 taint annotation satisfies Task 141 acceptance criterion 4.

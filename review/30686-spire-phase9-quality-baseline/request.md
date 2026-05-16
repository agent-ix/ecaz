# Review Request: SPIRE Phase 9 Quality Baseline

Checkpoint: `4ec7a087` (`Record SPIRE phase 9 quality baseline`)

## Scope

- Opens the canonical local pre-Phase-9.7 baseline packet required by
  `review/external/2026-05-09-phase-9-closeout-requirements/README.md`.
- Records packet-local real10k load, storage, explain, latency, and recall
  artifacts under `artifacts/`.
- Covers `nprobe=8,16,24,32` crossed with `rerank_width=0,25,50` on the same
  isolated SPIRE index.
- Notes the baseline packet in
  `plan/tasks/task30-phase9-spire-graph-architecture.md` while leaving the
  four Phase 9.7 treatment checkboxes open.

## Result Summary

Real10k saturates quickly on this 100-query local baseline:

| rerank_width | nprobe | recall@10 | latency p50 | latency p95 | latency p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 | 8 | 0.9950 | 576.3 ms | 616.3 ms | 629.9 ms |
| 0 | 16 | 1.0000 | 1019.7 ms | 1076.2 ms | 1127.0 ms |
| 0 | 24 | 1.0000 | 1438.7 ms | 1530.5 ms | 1540.2 ms |
| 0 | 32 | 1.0000 | 1896.5 ms | 1932.0 ms | 1954.4 ms |
| 25 | 8 | 0.9950 | 73.9 ms | 101.6 ms | 113.7 ms |
| 25 | 16 | 1.0000 | 112.0 ms | 125.2 ms | 145.4 ms |
| 25 | 24 | 1.0000 | 150.6 ms | 160.4 ms | 168.5 ms |
| 25 | 32 | 1.0000 | 188.1 ms | 197.6 ms | 231.5 ms |
| 50 | 8 | 0.9950 | 78.0 ms | 89.1 ms | 100.9 ms |
| 50 | 16 | 1.0000 | 117.1 ms | 123.5 ms | 130.0 ms |
| 50 | 24 | 1.0000 | 154.7 ms | 168.3 ms | 179.4 ms |
| 50 | 32 | 1.0000 | 192.7 ms | 224.7 ms | 255.2 ms |

The local checked-in fixture is therefore too saturated to prove anisotropic
centroid scoring or IMI quality upside by itself; adaptive `nprobe` still has a
visible latency target because `nprobe=8` is much cheaper with only one
recall@10 miss in the 100-query subset.

## Validation

- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- `target/debug/ecaz corpus load ... --prefix task30_p9_quality_base_c5ed545 ...`
- `target/debug/ecaz bench storage ...`
- `target/debug/ecaz dev sql ... --file artifacts/explain-real10k-nprobe-rerank-matrix.sql ...`
- `target/debug/ecaz bench latency ...` for `rerank_width=0`, `25`, `50`
- `target/debug/ecaz bench recall ...` for `rerank_width=0`, `25`, `50`
- `git diff --check`

## Review Focus

- Confirm the packet is sufficient as the canonical local baseline for Phase
  9.7 treatment packets.
- Confirm keeping the four Phase 9.7 items open is correct: this packet records
  baseline evidence only.
- Confirm the saturated recall result is enough evidence for the next adaptive
  `nprobe` slice and for later anisotropic/IMI disposition decisions.

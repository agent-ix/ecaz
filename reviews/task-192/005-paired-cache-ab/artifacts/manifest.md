# Artifact manifest — Task 192 packet 005

- Head / runner / extension SHA: `6578da92fdf43c14742e4395d71cb570bef31501`
- Task bucket / packet: `reviews/task-192/005-paired-cache-ab/`
- Source execution bucket: `reviews/task-192/002-isolated-ab-100k/artifacts/run/`
- Run window: 2026-07-21 22:19:40–22:53:45 PDT
- Host/lane: local Intel, three isolated PG18 instances, physical disjoint-owner generation
- Fixture: `ecaz bench suite`, one index per physical table plus a separate same-data single-index control table
- Storage / rerank / search: persisted training-landmark head, RaBitQ stored neighbor codes, exact co-located row-tier rerank, lazy10 materialization, BW=4, H=100
- Corpus/query: `ec_real_100k`; raw TSVs are intentionally not committed; query SHA-256 `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Installed extension: release profile, 24,212,288 bytes; target and installed SHA-256 `4065f1f55a89b43333facf3609e91499287d8f6f175a34c649dd4933cfa90e32`
- Command: `target/debug/ecaz bench suite run --config reviews/task-192/002-isolated-ab-100k/artifacts/task192-suite.json --database tqvector_bench --log-file reviews/task-192/002-isolated-ab-100k/artifacts/suite-run-release.log`
- Audit: `target/debug/ecaz bench suite audit --config reviews/task-192/005-paired-cache-ab/artifacts/task192-suite.json --database tqvector_bench --log-file reviews/task-192/005-paired-cache-ab/artifacts/suite-audit.log`
- Suite status: succeeded=1, failed=0, missing artifacts=0, stale=0; duration 2,123,988 ms

## Files

| Artifact | SHA-256 | Purpose |
|---|---|---|
| `task192-suite.json` | `c194a8afdd5ae491348281b30b1351c7ae58adb0d8e74381c0b0538f6ee55aa4` | Checked-in paired suite config |
| `suite/suite-manifest.json` | `e8a65e5613c38b2682d9cefd0179a5abf27f5c9e0049f4d63540ddf3dca6cc7b` | Exact command, runner SHA, timing, and success state; paths retain the source packet location |
| `suite/results.jsonl` | `8f002ebba256bfc1ac06a7f5e94dcf57859e4bd4fa4cd15d6a58822c74a23684` | Structured recall, latency, storage, topology, stage, and work rows |
| `suite/endpoint-cache-100k/distann-multinode-summary.log` | `b10b95ef60ccb04752272f42608fce65c2e359f4c11bf31d5c41065f93377158` | Provenance and complete parsed benchmark summary |
| `suite/endpoint-cache-100k/physical-owner-validation-uncached-recall.log` | `6565e303363928d2ee2e37bd398bc98de8d5389c5d455a1dd5ee3bf32a6c01dc` | Baseline recall table |
| `suite/endpoint-cache-100k/physical-owner-validation-uncached-latency.log` | `c888a840b5733c94f82e453e8047bada287bac9134e61143ab4405475f4d216a` | Baseline latency and counters |
| `suite/endpoint-cache-100k/physical-owner-validation-cached-recall.log` | `b16301d5c03adb69468ac8f7663995e53ff6cea3360ddf3c178fa67a38f1ed4c` | Candidate recall table |
| `suite/endpoint-cache-100k/physical-owner-validation-cached-latency.log` | `ad525c6ed2a9dc143a484de412415b6cd51a2d0315762d116a8fd0e8a293a81a` | Candidate latency and counters |
| `suite-audit.log` | `fbad5bff78a3e8c79adeb38878f901b9e1ac9aff3a6ab942728e2043a59219e1` | Preflight audit pass |

## Key cited rows

- Recall: uncached `0.9625` (CI 0.9532–0.9700); cached `0.9625` (same CI).
- Latency: uncached mean/p50/p95/p99 `23.90/23.80/27.10/28.60 ms`; cached `20.80/20.60/24.30/24.70 ms`.
- Open/validate: uncached `6.960390 ms/scan`; cached `0.026082 ms/scan`.
- Remote materialization: uncached `10.522218 ms/scan`; cached `7.148999 ms/scan`.
- Payload SQL: uncached `8.763685 ms/scan`; cached `8.927275 ms/scan`.
- Storage: both arms share `2,496,626,688` physical-generation bytes and `24,576` control-index bytes.

# Artifacts Manifest

## synthesis

- head SHA: `f93f130b`
- packet/topic: `30131-task28-ivf-current-gate-status`
- lane: Task 28 IVF current gate status synthesis
- fixture: source packets 30076, 30077, 30079, 30080, 30081, 30082, 30102, 30116, 30117, 30125, 30126, 30127, 30129, 30130
- storage format: synthesis
- rerank mode: synthesis
- command: not applicable; synthesis-only packet
- timestamp: 2026-04-28 17:55 PDT
- isolation: not applicable
- key result lines:
  - A2: packet 30129 records 1M vacuum at `nlists=8,32,64`, RSS peak 364476-370600 KB.
  - A9 100k: packet 30126 records recall@10 `0.9920`, recall@100 `0.9552`, p50/p95/p99 `169.3/191.2/194.4 ms`.
  - A9 990k: packet 30130 records recall@10 `0.9860`, recall@100 `0.9509`, p50/p95/p99 `1029.2/1169.1/1224.4 ms`, index size `177 MB`.

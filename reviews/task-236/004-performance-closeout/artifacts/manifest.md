# Task 236 packet 004 artifact manifest

- Head SHA: `0ed7ffaebe749e193e1324ac3ce2624bfb12baf4`
- Task / packet: `reviews/task-236/004-performance-closeout/`
- Date: `2026-08-24`
- Lane: review routing for the completed local PG18 release three-owner
  plaintext/TLS 10k/50k/100k A/B
- Storage / rerank: production RaBitQ neighbor codes, persisted head,
  `BW4/H100/L32`, lazy-10, no traversal replica
- Isolation: fresh one-index-per-table physical cluster per arm; no fixture
  reuse and no shared-table surface

This closeout packet does not duplicate the immutable benchmark artifacts.
The source of truth is:

- `benchmarks/task236-distann-secure-transport-ab/manifest.md`
- `benchmarks/task236-distann-secure-transport-ab/transport-ab-suite.json`
- `benchmarks/task236-distann-secure-transport-ab/artifacts/suite-manifest-{10k,50k,100k}.json`
- `benchmarks/task236-distann-secure-transport-ab/artifacts/results-{10k,50k,100k}.jsonl`
- `benchmarks/task236-distann-secure-transport-ab/artifacts/run-<scale>/<arm>/distann-multinode-summary.log`

The security/correctness source is packet 003:

- `reviews/task-236/003-pg18-tls-secret-matrix/artifacts/manifest.md`
- `reviews/task-236/003-pg18-tls-secret-matrix/artifacts/results-final-pass.jsonl`
- `reviews/task-236/003-pg18-tls-secret-matrix/artifacts/secret-exposure-scan.log`

Key result lines are reproduced in `request.md`; all cited values derive from
the normalized JSONL above. External PostgreSQL run directories and the
isolated Cargo target are not evidence and were removed after capture.

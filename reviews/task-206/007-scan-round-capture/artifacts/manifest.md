# Artifact manifest

- Task bucket: `reviews/task-206/007-scan-round-capture/`
- Code checkpoint: `8eea5f965` (`Capture physical head membership and scan rounds`).
- Validation date: 2026-08-04.
- Matrix rerun: no; this packet contains the parser fix and focused validation.
- Durable evidence: `validation.md`.
- The parser accepts both direct `ec_distann_scan_round` lines and the nested
  `[distann-multicluster] [postgres notice] ...` fixture-summary form.

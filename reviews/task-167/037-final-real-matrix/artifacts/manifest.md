# Task 167 packet 037 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `037-final-real-matrix`.
- Code/runtime checkpoint: `5568aba17026f74de7d5685816ce2a923f160d60`.
- Preregistered config:
  `reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json`
  (SHA-256
  `52861345c295613b11765059ad6a080c0111bfa9daf15f24930b444a0f6841cb`).
- Release CLI SHA-256:
  `d1a028e828a452717342935e0af11b971a4d5d0d110853994485e977463c5977`.
- Installed production PG18 `ecaz.so` SHA-256:
  `1d9e825a6b6645e4ca886089d0761ec7636252c5d9507981a64e1c22a6399337`.
- Build/install provenance and synthetic prerequisite:
  `reviews/task-167/036-rollback-vector-normalization/artifacts/manifest.md`.
- Scale order: 10k gate first; 50k and 100k only after 10k succeeds.
- Storage/rerank/index lane: three-owner physical `ec_distann`, graph degree
  32, head cap 4,096, beam width 4, 100 hop rounds, no rerank variant.
- Isolation: each scale uses one logical index in its own external run
  directory under `/home/peter/.ecaz/clusters/`; cluster directories are
  operational state and will be removed after their packet evidence is
  durable.
- Runtime status: not started. No recall, latency, storage, concurrency,
  parity, saturation, or append A/B result is claimed yet.


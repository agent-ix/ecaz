# Task 167 packet 037 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `037-final-real-matrix`.
- Initial failed runtime checkpoint:
  `5568aba17026f74de7d5685816ce2a923f160d60`.
- Initial preregistered config:
  `reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json`
  (SHA-256
  `52861345c295613b11765059ad6a080c0111bfa9daf15f24930b444a0f6841cb`).
- Corrected code checkpoint: `a49ffd92a` (packet 038).
- Corrected preregistered config:
  `reviews/task-167/037-final-real-matrix/artifacts/task167-final-real-suite.json`
  (SHA-256
  `d65a9690fa07e8c735de0e30172cca9a60b8f97359327e6f7b2d83bece5eefaa`).
- Counter mode: benchmark-only query-stage counters disabled; production
  Task 167 insert-work counters remain unconditionally captured and validated.
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
- First 10k command:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json --artifact-dir reviews/task-167/037-final-real-matrix/artifacts/10k-gate --only physical-10k --log-file reviews/task-167/037-final-real-matrix/artifacts/10k-gate/suite-run.log`.
- First 10k result: failed, suite exit code 1. Release preflight passed at clean
  SHA `5568aba17`, profile `release`, features `pg18`; ready/published topology,
  initial serving, and both remote-owner proofs passed. The recall child
  completed before the latency child failed.
- Failure boundary: `--distann-stage-counters` requested
  `ec_distann_stage_scoring_snapshot()`, but that SQL function is gated by the
  `distann-head-attribution-benchmark` extension feature and is absent from the
  required production `pg18` build.
- Failure evidence: `10k-gate/suite-manifest.json` SHA-256
  `cc6b61c3695aee38b702e30a928004c17c7a4de371778195f99de0560f141074`;
  `physical-10k/distann-local-multinode.log` SHA-256
  `6bb9ca72013e1594c5904743784d51c31dc97a29f8f5f36cfe353fcf8c415902`.
- Partial recall artifacts are retained as diagnostics only:
  `physical-production-recall.log` SHA-256
  `203bce6ac08ef6e274e9b34fbb14382a7ff47b74784df6306059819b6e79a6ba`;
  predictions SHA-256
  `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`.
- The run stopped before concurrency, natural retry, saturation, parity,
  append A/B, latency, and storage completed. No 10k result is claimed, and
  50k/100k were not selected.

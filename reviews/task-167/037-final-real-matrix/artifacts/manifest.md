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
- Corrected exact runtime head:
  `cce839647834e2bd3880ec826430af04c6175b0e`.
- Corrected release CLI SHA-256:
  `9fb0d0f310c11bdaa15cef29c43c5508b97ab6dc5bfd2a9f41a1d14b6bab5545`.
- Corrected installed production PG18 `ecaz.so` SHA-256:
  `86870052ba73f91c73016dab5d5273b538e895865f938f323e09921107443f3f`.
- Corrected build/install logs and suite audit:
  `reviews/task-167/038-production-counter-separation/artifacts/manifest.md`.
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
- Corrected 10k command:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/037-final-real-matrix/artifacts/task167-final-real-suite.json --artifact-dir reviews/task-167/037-final-real-matrix/artifacts/10k-final --only physical-10k --log-file reviews/task-167/037-final-real-matrix/artifacts/10k-final/suite-run.log`.
- Lane / fixture / storage / rerank: real `ec_real_10k`, 10,000 rows,
  dimension 1,536, three physical owners, graph degree 32, head cap 4,096,
  beam width 4, 100 hop rounds; physical generation storage; no rerank
  variant. One logical index was isolated under
  `/home/peter/.ecaz/clusters/task167-final-20260821-10k`.
- Corrected provenance: unanimous release nodes at embedded SHA
  `cce839647834e2bd3880ec826430af04c6175b0e`, features `pg18`, debug override
  false.
- Corrected query results: physical recall `0.9990`, mean/p95 latency
  `15.70/18.00 ms`; single-control recall `0.9990`, mean/p95 latency
  `2.73/3.19 ms`; cluster graph-side bytes `76,095,488`, raw-vector bytes
  `61,440,000`, amplification `1.238533x`.
- Insert results: 160 measured rows per arm; physical throughput `1.269 rows/s`
  versus single control `2.393 rows/s`; append enabled/disabled ratio
  `1.029029`; eight production insert-work metrics each reported 160 attempts
  and passed graph-degree bounds.
- Concurrency results: four scanners and two writers passed all 24 reverse-edge
  checks and both shared-target backlinks; natural retries `9`, steady retries
  `0`; routed delete/VACUUM and final topology passed.
- Corrected 10k blocking result: post-insert physical-vs-fresh distinct recall
  was `0.541667` for both append arms, below the required `0.80`; the summary
  recorded `pass=false`.
- Harness defect: the CLI returned the parity line without bailing when
  `parity_pass` was false, so `suite-manifest.json` incorrectly records the
  step as succeeded with exit code 0. Treat this as a failed 10k gate.
- Corrected artifact set: 15 files, 388 KiB; no corpus TSVs, truth caches,
  polling snapshots, tunnel state, or raw SSM trees are present.
- Artifact hashes: `10k-final/suite-manifest.json`
  `7bbbe104ce8bc28db60c3934d51eac59b7b5c8dcf64c688b7e34cf2ac88b075c`;
  `results.jsonl`
  `5e895ed450c75ca45685e29669dcab576748f05b3ed8bf94bd72bf7e40bfd047`;
  `physical-10k/distann-multinode-summary.log`
  `5f4414419b31be8321ea114307b8112d16877ea711935493de4807e72cc9d1a7`;
  `physical-10k/distann-local-multinode.log`
  `806313a972edc50b76c933be69ca0b78f212c37168367f808700341771f46dbf`.
- Recall/latency hashes: physical recall
  `67a80ee255c11a225d8808531c794660f904ae1571a3d6cb1140487f8261b243`,
  physical latency
  `72d8fe72f558a1d594fcad47a3d71d4ba2a2d922905acbfc8964c86b3746e274`,
  single recall
  `efef860e6037f5ec96e464a31c393c1e4c4c8403eb3fe42a1db9a5524310f657`,
  single latency
  `9a45e81a041994dabf7552026b3db95c8d8716481c1de2ad8513e21cce12d34a`.
- 50k/100k were not selected because the 10k correctness gate failed.

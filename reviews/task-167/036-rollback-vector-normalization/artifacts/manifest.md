# Task 167 packet 036 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `036-rollback-vector-normalization`.
- Harness checkpoint: `caa8ad63f`.
- Trigger: packet 035 passed the clean-head synthetic gate but the isolated
  `mi` rollback index emitted the unit-normalization warning.
- Change isolation: CLI fixture SQL only; no extension product code changed.
- Coverage: initial rollback corpus, injected failing insert, and stable-id
  replacement UPDATE use the shared deterministic unit-vector expression.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 2 passed, 0 failed, 497 filtered in
  `validation-test.log` (SHA-256
  `3045f30a93787d345fc301ee75f0abf43f5ddeec4aa7f52fcb290d58044e754f`).
- Exact runtime head: `5568aba17026f74de7d5685816ce2a923f160d60`.
- Release CLI build command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo build --release -p ecaz-cli`.
  Log: `build-cli.log`; SHA-256
  `ada8da49ad4496b37b190f66f5050d81783bc2831964bb79dcff6b80690f1d30`.
- PG18 extension install command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
  Log: `install-extension.log`; SHA-256
  `d76e917e5ba6579b87d2bccd0a2bfabff217869ee2e931bb2382d9263e6db11b`.
- Artifact hashes: CLI
  `d1a028e828a452717342935e0af11b971a4d5d0d110853994485e977463c5977`;
  installed PG18 `ecaz.so`
  `1d9e825a6b6645e4ca886089d0761ec7636252c5d9507981a64e1c22a6399337`.
- Suite audit passed four steps; `suite-audit.log` SHA-256
  `27bdc081d219d0b412ec2ee6b69c9a855ff68f612501aaef38bf81a64d31dc0d`.
- Synthetic command:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json --artifact-dir reviews/task-167/036-rollback-vector-normalization/artifacts/smoke-synthetic-final --only concurrency-synthetic --log-file reviews/task-167/036-rollback-vector-normalization/artifacts/smoke-synthetic-final/suite-run.log`.
- Lane / fixture / storage / rerank: synthetic, 2,000 rows, dimension 4,
  three physical owners, graph degree 8, head cap 4,096, beam width 4,
  20 hop rounds; physical generation storage; no rerank variant. One logical
  index was isolated under
  `/home/peter/.ecaz/clusters/task167-recovery-20260821-synthetic`.
- Provenance: clean embedded SHA `5568aba17026f74de7d5685816ce2a923f160d60`,
  release profile, features `pg18`, debug override false, unanimous across all
  three nodes.
- Key results: serving and both remote-owner CustomScan proofs passed;
  mid-insert rollback and stable replacement passed; controlled target filled
  `6 -> 8`; both writer backlinks survived; natural retries were `3` (one per
  owner); steady retries were `0`; routed delete/VACUUM and topology passed.
- Normalization result: no `expects unit-normalized source vectors` warning is
  present in the final suite output or owner logs.
- Result artifacts: `smoke-synthetic-final/suite-manifest.json` SHA-256
  `868e3748b543670f1b26e53022765381f04a3b958c3ba8ef9f9888956cd18de6`;
  `results.jsonl` SHA-256
  `7bc63e985e839fdab3a03699b2f79937217703d86ec7512c4be2f4b36e38014f`;
  summary SHA-256
  `636f6985b2965fdc9d2247c107d1523509fa590e35ef788b16abadc36943a72c`.
- Result: synthetic suite step passed with exit code 0.
- The 10k/50k/100k steps were not selected. No recall, latency, or storage
  closeout result is claimed yet.

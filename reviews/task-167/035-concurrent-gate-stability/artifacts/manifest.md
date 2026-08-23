# Task 167 packet 035 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `035-concurrent-gate-stability`.
- Harness checkpoint: `d799dc4fe`.
- Trigger: packet 034 synthetic diagnostics crossed the snapshot failure
  boundary but reported controlled target neighbors `6 -> 6`, natural retries
  `0`, and `physical_concurrent_insert_query pass=false`.
- Change isolation: CLI fixture only; no extension product code changed.
- Synthetic contract: deterministic physical-fixture vectors are normalized
  before `encode_to_ecvector`, matching `ecvector_distann_ip_ops` assumptions.
- Capacity contract: a target selected with two open graph slots receives only
  the two controlled writer near-duplicates before saturation is checked.
- Overlap contract: four scanners and two writers share one start barrier;
  scanners run at least 12 queries and continue while either writer is active,
  with a maximum of 192 queries per scanner.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 2 passed, 0 failed, 497 filtered in
  `validation-test.log` (SHA-256
  `79fa5b0a855c1269f0f4ccf3af0ba61ba0f15f200c9b77f07d99b533b479e2fa`).
- Exact runtime head: `31e6c9138bb43bce79cfd43714b4053a9b32c4c1`.
- Release CLI build command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo build --release -p ecaz-cli`.
  Log: `build-cli.log`; SHA-256
  `980bbde030203067a75b87e988193f6ee64d35734bc2e66e6fa6ca7d9d70ec33`.
- PG18 extension install command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
  Log: `install-extension.log`; SHA-256
  `07dad484aa7d0533eb36e9dc2a708f475dac16a158f7e51757aec2b7aa5d9b41`.
- Artifact hashes: CLI
  `36674074235def68700515416cdf22a81b71e3a166a20fb244d94df0f347c6d2`;
  installed PG18 `ecaz.so`
  `e051d9f0dd503a3f4618489c9152fce91baafca9e232bbc475585f27654d6659`.
- Suite audit passed four steps; `suite-audit.log` SHA-256
  `27bdc081d219d0b412ec2ee6b69c9a855ff68f612501aaef38bf81a64d31dc0d`.
- Synthetic command:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json --artifact-dir reviews/task-167/035-concurrent-gate-stability/artifacts/smoke-synthetic-clean-head --only concurrency-synthetic --log-file reviews/task-167/035-concurrent-gate-stability/artifacts/smoke-synthetic-clean-head/suite-run.log`.
- Lane / fixture / storage / rerank: synthetic, 2,000 rows, dimension 4,
  three physical owners, graph degree 8, head cap 4,096, beam width 4,
  20 hop rounds; physical generation storage; no rerank variant. The run used
  one isolated logical index in
  `/home/peter/.ecaz/clusters/task167-recovery-20260821-synthetic`.
- Provenance: clean embedded SHA `31e6c9138bb43bce79cfd43714b4053a9b32c4c1`,
  `extension_build_profile=release`, `extension_features=pg18`,
  `debug_override=false`, unanimous across three nodes.
- Key results: initial serving passed; both remote-owner CustomScan proofs
  passed; mid-insert rollback and stable UPDATE replacement passed; controlled
  target filled `6 -> 8`; both writer backlinks were present; natural retries
  were `3` (one per owner); steady retries were `0`; routed delete/VACUUM and
  final topology gates passed.
- Result artifacts: `smoke-synthetic-clean-head/suite-manifest.json` SHA-256
  `e996b4fd923a23fca767b06fb481564ac1a4b044df444adcd636296fc09e35d7`;
  `results.jsonl` SHA-256
  `c7d7e3b67399d468ff1efaec4c01d6cdf48aee5d3ace8b5b1da52adc3519be0b`;
  `concurrency-synthetic/distann-multinode-summary.log` SHA-256
  `636f6985b2965fdc9d2247c107d1523509fa590e35ef788b16abadc36943a72c`.
- Result: synthetic suite step passed with exit code 0.
- Carried follow-up: the isolated `mi` rollback subfixture still used the old
  unnormalized deterministic generator and emitted one build warning. Its
  rollback assertion passed, but the generator will be normalized before the
  final real-corpus matrix checkpoint.
- The 10k/50k/100k steps were not selected. No recall, latency, or storage
  closeout result is claimed by this packet.

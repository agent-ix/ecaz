# Task 167 packet 039 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `039-post-insert-parity-gate`.
- Code checkpoint: `6968f0a3d`.
- Trigger: packet 037's corrected production 10k run recorded
  `physical_benchmark_post_insert_fresh_rebuild ... pass=false` at distinct
  recall `0.541667`, but the suite manifest incorrectly recorded exit code 0.
- Change isolation: CLI result propagation only; no extension product code
  changed and no benchmark threshold changed.
- Behavior: exact `0.80` remains accepted; either append-enabled or overall
  distinct recall below `0.80` returns an error containing the complete
  measurement line after fixture cleanup.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 7 passed, 0 failed, 497 filtered in
  `validation-test.log` (SHA-256
  `05cedac360e285ebac335aa4b96c4376bade9c33eabc79cc5649a96d459869f3`).
- Measurement source:
  `reviews/task-167/037-final-real-matrix/artifacts/10k-final/physical-10k/distann-multinode-summary.log`
  at runtime SHA `cce839647834e2bd3880ec826430af04c6175b0e`.
- Blocking result: append-disabled recall `0.541667`, append-enabled recall
  `0.541667`, overall distinct recall `0.541667`, required `0.80`,
  `pass=false`.
- No runtime rerun is claimed by this packet. The production measurement is
  preserved in packet 037; 50k/100k remain unrun.

# Task 183 fixed-budget coverage manifest

- Pre-registration head: `de3b54f82`
- Task bucket / packet: `reviews/task-183/003-fixed-budget-coverage/`
- Implementation head: `f8612af1f`
- Lane: fixed-budget policy implementation and frozen 100k screen; measurement
  pending
- Frozen baseline: Task 183 packet 002 trained RaBitQ at 100k, recall 0.9625
  and warm p50 43.8 ms
- Frozen upper reference: same-generation owner-scan RaBitQ recall 0.9970;
  never selectable
- Candidate input: only disjoint training rows 201--400
- Evaluation input: held-out rows 1--200; unavailable to builders
- Fixed query work: cap 4,096, exact head scoring, 32 seeds, BW4/H100
- Fixed graph/codec: degree 32, RaBitQ neighbor codes/traversal, exact rerank
- Isolation: future policy arms use fresh one-index-per-table physical
  generations through a checked-in `ecaz bench suite` config
- Timestamp: 2026-07-17 America/Los_Angeles

## Validation artifacts

- `pg18-feature-check.log`
  - command: `cargo check --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
  - result: pass
  - SHA-256: `234e41dc1dbbb759d753feff9b14b4be83fe1a46f407b62fa21a10817c1d93e2`
- `policy-tests.log`
  - command: `cargo test --lib --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark' benchmark_landmark_policies_are_deterministic_and_bounded`
  - result: 1 passed, 0 failed
  - SHA-256: `4b417c23ecca3c0dd8a6d1ce37ade19660cf1ccae0648a6ee9f50776b8e0cc5e`
- `suite-tests.log`
  - command: `cargo test -p ecaz-cli distann_local_multinode_expands_task183_training_policies`
  - result: 1 passed, 0 failed
  - SHA-256: `8844897ec073c95b920bbd14a540335f5f549b3ed886786a4f3f3661ef0e0afa`
- `fixed-budget-100k-suite.json`
  - runner: `ecaz bench suite`
  - arms: `training_landmarks`, `training_region_balanced`, and
    `training_query_facility`
  - isolation: fresh physical generation per sequential step
  - fixed scan: exact score all 4,096 head entries; return 32; BW4/H100;
    RaBitQ traversal; exact rerank
  - SHA-256: `311f6d2d695013bea682d978f5c235006542175a02ad04c0c17ce15af81bce5c`
- `audit.log`
  - command: `target/debug/ecaz bench suite audit --config reviews/task-183/003-fixed-budget-coverage/artifacts/fixed-budget-100k-suite.json`
  - result: pass, 3 steps
  - SHA-256: `f2c78ac02355a9b6350e7b98fac794c95113b0d73056dcdc2b6002048f826b28`
- `dry-run.log` and `run/suite-manifest.json`
  - command: `target/debug/ecaz bench suite run --config reviews/task-183/003-fixed-budget-coverage/artifacts/fixed-budget-100k-suite.json --dry-run`
  - result: all three commands expand with exact head scoring, 32 returned
    seeds, and the intended single policy difference
  - log SHA-256: `0758fe562f44850f864893754f5d7fd95538f8a9315dfe600825866062d974cc`
  - dry-run manifest SHA-256: `46dbc30e24e0144ad00145c9c0093cb5a001535131c1ff08d3e1d47d990db46f`
- Installed release measurement head: `c644feb956c06d5b7250d9c718887fe9205fa40b`
- `implementation-install.log`
  - command: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
  - result: pass
  - log SHA-256: `a8301649135666ec2adc74f00fedc15f2d887bf539e55e1c7d03291680c5acbb`
  - installed `ecaz.so` SHA-256: `8295f3930b64ce483bee21bdf1d7b177a23fb4fda7b79b4ef09c7371394e4996`
- `cli-release-build.log`
  - command: `cargo build --release -p ecaz-cli`
  - result: pass; only the pre-existing unused `path` field warning
  - log SHA-256: `3109d00d9149b5f82c63ef18277d4d622977d9ad63ad968f187a44d3ff94c09e`
  - release CLI SHA-256: `0ad2cb92d9a07c0852f37c8838a95a7c49105f0251e963c53b893a0b7cc04810`

## Frozen policy names

- `training_landmarks`: Task 182 frequency/coverage control
- `training_region_balanced`: deterministic geometry-region round-robin
- `training_query_facility`: deterministic rotated query-neighborhood
  round-robin

No Phase 2 result is claimed yet. Corpus/query TSVs, truth caches, node logs,
and live run directories will not be committed.

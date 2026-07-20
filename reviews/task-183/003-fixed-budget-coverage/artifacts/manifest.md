# Task 183 fixed-budget coverage manifest

- Pre-registration head: `de3b54f82`
- Task bucket / packet: `reviews/task-183/003-fixed-budget-coverage/`
- Implementation head: `f8612af1f`
- Lane: fixed-budget policy implementation and completed 100k screen
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

## Measurement result

All three suite steps succeeded with no missing or stale artifacts. The
installed release extension SHA was unanimous at
`c644feb956c06d5b7250d9c718887fe9205fa40b`.

| Policy | Recall | Warm p50/p95/p99/max ms | Head cache bytes | Build/publish ms |
| --- | ---: | ---: | ---: | ---: |
| `training_landmarks` | 0.9625 | 41.0/54.5/57.9/60.1 | 25,892,203 | 883,763/1,015,695 |
| `training_region_balanced` | 0.9625 | 42.8/55.1/60.4/63.7 | 25,892,967 | 893,975/1,027,241 |
| `training_query_facility` | 0.9625 | 41.6/55.7/61.6/63.0 | 25,892,620 | 892,511/1,025,945 |

- Recall CI for every arm: 0.9532--0.9700 (200 queries / 2,000 top-10 trials).
- Physical generation bytes for every arm: 2,496,659,456.
- Head sample digests: control
  `50261d7627471fa3329535cd017ead6102cb220c62ca12dc9715178d05333b54`;
  region-balanced
  `5cf7924a28a990f8ff64f6fc9eaeb681ef66a382be49c048ca0c53a4d9c1e109`;
  query-facility
  `2d796191a69e2e717a77286b051cff58e3be7e39df069f144a8a59c2a1512c44`.
- Aggregate top-32 seed digest for every arm:
  `488caa73ad3f6c22864f9af309569ba4fe6edd72c8d535e71eec7bff78af6d50`.
- Coverage diagnostics for every arm: zero fraction 0.015, exact overlap
  0.51328125, owner membership 0.5503125, 182 represented query regions.
  Bounded overlap is 0.50671875 control, 0.5065625 region-balanced, and
  0.5059375 query-facility.
- Decision: both alternative builders are NO-GO. There is no Phase 2 winner;
  therefore Phase 3 is conditionally skipped and the retained control advances
  only to Phase 4 latency attribution.

## Measurement artifacts

- `report.md`
  - command: `target/release/ecaz bench suite report --artifact-dir reviews/task-183/003-fixed-budget-coverage/artifacts/run --output reviews/task-183/003-fixed-budget-coverage/artifacts/report.md`
  - result: 3 succeeded; 0 failed, skipped, missing, or stale
  - SHA-256: `2a12a52e0795fd851a89ab49da77e42bfa40c50e4312f5c7107d4ec6dbe6e968`
- `status.log`
  - command: `target/release/ecaz bench suite status --config reviews/task-183/003-fixed-budget-coverage/artifacts/fixed-budget-100k-suite.json --artifact-dir reviews/task-183/003-fixed-budget-coverage/artifacts/run`
  - result: completed 3, failed 0, missing 0, stale 0
  - SHA-256: `3d7e627c6214fb09d86bed65b9ab84be2b24bec0544949c7da2c7e96ff2f3e51`
- `run/suite-manifest.json`: SHA-256
  `4d2be06b8ac61754ef4f36c4bdef8052bc2d2d9d374187abb692cb7c6d85b5cc`
- `run/results.jsonl`: SHA-256
  `24e637e7c9a3c09466f7c5fc722655fda83b85fe3f17227aadd5e58f79636f59`
- Control summary / recall / latency SHA-256:
  `173fe4f2556cc117e358d05af50fc1b4c11e2cb80673ee06c7320c6630a8fafc`,
  `76c94d445705aca8aac642f8b741545b5711c7b779ec0cf652a385823b14084f`,
  `76fd18071556e48823c8b2065007a0a5814045706b51e95459683a8d4980baf4`.
- Region-balanced summary / recall / latency SHA-256:
  `3c16fd2c485af35b32400db72414428f782d71c3fbae8a681e3807723b52bd6b`,
  `e4c828e655d85efdf8c77b9258378d16cc984404ddfa4c6622ec8184105bf5a1`,
  `f30d1595e97fd05165abb5637e76b89ac75dae74fdf266712585369d8bb11cd3`.
- Query-facility summary / recall / latency SHA-256:
  `cbdae09bf98b4e90a9c0414214d185b1a29335edf38afa859ff8530861c063db`,
  `b8050b1222016459199c658c55bc64488b4815214113c30098c22e687af08a76`,
  `45637cbc99d12a8092cfab56dcf82852372ac9e55030c9317acdea41d28f0480`.

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

Corpus/query TSVs, truth caches, node logs, polling exhaust, and regenerable
live run state are not committed.

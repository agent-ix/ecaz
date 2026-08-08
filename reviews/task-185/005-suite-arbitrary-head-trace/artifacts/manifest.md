# Task 185 suite arbitrary-head trace manifest

- Task bucket / packet: `reviews/task-185/005-suite-arbitrary-head-trace/`
- Implementation head: `22ed70bb9`
- Scope: CLI/suite wiring plus corrected full-head Phase 1 attribution
- Fixture: physical three-owner sharded `ec_real_100k`
- Storage / lane: physical sharded path; isolated one-index-per-owner surfaces
- Required future runner: `ecaz bench suite`

## Validation artifacts

- `pg18-extension-check.log` — PG18 feature check with the attribution
  endpoint enabled; pass, exit 0; SHA-256
  `5041062555b15d3ec941f6f89d80238761883557d39c71ef57dbd38a8f3d4436`.
- `pg18-cli-check.log` — PG18 `ecaz-cli` check; pass, exit 0; SHA-256
  `34cc9c1ed5e77eb160378b3380f51cb674e278c0294a49dcb2adef17b168f16f`.
- `git-diff-check.log` — source whitespace check; pass, exit 0; SHA-256
  `c84a5896727b37cc104d131f6a9533f9d71337817312bf53eecb62e7c4d08d2c`.
- `pg18-full-head-fix-feature-check.log` — PG18 feature check at
  `22ed70bb9`; pass, exit 0; SHA-256
  `c0aaa10a2d101e9d8ec45aded767898f5b8e551741ceb8269f16c0c3575dc0cb`.
- `pg18-full-head-fix-featureless-check.log` — PG18 featureless check at
  `22ed70bb9`; pass, exit 0; SHA-256
  `53e5530e0bcd401f8a88867383fca65d966ae979ac8569493a7c031ba4f9085e`.

No corpus, truth cache, cluster directory, polling output, or benchmark result
is included. Packet 003 remains the immutable source for the returned-seed
attribution result; this packet adds the arbitrary full-head trace and its
compact exact-truth analysis.

## Runtime result

- Suite config: `artifacts/arbitrary-head-attribution-100k-suite.json`.
- Suite manifest/results: `artifacts/run/suite-manifest.json` and
  `artifacts/run/results.jsonl`.
- Run command: `/home/peter/.cargo-target/release/ecaz bench suite run
  --config reviews/task-185/005-suite-arbitrary-head-trace/artifacts/
  arbitrary-head-attribution-100k-suite.json --log-file
  reviews/task-185/005-suite-arbitrary-head-trace/artifacts/run/suite-run.log`.
- Timestamp: 2026-08-07; extension SHA
  `22ed70bb9d5a39685f0c06db40a4491489516da6`; release profile; three owners;
  remote-owner verification `2`; source rows `100000`.
- Disjoint query contract: evaluation rows `1-200`; training rows `201-400`;
  training slice SHA `30f11df03f6e988adfe531e2bf54b75b8515fa207fee1212dd0774acffec7471`;
  query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Full-head trace: 4,096 persisted candidates validated; positions
  `1,64,512,2048,4096`; `200 x 5 = 1,000` traces. The trace and compact
  analysis are `run/training-landmarks-arbitrary-head-100k/physical-control-
  gateway-head-candidate-trace.json` and
  `physical-control-gateway-head-candidate-analysis.json`.
- Artifact hashes: config
  `82badba578af1088a53e46d05d4e58dea422c6c6fb17c88a894770c17a8dbe2f`;
  suite manifest
  `2d1a116c48eaddec332dcd990cb095b73623988b40cd3ce4e46c523b47ddc5f6`;
  results
  `3669ef168fac58a5f281ad19fad5d656bf03cd8219600b36865885901d6ee864`;
  trace
  `15fef7c88d1486c7ceed428ef7c111fc4fd11b5b56511a760ed66f7ee297355e`;
  analysis
  `b5ced0a7fadac6f1deb1ca1d20c40259b4ec7516269f0e4c86c38b4cf071725f`.
- Control result: recall `0.9205`, 95% CI `[0.9078, 0.9316]`, warm latency
  `40.30 ms` after 10 warmups, construction `934711 ms`, physical generation
  `2496626688` bytes, amplification `1.351147`.
- Attribution result: ordered cumulative exact training truth hits
  `48,83,126,139,147` of `2000`; the final union reaches `69` of 200
  training queries. All `4096` head membership IDs and all trace hit IDs
  joined through the validated fixture identity mapping.
- This is Phase 1 diagnostic evidence only. No selector, default, persisted
  format, graph, traversal, or materialization change is claimed.

## Provenance

- PG18 config: `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- Extension command: `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check -q -p ecaz --no-default-features --features pg18,distann-head-attribution-benchmark`
- CLI command: `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check -q -p ecaz-cli`
- Source validation command: `git diff --check`

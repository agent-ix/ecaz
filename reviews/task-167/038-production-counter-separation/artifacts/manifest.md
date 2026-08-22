# Task 167 packet 038 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `038-production-counter-separation`.
- Code checkpoint: `a49ffd92a`.
- Trigger: packet 037's first 10k attempt requested benchmark-only query-stage
  SQL functions from a production `pg18` extension build.
- Change isolation: CLI preflight and physical benchmark measurement harness;
  no extension product code changed.
- Behavior: Task 167 production insert-work counters are always reset and
  captured for both append A/B arms. The query-stage counter switch is
  validated independently against the extension feature list before the
  fixture starts an expensive corpus build.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 5 passed, 0 failed, 497 filtered in
  `validation-test.log` (SHA-256
  `04a541f88f8c48a9ad5ddae9a3f9660d05542c02d84cc52f2d1dd75eb61bc9c8`).
- Exact runtime head: `cce839647834e2bd3880ec826430af04c6175b0e`.
- Release CLI build command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo build --release -p ecaz-cli`.
  Log: `build-cli.log`; SHA-256
  `56c79bdb2ffb31e098808999a19861dbffdca7d61fec49c9e027eb1ebfba357f`.
- PG18 extension install command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
  Log: `install-extension.log`; SHA-256
  `f186fc7b444c6b00d690c12a87d93eed8fa3dd19d9515e1a5f0e40081c11ace4`.
- Artifact hashes: release CLI
  `9fb0d0f310c11bdaa15cef29c43c5508b97ab6dc5bfd2a9f41a1d14b6bab5545`;
  installed PG18 `ecaz.so`
  `86870052ba73f91c73016dab5d5273b538e895865f938f323e09921107443f3f`.
- Suite audit command:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/037-final-real-matrix/artifacts/task167-final-real-suite.json --log-file reviews/task-167/038-production-counter-separation/artifacts/suite-audit.log`.
- Suite audit passed all three real-corpus steps; `suite-audit.log` SHA-256
  `63bdd2109ae0855c2de0d11eff5b6eaa8e0af93d8cf7a70bba9e3cfbf5a0a703`.
- Corrected real-corpus evidence is pending.

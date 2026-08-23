# Task 167 packet 047 artifact manifest — shipped-only 50k result

- Preregistration head: `63a78b6f0`.
- Owning packet: `reviews/task-167/047-shipped-only-50k/`.
- Suite config: `task167-shipped-only-50k-suite.json`.
- Suite config SHA-256:
  `eb24ee89c998215b191384741b5f0bfa86734731a6d56ef8ab336dc616367cdb`.
- Timestamp: `2026-08-22`.
- Lane: production physical distributed DistANN on PG18, three owners,
  RabitQ neighbor storage, exact fp32 truth, no rerank variant.
- Fixture: isolated `ec_real_50k`, one index per table, external run directory
  `/home/peter/.ecaz/clusters/task167-shipped-only-50k-20260822`.
- Search regime: beam width 4, candidate heap 32, hop rounds 100, top-k 10,
  200 heldout queries plus 48 inserted-neighborhood queries.
- Insert regime at the quality checkpoint: 160 shipped-default robust-prune
  inserts with ID base `2000000`. The append-when-room diagnostic ID base
  `3000000` is excluded until after the quality gate passes.
- Hard gates embedded by code checkpoint `c3b01290b`: inserted-neighborhood
  deficit at most `0.015`, heldout deficit at most `0.007`; source packet 045.
- Runtime output will be packet-local under `artifacts/final-suite/`. Corpus
  data, truth caches, PGDATA, PostgreSQL operational logs, and polling output
  will not be committed.

## Result

- Suite result: failed at the preregistered quality gate after `2177793 ms`
  (about 36.3 minutes). The failure is the expected control-flow result for an
  out-of-band measurement, not an incomplete fixture run.
- Pre-insert production recall: `0.9545` distinct recall over 200 queries,
  recorded in `physical-production-recall.log`.
- Shipped-only exact heldout result after 160 robust-prune inserts:
  physical `0.848722`, fresh rebuild `0.857333`, delta `-0.008611`, allowed
  deficit `0.007000`, gate `false`.
- The gate missed by `0.001611`. The threshold was not changed.
- `diagnostic_candidate_mutation_excluded=true`; because the hard gate stopped
  the step, the append-when-room candidate did not run.
- Compared with packet 045's contaminated `0.026250` heldout deficit, isolating
  the shipped path removed `0.017639` of apparent deficit. A residual shipped
  robust-prune deficit remains and blocks Task 167 closeout.
- The compact cited-result extract is `cited-results.log`, SHA-256
  `4996458e1e91260e16cf4eb4469b46bf8cec29c17cf92b7584bc0c60192c4141`.
- The hard failure occurred while returning the heldout population, before the
  then-current harness could emit the passing inserted-neighborhood line or a
  summary artifact. The child log is therefore the decision source for this
  run; the harness will be corrected before another measurement.

## Commands

- Audit:
  `/home/peter/.cargo-target/release/ecaz bench suite audit --config reviews/task-167/047-shipped-only-50k/artifacts/task167-shipped-only-50k-suite.json --log-file reviews/task-167/047-shipped-only-50k/artifacts/suite-audit-preregister.log`.
- Audit result: passed, 1 step. Log SHA-256:
  `ef40d406606a7ba2dcedaa8235758781959e4871d625eba3e4bfb8cd6e9a7a78`.
- Run after exact-runtime build and audit:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/047-shipped-only-50k/artifacts/task167-shipped-only-50k-suite.json --log-file reviews/task-167/047-shipped-only-50k/artifacts/suite-run.log`.
- Run result: exit 1 at the fixed quality gate. `suite-run.log` SHA-256:
  `4aa5611db16040350d5fcc8494f221449a77e57a8523598de763ea6f5b9ff8ec`.
- Suite manifest SHA-256:
  `c35e34072adbe8354c6b7ab2094b1090a2190b6999868d80d4cc1f867afbd03e`.
- Original failed-step child log SHA-256:
  `7cc07f1e685666a6a3d5c76984b2ed13ca11f7142f01a48ecaf16a04fe9dea12`.
- The manifest records runner head `cc3b05d4b23182657ee6b5ad897d311bde43c219`.
  The extension and CLI under measurement still embed the exact runtime head
  below; `cc3b05d4b` only added packet runtime metadata before launching the
  suite.

## Failed-step result extraction

- Report-only command (no benchmark rerun):
  `cargo run -p ecaz-cli --no-default-features -- bench suite report --manifest reviews/task-167/047-shipped-only-50k/artifacts/final-suite/suite-manifest.json --results-output reviews/task-167/047-shipped-only-50k/artifacts/final-suite/results.jsonl --log-file reviews/task-167/047-shipped-only-50k/artifacts/suite-report.log`.
- Parser checkpoints: `6d205bdbb` retains failed DistANN step artifacts,
  `7b20d18fa` structures Task 167 metrics, and `7e3d3d714` recognizes the
  hard-gate error form emitted by this run.
- `results.jsonl` contains 15 structured rows. Its decision row is
  `physical_benchmark_post_insert_exact_recall`, sourced from the original
  child log, with `quality_gate_pass=false` and
  `diagnostic_candidate_mutation_excluded=true`.
- `results.jsonl` SHA-256:
  `80e45d1307a47bc171dce152aefbc608b40ba4a1cc02c91e3b9bb0390aa7e128`.
- `suite-report.log` SHA-256:
  `ab2fbd815d4ad76201fb02dbb741ba1352e188ae0c880a16010a29e0b31db547`.

## Exact runtime

- Runtime head: `8bf0ac8a451f9cd73813dd0ab59ed305fab026bd`;
  both installed extension and release CLI embed this SHA with profile
  `release`.
- PG18 extension install command:
  `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`.
- Install result: passed. Committed `install-extension.log` SHA-256:
  `580f1405be92924ea6a7275b623e98aa3ffe8ed5bce8b2971066cef9bc03a4c1`.
- Installed `ecaz.so` SHA-256:
  `49f21b5151d071ba9709d6ba2a4f11c72653011361ed6b48a6ab2bedb6f6bd59`.
- CLI build command: `cargo build -p ecaz-cli --release --no-default-features`.
- Build result: passed with the pre-existing unrelated dead-code warning at
  `commands/corpus/load.rs:190`. Committed `build-cli.log` SHA-256:
  `eef89e71f15898d25765f73559ad9d9906144200bbe7cce700d88b36ac1c5760`.
- Release CLI SHA-256:
  `3f96c106338597793049b067bb3687bee955e4cdc6f691e8e2615e306077353a`.
- Exact-runtime audit result: passed, 1 step. Log SHA-256:
  `ef40d406606a7ba2dcedaa8235758781959e4871d625eba3e4bfb8cd6e9a7a78`.

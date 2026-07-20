# Task 191 implementation artifact manifest

- Task bucket: `reviews/task-191/`
- Packet: `reviews/task-191/002-production-implementation/`
- Implementation extension SHA: `7883cfcf8ca9769da72c3cf22856faa5537eb17f`
- Accepted suite runner SHA: `677e2d1d5af25426023af19015902bde1aa4e314`
- Timestamp: 2026-07-20 PDT
- Host/lane: Intel local, PG18 release, three loopback PostgreSQL owners
- Fixture: staged `ec_real_10k`, one byte-identical shared physical generation
- Quant/index: physical `ec_distann`, stored neighbor code `rabitq`
- Arms: feature-only eager control `0`; production lazy window `10`
- Rerank mode: physical payload materialization
- Isolation: one shared generation per scale; A/B arms differ only by the
  feature-gated materialization override and use identical seed digests

## Build and focused test artifacts

### `feature-release-install.log`

- Command: `cargo pgrx install --release --pg-config <PG18 pg_config>
  --no-default-features --features pg18,distann-head-attribution-benchmark`
- Result: feature extension install succeeded.
- Installed extension SHA-256:
  `6701d22880071a280668c84d4e2b43e4ebf997037a15a9032fe18879e997cd82`.

### `cli-release-rebuild-eager-control.log`

- Head: `677e2d1d5af25426023af19015902bde1aa4e314`
- Command: `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/home/peter/dev/ecaz/target
  cargo build --release -p ecaz-cli`
- Result: release build succeeded in 9m31s.

### `install-provenance-eager-control-final.log`

- Command: `git rev-parse HEAD; sha256sum <release ecaz> <installed ecaz.so>`
- CLI SHA-256:
  `204297e7ec7c62b4adc81f80e7153900d96a0d4701aa1ae50fc5358a70cb285c`.
- Extension SHA-256:
  `6701d22880071a280668c84d4e2b43e4ebf997037a15a9032fe18879e997cd82`.

### `eager-control-regression-test.log`

- Command: `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/home/peter/dev/ecaz/target
  cargo test -p ecaz-cli
  eager_materialization_control_is_forwarded_as_explicit_zero`
- Result: 1 passed, 0 failed; proves physical eager `0` is forwarded while the
  non-physical baseline receives no benchmark GUC.

## Semantic suite artifacts

### `semantic-suite.json`

- Checked-in `ecaz bench suite` config.
- Shape: staged 10k real corpus, 20 held-out queries, eager `0` versus lazy10,
  materialization semantics enabled, three owners, one shared generation.

### `suite-audit-eager-control-final.log`

- Command: `ecaz bench suite audit --config <semantic-suite.json>`
- Result: audit passed, one step.

### `suite-dry-run-eager-control-final.log`

- Command: `ecaz bench suite run --config <semantic-suite.json> --dry-run`
- Result: expands explicit variants
  `eager_control:...:rabitq:0` and `production_lazy10:...:rabitq:10`.

### `semantic-run/suite-manifest.json`

- Command: `ecaz bench suite run --config <semantic-suite.json>`
- Result: one succeeded step, exit code 0.
- Clean runner descriptor:
  `677e2d1d5af25426023af19015902bde1aa4e314`.

### `semantic-run/results.jsonl`

- Normalized suite result rows; this is the source of truth for all semantic,
  recall, latency, work, storage, topology, and provenance rows cited here.

### `semantic-run/production-semantics-10k/distann-multinode-summary.log`

- Compact accepted child summary.
- Key results:
  - eager/lazy recall: `0.9950` / `0.9950`;
  - eager/lazy mean latency: `43.60 ms` / `25.30 ms`;
  - eager/lazy remote candidates per scan: `31.333333` / `6.666667`;
  - duplicate remote candidates: zero in both arms;
  - eager merge/associate samples: `3` / `0`;
  - lazy merge/associate samples: `0` / `6`;
  - every `physical_materialization_correctness` row reports `pass=true`;
  - external-TOAST row reports `external_toast_ok=true`;
  - post-first-batch owner failure reports `pass=true`.

### `suite-status-eager-control-final.log`

- Command: `ecaz bench suite status --manifest <suite-manifest.json>`
- Result: completed 1; failed/skipped/dry-run/missing/stale all zero.

### `suite-report-eager-control-final.md`

- Command: `ecaz bench suite report --manifest <suite-manifest.json>`
- Result: generated normalized Markdown report over the accepted artifacts.

## Retention note

An earlier run used a runner that failed to forward explicit eager `0` after
the production default became lazy. Its output was rejected rather than used
as evidence. The corrected control-path commit is `677e2d1d5`; regenerable node,
recall, latency, and polling logs were pruned under the repository packet rules.

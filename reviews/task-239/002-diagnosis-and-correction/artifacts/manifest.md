# Task 239 packet 002 artifact manifest

- Task/packet: `task-239/002-diagnosis-and-correction`
- Harness code checkpoint: `8d8c181b889c8e0b5fb154b639cbfb9bd2ce34a9`
- Exact corrected CLI/config build checkpoint:
  `d03997c7aef2ff217d0535b47d0b8af765b8500f`
- Exact extension build checkpoint:
  `41392c011106cb040095fd6004c4d5c0f136f1a0`
- Detached corrected CLI/run checkout:
  `/home/peter/dev/ecaz/.worktrees/task239-corrected-run-build`
- Detached extension checkout:
  `/home/peter/dev/ecaz/.worktrees/task239-main-run-build`
- Host/lane: Intel local, PG18 attribution release
- Fixture: fresh three-node local multinode, one index per table
- Corpus: `ec_real_10k`; staged manifest SHA-256
  `cb3c68a3090ab4ff767f4e36448e5d90a95ae6416b50265a991d96184d00a561`;
  query SHA-256
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Search/runtime labels: persisted head 32/32, RaBitQ, BW4/H100/L32,
  eager-0 control and explicitly set/attested lazy-10 semantic candidate
- Measurement status: code/config preregistration only; no packet-002 extension
  install, live fixture, recall, or semantic run yet

## Config

- `crates/ecaz-cli/suites/task239-corrected-semantics-10k.json`
- SHA-256:
  `bd74199c5fc26d7dffc6b72582915529cbd1c7453ec4ff8fdaad82d7605e6f21`
- Run directory: `/home/peter/.ecaz/clusters/task239-corrected-semantics-10k`
- Ports: 44070--44072
- `reuse_fixture=false`; one index per table; run directory outside repository
  and `target/`
- Native/default sender and payload SQL. Task 224's feature-only fast sender is
  not enabled.

## Validation artifacts

Focused new regression:

```text
cargo test -p ecaz-cli materialization_semantics_always_restore_the_variant_batch_size
```

- `cargo-test-semantic-batch-restore.log` — 1/1 pass; SHA-256
  `300b397e120b637f12a5b7918cf75564085708b40083a4f843028b7d4404354d`

Focused materialization group:

```text
cargo test -p ecaz-cli materialization_
```

- `cargo-test-materialization-focused.log` — 6/6 pass; SHA-256
  `c845f60adae821c827fa2f59e591a974c5743cdc3e3009b8daef55af19ea0831`

Formatter gate:

```text
cargo fmt --all -- --check
```

- `cargo-fmt-check.log` — exit 0; stable-toolchain warnings only; SHA-256
  `06989bf4205a830d71e0d37c6d09bcc82bd5a987c7963c8dd66dac70ffc7575f`

Exact-checkpoint release CLI build:

```text
cargo build --release -p ecaz-cli
```

- `release-build-cli-d03997c7a.log` — pass; SHA-256
  `315523a0cce27a95c19edbff9ade26bdffb76d673306700e39b9a7cab3e4ea0a`
- Existing warning only: unread `LoadedDistributedPlacementConfig.path`.

Exact-checkpoint suite audit:

```text
/home/peter/.cargo-target/release/ecaz bench suite audit \
  --config <exact-detached-config> --log-file <packet-log>
```

- `suite-audit.log` — pass; SHA-256
  `850bc4cb9287014b7345ec946d5f689abec5bb8fcf6f9f9c755aedde2d6c6cf5`

Exact-checkpoint dry run:

```text
/home/peter/.cargo-target/release/ecaz bench suite run --dry-run \
  --config <exact-detached-config> --artifact-dir <absolute-packet-dry-run> \
  --log-file <absolute-packet-log>
```

- `dry-run.log` — expanded command; SHA-256
  `df1eacf727919ea854e408413d002ca0e9fe63ea3ae7b270239789edbec48043`
- `dry-run/suite-manifest.json` — `runner_git_commit=d03997c7a...`, exact
  config SHA, `dry_run=true`, step status `dry-run`; SHA-256
  `45419760c4b86c3c5ded374b32ea23f8278a8015ca0f4d533cacbec74e9602ea`

The dry run validates command construction only and has no recall, latency,
storage, or semantic decision weight.

## Live command prohibited pending review

After explicit outside authorization, run the following from the clean detached
extension checkout at exact `41392c011...`:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  cargo pgrx install --release \
  --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features --features pg18,distann-head-attribution-benchmark
```

Then invoke the checked-in suite once, from the corrected detached checkout,
without `--dry-run`, using the exact `d03997c7a...` release CLI and absolute
packet-local artifact/log paths. No continuation, resume, selected step, or
replacement run. Inspect unanimous extension SHA `41392c011...` and exact
features, verify the live suite manifest records runner SHA `d03997c7a...`,
apply request.md's fixed gate, capture compact evidence, and remove the stopped
run directory afterward.

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
- Measurement status: the sole authorized live run was consumed and failed
  before semantics on a 40-versus-37 CLI/extension stage-row mismatch; no
  packet-002 rerun is authorized

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

## Authorized live command — consumed

Reviewer seq02 authorized the following command from the clean detached
extension checkout at exact `41392c011...`; it completed successfully:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  cargo pgrx install --release \
  --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features --features pg18,distann-head-attribution-benchmark
```

The exact `d03997c7a...` release CLI then invoked the checked-in suite once,
without `--dry-run`, continuation, resume, selected-step execution, or a
replacement. That authorization is now exhausted.

## Live artifacts and outcome

- `extension-install-main-41392c.log` — exact-main attribution install passed;
  SHA-256
  `d1a7d961b651cd4b9566970d2ad8714946c4d20bec4e1b8e67363e966caca213`
- `live-runner-build-d03997c7a.log` — exact corrected release CLI rebuild
  passed with the one existing warning; SHA-256
  `58c6103136502059eaaf756f3d75968f9394208a27086fb7f8cba81382985662`
- Exact runner binary immediately before invocation: SHA-256
  `0f48f41f37d17a12ea2ddbd018ce306d1d6fc837b903c6a6d70e56402ed350e0`
- `live-suite.log` — suite driver output, step exit 1; SHA-256
  `efb82d10a62b638c7cde3bb8f7555fc2e3c0c85d768d10f4660ccbe61645bd97`
- `live-run/suite-manifest.json` — `dry_run=false`, runner `d03997c7a...`,
  exact config hash, one step `status=failed`, exit 1, duration 139,319 ms;
  SHA-256
  `efbe452ca6c7cb75fe507c5524fcb4fa3a7ec0d7c6907a241b005a3dbb326646`
- `live-run/corrected/distann-local-multinode.log` — exact-main unanimous
  release/feature preflight followed by `expected 40 ... got 37`; zero semantic
  rows; SHA-256
  `817be94a4e829a9756306b1e74785b6bf2fcdc480c725ac393575993fbc0a4bd`
- `live-run/corrected/physical-eager-control-recall.log` — recall 0.9990,
  200 queries / 2,000 trials; SHA-256
  `25777989f40f8393d60383461dde6ea92f8d304d08de73243e35f18c7b547efd`
- `live-run/corrected/physical-eager-control-predictions.json` — SHA-256
  `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`
- `live-run-decision.md` — authoritative failed-run disposition and causal
  derivation.

The eager latency and memory-series logs plus three node PostgreSQL logs are
retained as compact failure context. No summary or `results.jsonl` exists
because the child failed before the semantic matrix. The harness stopped the
nodes; the stopped 1.2 GB external run directory was removed after capture and
is recoverable only by regeneration. No packet-002 rerun is permitted.

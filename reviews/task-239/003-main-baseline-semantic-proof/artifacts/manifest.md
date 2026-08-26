# Task 239 packet 003 artifact manifest

- Task/packet: `task-239/003-main-baseline-semantic-proof`
- Baseline: `41392c011106cb040095fd6004c4d5c0f136f1a0`
- Ported code checkpoint: `21c013079723decfecb6880f40d099af5b37d627`
- Config checkpoint: `0adea669ba2dfaacb7c20a81f41af827280e2a48`
- Frozen validation/proposed build checkpoint:
  `4ab2aa9a90f14b045298ac9fe408f9a4b586bf3c`
- Detached exact validation checkout:
  `/home/peter/dev/ecaz/.worktrees/task239-main-port-run-build`
- Host/lane: Intel local, PG18 attribution release
- Fixture if later authorized: fresh three-node local multinode, one index per
  table, ports 44070--44072
- Run directory if later authorized:
  `/home/peter/.ecaz/clusters/task239-main-port-semantics-10k`
- Corpus: `ec_real_10k`; staged manifest SHA-256
  `cb3c68a3090ab4ff767f4e36448e5d90a95ae6416b50265a991d96184d00a561`;
  query SHA-256
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Search/runtime labels: persisted head 32/32, RaBitQ, BW4/H100/L32,
  eager-0 control and explicitly set/attested lazy-10 semantic candidate
- Measurement status: code/config validation and dry-run only; no packet-003
  extension install, fixture, recall, latency, storage, or semantic run

## Config

- `crates/ecaz-cli/suites/task239-main-port-semantics-10k.json`
- SHA-256:
  `53e13d779e2452a4282f8a076c17eb082396df615efd3e45393d2054257a4532`
- `reuse_fixture=false`; run directory outside the repository and `target/`
- Native/default sender and payload SQL; no Task 224 runtime flags or stage
  schema; routed DELETE+VACUUM remains enabled because exact main has no skip
  option

## Validation artifacts

Formatter:

```text
cargo fmt --all -- --check
```

- `cargo-fmt-check.log` — pass; SHA-256
  `6f175d7126514288c7663a04aeb90b7f7b29e65b1188ccbecfcac3551d339305`

Focused tests:

```text
cargo test -p ecaz-cli materialization_
```

- `cargo-test-materialization-focused.log` — 7/7 pass; SHA-256
  `02c4af9165d24aae92ace70bfc3b28c1b8734a57703e69461985b05889d50a0e`

Exact release runner build:

```text
cargo build --release -p ecaz-cli
```

- `release-build-cli-4ab2aa9a9.log` — pass, existing
  `LoadedDistributedPlacementConfig.path` warning only; SHA-256
  `a537a695e5652171c959f900b5a9991d47671902cdfca5668cb6b1c7aea1f5ef`
- Exact runner binary immediately after build: SHA-256
  `ce5bfeb1ea486c2fbed3027a703bba49122ac5123f46672cd0aaf1b4b0eb5163`

Suite audit:

```text
/home/peter/.cargo-target/release/ecaz bench suite audit \
  --config <exact-detached-config> --log-file <packet-log>
```

- `suite-audit.log` — one step, pass; SHA-256
  `e9c6c1acee8bb44567d2fa19722cc0476410607a5a1fb802dc7e549f928a130e`

Dry-run:

```text
/home/peter/.cargo-target/release/ecaz bench suite run --dry-run \
  --config <exact-detached-config> --artifact-dir <absolute-packet-dry-run> \
  --log-file <absolute-packet-log>
```

- `dry-run.log` — expanded command; SHA-256
  `74c1a5c8e5c736a4b72ac268eb7f24273d33cf0f91529e43b03e23b37a4948a4`
- `dry-run/suite-manifest.json` — runner `4ab2aa9a9...`, config hash exact,
  `dry_run=true`, one selected step at `status=dry-run`; SHA-256
  `3612b3cc2c37bfcbe0349c1b025af8d36988cedc211ad3a5b4e5a0524a339723`

The dry-run validates command construction only and has no semantic,
recall, latency, or storage decision weight. The configured live run directory
does not exist.

## Live action prohibited pending review

No `cargo pgrx install`, fixture creation, or non-dry suite run is authorized.
If outside review authorizes a one-shot run, build/install the release
attribution extension and release CLI from the same clean detached checkpoint
`4ab2aa9a9...`, verify exact runner/extension SHA and features, invoke the
checked-in suite once with absolute packet-local paths, apply `request.md`'s
fixed gate, and remove the stopped external run directory after capture.

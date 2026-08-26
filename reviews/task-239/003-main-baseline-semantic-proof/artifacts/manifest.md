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
- Measurement status: sole authorized live run complete; all fixed semantic
  and recall gates pass; outside result review pending

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

## Sole live run

Outside review `feedback/2026-08-26-01-reviewer.md` returned DONE and authorized
exactly one live invocation under C1--C5. The extension and CLI were rebuilt
from the clean detached `4ab2aa9a9...` checkpoint. Immediately before the run,
the worktree was clean, ports 44070--44072 were free, the config hash was exact,
and the runner binary SHA-256 was
`ce5bfeb1ea486c2fbed3027a703bba49122ac5123f46672cd0aaf1b4b0eb5163`.

```text
/home/peter/.cargo-target/release/ecaz bench suite run \
  --config <exact-detached-config> \
  --artifact-dir <absolute-packet-live-run> \
  --log-file <absolute-packet-live-suite-log>
```

There was one invocation only: no `--continue-on-error`, resume, `--only`,
retry, or replacement. The suite manifest records `dry_run=false`, runner
`4ab2aa9a9...`, the exact config hash, one step at `status=succeeded`, and exit
code 0. Extension preflight reports the same SHA and release profile unanimously
across three nodes, features exactly `distann-head-attribution-benchmark,pg18`,
and `debug_override=false`.

Decision: **HARNESS REGRESSION CORRECTED; EXACT-MAIN LAZY-10 SEMANTIC PATH
RESTORED TO 10/10**. See `live-run-decision.md` for the fixed-gate adjudication.
The main and summary logs each contain exactly one row for all nine scenarios,
all passing. The decisive `exactly_one_window` row records 10 rows, 6 remote
requests, 4 local consumptions, 10 reads against bound 10, zero duplicates, and
digest `df979e2d...6cfc77d`. Both recall arms are 0.9990 over 200 queries / 2,000
trials; both prediction files hash to packet 001's
`801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`.
Routed DELETE+VACUUM passes. Timing and storage are diagnostic only.

After capture, the stopped external run directory was 1.2 GB. It was removed
and confirmed absent before final artifact hashing.

### Final live artifact hashes

`live-artifacts.sha256` records and verifies every generated live artifact
listed below the packet's `artifacts/` root; its SHA-256 is
`fbf4bce68f03bb0884a853544f792da91f346fe6e339444884fd7728343cecf9`.
`sha256sum --check live-artifacts.sha256` passes 20/20.

- `extension-install-4ab2aa9a9.log`:
  `c548d1362efd03f7ff3dfc6891ee3322fae3f89afbe6c75dfa3fd875c6d5ff33`
- `live-runner-rebuild-4ab2aa9a9.log`:
  `00dcaa75a5a972ff98ba0ebd74f67c9539aacd7bf32ee6f0eb8b8a3def0e408d`
- `pre-run-attestation.log`:
  `c14f8c8e12d1cc758f21b2e4f8415bbf4201961faf1c9473a4898ca266b1e3dd`
- `live-suite.log`:
  `7eee08659c8c49e1224722b2ee5c38aff661276599a1237e98cdb13e6d9c236f`
- `live-run/suite-manifest.json`:
  `015cece64e0faf7e627335ae427b8e8d9254b934749bb8184f834b8f15becd92`
- `live-run/results.jsonl`:
  `7ede530aa2fc8bb3fd58d3d74bf7add35b4f9a94f1ca1c5737814dc0cbbdb1aa`
- `live-run/main-port/distann-local-multinode.log`:
  `c2c5da71e75243082aae5e065d35c10ff6ea858c89741864db0bd2ee22bad8b4`
- `live-run/main-port/distann-multinode-summary.log`:
  `2adc0db4e686e3e9a97800b55ab2c30fcc3011b593a8d685942f79f8d38b9874`

# Task 239 packet 001 artifact manifest

- Task/packet: `task-239/001-current-main-reproduction`
- Packet/config branch checkpoint: `81e26f8d0`
- Exact source/runner SHA: `41392c011106cb040095fd6004c4d5c0f136f1a0`
- Source checkout: clean detached worktree
  `/home/peter/dev/ecaz/.worktrees/task239-main-run-build`
- Host/lane: Intel local, PG18
- Fixture: fresh three-node local multinode; one index per table
- Corpus: `ec_real_10k` from
  `/home/peter/dev/ecaz/data/staged-current`
- Staged manifest SHA-256:
  `cb3c68a3090ab4ff767f4e36448e5d90a95ae6416b50265a991d96184d00a561`
- Query SHA-256:
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Storage/search/rerank: production payload projection, RaBitQ neighbors,
  persisted head, BW4/H100/L32, eager-0 and production lazy-10
- Measurement status: preregistration only; no extension installation, live
  suite, cluster, latency result, storage result, or recall result yet

## Preregistered configs

- `crates/ecaz-cli/suites/task239-current-main-production-10k.json`
  - SHA-256:
    `6778beb12f920a413fdd6cca99736616c5b3acbcbc6f0000b57572678ce6f110`
  - normal release `pg18`; no attribution hooks; seven production semantic
    scenarios plus eager/lazy-10 recall
  - run directory:
    `/home/peter/.ecaz/clusters/task239-current-main-production-10k`
  - ports: 44050--44052
- `crates/ecaz-cli/suites/task239-current-main-attribution-10k.json`
  - SHA-256:
    `2e1b2523bf27877d650bcee362464d0e4dbe22e297a567fd5a2437845a28c3cd`
  - release `pg18,distann-head-attribution-benchmark`; native sender;
    profiler disabled; nine semantic scenarios plus eager/lazy-10 recall
  - run directory:
    `/home/peter/.ecaz/clusters/task239-current-main-attribution-10k`
  - ports: 44060--44062

Both fixtures are isolated and non-reused across lanes. The production cluster
must be removed before the attribution installation and run. Neither run
directory is review evidence and both must be removed after cited results are
captured.

## Commands and preregistration evidence

All commands below use the exact-main release binary
`/home/peter/.cargo-target/release/ecaz`. Live commands use absolute config,
artifact, and log paths under the Task 239 branch while retaining the detached
exact-main checkout as CWD.

Remote-ref verification on 2026-08-26:

```text
gh api repos/agent-ix/ecaz/git/ref/heads/main --jq .object.sha
```

Result: `41392c011106cb040095fd6004c4d5c0f136f1a0`.

Exact-main release CLI build:

```text
cargo build --release -p ecaz-cli
```

- `release-build-cli-main-41392c.log` — pass; generated 2026-08-26 00:23 PDT;
  SHA-256
  `884031f0550c6da2ad6a42fd982f105f3ad1a15eb07ccba4b6862e79e3906f82`
- The only warning is the existing unread
  `LoadedDistributedPlacementConfig.path` field.

Suite audits, invoked from the Task 239 branch with each checked-in config:

```text
ecaz bench suite audit --config <config>
```

- `production-audit.log` — pass; SHA-256
  `344217e975b04478a752875bb252729942be462b6345a136d6bbb2f22ba5c522`
- `attribution-audit.log` — pass; SHA-256
  `66aa7143430dd61071d04f164740b3aab278398842d624cad2c4ab02f2da3cf2`

Exact-main dry runs, invoked from the detached checkout:

```text
ecaz bench suite run --dry-run --config <absolute-config> \
  --artifact-dir <absolute-packet-dry-run-dir> --log-file <absolute-log>
```

- `dry-run/production.log` — command expansion; SHA-256
  `32aab2f2fbd28b671842c8a011d7f1f171d3f49f2dd37d8d556b3155992ed44a`
- `dry-run/production/suite-manifest.json` — exact runner SHA and `dry-run`
  status; config SHA exact; SHA-256
  `bd9550b7586ba10b67e7ada0b6ab6447d0860214611ef345660edb0e3687ce1e`
- `dry-run/attribution.log` — command expansion; SHA-256
  `74a178f9872d8d2e991f3f2482668081be8529c486a05777ce8a955056df042d`
- `dry-run/attribution/suite-manifest.json` — exact runner SHA and `dry-run`
  status; config SHA exact; SHA-256
  `475d36c6c618e50013e29138540549030f906cba6e978f800292e295ade888a0`

`dry-run/production-pre-fix.log` is retained as non-decision audit history. It
records the preliminary config's intentional validation failure when a normal
release requested attribution-only owner-shape/counter fields. The checked-in
production config removed those fields and uses `metrics_mode=benchmark`; the
final audit and exact-main dry run above supersede it.

## Live commands prohibited pending review

After outside authorization, the production extension installation will be:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  cargo pgrx install --release \
  --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features --features pg18
```

Only after the production suite passes and its cluster is removed, the
attribution installation will use the same command with features
`pg18,distann-head-attribution-benchmark`.

Each live suite command is the corresponding dry-run command without
`--dry-run`, with an absolute packet-local artifact directory and log file. No
`--continue-on-error`, resume, selected-step execution, or replacement run is
allowed. The request's fixed ordered stop/decision rules are the source of
truth.

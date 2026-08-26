# Task 239 packet 001 artifact manifest

- Task/packet: `task-239/001-current-main-reproduction`
- Initial config checkpoint: `81e26f8d0`
- Seq01 corrected config checkpoint: `44c9eac00`
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
- Storage/search/rerank: exact-main native payload SQL/sender, RaBitQ neighbors,
  persisted head, BW4/H100/L32. The configs retain eager-0/lazy-10 labels; the
  featureless lane executes production lazy-10 for both, while the attribution
  semantic candidate inherits eager-0 from its control.
- Measurement status: preregistration only; no extension installation, live
  suite, cluster, latency result, storage result, or recall result yet

## Preregistered configs

- `crates/ecaz-cli/suites/task239-current-main-production-10k.json`
  - SHA-256:
    `3ddd441a401feee50b03fd89d5bc1b10cf7c77f6d6f14c260bd85af2f16fcf3f`
  - normal release `pg18`; no attribution hooks; seven production semantic
    scenarios plus two same-production-configuration recall labels
  - run directory:
    `/home/peter/.ecaz/clusters/task239-current-main-production-10k`
  - ports: 44050--44052
- `crates/ecaz-cli/suites/task239-current-main-attribution-10k.json`
  - SHA-256:
    `24b261617eed9a940391dd6ddab433c1ab888d1258a12c67a428cb4495c26292`
  - release `pg18,distann-head-attribution-benchmark`; exact-main native
    sender/payload SQL; nine semantic scenarios whose nominal candidate
    inherits eager-0 from the control, plus two recall labels
  - run directory:
    `/home/peter/.ecaz/clusters/task239-current-main-attribution-10k`
  - ports: 44060--44062

Both fixtures are isolated and non-reused across lanes. The production cluster
must be removed before the attribution installation and run. Neither run
directory is review evidence and both must be removed after cited results are
captured.

The corrected configs intentionally omit the four post-main Task 224 fields
that exact main silently ignores: `owner_payload_shape`,
`skip_owner_locality_profile`, `owner_fast_real_array_send`, and
`skip_routed_delete_vacuum_drill`. Consequently both lanes run exact main's
routed DELETE+VACUUM drill after semantic capture; the request preregisters how
a later drill failure is classified without discarding completed semantic
rows.

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

Corrected-suite audits, invoked from the detached exact-main checkout with the
exact-main release binary `/home/peter/.cargo-target/release/ecaz` and each
absolute checked-in config path:

```text
ecaz bench suite audit --config <config>
```

- `production-audit-seq01.log` — pass; SHA-256
  `344217e975b04478a752875bb252729942be462b6345a136d6bbb2f22ba5c522`
- `attribution-audit-seq01.log` — pass; SHA-256
  `66aa7143430dd61071d04f164740b3aab278398842d624cad2c4ab02f2da3cf2`

The byte-identical initial `production-audit.log` and
`attribution-audit.log` are retained as the original packet evidence, but the
seq01 logs explicitly pin the binary/CWD used after the unknown-key correction.

Exact-main dry runs, invoked from the detached checkout:

```text
ecaz bench suite run --dry-run --config <absolute-config> \
  --artifact-dir <absolute-packet-dry-run-dir> --log-file <absolute-log>
```

- `dry-run-seq01/production.log` — corrected command expansion; SHA-256
  `00088585f93a698c9cfda8720769a6d0e7ab84377181f5a450f1470d14e0d50e`
- `dry-run-seq01/production/suite-manifest.json` — exact runner SHA and `dry-run`
  status; config SHA exact; SHA-256
  `59c21e73b1a461274f3985ccb7f4fc8957949aa6391a7aa276795bc93ee4add6`
- `dry-run-seq01/attribution.log` — corrected command expansion; SHA-256
  `73502a1708b39e7fc99b313852adf6d1de4170831ff2db050e3a297841e7cc86`
- `dry-run-seq01/attribution/suite-manifest.json` — exact runner SHA and `dry-run`
  status; config SHA exact; SHA-256
  `abd0714f3c6250fa83b4b831a733a41bd2dba139efc98b369d929d9ea9a8f79a`

The initial `dry-run/` tree remains immutable review history for the original
config hashes. The corrected `dry-run-seq01/` expansions prove that exact main
receives none of the four post-main Task 224 switches and that neither lane
suppresses the routed DELETE+VACUUM drill.

`dry-run/production-pre-fix.log` is retained as non-decision audit history. It
records the preliminary config's intentional validation failure when a normal
release requested attribution-only owner-shape/counter fields. The checked-in
production config removed those fields and uses `metrics_mode=benchmark`; the
final audit and exact-main dry run above supersede it. SHA-256:
`862291a542b6231644d09ff16ce2f8ad859d451f7e017cccf14701bea481b771`.

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

Exact main's extension preflight enforces cross-node unanimity, release
profile, and absence of `pg-test`; it does not compare against this packet's
pinned SHA/features. Before accepting either lane, the operator must inspect
its emitted `release_profile_preflight` row for exact SHA `41392c011...` and
the expected lane feature list. The featureless lane's required
`attribution_available=false` semantic rows are the second stale-build
backstop. A zero process exit with Task 167
`reason=candidate_default_quality_gate_failed` is an invalid semantic lane,
not a pass.

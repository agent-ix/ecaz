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
- Measurement status: both seq02-authorized live lanes executed exactly once;
  production gate passed, attribution lane hit preregistered rule 1; no resume
  or replacement run

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

## Authorized live commands and results

Reviewer seq02 explicitly authorized the ordered installs/runs. The production
extension installation was:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  cargo pgrx install --release \
  --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features --features pg18
```

Only after the production suite passed and its cluster was removed, the
attribution installation used the same command with features
`pg18,distann-head-attribution-benchmark`.

Each live suite command was the corresponding dry-run command without
`--dry-run`, with an absolute packet-local artifact directory and log file. No
`--continue-on-error`, resume, selected-step execution, or replacement run was
used. The request's fixed ordered stop/decision rules remain the source of
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

### Production lane

- Install log: `production-install-main-41392c.log`; exact-main release PG18
  install passed; SHA-256
  `3f69bc5f6fcdeef207821684c015694bf05d7868d7ec6136e1a1e494e53aa3b0`.
- Suite log: `production-run.log`; exit 0; SHA-256
  `6fb01e4a02dd6896df70287465604c7c316d5012c23a50b2f4856eafeac05100`.
- Suite manifest: `production-run/suite-manifest.json`; status `succeeded`,
  exit 0, exact runner/config SHAs; SHA-256
  `ef2f59f1c4e21fcb7102562e7ca2616fb25139f8013521c3a9a28b1595af64c6`.
- Structured results: `production-run/results.jsonl`; 57 rows; SHA-256
  `2bdde681a3b08ceefba7c088dcd42b914b50484c3a9ead173dc4af2097a3f1a5`.
- Preflight: exact SHA `41392c011...`, release profile, features `pg18`, three
  nodes unanimous.
- Semantic gate: all seven core rows present once, exact result identity,
  `attribution_available=false`; feature-isolation row present; no Task 167
  skip; routed DELETE+VACUUM passed.
- Recall: 0.9990 for both same-production-configuration labels over 200
  queries / 2,000 trials. Both predictions have SHA-256
  `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`.

### Attribution lane

- Install log: `attribution-install-main-41392c.log`; exact-main release
  attribution/PG18 install passed; SHA-256
  `4fcedb5fbecb6ff40d7851549ab164e29a92a7f3cde420fb6a239120cbcb7a95`.
- Suite log: `attribution-run.log`; expected exit 1 at rule 1; SHA-256
  `8656b90a973e247303ae2778f486beb2901f5204ab89f48d85eda6de71efd827`.
- Suite manifest: `attribution-run/suite-manifest.json`; status `failed`, exit
  1, exact runner/config SHAs; SHA-256
  `d98fc81e0322940c873a258455f18a5fa4aa5ee4c4c65c9995f6de9156c49e97`.
- Exact failure log: `attribution-run/attribution/distann-local-multinode.log`;
  correct/identical 10 rows, remote 8 + local 4 = 12/10, duplicate 0; SHA-256
  `e98b5a78c6bad5799d3a072e3950877a6460c1790e445205421f499f97b46ae5`.
- Preflight: exact SHA `41392c011...`, release profile, features
  `distann-head-attribution-benchmark,pg18`, three nodes unanimous.
- Both separate-process recall children completed at 0.9990 and emitted the
  same prediction SHA as lane 1.
- Production-lazy-10 child context:
  `physical-lazy10-production-latency.log`, SHA-256
  `bd69451140fd1e66936978531a834f4d5399a0a7257fb20639de91001e66b86f`;
  remote requested 6, local consumed 4, returned 10, duplicates 0.
- Eager child context: `physical-eager-control-latency.log`, SHA-256
  `ae2a734fb3212e4c237bea974d57a77d78e84a8d86f2544cfd5fbb9d7a403d85`;
  remote requested 27, local consumed 4, returned 10, duplicates 0.

Decision record: `reproduction-decision.md`; SHA-256
`2cae26d6d857f1d9932c96d216961af8a0960c7cb41157d59c653a42f0eb1ce3`.
Every live install/run artifact is enumerated in
`live-artifact-sha256.txt`; ledger SHA-256
`681208bfeebe8a99598344c66499e39cfe7518cec53e9be6bc19e4e106b8c449`.

Both fixtures were stopped by the harness. After compact evidence capture, the
two exact 1.2 GB run directories under `/home/peter/.ecaz/clusters/` were
removed as required. They were regenerable operational state, not evidence.

Disposition: **REPRODUCED — EAGER-PATH COUNTER SHAPE ON EXACT CURRENT MAIN**.
The production lazy-10 child is 10/10 on the same attribution fixture. Packet
002 owns the semantic harness correction and complete nine-scenario proof; no
bound or production runtime behavior changed in packet 001.

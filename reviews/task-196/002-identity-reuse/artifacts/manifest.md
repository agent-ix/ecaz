# Artifact manifest — Task 196 packet 002

- Head / runner / fixed extension SHA:
  `4a0d722e9f5365585ce3734bfa64233e26c5459f`
- Production fix commit: `77adfb6b40430e4fb4684bb9403717e1e1a42f68`
- Task bucket / packet: `reviews/task-196/002-identity-reuse/`
- Lane / fixture: local Intel, three independent PG18 physical owners, real
  100k staged corpus, one index per table
- Search / materialization: exact training-landmark head, RaBitQ neighbors,
  BW4/H100, eager control versus production lazy10
- Scenarios: fewer/exact/more than one window, first/multiple rejected windows,
  NULL payload, external-TOAST projection/qual, mixed local/remote ownership,
  and owner failure after the first materialization batch
- Profile: release PG18 plus attribution feature
- Timestamp: 2026-07-22 America/Los_Angeles

## Commands and binary identity

The fixed extension was built and installed with:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark
```

Target and installed binaries were 24,269,976 bytes and byte-identical at
SHA-256 `4b4c7d0a7689aa510687bf18621375bf86725a8d96153ef8154dd2fdd5154e13`.
The release runner was 23,198,448 bytes at SHA-256
`8daae60a0fa849e06954a25f5004388af77d354c2542f57fe58d304ddaab2dc4`.

The checked-in packet-001 suite was audited and rerun against the fixed binary:

```text
target/release/ecaz bench suite run --config reviews/task-196/001-reproducer/artifacts/task196-stable-prefix-reproducer-100k.json --artifact-dir reviews/task-196/002-identity-reuse/artifacts/semantic-run
```

The manifest records a clean runner SHA, one succeeded step, zero failures,
and 2,087,220 ms duration. `suite status` reports one completed step, zero
failed, missing, or stale artifacts; `suite audit` passes.

## Semantic result

All nine semantic scenarios passed and reported `duplicate_requested=0`.
Every ordinary lazy10 result digest matched eager materialization. In the
formerly failing `reject_multiple_windows` case, lazy10 returned all 10 rows
with digest `5a2cdd94275ddef53ea47a68a409fc0819deee816b411648dd3ea9682cbaf92b`,
after 31 remote and 17 local payload reads, without re-requesting an immutable
remote vec_id. Mixed local/remote ownership also passed with 7 remote and 3
local rows. The post-first-batch owner outage preserved the expected error
digest and made no duplicate request.

The implementation now takes a previously materialized payload only when its
immutable vec_id matches the newly ranked candidate, searching the already
proven prefix rather than assuming raw rank stability. Current deepened search
results remain authoritative for output ordering.

## Files

| Artifact | SHA-256 | Purpose |
|---|---|---|
| `fixed-binary-identity.log` | `60120ed4a32024106350522011474773176e007ec3e66f9d7f391b5f08a685b1` | Fixed release target/installed/runner identity |
| `fixed-release-install.log` | `5df2234a6b656d4ee4a69773826f38e31a460d39ed5e0ada9a6f47e4c1784fd9` | Fixed release build/install transcript |
| `suite-audit.log` | `456be81a89e8ceb5579715a7e5aa91c8afc080d8ec1ffd0da6b0bfdadc03f9fa` | Suite input audit passed |
| `suite-status.log` | `0737a10c4bee31aee09154b6b6caeb3caa1b2af90d4532e9f3f5709442dd53b6` | One succeeded, zero failed/missing/stale |
| `suite-run-semantic.log` | `a95c46a45012bdb220c5a8fa2a2dd7897afc48807afd083667d6e5c723ba7919` | Compact run transcript with all nine correctness lines |
| `semantic-run/suite-manifest.json` | `95cc4520ba6be8f097a6070a85e8f313455294e9b53247ee82f98dea6665505d` | Clean runner/config/step provenance |
| `semantic-run/results.jsonl` | `b7379c15b27bee4275ac7678acaa1f8a2ef8252ebddca37149e5bb01bc397d6a` | Structured benchmark result rows |
| `semantic-run/reject-multiple-windows-stable-prefix-100k/distann-multinode-summary.log` | `009bc046e5129ed1458918fe59c4d341fbf3289315e7a21e3485a5ddbc80b383` | Retained compact fixture summary and semantic evidence |
| `semantic-run/reject-multiple-windows-stable-prefix-100k/physical-production-lazy10-recall.log` | `8b78a068d829bc1b0b6dca79cc6d3c8e8818c2d0d6f5e6db33a0e90f95dfbfa3` | Production-arm recall output |
| `semantic-run/reject-multiple-windows-stable-prefix-100k/physical-production-lazy10-latency.log` | `09d3ffb13f7736fcbf9e1ccff01551df2b1a0d4673376ef22e8b87d285d953b7` | Production-arm latency output |

Node PostgreSQL logs, the full fixture transcript, single-control logs, and
other operational exhaust are deliberately not committed.

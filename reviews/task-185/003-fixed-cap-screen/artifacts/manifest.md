# Task 185 packet 003 artifact manifest

Date: 2026-07-23 (America/Los_Angeles)

## Identity and frozen lane

- Task / packet:
  `reviews/task-185/003-fixed-cap-screen/`.
- Runner head:
  `c83ea6ea8426df0ae5ddc4e8dec55f68db801a94`.
- Last extension-source head:
  `23154d722eee818df1ef4b086b1e76d1d7ceb58e`.
  The later CLI-only commits through the runner head change only CLI-side
  diagnostic aggregation, not extension code.
- Extension profile: release, unanimous across all three PG18 nodes.
- Installed and target extension:
  35,342,808 bytes, SHA-256
  `f77c2f0b50cd2a55aad1ecf186e3550ff64371d36415e1e00a4c1dfd50bbeec4`.
- Runner:
  23,229,680 bytes, SHA-256
  `f86e33eaa62e6f2b32652524beae11d6da8f3bfe04244aa868ff392b7baba5b7`.
- Corpus: `ec_real_100k`, staged under `data/staged-current/`, 100,000
  source rows.
- Corpus SHA-256:
  `07275cfdc742805011552eb931298931576ef61583079722622985ac058a3375`.
- Query-file SHA-256:
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Evaluation / training / validation:
  rows 1--200 / 201--400 / 401--600. Training slice SHA-256 is
  `30f11df03f6e988adfe531e2bf54b75b8515fa207fee1212dd0774acffec7471`;
  validation slice SHA-256 is
  `0ff75eb04f5e9a4aef730ff30b9a7055285b2478f5be39b639ff695a001fb21a`.
- Topology: one coordinator, three exact/disjoint physical owners,
  100,000 owned rows, zero non-owned rows, zero orphans, and two remote
  owners verified through CustomScan.
- Frozen search: cap 4,096, exact head scoring, 32 returned seeds, graph
  degree 32, BW4/H100, RaBitQ neighbor scoring, exact final ranking,
  materialization batch 10.
- Measurement: 200 held-out queries / 2,000 top-10 trials; 50 warm
  concurrency-1 samples after 10 warmups.
- Storage format: unchanged Task 182 head plus RaBitQ graph neighbor values.
- Isolation: one index per physical owner table plus a separate coordinator
  source; the single-index reference is also isolated.

## Commands and suite state

The checked-in config is
`fixed-cap-screen-100k-suite.json` (SHA-256
`fbc30ea711f0b6a41a74803cf01edb373a179ead1790ed37bff0180d819a995f`).
This bespoke SuiteConfig is intentional: the four-cell gateway/basin A/B has
no counterpart in the reusable current lane configs, so a task-local checked-
in config is required to preserve its distinct membership/selector axes.
The complete command was:

```text
target/release/ecaz bench suite run \
  --config reviews/task-185/003-fixed-cap-screen/artifacts/fixed-cap-screen-100k-suite.json \
  --artifact-dir reviews/task-185/003-fixed-cap-screen/artifacts/run
```

The matching `dry-run`, `audit`, `status`, and focused CLI test commands and
outputs are retained in `suite-dry-run.log`, `suite-audit.log`,
`suite-status.log`, and `cli-distann-tests.log`. The suite finished with two
succeeded steps, no failures, no skipped steps, no missing artifacts, and no
stale artifacts. Step durations were 2,567,094 ms for the frequency build and
2,626,242 ms for the gateway build.

## Review caveats carried from packet 002

- The gateway builder's per-seed reachability is an **isolated-budget upper
  bound**: each single seed receives the full BW4/H100 budget. It is not the
  marginal contribution of that seed while competing inside the production
  joint 32-seed beam. The held-out joint-path evaluation below is the arbiter.
- Tail fill differs by arm. `training_landmarks` geometry-fills unused control
  slots; `training_gateway_set_cover` frequency-fills unused slots and fails
  if it cannot reach the cap. Thus membership may differ for two reasons even
  though the gateway objective is the intended intervention.
- Candidate selection uses only evaluation recall from rows 1--200. The
  training and validation proxy fields are diagnostics and never break a tie.
- The gateway screen is structurally constrained: only 127 positive marginal
  picks control the cap, while 3,969 of 4,096 slots are frequency-filled from
  the control's candidate pool. Its Jaccard-1.0 result therefore does not
  refute a whole-cap objective over a larger candidate pool.
- The basin selector's runtime basin is a 32-wide walk over the persisted
  4,096-node head graph, not the base-graph traversal basin used by the Phase-1
  attribution. Its ~46--48 ms cost is a prototype implementation cost, not a
  family-wide cost claim.
- `estimated_peak_extra_bytes` is a lower bound: the compact attribution
  estimate excludes per-query expanded-set vectors, which dominated the
  excluded statement-context OOM attempts documented in
  `memory-context-failure.md`.

## Key result lines

| 100k arm | Recall (95% CI) | Warm mean / p50 / p95 / p99 / max ms |
|---|---:|---:|
| frequency + exact | 0.9625 (0.9532--0.9700) | 20.40 / 20.50 / 23.80 / 25.40 / 26.00 |
| gateway + exact | 0.9625 (0.9532--0.9700) | 19.80 / 19.70 / 23.10 / 24.20 / 24.30 |
| frequency + basin | 0.9625 (0.9532--0.9700) | 66.10 / 65.80 / 74.40 / 80.70 / 83.60 |
| gateway + basin | 0.9625 (0.9532--0.9700) | 67.40 / 66.70 / 76.50 / 78.70 / 80.30 |

All four arms used the same 2,496,626,688-byte physical generation and
24,576 bytes of control indexes. Frequency and gateway head cache estimates
were 25,892,203 and 25,893,265 bytes respectively. Construction time was
931,189 ms for frequency and 985,165 ms for gateway.

The gateway arm selected the **same 4,096-node membership** as the control
(`control_gateway_jaccard=1.0`) but in a different persisted order. Exact
scoring is order-insensitive, so the intervention did not change the exact
seed frontier. Evaluation exact-overlap and owner-membership diagnostics were
also identical at 0.51328125 and 0.5503125. The training isolated-budget
diagnostic nevertheless reached 1,997 / 2,000 truth pairs and chose 127
positive set-cover seeds; this is precisely the non-transfer from isolated
reachability to joint-budget membership that packet 002 warned could occur.

The basin selector changed the returned order/set for every evaluation query,
but mean basin overlap fell only from 0.988369 to 0.987938 on frequency and
0.989391 to 0.989032 on gateway. It reduced overlap on only 13.5% and 15.0%
of queries and cost roughly 46--48 ms/query without moving recall.

The validation-only joint-path proxy was tied at 0.924 for control and
gateway. It is not used for selection.

## Durable artifacts

- `run/suite-manifest.json` — command expansion and completion state,
  SHA-256
  `cd891a71152e048b2a5affd6cdc044af72eee1c3514b7171a3673f50cf852008`.
- `run/results.jsonl` — structured result source of truth, SHA-256
  `e7cd23aa76c04fa1f85f3ce290b1375b4215325ea6759be50315c58a83918fb8`.
- `run/report.md` — generated suite report, SHA-256
  `a3f67638fff58090d85d3f8af540341276ba75691bd0ff714a24469f737a937d`.
- `run/frequency-membership-100k/distann-multinode-summary.log` — complete
  compact frequency result, SHA-256
  `bdc2b1db293bdf38a828060334e46f45ed4790c69e2dd6d52e2d91e0304119a4`.
- `run/gateway-membership-100k/distann-multinode-summary.log` — complete
  compact gateway result, SHA-256
  `1c3f4598b93148a8bc856c1e0ed17e6bfb3685803eb14797ba83a678d8dc6b09`.
- The eight `physical-*-{recall,latency}.log` files under the two step
  directories are the directly cited recall and latency outputs.
- `binary-identity.log` attests installed/target hashes and sizes.
- `memory-context-failure.md` records why two pre-measurement attempts were
  excluded and the bounded statement-context repair. Neither attempt emitted
  a decision result.

Corpus/query TSVs, node PostgreSQL logs, suite polling exhaust, truth caches,
and raw local-multinode transcripts are intentionally excluded.

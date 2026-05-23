# Full AWS SPIRE Test Matrix — Sweep Plan

## Goal

Complete the full SPIRE AWS test array (not just the synth-10k cell
that packet 958 shipped). Cells to cover:

- Real corpora at multiple scales (10k / 100k / 1M dbpedia)
- All 3 built-in suites (correctness, representative, stress)
- Inter-node network + SPIRE dispatch latency (NEW per user)
- Multi-AM comparison on the same corpus (ec_spire vs ec_ivf at minimum)
- Multi-cluster vs single-node baseline (so we can attribute dispatch
  overhead vs raw query time)

## Constraints

| Constraint | Value | Implication |
|---|---|---|
| r/m/c/t-family vCPU quota | 16 | Spec sizing (40 vCPU) doesn't fit — need quota raise OR downsize |
| F2 page-overflow risk at 1M | unproven | First 1M build may fail; capture safe nlists window as evidence |
| F29 fault drills | broken | Skip via `pass-*-bench` variants until F29+F34 fixed |
| Operator-laptop session | RaBitQ tests in progress | Paused — no AWS spend until cleared |

## Sizing per cell

Recommended **base topology** for everything below: 1 × `r8g.2xlarge`
coord + 1 × `r8g.2xlarge` remote (16 vCPU, 64 GB/node, exact quota
fit). This is **bigger than packet 958's 1+1 r8g.xlarge** so 1M
builds fit in RAM.

For 3-remote spec topology: **needs quota raise to 64+ vCPU** before
we can run it. File an AWS Support request as soon as RaBitQ pause
clears. Realistic ETA: 1-4 hr after submit.

## Test matrix — cells in priority order

### Cell A — `pass-representative-bench` on **real 10k** (sanity check before 1M)

Goal: validate the real-data path works end-to-end on multi-cluster
AWS before committing to a 1M run. Apples-to-apples with
bench-on-live-db single-node baseline.

- Topology: 1+1 r8g.2xlarge
- Corpus: dbpedia 10k subset
- Suite: same shape as suite-correctness but pointed at real_10k
- Wall-time: ~30-40 min
- Cost: ~$2
- New artifact: `real-10k-multi-cluster-recall.{md,json}` with recall
  at the nprobe sweep, comparable to IVF coder's existing prior 10k
  real numbers
- Gate: must complete before Cell B

**Blockers**:
- load.sh needs a new tier (`real_10k`) — currently has only
  `correctness`/`representative`/`stress`. Add `--tier-prefix
  ec_spire_aws_real_10k` and corpus fetch from dbpedia parquet.
- Or: re-use IVF coder's already-loaded `real_10k_*` corpus and just
  rebuild as `ec_spire` profile via `CREATE INDEX ... USING ec_spire`
  manually. Cheaper but less clean.

### Cell B — `pass-representative` on **1M dbpedia** (THE actual goal)

Goal: the original Phase 13 ship target. F2 page-overflow probe at
1M is part of the deliverable — either the build succeeds and we get
real numbers, or it fails and we get the safe nlists upper bound.

- Topology: 1+1 r8g.2xlarge (or 1+3 if quota raise lands first)
- Corpus: qdrant-dbpedia-openai3-large-1536-1m (1M × dim 1536)
- Suite: full `suite-representative.json` (recall k=10, k=100 ×
  nprobe sweep × 1000 queries each; latency at c=1,4,8,32)
- Wall-time: ~60-90 min (load + index build is the long pole)
- Cost: ~$5-10
- New artifacts: `suite-manifest-representative.json` +
  `suite-results-representative.jsonl` with all bench cells
- Gate: 1M build must succeed (F2 outcome captured)

**Blockers**:
- F2 outcome unknown. Mitigation: walk the nlists ladder if first
  attempt overflows — try nlists=128, 160, 200 and capture the safe
  window. Each retry burns ~10 min of instance time.

### Cell C — Inter-node latency probes (NEW)

Goal: quantify the SPIRE dispatch overhead vs raw network + raw PG.

Three measurements per topology size:

1. **Raw network RTT** coord ↔ remote: `ping -c 100`, `iperf3 -c`
   or `mtr --report-cycles 50` — same-AZ, private IPs, ENA driver.
   Expected: ~0.1-0.3 ms.
2. **PG libpq connect + simple query** time: `psql -h <remote>
   -c 'SELECT 1'` from coord, 100 iterations. Expected: ~1-3 ms.
3. **SPIRE coord→remote dispatch overhead**: the smoke handoff
   summary already records `coord_local_search_micros`,
   `coord_remote_dispatch_micros`, `coord_merge_micros`. Extract
   these from every recall/latency step and compute the multi-cluster
   tax vs single-node baseline (run a single-node ec_spire on the
   coord with the remote disabled, same query workload).

- Topology: piggyback on Cell A and Cell B instances; no new
  provisioning
- Wall-time: +10 min on top of the bench runs
- Cost: negligible
- New artifacts: `network-rtt.log`, `pg-connect-rtt.log`,
  `dispatch-overhead-extract.md`

### Cell D — Multi-AM comparison on the same corpus

Goal: relative performance numbers ec_spire (multi-cluster) vs ec_ivf
(single-node) on the SAME corpus, SAME hardware.

- Topology: 1+1 r8g.2xlarge (Cell A/B instances reused)
- Corpora: 10k real + 1M real (Cells A and B)
- Indexes: build ec_ivf AND ec_spire on each corpus on the coord;
  run identical bench parameters
- Wall-time: +30-45 min total (extra indexes + extra bench runs)
- Cost: ~$1-2
- New artifacts: `am-comparison-{10k,1m}-recall.{md,json}`

**Blockers**:
- ec_ivf bench infrastructure already proven (bench-on-live-db). Just
  need to also build the index on our test coord.

### Cell E — `pass-stress` on **10M synthetic** (reviewer-gated)

Goal: scale ceiling test. Phase 13a.9 requires reviewer signoff
before running stress.

- Topology: needs the quota-raised 1+3 spec
- Corpus: 10M synthetic
- Suite: full `suite-stress.json`
- Wall-time: ~2-4 hr
- Cost: ~$20-40
- **Defer** until A+B+C+D done AND reviewer signoff.

### Cell F (optional) — Cross-engine comparators

Goal: numbers vs pgvector / pgvectorscale / vchord on the same
corpus, same hardware. The IVF coder has been collecting this — we
should match their methodology.

- Topology: piggyback
- Wall-time: +30 min
- Cost: ~$1-2

## Execution order (post-pause)

When user clears the pause:

0. **Submit AWS quota raise** (16 → 64 vCPU r/m/c/t-family) — async
1. Cell A (real 10k smoke) — validates the real-data path
2. Cell C network/dispatch probes piggyback on Cell A
3. Cell B (1M dbpedia) — the actual goal
4. Cell D (multi-AM) — piggyback on Cell B instances (build ec_ivf
   in addition to ec_spire, run bench against both)
5. Cell C dispatch probes on the 1M topology
6. If quota raise landed: re-run Cell B on the spec 1+3 topology to
   get true multi-remote numbers
7. Cell E (stress 10M) — reviewer gate; skip unless authorized
8. Cell F (comparators) — if time/budget remains

## Total estimate

- Wall-time: 4-6 hr (without stress 10M)
- AWS spend: $10-15 (without stress)
- Plus stress 10M if authorized: +2-4 hr / +$20-40

## Per-cell cost-bleed safety

Same as packet 958: every cell ends in snapshot + teardown.
`aws ec2 describe-instances --filters Phase=...` empty check between
cells. Option A discipline: keep instances UP between *stage*
failures within a cell, only teardown when cell completes or a fix
requires a rebuild.

## Output: rolled into the same packet (958) or a new one?

Recommend new packet `reviews/task-30/959-spire-aws-pass-representative/`
for cells A+B+C+D, since the corpus + scope are materially different
from 958. Keep 958 as the "plumbing + synth-10k smoke" packet.
Cell E (stress) gets packet 960.

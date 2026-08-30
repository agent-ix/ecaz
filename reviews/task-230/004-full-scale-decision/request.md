---
task: 230
packet: 004-full-scale-decision
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 04
---

# Task 230 full-scale decision: STOP, re-scoped basis

Review the corrected interpretation of the completed frozen 20-step A/B at
benchmark head `8bcccb56c6381527c4d2f3a4f4c9931b66b9235c`. The disposition remains
**STOP: do not promote the hot/cold row tier**. This revision implements all
four requests in `feedback/2026-08-29-03-reviewer.md`: it separates
non-discriminating harness limitations from candidate failures, records the
position-sensitivity result, classifies the DML outlier's reproducibility, and
corrects the CLI-runner provenance.

## Decisive candidate evidence

The frozen primary rule requires the candidate to beat its matched row-heap
control in **both** independent 100k pairs by at least 5.0% and 0.50 ms. There
is no averaging or tie-break.

| 100k pair | Position order | Control mean | Candidate mean | Candidate result | Frozen gate |
|---|---|---:|---:|---:|---|
| A | row-heap first, hot/cold second | 12.40 ms | 9.47 ms | +23.63%, +2.93 ms | PASS |
| B | hot/cold first, row-heap second | 8.67 ms | 9.47 ms | -9.23%, -0.80 ms | **FAIL** |

Pair B is an uncontaminated, discriminating, sufficient STOP gate. Its tails
also fail the combined guardrail: p95 is 10.30 → 11.40 ms (+10.68%, +1.10 ms)
and p99 is 11.00 → 11.90 ms (+8.18%, +0.90 ms).

The reproduced 100k recall deficit supports STOP but is stated against the
measured noise floor:

- candidate recall is 0.9275 in both counterbalance positions;
- controls are 0.9300 and 0.9295, a 0.0005 control-vs-control spread; and
- candidate-minus-control is therefore -0.0025 and -0.0020, both worse than
  the frozen -0.001 tolerance and both materially larger than the observed
  control spread.

## Position sensitivity: the result inherited by Tasks 231/232

The counterbalance exposes the control—not the candidate—as the unstable arm.
The two identically configured row-heap controls span 12.40 and 8.67 ms, a
3.73 ms difference and 43.02% of the faster value. The hot/cold candidate is
9.47 ms in both opposite positions to two decimal places.

The evidence therefore does **not** say “hot/cold is generally slower.” It says
that hot/cold trades peak warm latency for position-insensitive latency: it
beats the first/cooler row-heap control and loses to the second/warmer one. This
is consistent with the measured mechanism: the candidate's PLAIN inline hot
tier has no vector TOAST pages to warm, while the row-heap control does. This is
an evidence-backed interpretation, not an override of the frozen conjunctive
gate; the gate still requires STOP.

## Run validity and harness limitations

- Suite status is `completed=20 failed=0 skipped=0 dry_run=0
  missing_artifacts=0 stale=0`; audit is `audit passed: 20 steps`.
- Config SHA-256 is the frozen
  `e141ac65a7e18eaf4512509c549ba750e3106a2a045942e0eb6a5ac8fcc5437c`.
- The PG18 **extension** was release-built after the Packet 003 test install;
  all 20 arms report the benchmark head, unanimous release extension
  provenance, and no debug override.
- The **CLI runner** was `/home/peter/.cargo-target/debug/ecaz`, built in the
  Cargo `dev` profile with debuginfo. It was not a release CLI. This matches the
  Task 229 precedent and biases the absolute/percentage results conservatively;
  it does not rescue the failed conjunctive gate. The receipt is correctly
  named `artifacts/cargo-build-cli-debug-runner.log`.
- CLI clippy retained the accepted 77-warning binary / 78-warning test
  baseline.
- Five PostgreSQL startup-only port collisions occurred before the affected
  arm's preflight or measurement; the compact receipt proves same-manifest
  cleanup/resume and exclusion of failed rows.
- All 20 distinct run directories were children of `~/.ecaz/clusters` and were
  removed after capture.

Two preregistered checks fail as run-validity/reproducibility checks but do not
discriminate between layouts and are **not candidate failures**:

1. All four 50k arms miss the 0.980 absolute recall floor at 0.9540–0.9545.
   Candidate equals control in pair A and exceeds it by 0.0005 in pair B. This
   is a lane/configuration threshold failure, not a hot/cold deficit.
2. Physical predictions are byte-identical at 10k and in 50k pair A. In 50k
   pair B, three of four 50k arms—including the candidate—share
   `1abc27ff…`; the outlier is the row-heap control (`1d5139fa…`). At 100k all
   four hashes differ, including control-vs-control `02dcd617…` versus
   `667bfe30…`. Prediction byte parity is therefore unmeasurable as semantic
   parity at the decision scale; it exposes declining run-to-run harness
   reproducibility, not candidate semantic failure.

## Storage, build, and DML

Tier/storage mechanism results are strong and remain positive. Candidate hot
main-heap bytes are 82,018,304 / 409,812,992 / 819,511,296 against raw-vector
bytes 61,440,000 / 307,200,000 / 614,400,000. At 100k the ratio is 1.3337×,
confirming the preregistered 8192/6144 = 1.3333 PLAIN-page prediction. Total
generation storage, build, publish, insert throughput, exact row distributions,
and delete p95 pass every candidate gate.

The 50k pair-B replacement p95 gate fails literally but **does not reproduce**
across the two counterbalanced pairs:

| 50k pair | Control p95 | Candidate p95 | Candidate/control | Gate |
|---|---:|---:|---:|---|
| A | 1294.833 ms | 911.177 ms | 0.704× | PASS |
| B | 768.535 ms | 2101.234 ms | 2.734× | **FAIL** |

Thus one of two candidate measurements is an outlier, not a stable layout
effect. It remains disclosed as a mandatory single-pair gate failure, but it is
not needed for—and is not presented as the primary basis of—STOP. The 50k
prediction outlier is in the opposite, row-heap arm of pair B, so those two
outliers share a pair but not an arm.

## Mechanism and directional predictions

Tier laziness passes. Every candidate id-only and exact-vector cold relation
reports zero for all six heap/TOAST/tidx counters; cold-only reports zero for
all six hot-tier counters. `artifacts/io-attribution.md` contains all 90
per-node/per-relation rows plus 20 arm totals and shared-buffer hit ratios.

| Shape | Control elapsed / 50 | Candidate elapsed / 50 | Direction | Numeric gate | Prediction |
|---|---:|---:|---|---|---|
| exact-vector | 559.127 ms | 566.784 ms | +1.37%, +0.153 ms/query worse | PASS | **FALSIFIED** |
| cold-only | 491.319 ms | 474.840 ms | 3.35%, 0.330 ms/query better | PASS | **FALSIFIED** |
| mixed | 421.456 ms | 453.966 ms | +7.71%, +0.650 ms/query worse | PASS | SUPPORTED |
| select-all | 846.281 ms | 867.906 ms | +2.56%, +0.433 ms/query worse | PASS | SUPPORTED |

Id-only/hot-scalar is **FALSIFIED as a general improve-or-neutral prediction**:
five pairs improve, but 100k pair B materially regresses. Id-only and
hot-scalar remain one identical frozen measurement under two labels.

## Decision

**STOP.** The decisive basis is the clean 100k pair-B mean failure under the
conjunctive primary rule, with its tail regression and the reproduced 100k
recall deficit as supporting discriminating evidence. The 50k floor and
prediction divergence are harness limitations, not candidate failures; the
50k replacement outlier does not reproduce. Hot/cold's structural mechanism,
storage bound, and position-insensitive ~9.47 ms latency are real positive
findings, but it does not beat the warm row-heap control and therefore cannot
be promoted under the frozen contract.

## Review request

Please verify that seq-03's four findings are fully resolved and close the
Packet 004 STOP disposition if DONE.

---
task: 230
packet: 004-full-scale-decision
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 03
---

# Task 230 full-scale decision: STOP

Review the completed 20-step, frozen-config A/B decision at benchmark head
`8bcccb56c6381527c4d2f3a4f4c9931b66b9235c`. The disposition is **STOP: do
not promote the hot/cold row tier**. Multiple independent mandatory gates fail;
no averaging or secondary result can rescue the candidate.

## Run validity

- Suite status: `completed=20 failed=0 skipped=0 dry_run=0
  missing_artifacts=0 stale=0`.
- Suite audit: `audit passed: 20 steps`.
- Config SHA-256 is the frozen
  `e141ac65a7e18eaf4512509c549ba750e3106a2a045942e0eb6a5ac8fcc5437c`.
- The PG18 extension was reinstalled in release mode after the Packet 003 test
  install; all 20 arms report the benchmark head, unanimous release provenance,
  and no debug override.
- `cargo clippy -p ecaz-cli --all-targets` retained the accepted 77-warning
  binary / 78-warning test baseline.
- Five PostgreSQL startup-only port collisions occurred before the affected
  arm's release preflight or measurement. Each port set was verified idle, the
  failed fixture was cleaned, and the same manifest was resumed. No failed
  result row was admitted. The compact receipt is
  `artifacts/step2-port-collision-retry.log`.
- All 20 run directories were distinct children of `~/.ecaz/clusters` and have
  been removed after durable capture.

## Primary results and gates

Candidate deltas use `candidate - control` for recall and the preregistered
`100 * (1 - candidate/control)` for mean improvement.

| Scale/pair | Recall control → candidate | Recall gate | Mean ms control → candidate | Mean gate | Prediction SHA parity |
|---|---:|---|---:|---|---|
| 10k A | 0.9990 → 0.9990 | PASS | 9.28 → 8.05 (+13.25%) | PASS | PASS |
| 10k B | 0.9990 → 0.9990 | PASS | 9.02 → 7.72 (+14.41%) | PASS | PASS |
| 50k A | 0.9545 → 0.9545 | **FAIL floor** | 9.47 → 9.14 (+3.48%) | PASS | PASS |
| 50k B | 0.9540 → 0.9545 | **FAIL floor** | 9.77 → 9.72 (+0.51%) | PASS | **FAIL** |
| 100k A | 0.9300 → 0.9275 (-0.0025) | **FAIL delta** | 12.40 → 9.47 (+23.63%, +2.93 ms) | PASS | **FAIL** |
| 100k B | 0.9295 → 0.9275 (-0.0020) | **FAIL delta** | 8.67 → 9.47 (-9.23%, -0.80 ms) | **FAIL** | **FAIL** |

All four 50k physical arms miss the frozen absolute recall floor of 0.980.
The suite therefore exited nonzero on four threshold checks even though every
step itself completed successfully. Prediction files are byte-identical in
both 10k pairs and 50k pair A, but not 50k pair B or either 100k pair.

The 100k pair-B p95/p99 values also fail the combined tail guardrail:
10.30 → 11.40 ms (+10.68%, +1.10 ms) and 11.00 → 11.90 ms (+8.18%,
+0.90 ms). The second conjunctive 100k mean win is absent.

## Storage, build, and DML

Both candidate storage gates pass at every scale. Candidate hot main-heap bytes
are 82,018,304 / 409,812,992 / 819,511,296 against raw-vector bytes
61,440,000 / 307,200,000 / 614,400,000, approximately 1.334×. Candidate total
generation bytes are below their matched row-heap controls in all pairs.
Physical build, publish, insert throughput, exact row distributions, and delete
p95 also pass every gate.

One DML gate fails: 50k pair-B replacement p95 is 768.535 → 2101.234 ms,
2.734× control versus the 1.50× ceiling. Every other replacement gate passes.

## Mechanism and secondary projections

Tier laziness passes. Every candidate id-only and exact-vector cold relation
reports zero for all six heap/TOAST/tidx counters; cold-only reports zero for
all six hot-tier counters. `artifacts/io-attribution.md` consolidates all 90
per-node/per-relation rows plus the 20 arm totals and shared-buffer hit ratios.

| 100k shape | Control elapsed / 50 | Candidate elapsed / 50 | Candidate direction | Numeric gate | Prediction |
|---|---:|---:|---|---|---|
| exact-vector | 559.127 ms | 566.784 ms | +1.37%, +0.153 ms/query worse | PASS | **FALSIFIED** (predicted improve) |
| cold-only | 491.319 ms | 474.840 ms | 3.35%, 0.330 ms/query better | PASS | **FALSIFIED** (predicted regress) |
| mixed | 421.456 ms | 453.966 ms | +7.71%, +0.650 ms/query worse | PASS | SUPPORTED |
| select-all | 846.281 ms | 867.906 ms | +2.56%, +0.433 ms/query worse | PASS | SUPPORTED |

The id-only/hot-scalar prediction is **FALSIFIED as a general prediction**:
five matched pairs improve, but 100k pair B materially regresses. Id-only and
hot-scalar remain one identical measurement under two labels, as frozen.

## Decision

**STOP.** The candidate fails semantic parity, quality, the conjunctive 100k
latency win, a 100k tail guardrail, and one DML guardrail. The structural
mechanism works and its storage cost is bounded, but the required end-to-end
benefit is not repeatable. No Task 230 production/default promotion is
authorized.

## Review request

Please verify the run validity, each frozen gate calculation, prediction
classification, consolidated I/O attribution, provenance, and STOP
disposition. This request remains review-open until an outside reviewer writes
a verdict under this packet's `feedback/` directory.

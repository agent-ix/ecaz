# Task 106 packet 004 - AWS targeted bench manifest

- Head SHA: `9fe067511` at config creation.
- Branch: `task-106-unified-driver-closeout`.
- Packet: `reviews/task-106/004-aws-targeted-bench/`.
- Date: 2026-06-13.
- Purpose: package targeted AWS bench configs for the Task 106 affected
  surfaces only. This is not a full sweep.

## Suite Configs

| Config | Steps | Purpose |
| --- | ---: | --- |
| `task106-aws-targeted-fixture-prep.json` | 5 | Optional fixture fetch/prepare for 10k, 50k, 100k, 1m under `/var/lib/pgsql/18/datasets/staged-task106-targeted`. |
| `task106-aws-intel-targeted.json` | 129 | AWS Intel recall/latency/pipeline suite for Task 106 affected cells. |
| `task106-aws-graviton-targeted.json` | 129 | AWS Graviton recall/latency/pipeline suite for Task 106 affected cells. |
| `task106-aws-spire-pqfastscan-negative-smoke.json` | 3 | Separate expected-failure smoke for SPIRE pq_fastscan product gap at 10k. |

Generated summary: `generated-config-summary.json`.

## Matrix Boundary

Included:

- `ec_ivf` + `rabitq` + `quant_bits={1,2,4,8}`, scratch SoA on/off,
  recall + latency, all four scales.
- `ec_ivf` default `Auto`, scratch SoA on/off, recall + latency, all four
  scales.
- `ec_spire` + `rabitq`, candidate batch scoring on/off, recall + latency +
  `spire-pipeline`, all four scales.
- `ec_spire` + `pq_fastscan`, 10k negative smoke only.

Excluded:

- HNSW grouped-PQ, DiskANN, explicit TurboQuant comparator lanes, broad
  PQ-FastScan benches, and unrelated quant/index/option combinations.

## Local Validation

Commands run locally:

- `target/release/ecaz bench suite audit --config reviews/task-106/004-aws-targeted-bench/task106-aws-targeted-fixture-prep.json --log-file reviews/task-106/004-aws-targeted-bench/artifacts/audit-fixture-prep.log`
  - Result: passed, 5 steps.
- `target/release/ecaz bench suite run --dry-run --config reviews/task-106/004-aws-targeted-bench/task106-aws-intel-targeted.json --log-file reviews/task-106/004-aws-targeted-bench/artifacts/dry-run-aws-intel.log`
  - Result: passed; wrote `aws-intel/suite-manifest.json`.
- `target/release/ecaz bench suite run --dry-run --config reviews/task-106/004-aws-targeted-bench/task106-aws-graviton-targeted.json --log-file reviews/task-106/004-aws-targeted-bench/artifacts/dry-run-aws-graviton.log`
  - Result: passed; wrote `aws-graviton/suite-manifest.json`.
- `target/release/ecaz bench suite run --dry-run --config reviews/task-106/004-aws-targeted-bench/task106-aws-spire-pqfastscan-negative-smoke.json --log-file reviews/task-106/004-aws-targeted-bench/artifacts/dry-run-spire-pqfastscan-negative.log`
  - Result: passed; wrote `spire-pqfastscan-negative/suite-manifest.json`.

Local audit limitation:

- `audit-aws-intel-local-missing-inputs.log`,
  `audit-aws-graviton-local-missing-inputs.log`, and
  `audit-spire-pqfastscan-negative-local-missing-inputs.log` show expected
  local failures because `/var/lib/pgsql/18/datasets/staged-task106-targeted`
  does not exist on this workstation.

## AWS Execution Order

On each EC2 host:

1. Confirm the branch head is the intended Task 106 SHA and the extension is
   release-installed.
2. If staged fixtures are missing, run:
   `target/release/ecaz bench suite run --config reviews/task-106/004-aws-targeted-bench/task106-aws-targeted-fixture-prep.json`
3. Run the lane audit:
   `target/release/ecaz bench suite audit --config reviews/task-106/004-aws-targeted-bench/task106-aws-intel-targeted.json`
   or
   `target/release/ecaz bench suite audit --config reviews/task-106/004-aws-targeted-bench/task106-aws-graviton-targeted.json`
4. Run the lane suite with packet-local manifest/results outputs from the
   config's `artifact_dir`.
5. Run the negative smoke separately with `--continue-on-error`, and confirm
   the SPIRE pq_fastscan load fails for the known grouped-PQ persistence gap.

## AWS Result Acceptance

For each AWS lane, the result packet is complete when:

- the lane suite finishes with successful recall/latency/pipeline results;
- `suite-manifest.json`, `results.jsonl`, raw logs, and suite report are
  packet-local;
- fixture hashes and host metadata are recorded;
- IVF RaBitQ counter attribution is summarized for bits 1/2 versus 4/8;
- IVF Auto scratch-on emits TurboQuant/QJL batch counters;
- SPIRE RaBitQ on/off pipeline behavior is recorded;
- SPIRE pq_fastscan negative smoke records the expected product-gap failure.

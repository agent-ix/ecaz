# Task 106 packet 004 - AWS targeted bench manifest

- Head SHA: `9fe067511` at config creation; AWS execution used branch head
  `36f6ff793`.
- Branch: `task-106-unified-driver-closeout`.
- Packet: `reviews/task-106/004-aws-targeted-bench/`.
- Date: 2026-06-13.
- Purpose: package targeted AWS bench configs for the Task 106 affected
  surfaces only. This is not a full sweep.

## Suite Configs

| Config | Steps | Purpose |
| --- | ---: | --- |
| `task106-aws-targeted-fixture-prep.json` | 5 | Optional fixture fetch/prepare for 10k, 50k, 100k, 1m under `/var/lib/pgsql/18/datasets/staged-task106-targeted`. |
| `task106-aws-intel-targeted.json` | 149 | AWS Intel recall/latency/pipeline suite for Task 106 affected cells, including HNSW grouped-PQ gap-2 cells. |
| `task106-aws-graviton-targeted.json` | 149 | AWS Graviton recall/latency/pipeline suite for Task 106 affected cells, including HNSW grouped-PQ gap-2 cells. |

Generated summary: `generated-config-summary.json`.

## Matrix Boundary

Included:

- `ec_ivf` + `rabitq` + `quant_bits={1,2,4,8}`, scratch SoA on/off,
  recall + latency, all four scales.
- `ec_ivf` default `Auto`, scratch SoA on/off, recall + latency, all four
  scales.
- `ec_spire` + `rabitq`, candidate batch scoring on/off, recall + latency +
  `spire-pipeline`, all four scales.
- `ec_hnsw` + grouped-PQ (`storage_format=pq_fastscan`, `m=16`,
  `ef_construction=128`), candidate batch scoring on/off, recall + latency,
  all four scales.

Excluded:

- DiskANN, explicit TurboQuant comparator lanes, broad PQ-FastScan benches,
  SPIRE pq_fastscan, and unrelated quant/index/option combinations. SPIRE
  pq_fastscan is not implemented for SPIRE and is not a benchmark target.

## Local Validation

Commands run locally:

- `target/release/ecaz bench suite audit --config reviews/task-106/004-aws-targeted-bench/task106-aws-targeted-fixture-prep.json --log-file reviews/task-106/004-aws-targeted-bench/artifacts/audit-fixture-prep.log`
  - Result: passed, 5 steps.
- `target/release/ecaz bench suite run --dry-run --config reviews/task-106/004-aws-targeted-bench/task106-aws-intel-targeted.json --log-file reviews/task-106/004-aws-targeted-bench/artifacts/dry-run-aws-intel.log`
  - Result: passed; wrote `aws-intel/suite-manifest.json`.
- `target/release/ecaz bench suite run --dry-run --config reviews/task-106/004-aws-targeted-bench/task106-aws-graviton-targeted.json --log-file reviews/task-106/004-aws-targeted-bench/artifacts/dry-run-aws-graviton.log`
  - Result: passed; wrote `aws-graviton/suite-manifest.json`.

Local audit limitation:

- `audit-aws-intel-local-missing-inputs.log`,
  `audit-aws-graviton-local-missing-inputs.log` show expected local failures
  because `/var/lib/pgsql/18/datasets/staged-task106-targeted` does not exist
  on this workstation.

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

## AWS Result Acceptance

For each AWS lane, the result packet is complete when:

- the lane suite finishes with successful recall/latency/pipeline results;
- `suite-manifest.json`, `results.jsonl`, raw logs, and suite report are
  packet-local;
- fixture hashes and host metadata are recorded;
- IVF RaBitQ counter attribution is summarized for bits 1/2 versus 4/8;
- IVF Auto scratch-on emits TurboQuant/QJL batch counters;
- SPIRE RaBitQ on/off pipeline behavior is recorded;

## AWS Host / Install Evidence

- Account/region: `932658697181`, `us-west-2`.
- Intel lane: profile `10k-intel`, instance `i-073714375f63a68af`,
  private IP `10.42.1.169`, bucket
  `s3://ecaz-cloud-10k-intel-e06ee4a0`.
- Graviton lane: profile `10k-medium`, instance `i-0fafab662921324eb`,
  private IP `10.42.1.233`, bucket
  `s3://ecaz-cloud-10k-medium-268ea93e`.
- Branch checkout on both hosts: `36f6ff793`.
- Extension version on both hosts: `ecaz|0.1.1`.
- Intel release artifacts:
  - `.so` sha256
    `9bbb23e95527db4f07807c26cb111cb90f012c1c428ff8d44c2a8685514961fd`
  - CLI sha256
    `7c715d4253e58291b7b634e50d6e59c351524a2036f5e1d036d48808af4d67cb`
- Graviton release artifacts:
  - `.so` sha256
    `061b6f7459174b4aca8004bfe48f1dcb2fca198f8bd620cdc9c7f56d8f301203`
  - CLI sha256
    `e3395497d2bc2ec529e459fdb2fb2649407d0f0c2df20b2d9fd09df15eb27027`
- EBS volumes were expanded from 400G to 800G after the initial no-space
  failure. Final sampled usage on both lanes was 405G used / 396G available
  on the 800G XFS volume.

## AWS Fixture / Audit Evidence

- Fixture prep completed:
  - Intel SSM `38743719-2f19-4f20-a08c-92ee5870fc64`, S3 prefix
    `s3://ecaz-cloud-10k-intel-e06ee4a0/bench-artifacts/task106-fixture-prep-intel/20260614T004257Z/`.
  - Graviton SSM `b80edbfd-a00f-499f-a07a-5ea1fe8ac0ec`, S3 prefix
    `s3://ecaz-cloud-10k-medium-268ea93e/bench-artifacts/task106-fixture-prep-graviton/20260614T004257Z/`.
- AWS audits passed after fixture staging:
  - Intel SSM `c4655ea6-befc-4b55-aa18-483810580ff2`:
    `[suite:task106-aws-intel-targeted] audit passed: 149 steps`.
  - Graviton SSM `7978b869-d75b-49df-8207-8a5866a1c17a`:
    `[suite:task106-aws-graviton-targeted] audit passed: 149 steps`.

## AWS Suite Results

- Intel suite:
  - Initial SSM `eba08084-e11b-4ec0-ae62-ad420f500985` failed at
    `load-1m-ivf-rabitq-b8` with `No space left on device`.
  - Resume SSM `3ae1893a-cf37-4441-b631-2718961be22e` completed with
    `Status=Success`, `ResponseCode=0`.
  - S3 prefix:
    `s3://ecaz-cloud-10k-intel-e06ee4a0/bench-artifacts/task106-aws-intel-targeted/20260614T005756Z/`.
  - Packet-local artifacts:
    `artifacts/aws-intel/suite-manifest.json`,
    `artifacts/aws-intel/results.jsonl`, and raw per-step logs.
  - `suite-manifest.json` has 149 succeeded steps and no failed, running,
    pending, or skipped steps.
  - `results.jsonl` has 826 rows.
- Graviton suite:
  - Initial SSM `85208de2-e707-410e-888e-ea262b60ee61` failed at
    `load-1m-ivf-rabitq-b8` with `No space left on device`.
  - Resume SSM `c7b9c748-b5fe-48ff-b172-4e67963fc166` completed with
    `Status=Success`, `ResponseCode=0`.
  - S3 prefix:
    `s3://ecaz-cloud-10k-medium-268ea93e/bench-artifacts/task106-aws-graviton-targeted/20260614T005756Z/`.
  - Packet-local artifacts:
    `artifacts/aws-graviton/suite-manifest.json`,
    `artifacts/aws-graviton/results.jsonl`, and raw per-step logs.
  - `suite-manifest.json` has 149 succeeded steps and no failed, running,
    pending, or skipped steps.
  - `results.jsonl` has 826 rows.

Operational note: the no-space recovery did targeted cleanup before resuming
the failed lane. After resume, no indexes were dropped while benchmark steps
were in flight.

## Single-Node SPIRE Boundary

The SPIRE cells in both main configs are single-node SPIRE, not distributed
SPIRE: `profile=ec_spire`, `local_store_count=1`, one EC2 host per AWS lane.
The suite labels and artifact names use `spire-rabitq`; no SPIRE pq_fastscan
cell is present.

## HNSW Grouped-PQ Gap-2 Evidence

- 1m HNSW grouped-PQ load:
  - Intel: copied corpus 299.36s, encoded corpus 408.06s, copied queries
    3.36s, built
    `t106_aws_intel_1m_hnsw_groupedpq_pq_fastscan_m16_idx` in 2278.82s,
    total 3111.77s.
  - Graviton: copied corpus 282.40s, encoded corpus 423.17s, copied queries
    2.93s, built
    `t106_aws_graviton_1m_hnsw_groupedpq_pq_fastscan_m16_idx` in 2507.15s,
    total 3363.71s.
- 1m HNSW grouped-PQ recall, batch-on:
  - Intel: ef 40/80/120 recall@k `0.7960` / `0.8815` / `0.9150`; ndcg@k
    `0.9580` / `0.9802` / `0.9875`.
  - Graviton: ef 40/80/120 recall@k `0.7960` / `0.8815` / `0.9150`;
    ndcg@k `0.9580` / `0.9802` / `0.9875`.
- 1m HNSW grouped-PQ latency, batch-on:
  - Intel: ef40 mean 7.03ms p95 11.4ms p99 13.5ms; ef80 mean 11.1ms p95
    16.9ms p99 21.5ms; ef120 mean 13.8ms p95 22.6ms p99 26.3ms.
  - Graviton: ef40 mean 5.11ms p95 8.07ms p99 9.48ms; ef80 mean 8.50ms
    p95 12.7ms p99 15.5ms; ef120 mean 11.2ms p95 16.9ms p99 20.1ms.
- Gap-2 counter observation:
  - Searching all HNSW grouped-PQ latency logs found no
    `block-kernel-counters` rows and no `width_*` histogram rows.
  - The latency logs do contain `task87-counters` rows for `surface=hnsw`,
    `ivf`, `spire`, and `unknown`; in the HNSW grouped-PQ cells those rows
    report `flushes=0`, `candidates=0`, `lut32_flushes=0`, and
    `lut32_candidates=0`.
  - Interpretation: the AWS benchmark suites completed successfully, but the
    requested flush-width histogram was not observed. This does not indicate
    recall/latency failure. It means the run did not provide positive evidence
    for a grouped-PQ traversal block-kernel investment; the likely next action
    is to inspect the probe wiring or counter surface before treating gap 2 as
    histogram-decided.

## Local Gap-2 Probe Diagnosis

- Date: 2026-06-14.
- Pre-fix head: `81eeccb07`.
- Local PG18 fixture:
  - `ec_hnsw` HNSW grouped-PQ / `storage_format=pq_fastscan`, `m=8`,
    `ef_search=40`, 1,000 corpus rows, 50 query rows.
  - Fixture and logs under `artifacts/local-gap2-repro/`.
- Pre-fix catalog checks:
  - `local-gap2-catalog-check.log`: extension `ecaz|0.1.1` installed;
    `ec_block_kernel_scoring_snapshot()` present.
  - `local-gap2-empty-snapshot.log`: snapshot function returned the expected
    width bucket columns and zero rows after reset.
- Pre-fix single-session repro:
  - Command used `target/release/ecaz dev sql --pg 18 ...` with
    `ec_hnsw.candidate_batch_scoring=on`, `ec_hnsw.ef_search=40`, and
    `enable_seqscan=off`.
  - `single-session-query-snapshot.log` shows the query used
    `t106_local_gap2_hnsw_groupedpq_pq_fastscan_m8_idx`, returned 10 rows,
    and then `ec_block_kernel_scoring_snapshot()` returned 0 rows.
- Root cause:
  - The Task 106 probe counted grouped-PQ traversal width only inside the
    direct grouped scoring subpath.
  - Default HNSW PqFastScan traversal can use the binary traversal score path
    for grouped candidates, bypassing that increment while still using the
    HNSW grouped-PQ index. The AWS logs therefore reflected probe placement,
    not absence of grouped-PQ traversal work.
- Fix:
  - Move the width increment to the top of each
    `CandidateScoreDispatch::Grouped(_)` arm in
    `src/am/ec_hnsw/scan.rs`, before binary traversal, exact-budget, or other
    scoring branches choose the scoring path.
  - This remains width-only evidence: scoring flush/kernel/scalar counts stay
    zero because no grouped-PQ traversal block kernel exists yet.
- Post-fix local validation:
  - `cargo test --lib grouped_pq_traversal_flush_width_probe_records_width_only_histogram --no-default-features --features pg18`
    passed: 1 test passed. Log:
    `local-gap2-repro/cargo-test-counter-after-fix.log`.
  - `rustfmt --check src/am/ec_hnsw/scan.rs` passed. Repo-wide
    `cargo fmt --check` still fails on existing quant-file formatting diffs
    outside this change. Log:
    `local-gap2-repro/rustfmt-scan-after-fix.log`.
  - Installed the patched release extension into local PG18 with
    `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
    and restarted `/home/peter/.pgrx/data-18`.
  - `single-session-query-snapshot-after-fix.log` shows the same HNSW
    PqFastScan index plan and a positive width-only row:
    `surface=hnsw`, `quant_kind=grouped_pq`, `isa=scalar`,
    `width_8_15_flushes=23`, `width_16_31_flushes=20`.
  - `latency-hnsw-groupedpq-batch-on-after-fix.log` shows the bench logging
    path now emits:
    `width_lt8=0 width_8_15=256 width_16_31=194 width_ge32=0`.
- AWS implication:
  - The completed AWS recall/latency results remain valid.
  - Gap 2 still needs an AWS rerun of the HNSW grouped-PQ gap-2 latency cells
    on a branch/head containing this probe fix before the histogram can be
    treated as AWS-measured.

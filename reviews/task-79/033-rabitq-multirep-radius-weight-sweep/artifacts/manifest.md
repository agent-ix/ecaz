# Task 79 Packet 033 Manifest: RaBitQ Multi-Representative Radius-Weight Sweep

- head SHA: `f954e3fb47cde15c92cd83012a581d03b856aefb`
- implementation commit: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/033-rabitq-multirep-radius-weight-sweep/`
- timestamp: `2026-06-02T08:38:11Z`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- installed clean backend SHA256: `929efde6155ae01ac72dd90395eea24334c1d496fc48b3f738ce7f52a1c1b15a`
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: `ec_spire.leaf_block_rows=32`
- summary representation: RaBitQ V4 leaf-block summaries with two cluster-mean representatives per block
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- selector: final global block cap 640, radius weights 0.00/0.10/0.20/0.25/0.30/0.40/0.50, no sampled rescue

## Commands

- clean backend install:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/install-clean-ecaz-pg18.log`
- restart local PG18:
  `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/pg18-restart.log restart -m fast`
- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/033-rabitq-multirep-radius-weight-sweep/suite-rabitq-multirep-radius-weight-sweep.json" reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/033-rabitq-multirep-radius-weight-sweep/suite-rabitq-multirep-radius-weight-sweep.json --manifest-output reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/033-rabitq-multirep-radius-weight-sweep/suite-rabitq-multirep-radius-weight-sweep.json --log-file reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/suite-manifest.json --log-file reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/suite-manifest.json --results-output reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/report-results.jsonl --log-file reviews/task-79/033-rabitq-multirep-radius-weight-sweep/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-multirep-radius-weight-sweep.json`: checked-in SuiteConfig for the local radius-weight sweep.
- `artifacts/install-clean-ecaz-pg18.log`: clean backend install log. Key line: `sha256=929efde6155ae01ac72dd90395eea24334c1d496fc48b3f738ce7f52a1c1b15a`.
- `artifacts/pg18-restart.log`: local PG18 restart log.
- `artifacts/suite-audit.log`: suite audit output; 9 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=9 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-multirep.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=16549`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
row	final_blocks	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
multirep	640	0.00	4015761	45.697	54.352	0.9840	2000	fail
multirep	640	0.10	4035744	45.529	52.734	0.9850	2000	fail
multirep	640	0.20	4046743	44.858	50.847	0.9865	2000	fail
multirep	640	0.25	4050758	44.696	53.143	0.9870	2000	fail
multirep	640	0.30	4054052	45.145	53.398	0.9865	2000	fail
multirep	640	0.40	4059747	44.786	51.076	0.9840	2000	fail
multirep	640	0.50	4063715	44.973	53.222	0.9810	2000	fail
```

Interpretation:

- Radius weight peaks at 0.25, matching packet 029's best cap640 recall of 0.9870.
- Candidate surface stays in the desired band at about 4.02M-4.06M candidates, and several rows satisfy p50, but none approach the 0.9925 recall gate.
- Radius-weight tuning alone cannot recover the 11 missed exact top-10 targets identified by packet 030. The next viable local path is a scoring change with more discriminating signal than a scalar radius blend.

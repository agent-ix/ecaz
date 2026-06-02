# Task 79 Packet 036 Manifest: RaBitQ Leaf-Contrast Calibration

- head SHA: `08f92d085249b73b3723b83227c4898640681732`
- implementation base commit: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- temporary code patch: `artifacts/leaf-contrast-source.patch` (negative research patch; not kept as production code)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/036-rabitq-leaf-contrast-calibration/`
- timestamp: `2026-06-02T02:51:46-07:00`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- installed leaf-contrast backend SHA256: `89d29c0345cc756864a8812be5aa9ba7147646cf41aaf971d7b37187561924ae`
- restored clean backend SHA256: `929efde6155ae01ac72dd90395eea24334c1d496fc48b3f738ce7f52a1c1b15a`
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: `ec_spire.leaf_block_rows=32`
- summary representation: RaBitQ V4 leaf-block summaries with two cluster-mean representatives per block
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- selector: final global block cap 640, radius weight 0.25, leaf-contrast weights 0.00/0.25/0.50/1.00/2.00/4.00, no sampled rescue

## Commands

- focused calibration test:
  `script -q -c "cargo test --no-default-features --features pg18 leaf_block_score_contrast_amplifies_leaf_local_outliers" reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/cargo-test-leaf-contrast.log`
- default global selector regression:
  `script -q -c "cargo test --no-default-features --features pg18 select_global_leaf_block_row_ranges" reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/cargo-test-global-block-selection.log`
- temporary patch capture:
  `git diff --output=reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/leaf-contrast-source.patch -- src/am/ec_spire/options/mod.rs src/am/ec_spire/scan.rs src/am/ec_spire/scan/candidates.rs src/am/ec_spire/scan/tests.rs src/am/ec_spire/scan/tests/candidates.rs`
- leaf-contrast backend install:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/install-leaf-contrast-ecaz-pg18.log`
- restart local PG18:
  `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/pg18-restart.log restart -m fast`
- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/036-rabitq-leaf-contrast-calibration/suite-rabitq-leaf-contrast-calibration.json" reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/036-rabitq-leaf-contrast-calibration/suite-rabitq-leaf-contrast-calibration.json --manifest-output reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/036-rabitq-leaf-contrast-calibration/suite-rabitq-leaf-contrast-calibration.json --log-file reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-manifest.json --log-file reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-manifest.json --results-output reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/report-results.jsonl --log-file reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/suite-report.log`
- restore clean local PG18 backend:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/install-clean-after-leaf-contrast-ecaz-pg18.log`
- restart clean local PG18:
  `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/036-rabitq-leaf-contrast-calibration/artifacts/pg18-clean-restart.log restart -m fast`

## Artifacts

- `suite-rabitq-leaf-contrast-calibration.json`: checked-in SuiteConfig for the local contrast sweep.
- `artifacts/leaf-contrast-source.patch`: temporary source patch used for this measurement.
- `artifacts/cargo-test-leaf-contrast.log`: focused unit test log. Key line: `1 passed; 0 failed`.
- `artifacts/cargo-test-global-block-selection.log`: default selector regression log. Key line: `2 passed; 0 failed`.
- `artifacts/install-leaf-contrast-ecaz-pg18.log`: patched backend install log. Key line: `sha256=89d29c0345cc756864a8812be5aa9ba7147646cf41aaf971d7b37187561924ae`.
- `artifacts/pg18-restart.log`: local PG18 restart log after the backend install.
- `artifacts/suite-audit.log`: suite audit output; 8 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/install-clean-after-leaf-contrast-ecaz-pg18.log`: clean backend reinstall after the negative experiment. Key line: `sha256=929efde6155ae01ac72dd90395eea24334c1d496fc48b3f738ce7f52a1c1b15a`.
- `artifacts/pg18-clean-restart.log`: local PG18 restart log after restoring the clean backend.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck. Key lines: `corpus_rows=100000`, `query_rows=1000`, `ec_spire.leaf_block_pruning_leaf_contrast_weight`.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-leaf-contrast.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=16456`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
row	global_blocks	leaf_contrast_weight	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
leaf_contrast	640	0.00	0.25	4050758	44.870	52.173	0.9870	2000	fail_recall
leaf_contrast	640	0.25	0.25	4052735	45.506	51.544	0.9860	2000	fail_recall_p50
leaf_contrast	640	0.50	0.25	4053651	44.801	52.646	0.9860	2000	fail_recall
leaf_contrast	640	1.00	0.25	4054561	44.925	51.436	0.9800	2000	fail_recall
leaf_contrast	640	2.00	0.25	4055175	45.168	52.805	0.9705	2000	fail_recall_p50
leaf_contrast	640	4.00	0.25	4055601	45.277	52.652	0.9540	2000	fail_recall_p50
```

Interpretation:

- Leaf-local mean-contrast calibration does not rescue the missed exact top-10 targets at the 640-block candidate envelope.
- Nonzero contrast monotonically worsens recall after 0.50 and never beats the contrast=0 control. It also slightly raises selected candidate rows by preferring fuller blocks.
- This closes the simple leaf-local contrast axis as a direct Task 79 fix. The remaining viable paths are richer two-stage summary scoring or a build-time calibration term informed by block-level residual/radius quality rather than query-local deviation from a leaf mean.

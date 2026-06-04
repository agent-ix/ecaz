# Task 79 Packet 037 Manifest: RaBitQ Two-Stage K3 Rescore

- head SHA: `eb251542973bfd64dd28c4e13290bd6cc67e07b5`
- implementation base commit: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- temporary code patch: `artifacts/two-stage-k3-rescore.patch` (negative research patch; not kept as production code)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/037-rabitq-two-stage-k3-rescore/`
- timestamp: `2026-06-02T03:48:26-07:00`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- installed two-stage k3 backend SHA256: `b156e25f1070b9555ec19796ba0000765572cd04e8d93c301c25e33dfa1ddc6f`
- restored clean backend SHA256: `929efde6155ae01ac72dd90395eea24334c1d496fc48b3f738ce7f52a1c1b15a`
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: `ec_spire.leaf_block_rows=32`
- summary representation: temporary RaBitQ V4 leaf-block summaries with three cluster-mean representatives per block
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- selector: full k3 controls and two-stage first-pass representative limit 2 with k3 rescore shortlist sizes 896/1024/1280/1536; radius weight 0.25; no sampled rescue

## Commands

- representative-limit unit test:
  `script -q -c "cargo test --no-default-features --features pg18 leaf_block_summary_representative_limit_scores_prefix_only" reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/cargo-test-representative-limit.log`
- full-rescore unit test:
  `script -q -c "cargo test --no-default-features --features pg18 full_rescore_promotes_block_with_late_representative_hit" reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/cargo-test-full-rescore.log`
- k3 build summary regression:
  `script -q -c "cargo test --no-default-features --features pg18 rabitq_leaf_block_summary_records_three_cluster_representatives" reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/cargo-test-k3-leaf-block.log`
- temporary patch capture:
  `git diff --output=reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/two-stage-k3-rescore.patch -- src/am/ec_spire/options/mod.rs src/am/ec_spire/scan.rs src/am/ec_spire/scan/candidates.rs src/am/ec_spire/scan/tests.rs src/am/ec_spire/scan/tests/candidates.rs src/am/ec_spire/build/recursive.rs src/am/ec_spire/build/tests/recursive.rs`
- two-stage k3 backend install:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/install-two-stage-k3-ecaz-pg18.log`
- restart local PG18:
  `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/pg18-restart.log restart -m fast`
- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/037-rabitq-two-stage-k3-rescore/suite-rabitq-two-stage-k3-rescore.json" reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/037-rabitq-two-stage-k3-rescore/suite-rabitq-two-stage-k3-rescore.json --manifest-output reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/037-rabitq-two-stage-k3-rescore/suite-rabitq-two-stage-k3-rescore.json --log-file reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-manifest.json --log-file reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-manifest.json --results-output reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/report-results.jsonl --log-file reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/suite-report.log`
- restore clean local PG18 backend:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/install-clean-after-two-stage-k3-ecaz-pg18.log`
- restart clean local PG18:
  `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/pg18-clean-restart.log restart -m fast`

## Artifacts

- `suite-rabitq-two-stage-k3-rescore.json`: checked-in SuiteConfig for the local two-stage k3 sweep.
- `artifacts/two-stage-k3-rescore.patch`: temporary source patch used for this measurement.
- `artifacts/cargo-test-representative-limit.log`: focused representative-limit unit test log. Key line: `1 passed; 0 failed`.
- `artifacts/cargo-test-full-rescore.log`: focused full-rescore unit test log. Key line: `1 passed; 0 failed`.
- `artifacts/cargo-test-k3-leaf-block.log`: k3 build-summary regression log. Key line: `1 passed; 0 failed`.
- `artifacts/install-two-stage-k3-ecaz-pg18.log`: patched backend install log. Key line: `sha256=b156e25f1070b9555ec19796ba0000765572cd04e8d93c301c25e33dfa1ddc6f`.
- `artifacts/pg18-restart.log`: local PG18 restart log after the backend install.
- `artifacts/suite-audit.log`: suite audit output; 10 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/install-clean-after-two-stage-k3-ecaz-pg18.log`: clean backend reinstall after the negative experiment. Key line: `sha256=929efde6155ae01ac72dd90395eea24334c1d496fc48b3f738ce7f52a1c1b15a`.
- `artifacts/pg18-clean-restart.log`: local PG18 restart log after restoring the clean backend. It shows the server ready on port 28818; later automatic-vacuum errors are from pre-existing Task 51 IVF/RaBitQ indexes in `tqvector_bench`, not from the Task 79 packet surface.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck. Key lines: `corpus_rows=100000`, `query_rows=1000`, `ec_spire.leaf_block_pruning_first_pass_representatives`, `ec_spire.leaf_block_pruning_full_rescore_blocks`.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-k3-two-stage.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=18150`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
row	global_blocks	first_pass_representatives	full_rescore_blocks	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
full_k3	736	0	0	4657668	48.853	56.690	0.9925	2000	fail_p50
full_k3	768	0	0	4860209	49.017	56.424	0.9925	2000	fail_p50
two_stage	736	2	1024	4657668	49.856	60.253	0.9925	2000	fail_p50
two_stage	736	2	1280	4657668	48.651	59.354	0.9925	2000	fail_p50
two_stage	768	2	896	4860209	49.648	57.167	0.9925	2000	fail_p50
two_stage	768	2	1024	4860209	49.281	56.671	0.9925	2000	fail_p50
two_stage	768	2	1280	4860209	49.088	57.734	0.9925	2000	fail_p50
two_stage	768	2	1536	4860209	49.543	56.906	0.9925	2000	fail_p50
```

Interpretation:

- Two-stage k=2 first-pass scoring plus k=3 late-representative rescore preserves the k=3 recall/candidate breakthrough but does not materially reduce scan latency.
- The best row, `global736/rescore1280`, reaches 0.9925 recall with 4.66M candidates, but p50 remains 48.651 ms.
- Full k3 at `global736` is a useful near-miss in its own right: it hits recall and candidate gates with fewer candidates than packet 035's `global768`, but p50 is still 48.853 ms.
- This closes the conservative two-stage Path B variant. The remaining direct paths should focus on reducing full k3 scorer CPU cost or avoiding k3 scan-time cost through build-time k2-compatible calibration.

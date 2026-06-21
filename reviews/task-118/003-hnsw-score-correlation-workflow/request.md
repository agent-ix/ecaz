# Review Request: Task 118 HNSW Score Correlation Workflow

## Head

`aba7b40e6b483ca20b9887a7c1bd1527f1f55a10`

## Scope

This checkpoint adds the Task 118 approximate/exact score-correlation workflow:

- adds a batched pg_test SQL export, `ec_hnsw_graph_scan_score_correlation_rows(...)`, for per-query score and rank drift rows;
- adds `ecaz bench hnsw-score-correlation`, which summarizes each `ef_search` and can emit per-query JSONL with compared row ids, approximate ranks/scores, exact scores, and exact ranks;
- adds `kind: "hnsw-score-correlation"` to `ecaz bench suite`, including validation, command expansion, default artifact paths, artifact-dir template rewriting, expected artifacts, and produced paths;
- updates `crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json` so the 10k / 50k / 100k x TurboQuant / PqFastScan / RaBitQ matrix includes score-correlation steps alongside recall, frontier, latency, and storage.

This is still not the final Task 118 decision packet. It adds the durable runner path for Phase 4 evidence.

## Evidence

See `artifacts/manifest.md` for commands and artifact metadata.

- `artifacts/cargo-check-pg18-pgtest.log`: PG18 + pg_test check passed.
- `artifacts/cargo-test-ecaz-cli-hnsw-score-correlation.log`: focused CLI tests passed, including suite expansion for the new step.
- `artifacts/suite-dry-run.log`: Task 118 suite dry-run expanded 54 selected steps, including nine `hnsw-score-correlation` steps.
- `artifacts/suite-dry-run-manifest.json`: normalized dry-run manifest with expected artifacts.

## Reviewer Focus

- Check whether the score-correlation row shape is sufficient for Task 118 Phase 4: score deltas, rank shifts, Spearman correlation, and compared candidate arrays.
- Check whether deriving exact ranks among emitted compared candidates is the right diagnostic scope for this slice, given the existing frontier diagnostic separately captures retained pre-output frontier candidates.
- Check whether the suite integration covers all runner surfaces reviewers expect: dry-run, manifest, artifact paths, and audit produced-path accounting.

## Known Follow-Up

The full benchmark matrix still needs to run on a host with staged `data/staged-current/` corpus inputs and a pg_test-enabled extension exposing the Task 118 diagnostic SQL functions. Phase 3 build-source A/B evidence and the final decision packet remain open.

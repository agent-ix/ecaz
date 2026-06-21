# Review Request: Task 118 HNSW Frontier Suite Workflow

## Head

`c143616ed2bc9ac7cffd7361018754247bff6095`

## Scope

This checkpoint turns the Task 118 HNSW frontier diagnostic into a suite-runner workflow:

- adds `ecaz bench hnsw-frontier` for batched per-query frontier containment summaries and JSONL output;
- adds `kind: "hnsw-frontier"` to `ecaz bench suite`, including validation, command expansion, default artifact paths, artifact-dir template rewriting, expected artifacts, and produced paths;
- adds `crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json`, covering 10k / 50k / 100k across TurboQuant, PqFastScan, and RaBitQ with load, recall, frontier, latency, and storage steps;
- adds a batched pg_test SQL export for `ec_hnsw_graph_scan_recall_frontier_containment_rows(...)` so the CLI does not rebuild exact-truth context once per query.

This is not the final Task 118 measurement packet. It establishes the reproducible runner shape that the benchmark host can execute for the required attribution evidence.

## Evidence

See `artifacts/manifest.md` for commands and artifact metadata.

- `artifacts/cargo-check-pg18-pgtest.log`: PG18 + pg_test check passed.
- `artifacts/cargo-test-ecaz-cli-hnsw-frontier.log`: focused CLI tests passed, including suite expansion for the new step.
- `artifacts/suite-dry-run.log`: Task 118 suite dry-run expanded 45 selected steps.
- `artifacts/suite-dry-run-manifest.json`: normalized dry-run manifest with expected artifacts.

## Reviewer Focus

- Check whether the `hnsw-frontier` CLI output is sufficient for candidate-containment attribution before final emission.
- Check whether the suite step integration covers the runner surfaces reviewers expect: dry-run, manifest, artifact paths, and audit produced-path accounting.
- Check whether the Task 118 suite matrix is scoped correctly for the required 10k/50k/100k x TurboQuant/PqFastScan/RaBitQ evidence.

## Known Follow-Up

The full benchmark matrix still needs to run on a host with staged `data/staged-current/` corpus inputs and a pg_test-enabled extension exposing the frontier diagnostic SQL function. This packet intentionally contains dry-run evidence only.

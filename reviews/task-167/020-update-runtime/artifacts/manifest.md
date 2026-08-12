head_sha: 406817f15
task_bucket: reviews/task-167
packet: 020-update-runtime
timestamp: 2026-08-12 (America/Los_Angeles)
fixture: ecaz dev distann-multicluster local-multinode-pg18 --pg 18 --nodes 3 --physical-benchmark
database: tqvector_bench; PostgreSQL 18.3; release extension profile
suite_config: artifacts/task167-physical-suite.json
suite_manifest: artifacts/suite-manifest.json
results: artifacts/results.jsonl
transient_archive: /home/peter/.ecaz/task167-020-transient-archive-20260812 (recoverable, not review evidence)

The suite command was driven exclusively by `ecaz bench suite` at 10k, 50k,
and 100k using `data/staged-current` prefixes `ec_real_10k`, `ec_real_50k`,
and `ec_real_100k`. Each arm used one physical index per owner table and one
single-index control table; the physical lane used three disjoint owners and
the coordinator outside the owner roster. No corpus or cluster directory is
committed.

Command:

    /home/peter/.cargo-target/debug/ecaz bench suite run --config reviews/task-167/016-physical-benchmark-suite/artifacts/task167-physical-suite.json --artifact-dir reviews/task-167/020-update-runtime/artifacts/final2 --manifest-output reviews/task-167/020-update-runtime/artifacts/final2/suite-manifest.json --results-output reviews/task-167/020-update-runtime/artifacts/final2/results.jsonl

The 10k and 50k steps completed in the original suite run. The pending 100k
step was completed in the same packet artifact directory, then the suite was
resumed with `--resume-from artifacts/final2/suite-manifest.json`; the resume
reused all three successful step records and wrote the normalized
`artifacts/results.jsonl` and final manifest. The cited compact lines are in
`artifacts/cited-results.log`; the stable UPDATE and failure drill lines are in
`artifacts/update-drill.log`.

Key evidence:

- 10k / 50k / 100k recall, latency, storage, and insert A/B rows are in `results.jsonl`.
- Insert-neighborhood parity used 48 distinct inserted source vectors at each scale and passed against a fresh rebuild.
- Mid-insert rollback, concurrent insert/query, topology, remote-owner materialization, and stable-vec_id UPDATE drills passed at every scale.
- The update drill observed one stable vec_id with version/count transition `(1,1)` to `(2,1)`.

# Artifact manifest — Task 193 packet 005

- Task bucket / packet: `reviews/task-193/005-owner-plan-candidate/`.
- Implementation SHA: `e444f6474`.
- Lane: local Intel, three isolated PG18 owner instances.
- Fixture: `ecaz bench suite`; one physical generation shared by both A/B
  arms plus a separate same-data single-index control.
- Storage / rerank / search: trained exact landmark head, RaBitQ stored
  neighbor codes, exact co-located row-tier rerank, lazy10, BW=4/H=100.
- Isolation: both arms explicitly disable the Task 192 validation cache. The
  control explicitly disables and the candidate explicitly enables only the
  Task 193 owner payload plan cache.
- Protocol: 200 recall queries / 2,000 trials and 10 warmups + 50 measured
  latency iterations. Stage counters and materialization correctness drills
  are enabled.
- Corpus/query: `ec_real_100k`; corpus TSVs are intentionally not committed.
- Suite config: `task193-owner-plan-100k.json`.
- Planned command: `target/debug/ecaz bench suite run --config
  reviews/task-193/005-owner-plan-candidate/artifacts/task193-owner-plan-100k.json
  --database tqvector_bench --log-file
  reviews/task-193/005-owner-plan-candidate/artifacts/suite-run.log`.

The release install identity, suite audit, structured results, summary, recall,
latency, storage, and correctness artifacts will be added after the run. Node
logs, fixture transcripts, single-control raw logs, and generated corpus/truth
data are operational exhaust and will not be committed.

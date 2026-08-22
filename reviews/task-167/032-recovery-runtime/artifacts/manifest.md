# Task 167 packet 032 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `032-recovery-runtime`.
- Status: preregistered; no runtime result claimed yet.
- Suite config: `task167-recovery-suite.json`.
- Lane: local Intel PG18, three physical owners, one index per owner table and
  a single-index control.
- Storage format: RaBitQ neighbor codes with exact owner-routed payload rerank.
- Real-corpus reloptions: shipped defaults `graph_degree=32` and
  `head_index_cap=4096`; BW4/H100 retained production traversal posture.
- Scales: staged `ec_real_10k`, `ec_real_50k`, and `ec_real_100k` under
  `data/staged-current`; corpus/query TSVs are not packet artifacts.
- Stress fixture: 2k rows, dimension 4, degree 8, three owners.
- External run directories: dated Task 167 directories under
  `/home/peter/.ecaz/clusters/`, never under the repository or Cargo target.
- Execution policy: production extension built with
  `--release --no-default-features --features pg18`; no `pg_test` or debug
  override; preliminary results are diagnostic and the final matrix must use
  one exact SHA/config.


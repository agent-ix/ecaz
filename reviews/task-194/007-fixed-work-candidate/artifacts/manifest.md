# Artifact manifest — Task 194 packet 007

- Task bucket / packet: `reviews/task-194/007-fixed-work-candidate/`.
- Implementation SHA: `e444f6474`.
- Lane: local Intel, three isolated PG18 owner instances.
- Fixture: `ecaz bench suite`; one physical generation shared by both A/B
  arms plus a separate same-data single-index control.
- Storage / rerank: trained exact landmark head, RaBitQ stored neighbor codes,
  exact co-located row-tier rerank, lazy10.
- Isolation: both arms explicitly disable the Task 192 validation cache and
  Task 193 prepared-plan cache. The control is BW=4/H=100 and the candidate is
  BW=8/H=50; both have the same nominal BW×H=400 expansion bound.
- Protocol: 200 recall queries / 2,000 trials and 10 warmups + 50 measured
  latency iterations. Nine-way stage/work counters are enabled.
- Corpus/query: `ec_real_100k`; corpus TSVs are intentionally not committed.
- Suite config: `task194-fixed-work-100k.json`.
- Suite audit: passed, one step.
- Validation: strict PG18 attribution-feature clippy passed with warnings
  denied; the focused CLI variant tests passed; the runner build completed.
- Installed extension preflight: release target and installed PG18 library are
  both 24,244,984 bytes with SHA-256
  `ec58009be20adf9db45af01fcc9bf0a947b9ec893ee6541f9c47d194f5ea8031`.
- Planned command: `target/debug/ecaz bench suite run --config
  reviews/task-194/007-fixed-work-candidate/artifacts/task194-fixed-work-100k.json
  --database tqvector_bench --log-file
  reviews/task-194/007-fixed-work-candidate/artifacts/suite-run.log`.

The structured results, summary, recall, latency, and storage artifacts will be
added after the run. The release-install transcript is owned by Task 193 packet
005 because one verified install serves the sequential Task 193/194 run;
`validation.md` records the identical preflight locally. Operational node logs,
fixture transcripts, single-control raw logs, and generated corpus/truth data
will not be committed.

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
- Runner SHA: `e444f6474`; extension SHA
  `fb0c512bf3bb9c7358ea905bf4e8565bd53fc181`, unanimous release profile.
- Suite duration: 2,084,540 ms; 1 succeeded, 0 failed, 0 missing, 0 stale.

The structured results, summary, recall, latency, and storage artifacts will be
added after the run. The release-install transcript is owned by Task 193 packet
005 because one verified install serves the sequential Task 193/194 run;
`validation.md` records the identical preflight locally. Operational node logs,
fixture transcripts, single-control raw logs, and generated corpus/truth data
will not be committed.

## Decision artifacts

| Artifact | SHA-256 |
|---|---|
| `task194-fixed-work-100k.json` | `218c461ea0d725bcdd2153f227f2f1a0d0bbb443fba6f9195e41f42e2d79ab8b` |
| `run/suite-manifest.json` | `0ff89f94f7940d413ba799de557057fb14b40ae3ffe2cd177ef55eb5b7905718` |
| `run/results.jsonl` | `14e5c5da2b1c20b7eb786c91bd53832c24f66a4740bc2b6b2b3be041a40f345a` |
| `run/fixed-work-ab-100k/distann-multinode-summary.log` | `da56a9c964dc0f801f26143713c30db4ee1cb34ba958e2aeda1fd37cdcde358c` |
| `run/fixed-work-ab-100k/physical-production-rounds-recall.log` | `fab7414c6f70a17a9ad9e60397887009f05f03025ad1d3b7ab57482aa88489df` |
| `run/fixed-work-ab-100k/physical-production-rounds-latency.log` | `6e271a37c682c7f4fdc472d15e5b85d0f663b26d9e1ea26700f8b4538307ca4d` |
| `run/fixed-work-ab-100k/physical-wider-rounds-recall.log` | `0c5ca3b896facafec1efa96ab713f1aaab904eca3436d0f0acac780f7aa9fcd6` |
| `run/fixed-work-ab-100k/physical-wider-rounds-latency.log` | `ac556fc16487e5a03c7c735fec5600443970e4a93dacd7df93440a8b6b4093f7` |

## Key results (production/candidate)

- Recall `0.9625/0.9675`; warm mean `24.30/24.20 ms`; p95
  `27.80/28.30 ms`; storage `2,496,659,456` bytes both.
- Hops `10.0/5.88`; nodes `40.0/47.04`; traversal
  `7.685150/7.081538 ms`; transport wait `4.179608/3.435451 ms`;
  straggler spread `0.410776/0.736326 ms`.
- Decision: STOP; the traversal-local win did not produce useful end-to-end
  latency and the tail regressed.

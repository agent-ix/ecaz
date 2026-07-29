# Task 200 Phase 1 reproduction artifacts

- Packet: `reviews/task-200/001-reproduction/`
- Task: `plan/tasks/200-ec-distann-backend-memory-retention.md`
- Checkout head at packet capture: `9de8b4fa2` (`chore/build-artifact-disk-usage`)
- Measurement binary: `target/debug/ecaz`, built from the equivalent pre-commit working tree immediately before `9de8b4fa2`
- PostgreSQL: PG18.3; three-owner physical fixture; TCP ports 42080–42082
- Fixture: `/home/peter/.ecaz/clusters/task200-counters-off-100k`; stopped after each run and retained for reuse
- Extension provenance reported by every node: `extension_git_sha=897c69045249a876de151c1da0544001ead82352-dirty`, `extension_build_profile=release`
- Corpus: `ec_real_100k`; staged corpus/query files under `data/staged-current`; query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Generation: graph degree 32, head cap 16384, `head_sample_exact`, search width 32, seed count 32, `rabitq`, beam width 4, hop rounds 100
- Latency mode: physical production arm, 300 timed queries, 10 warmups, `worker_batch_size=0`, one backend, RSS/HWM sample interval 250 ms

## Cited artifacts

| Artifact | Command / purpose | Key result |
| --- | --- | --- |
| `task200-backend-memory-reproduction-suite.json` | `ecaz bench suite` config | Counters-off builds once; counters-on sets `reuse_fixture=true` and shares the same run directory. |
| `run-latency-rerun/counters-off-100k/physical-production-latency.log` | Suite counters-off physical latency arm | `count=300`, mean 27.50 ms, `worker_batch_size=0`; RSS series 33 samples, 260104→261028 KB, 8067 ms, slope 114.54 KB/s. |
| `run-latency-rerun/counters-on-100k/physical-production-latency.log` | Same fixture, stage counters enabled | `count=300`, mean 26.50 ms, `worker_batch_size=0`; RSS series 32 samples, 260024→261024 KB, 7817 ms, slope 127.93 KB/s. |
| `run-latency-rerun/counters-{off,on}-100k/physical-production-latency.memory-series.log` | Streamed fixed-interval RSS/HWM series | Both production paths remain near 260–261 MB; no multi-GB growth. |
| `run-latency-rerun/counters-on-100k/distann-local-multinode.log` | Reuse decision and provenance | `fixture_decision action=reuse`, source rows 100000, unanimous release extension provenance. |
| `run-latency-rerun/coverage-only.log` | Standalone coverage statement | Backend PID 589846 entered the coverage statement; the statement was canceled after the RSS safety limit. |
| `run-latency-rerun/diagnostic-node1.log` | PostgreSQL memory-context dumps | At ~6.8 GB RSS, `Grand total: 8323959872 bytes ... 8314784136 used`; the SQL statement is recorded in the node log. |
| `run-latency-rerun/reuse-suite-manifest.json` and `reuse-results.jsonl` | Suite provenance/result records | Records the reused run command and normalized result rows. |

The large diagnostic logs are intentionally packet-local. No corpus TSV, PGDATA, or external cluster directory is committed.

`run-latency-rerun/coverage-separate-200.log` is retained but excluded from
the cited evidence. The node log shows all calls were sent in one simple-query
protocol message, hence one implicit transaction; it cannot support a claim
about retention across transaction or statement boundaries.

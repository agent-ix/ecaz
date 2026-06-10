# Task 97 Packet 009 Artifact Manifest

- head SHA: `70f6f2cf3c2f3c06a67139754242ce2c465d1f3e`
- task bucket: `reviews/task-97/009-local-qjl32-suite`
- lane: Task 97 TurboQuant QJL block kernel
- fixture: local PG18, deterministic synthetic corpus, `dim=1024`, `bits=4`, `seed=42`, `queries_seed=43`
- storage format: `turboquant`
- rerank / exact mode: production QJL (`MseLutQjl`), not no-QJL / LUT32
- host ISA: local x86_64 AVX2 where block width reaches 32; scalar tails otherwise
- AWS / CI: not run

## Commands

- Generate fixture:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/009-local-qjl32-suite/artifacts/suite-generate-cli.log bench suite run --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json --only generate-qjl32-corpus --only generate-qjl32-queries --manifest-output reviews/task-97/009-local-qjl32-suite/artifacts/suite-generate-manifest.json --results-output reviews/task-97/009-local-qjl32-suite/artifacts/generate-results.jsonl`
- Audit:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/009-local-qjl32-suite/artifacts/suite-audit-cli.log bench suite audit --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json`
- Local PG18 install for final evidence:
  `target/debug/ecaz --log-file reviews/task-97/009-local-qjl32-suite/artifacts/local-ecaz-pg18-install-after-hotcold-batch.log dev install ecaz-pg-test --pg 18`
- Current kernel-on local suite:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/009-local-qjl32-suite/artifacts/suite-rerun-kernel-on-current-cli.log bench suite run --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json --artifact-dir reviews/task-97/009-local-qjl32-suite/artifacts --only-tag kernel_on --manifest-output reviews/task-97/009-local-qjl32-suite/artifacts/suite-rerun-kernel-on-current-manifest.json --results-output reviews/task-97/009-local-qjl32-suite/artifacts/results-rerun-kernel-on-current.jsonl`

## Primary Artifacts

- `task97-local-qjl32-suite.json`: suite config for the clarified `dim=1024,bits=4` QJL fixture.
- `task97_qjl32_corpus.tsv`, `task97_qjl32_queries.tsv`: generated deterministic fixture inputs.
- `cargo-fmt-check.log`: `cargo fmt --check` packet-local validation.
- `cargo-test-hnsw-qjl-codec.log`: focused codec-level HNSW QJL batch test.
- `cargo-test-turboquant-exact-payload-batch.log`: focused HNSW exact-payload batch tests for QJL and full-LUT no-QJL.
- `local-ecaz-pg18-install-after-hotcold-batch.log`: final local PG18 install; backend assertion passed, sha256 `041ce14cf789394aa9a91886d873fe11bcd1b35aae71a69dda1f320c0a54facb`.
- `suite-rerun-kernel-on-current-cli.log`, `suite-rerun-kernel-on-current-manifest.json`, `results-rerun-kernel-on-current.jsonl`: current local kernel-on run at the code checkpoint.
- `latency-ivf-turboquant-qjl32-batch-on.log`: IVF latency and direct counters.
- `latency-spire-turboquant-qjl32-batch-on.log`: SPIRE latency and direct counters.
- `latency-hnsw-turboquant-qjl32-batch-on.log`: HNSW latency and direct counters.

## Key Result Lines

- IVF `nprobe=8`: mean `1.31 ms`; direct row `surface=ivf quant=turboquant_qjl isa=avx2 kernel_candidates=24096`, plus scalar tails `scalar_candidates=1263`.
- IVF `nprobe=16`: mean `1.67 ms`; direct row `surface=ivf quant=turboquant_qjl isa=avx2 kernel_candidates=51200`.
- SPIRE `nprobe=8`: mean `9.40 ms`; direct row `surface=spire quant=turboquant_qjl isa=avx2 kernel_candidates=13696`, plus scalar tails `scalar_candidates=11663`.
- SPIRE `nprobe=16`: mean `17.7 ms`; direct row `surface=spire quant=turboquant_qjl isa=avx2 kernel_candidates=28800`, plus scalar tails `scalar_candidates=22400`.
- HNSW `ef_search=32`: mean `1.98 ms`; direct row `surface=hnsw quant=turboquant_qjl isa=scalar scalar_candidates=29763`.

## Notes

- HNSW uses the m=8 suite fixture from the local packet. Its graph expansions are below block width 32, so direct rows are scalar-tail rows under `quant=turboquant_qjl`; this is expected width gating, not a missing QJL route.
- Earlier diagnostic logs in this packet record the stale-extension and pre-hot/cold attempts that produced zero HNSW rows. The current evidence source is `suite-rerun-kernel-on-current-*` plus the final `latency-*-batch-on.log` files.

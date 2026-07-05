# Packet 002 — artifact manifest (Phase 3 benchmark gate)

- Task bucket: `reviews/task-111g/`
- Packet: `002-rerank-rep-benchmark-gate`
- Branch: `task-111g-coarse-rerank-representations`
- Head SHA: recorded at commit time (see packet commit)
- Lane: not yet run — config validated on local Intel desktop (PG18), corpora not staged
- Storage formats under test (config): `coarse_rerank` (rerank_format f32/f16/rabitq4), `rabitq` (dense-rb8 baseline; row-f32 baseline)
- Surfaces: isolated one-index-per-table via `ecaz corpus load` per `prefix`
- Code change under review: no (config + supporting profiles.rs reloption addition only). Measurement numbers: NONE (run is blocked — see request.md)

## Commands (validation only; full run is blocked)

- `ecaz bench suite audit --config reviews/task-111g/002-rerank-rep-benchmark-gate/artifacts/task111g-rerank-rep-suite.json`
- `ecaz bench suite run --dry-run --config reviews/task-111g/002-rerank-rep-benchmark-gate/artifacts/task111g-rerank-rep-suite.json`
- `cargo build -p ecaz-cli`
- `cargo test -p ecaz-cli profiles`

## Artifacts

| File | What it shows | Key lines |
| --- | --- | --- |
| `task111g-rerank-rep-suite.json` | The bespoke SuiteConfig (coarse_rerank × {f32,f16,rabitq4} × {50k,100k} + dense-rb8 / row-f32 baselines) | n/a (config) |
| `suite-audit.log` | Config parses; only 30 missing-corpus issues | `audit: step "..." references missing input data/staged-current/...`; `suite audit found 30 issue(s)` |
| `suite-dry-run.log` | Reloption passthrough expands correctly | `--storage-format coarse_rerank ... --reloption rerank_format=f16`; `--reloption quant_bits=8 --reloption rerank=off` (dense-rb8); `--reloption rerank=heap_f32 --reloption rerank_width=200` (row-f32) |

## Run prerequisites (not satisfied in this sandbox)

- Branch `ecaz.so` installed on the PG18 instance (coarse_rerank f16/rabitq4 rerank).
- Real 50k + 100k corpora staged at `data/staged-current/` as `ec_real_{50k,100k}_{corpus,queries}.tsv` + `_manifest.json`.

When the run lands, add: `suite-manifest*.json`, `suite-results*.jsonl`, the
recall/latency/storage logs, and the matched-recall post-hoc table, then record
the head SHA and key result lines here and the promote/iterate verdict in
`request.md`.

Timestamp: 2026-06-18 (config validation only).

# Artifact manifest

- Head SHA: `2a4a70b23161f556c44d6d1d2c960541fbcb1bdb`
- Task bucket: `reviews/task-199`
- Packet: `reviews/task-199/002-operations-lifecycle-and-isolation`
- Timestamp: `2026-07-25T15:53:02-07:00`
- Lane: local WSL, managed PG18 `18.3`, three isolated local nodes
- Fixture: staged `ec_real_10k`; query SHA-256
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Storage/rerank: physical `ec_distann`, degree 32, trained exact cap-4,096
  head, ordered 32 persisted-head seeds, BW4/H100, RaBitQ neighbor scoring,
  exact final scoring, lazy10 materialization
- Isolation: one control index per physical corpus table
- Runner: checked-in `ecaz bench suite`, clean exact SHA above
- Run interval: `1785019610617` through `1785019982246` Unix milliseconds;
  duration `378503 ms`

Corpus/query TSVs are intentionally not committed. Hashes below are hashes of
the committed blobs; Git's LF normalization is therefore reflected for the
two `script`-captured build logs.

## Commands and artifacts

| Artifact | SHA-256 | Command / key result |
| --- | --- | --- |
| `task199-normal-lifecycle-10k.json` | `bf3510fbdeacb98edcfa9db306cb8b9848d6d774d07c52f27505058d35a003e6` | checked-in one-step `SuiteConfig` |
| `normal-release-install-r27.log` | `274d5d2bb947cc4dd280d81225895653d432446646b4bc806b0e386483b1c434` | clean detached `cargo pgrx install --release ... --features pg18`; 311 SQL entities, two triggers |
| `ecaz-cli-release-build-r27.log` | `987811db5e8f19b866e7b0f0643397fca63a026a8270205bde69223bf2462fa7` | clean detached `cargo build --release -p ecaz-cli`; one pre-existing corpus-loader warning |
| `suite-run-r27.log` | `f265ed2b51283057b0cb447ff1bfad0c91a3f5c98407a50d9fef3a0ef2972a98` | `ecaz bench suite run --config .../task199-normal-lifecycle-10k.json --artifact-dir .../artifacts/run`; succeeded |
| `run/suite-manifest.json` | `932119090ef4e0f1cfda62eb9e9f20c3053c98efe38c237e53076595cf087436` | runner exact `2a4a70b23`; one succeeded step, exit 0 |
| `run/results.jsonl` | `4fc5641b35b5c85badbd09a096a38f66fbb16641057f038891699e1bf6704c7b` | normalized topology, lifecycle, recall, latency, storage, and semantic rows |
| `run/normal-replica-lifecycle-10k/distann-multinode-summary.log` | `4639e4f6910c16f0124bea126b86224fc392ae89f639a0a62f9753579bf77068` | every cited lifecycle scenario `pass=true`; exact release provenance |
| `run/normal-replica-lifecycle-10k/distann-local-multinode.log` | `863fe1069124fffb52bee6ccd0016ea3919ea2fad4c946769b92320cb47d2cb1` | three-node topology, lifecycle drills, feature isolation, semantics |
| `run/normal-replica-lifecycle-10k/task199-enospc-provider.marker` | `dc8970d382026cbc0f25cd853c4e196fcfd10d1b35cc01038ede35b11767cd4d` | raw provider witnesses: create `open errno=28`; mid-copy `pwrite count=2 errno=28` |
| `suite-audit-r27.log` | `474c702b085e7e0b191462c3c47f912251d560978e49a7700d81d62a7d2d2e5b` | audit passed, one step |
| `suite-status-r27.log` | `113218ccc85414eb9f303358c5699f17534d9b0aa88ec59e5b9c948b8bcbd248` | completed 1, failed 0, missing 0, stale 0 |
| `suite-report-r27.md` | `ea1fff2b1d21691f99bb79148f03f0f26b8ae562add9146148adf10094ff8b73` | compact parsed report |

## Cited result lines

- `physical_benchmark_no_replica_insert ... pass=true ... rows_per_second=2481.671`
- `scenario=stronger_isolation_mutation_fenced pass=true ... stale_snapshot=true xid_assigned_before_ready=true ready_committed_after_snapshot=true sqlstate=40001 ... inserted_rows=0`
- `scenario=enospc_create_cleanup pass=true sqlstate=53100 ... catalog_residue=0 relation_residue=0 ... recovery_build_state=Ready`
- `scenario=enospc_midcopy_cleanup pass=true sqlstate=53100 ... building_row_created=true ... catalog_residue=0 relation_residue=0 ... tablespace_restored=true recovery_build_state=Ready`
- Initial Ready image: 10,000 records, 131,520,000 copied bytes,
  158,326,784 relation bytes, 137,540,056 WAL bytes, 4,980 ms build.
- Owner/replica lifecycle recall: `0.9900` / `0.9900`; two-sample warm
  latency: `19.80` / `15.20` ms.

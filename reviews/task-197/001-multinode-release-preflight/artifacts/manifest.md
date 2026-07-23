# Task 197 packet 001 artifact manifest

- Evidence head: `be2f3497f0bbae16a861bcbb91ac44bab7595e92`
- Task bucket / packet: `reviews/task-197/001-multinode-release-preflight/`
- Timestamp: 2026-07-22 19:12 America/Los_Angeles
- Lane: local PG18 self-hosted `distann-local-multinode`
- Fixture: synthetic 32-row, 16-dimension physical fixture; one owner; port 39970
- Storage / quantizer: normal `ec_distann` physical generation / RaBitQ
- Rerank mode: normal physical exact rerank; performance measurement not in scope
- Isolation: one index per table; the run directory is outside the review packet
- Runner: `target/debug/ecaz`, SHA-256 `88ebb36855721b290c95e8a8eae860caf449f850cd397e3eee0308d04ac495cc`
- Installed extension: release profile, git SHA `a5e567c45a5c96f67a842163e2293843d0a3774a`, SHA-256 `d5a4e92a9d13310a045f26753a41b6fea00b61661a5bfc384a9804e66d00a1ad`

## Commands and artifacts

| Artifact | Command / purpose |
| --- | --- |
| `extension-preflight-tests.log` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --offline -p ecaz-cli extension_preflight -- --nocapture`; four tests cover unanimous release acceptance, default debug rejection, mixed-node rejection even with override, and explicit unanimous-debug override. |
| `structured-result-test.log` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --offline -p ecaz-cli distann_multinode_rows_parse_recall_identity_gate_and_drills -- --nocapture`; proves the flushed line becomes `multinode_release_preflight`. |
| `suite-expansion-test.log` | `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --offline -p ecaz-cli distann_local_multinode_step_expands_head_index_cap -- --nocapture`; proves `allow_debug_extension` serializes to `--allow-debug-extension`. |
| `task197-suite.json` | Checked-in `SuiteConfig` for the short physical integration smoke. |
| `suite-audit.log` | `target/debug/ecaz --log-file .../suite-audit.log bench suite audit --config .../task197-suite.json`; audit passed for one step. |
| `suite-dry-run.log`, `dry-run-manifest.json` | `target/debug/ecaz --log-file .../suite-dry-run.log bench suite run --config .../task197-suite.json --dry-run --manifest-output .../dry-run-manifest.json`; command expansion includes the fixture and packet-local logging surfaces. |
| `suite-run.log`, `suite-manifest.json`, `results.jsonl` | `target/debug/ecaz --database tqvector_bench --log-file .../suite-run.log bench suite run --config .../task197-suite.json --manifest-output .../suite-manifest.json --results-output .../results.jsonl`; step succeeded in 2278 ms and manifest runner SHA is the evidence head. |
| `suite/release-preflight-smoke/distann-local-multinode.log` | Full compact fixture log. Line 4 is the passed release preflight; line 5 is `physical_setup_start`; ready/published topology and serving subsequently pass. |
| `suite/release-preflight-smoke/distann-multinode-summary.log` | Compact successful fixture summary. Regenerable PostgreSQL node logs were pruned and are not committed. |

## Key results

- `release_profile_preflight status=passed nodes=1 unanimous=true ... extension_build_profile=release debug_override=false`
- The accepted preflight is line 4 and `physical_setup_start` is line 5, proving the gate was flushed before corpus/generation setup.
- `results.jsonl` records `metric=multinode_release_preflight`, `pass_numeric=1`, `unanimous=true`, and `debug_override=false`.
- All four provenance-validator cases pass; the mixed-node case exercises two different node/port observations and remains rejected under the diagnostic override.
- The task changes only CLI fixture/suite tooling. It does not change quantizer, index, scan, rerank, posting, or storage behavior, so the task-defined 10k/50k/100k performance-matrix exemption applies.

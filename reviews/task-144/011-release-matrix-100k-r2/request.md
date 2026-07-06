# Task 144 Packet 011: 100k Release Matrix r2

## Request

Review the completed 100k release matrix slice from the approved Task 144 r2 suite config.

This packet adds the missing 100k evidence after the approved 50k packet. The run used release PostgreSQL 18.3 (`ecaz_build_profile=release`), the approved r2 suite config, and `ecaz bench suite`.

## Evidence

- Artifact manifest: `artifacts/manifest.md`
- Suite manifest: `artifacts/suite-manifest-100k-r2.json`
- Suite results: `artifacts/results-100k-r2.jsonl`
- Suite run log: `artifacts/suite-run-100k-r2.log`
- Release precheck: `artifacts/precheck-release-profile.log`
- Pipeline logs: `artifacts/pipeline-100k-*.log`

The suite completed all 30 pipeline cells: 5 index variants x 6 probe modes. The structured results include 3030 `spire-pipeline` rows, 900 row-scan rows, 25 load timing rows, and 60 storage rows.

## Result

Only `closure_e050_b8` reaches recall >= 0.99 at 100k:

| row | nprobe | recall | candidate rows | ready rows | production p50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| closure_e050_b8-ratio200 | 96 | 0.9925 | 78.6594% | 31.7537% | 73.132 ms |
| closure_e050_b8-ratio800 | 96 | 0.9925 | 79.1109% | 31.9043% | 71.754 ms |
| closure_e050_b8-ratio400 | 96 | 0.9925 | 79.1109% | 31.9043% | 72.181 ms |
| closure_e050_b8-fixed | 96 | 0.9925 | 79.1109% | 31.9043% | 72.587 ms |
| closure_e050_b8-adaptive | 96 | 0.9925 | 79.5005% | 32.6196% | 73.023 ms |

The best `fixed_b2` row is `fixed_b2-adaptive @ np96`: recall 0.9775, 33.1731% candidate rows, 21.3365% ready rows, 33.266 ms production p50. The best `closure_e025_b8` row is `closure_e025_b8-adaptive @ np96`: recall 0.9835, 54.9097% candidate rows, 52.271 ms production p50.

Storage also worsens at the only 0.99-recall family: `closure_e050_b8` is 568.8 MiB with 7.1315 mean replicas/vector, versus `fixed_b2` at 246.0 MiB and 3.0000 mean replicas/vector.

## Ask

Please review the 100k evidence and confirm the Task 144 matrix conclusion: no promotion candidate. The data supports iterate/escalate, with ratio pruning retired as a headline lever.

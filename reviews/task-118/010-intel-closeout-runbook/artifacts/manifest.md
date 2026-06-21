# Task 118 Packet 010 Artifact Manifest

- head SHA: `fb756b6d6fae31e0e2f645e55c865a3930eb4dc4`
- task bucket: `reviews/task-118/010-intel-closeout-runbook`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: Intel final closeout runbook for 10k/50k/100k source-build and compressed-build HNSW suite lanes.
- isolated surface: the runbook preserves the existing suite shape: one HNSW index per loaded prefix.
- refresh note: packet 017 updates this runbook so packet 016's AMD-local
  diagnostics are treated as preview evidence only, and final closeout requires
  Intel artifacts at all three required scales.

## Artifacts

### `intel-closeout-runbook.md`

- purpose: exact operator runbook for the final Intel Task 118 evidence pass.
- includes:
  - extension install command with `pg18 pg_test` diagnostics enabled;
  - 10k `ecaz bench suite` command;
  - 50k `ecaz bench suite` command;
  - 100k `ecaz bench suite` command;
  - status and report regeneration commands;
  - post-run `jq` checks for recall, frontier, score-correlation, latency, and storage evidence;
  - packet 018 final-table extractor command for `final-decision-table-intel.tsv`;
  - commit-scope guardrails to avoid committing truth caches, raw per-query JSONL, corpus TSVs, or scratch exhaust;
  - final decision table template.

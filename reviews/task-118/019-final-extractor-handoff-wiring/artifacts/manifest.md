# Task 118 Packet 019 Artifact Manifest

- head SHA: `2fb288b30f3aa131da68848a1de1110bd962b01d`
- task bucket: `reviews/task-118/019-final-extractor-handoff-wiring`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: final Intel closeout handoff
  wiring for the packet 018 offline table extractor.
- isolated surface: consumes final `ecaz bench suite` JSONL result files from
  the one-index-per-prefix suite layout.

## Artifacts

### `../010-intel-closeout-runbook/artifacts/intel-closeout-runbook.md`

- purpose: operator runbook for final Intel Task 118 evidence pass.
- change in this checkpoint:
  - replaces manual multi-command row extraction with packet 018's
    `task118-final-table.jq` extractor;
  - adds `final-decision-table-intel.tsv` to the final commit scope;
  - states that the final packet must fill the interpretation columns while
    preserving generated measurement values.

### `../011-final-closeout-audit-template/artifacts/final-closeout-audit-template.md`

- purpose: final quality-control checklist for packet 006 closeout.
- change in this checkpoint:
  - adds the exact extractor command;
  - adds an `awk` width/count check expecting 15 columns, 18 data rows, and no
    malformed rows;
  - updates the required decision-table columns to match the extractor output.

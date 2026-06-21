# Task 118 Packet 011 Artifact Manifest

- head SHA: `b45dfe8647cca5841df89920c7b20b1daf1032a0`
- task bucket: `reviews/task-118/011-final-closeout-audit-template`
- generated: `2026-06-21`
- lane / fixture / storage format / rerank mode: final Task 118 closeout audit template for the future Intel 50k/100k suite evidence.
- isolated surface: template assumes the checked-in Task 118 suite's one-index-per-prefix layout.

## Artifacts

### `final-closeout-audit-template.md`

- purpose: requirement-by-requirement audit template for proving Task 118 closeout after Intel evidence lands.
- includes:
  - expected final artifacts;
  - selected-step status checks;
  - result-row completeness checks;
  - acceptance-criteria-specific `jq` extraction commands;
  - final decision table schema;
  - allowed dominant-loss labels;
  - commit hygiene checklist.

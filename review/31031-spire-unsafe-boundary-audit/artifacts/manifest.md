# Artifact Manifest

Head SHA: `1ea3b750c29a60627f8c3e196afd7110ba252887`

Packet/topic: `31031-spire-unsafe-boundary-audit`

Lane / fixture / storage format / rerank mode: static unsafe-boundary audit;
no SQL fixture; no storage format; no rerank mode.

Timestamp: `2026-05-14T03:22:54Z`

Isolated one-index-per-table or shared-table surfaces: not applicable.

## Artifacts

- `scoped-unsafe-before.log`
  - Command: `git grep -n unsafe b9a028cd -- src/am/ec_spire/dml_frontdoor src/am/ec_spire/update`
  - Purpose: pre-checkpoint inventory for the scoped audit.

- `scoped-unsafe-counts-before.log`
  - Command: `git grep -n unsafe b9a028cd -- src/am/ec_spire/dml_frontdoor src/am/ec_spire/update | cut -d: -f2 | sort | uniq -c | sort -nr`
  - Purpose: pre-checkpoint count by file.
  - Key result: `update/publish/relation.rs` had 15 unsafe-bearing lines.

- `scoped-unsafe-after.log`
  - Command: `rg -n 'unsafe' src/am/ec_spire/dml_frontdoor src/am/ec_spire/update`
  - Purpose: final scoped inventory after the code checkpoint.

- `scoped-unsafe-counts-after.log`
  - Command: `rg -n 'unsafe' src/am/ec_spire/dml_frontdoor src/am/ec_spire/update | cut -d: -f1 | sort | uniq -c | sort -nr`
  - Purpose: final count by file.
  - Key result: `update/publish/relation.rs` dropped from 15 to 12
    unsafe-bearing lines.

- `ec-spire-unsafe-total-before.log`
  - Command: `git grep -n unsafe b9a028cd -- src/am/ec_spire | wc -l`
  - Purpose: full pre-checkpoint count.
  - Key result: `1430`.

- `ec-spire-unsafe-total-after.log`
  - Command: `rg -n 'unsafe' src/am/ec_spire | wc -l`
  - Purpose: full post-checkpoint count.
  - Key result: `1427`.

- `classification.md`
  - Command: manual review of the scoped inventory and surrounding code.
  - Purpose: classifies remaining scoped unsafe sites as FFI/SPI or
    storage/relation boundary sites.

- `cargo-test-relation-scheduled-input.log`
  - Command: `cargo test -p ecaz relation_scheduled_replacement_execution_input_uses_publish_plan`
  - Purpose: focused compile/behavior check for the touched update relation
    execution area.
  - Key result: `1 passed; 0 failed`.

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Purpose: formatting validation.
  - Key result: passed with the repository's existing stable-rustfmt warnings
    about unstable import options.

- `git-diff-check.log`
  - Command: `git diff --check -- plan/tasks/task30-phase12b-spire-cleanup.md review/31031-spire-unsafe-boundary-audit`
  - Purpose: whitespace validation for the tracker and review packet.
  - Key result: no output.

# Artifact Manifest

Head SHA: `d216f142151d4b989c13e6d7b083e844e3d1d0c5`

Packet/topic: `31030-spire-non-test-unwrap-expect-audit`

Lane / fixture / storage format / rerank mode: static SPIRE non-test
unwrap/expect audit; no SQL fixture; no storage format; no rerank mode.

Timestamp: `2026-05-14T03:08:27Z`

Isolated one-index-per-table or shared-table surfaces: not applicable.

## Artifacts

- `non-test-unwrap-expect-rg.log`
  - Command: `rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire --glob '!**/tests*'`
  - Purpose: pre-checkpoint hit inventory used to select avoidable code fixes.

- `non-test-unwrap-expect-counts-by-file.log`
  - Command: `rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire --glob '!**/tests*' | cut -d: -f1 | sort | uniq -c | sort -nr`
  - Purpose: pre-checkpoint count by file.

- `non-test-unwrap-expect-rg-after.log`
  - Command: `rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire --glob '!**/tests*'`
  - Purpose: final hit inventory after the code checkpoint.
  - Key result: two avoidable hits removed; remaining hits are classified in
    `classification.md`.

- `non-test-unwrap-expect-counts-by-file-after.log`
  - Command: `rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire --glob '!**/tests*' | cut -d: -f1 | sort | uniq -c | sort -nr`
  - Purpose: final count by file.
  - Key result: highest remaining files are `top_graph.rs` and
    `leaf_v2_parts.rs` at 11 hits each; both are fixed-width decoder groups.

- `non-test-unwrap-expect-total-after.log`
  - Command: `rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire --glob '!**/tests*' | wc -l`
  - Purpose: final total count.
  - Key result: `114`.

- `classification.md`
  - Command: manual review of the final inventory and surrounding code.
  - Purpose: records accepted category (a) groups and confirms category (c)
    is zero.

- `cargo-test-local-heap-delivery-gate.log`
  - Command: `cargo test -p ecaz local_heap_delivery_gate_blocks_remote_placements`
  - Purpose: focused validation for the `scan/relation.rs` fallback change.
  - Key result: `1 passed; 0 failed`.

- `cargo-test-bounded-deduped-candidates.log`
  - Command: `cargo test -p ecaz rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates`
  - Purpose: focused validation for the `scan/candidates.rs` fallback change.
  - Key result: `1 passed; 0 failed`.

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Purpose: formatting validation.
  - Key result: passed with the repository's existing stable-rustfmt warnings
    about unstable import options.

- `git-diff-check.log`
  - Command: `git diff --check -- src/am/ec_spire/scan/relation.rs src/am/ec_spire/scan/candidates.rs plan/tasks/task30-phase12b-spire-cleanup.md review/31030-spire-non-test-unwrap-expect-audit`
  - Purpose: final whitespace validation after tracker and packet text.
  - Key result: no output.

- `git-diff-check-pretracker.log`
  - Command: `git diff --check -- src/am/ec_spire/scan/relation.rs src/am/ec_spire/scan/candidates.rs plan/tasks/task30-phase12b-spire-cleanup.md review/31030-spire-non-test-unwrap-expect-audit`
  - Purpose: whitespace check before tracker and packet text were finalized.
  - Key result: no output.

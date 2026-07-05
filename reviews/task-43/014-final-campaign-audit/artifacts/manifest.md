# Task 43 Packet 014 Artifact Manifest

Task bucket: `reviews/task-43/014-final-campaign-audit`

Validation head SHA: `c44d0bccb9647a1b50c14f3b68f7fb857c763126`

Validation scope: pure hardening lanes for Task 43 Miri/cargo-careful safety
campaign. No PostgreSQL table/index fixture, storage format fixture, or rerank
mode applies to this packet.

## Artifacts

| Artifact | Command | Timestamp | Key result |
| --- | --- | --- | --- |
| `cargo-fmt-check.log` | `script -q -c 'cargo fmt --all -- --check' reviews/task-43/014-final-campaign-audit/artifacts/cargo-fmt-check.log` | 2026-05-18 13:53:18-07:00 | Exit 0. Rustfmt emitted stable-channel warnings for unstable import options, but no formatting diff. |
| `git-diff-check.log` | `script -q -c 'git diff --check' reviews/task-43/014-final-campaign-audit/artifacts/git-diff-check.log` | 2026-05-18 14:37:49-07:00 | Exit 0 after final tracker and packet edits. |
| `careful-harness-cargo-test.log` | `script -q -c 'cargo test --manifest-path hardening/careful/Cargo.toml --lib' reviews/task-43/014-final-campaign-audit/artifacts/careful-harness-cargo-test.log` | 2026-05-18 13:53:18-07:00 | `test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. |
| `make-careful.log` | `script -q -c 'make careful' reviews/task-43/014-final-campaign-audit/artifacts/make-careful.log` | 2026-05-18 13:53:18-07:00 | Library tests: 69 passed, 0 failed. Doc-tests: 0 passed, 0 failed. Exit 0. |
| `make-miri-expanded.log` | `script -q -c 'make miri-expanded' reviews/task-43/014-final-campaign-audit/artifacts/make-miri-expanded.log` | 2026-05-18 13:53:54-07:00 | `test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 1710 filtered out`. Exit 0. |
| `make-miri-tree.log` | `script -q -c 'make miri-tree' reviews/task-43/014-final-campaign-audit/artifacts/make-miri-tree.log` | 2026-05-18 13:55:36-07:00 | `test result: ok. 87 passed; 0 failed; 0 ignored; 0 measured; 1710 filtered out`. Exit 0. |
| `make-miri-many-seeds.log` | `script -q -c 'make miri-many-seeds' reviews/task-43/014-final-campaign-audit/artifacts/make-miri-many-seeds.log` | 2026-05-18 13:57:26-07:00 | Exit 0. The sanitized log contains 128 distinct `Trying seed:` entries, covering seeds 0 through 127. Output is interleaved by concurrent seed jobs; completed summaries report 87 passed, 0 failed. |

## Reviewer Feedback Coverage

Packet 014 uses this manifest plus `request.md` and the updated campaign
tracker to close G8:

- Packet 005 critical feedback, "many-seeds is structurally empty": closed by
  packet 007 threaded common-parallel coverage and the aggregate packet 014
  many-seeds run.
- Packet 005 breadth gaps: closed by packets 008-011.
- Packet 005 mutation gap: closed by packet 013.
- Packet 012 SPIRE careful gap: retained as explicit G6 blockers; not hidden
  by the final audit.
- Packet 013 mutation-depth caveat: recorded as sensitivity evidence, not an
  exhaustive proof claim.

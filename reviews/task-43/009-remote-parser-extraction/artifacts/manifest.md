# Task 43 Packet 009 Artifact Manifest

Head SHA: `19834f9ee6b6c7986dcf5ecd5eefc451bff7ab64`

Task bucket: `reviews/task-43/009-remote-parser-extraction`

Timestamp: `2026-05-18T13:05:29-07:00`

Surface: pure field-level SPIRE remote typed tuple payload validation. No
PostgreSQL table, index, storage format fixture, or rerank runtime table was
created; isolated/shared table distinction is not applicable.

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `miri-remote-typed-payload-fields.log` | `cargo +nightly miri test --lib miri_remote_typed_payload_fields` | 2 passed; 0 failed; 1794 filtered; valid Row-independent typed payload plus adversarial shape validation. |
| `miri-remote-payload-caps.log` | `cargo +nightly miri test --lib miri_remote_payload_caps_reject_oversized_rows_and_batches` | 1 passed; 0 failed; 1795 filtered; existing cap path still passes after extraction. |
| `cargo-fmt-check.log` | `cargo fmt --all -- --check` | Exit 0; rustfmt emitted existing unstable-option warnings. |
| `git-diff-check.log` | `git diff --check` | Exit 0; no whitespace errors. |

## Coverage Notes

- The libpq `postgres::Row` decoder now delegates typed tuple payload
  validation to a pure field-level decoder.
- The new pure decoder validates width consistency, positive attnums, type OID
  parsing and nonzero type OIDs, collation OID parsing, hex byte count, invalid
  hex, row byte caps, tuple transport, transport status, and per-column payload
  format.
- A focused normal `cargo test --lib miri_remote_typed_payload_fields` build
  completed compilation but could not execute the pgrx-linked test binary
  outside PostgreSQL due `undefined symbol: LockBuffer`; Miri is the validation
  lane for this pure helper in this packet.

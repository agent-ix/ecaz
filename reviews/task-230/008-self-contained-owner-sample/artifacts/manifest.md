# Task 230 packet 008 artifact manifest

- Head SHA: `cb6666410ea75faaf333ca8d017a1d6f044dbbe4`
- Task bucket: `reviews/task-230/008-self-contained-owner-sample/`
- Timestamp: 2026-08-29T08:33:18-07:00
- Lane / fixture / storage format / rerank mode: local Intel PG18; real 10k
  one-step hot/cold validation smoke; descriptor V4 / Graph V2; no rerank
- Results state: none. Static validation, suite audit, and dry-run only.

## Failure receipt

`failed-rowheap-owner-sample-summary.log` records the accepted-head release
preflight and pre-measurement malformed sample. No decision row is admitted.

## Static validation

- `cargo-fmt-check.log`: exit 0.
- `cargo-test-focused.log`: exit 0; 1 passed, 0 failed, 552 filtered out.
- `cargo-clippy-cli.log`: exit 0; 77 binary / 78 test warnings.

## Smoke preregistration

- Config: `tiny-hotcold-smoke.json`.
- SHA-256: `7abf830f498f96ac0211714235635cfd760b10d36e01bcddf1aa7ec97394b9ee`.
- Audit: `smoke-audit.log`, exit 0, one step.
- Dry run: `smoke-dry-run.log` and `smoke-dry-run-manifest.json`, one selected
  step; manifest SHA-256
  `e8b8295a0bc7f48f7a0e71edffc9fe4dacfbe07348aa2feee2de11c4cdf2409f`.
- Dry-run runner head: `cb6666410ea75faaf333ca8d017a1d6f044dbbe4`.
- Run directory: `/home/peter/.ecaz/clusters/task230-packet008-hotcold-smoke`;
  removed after durable capture.
- Isolation: fresh one-index-per-table hot/cold fixture; no reuse and no debug
  override.

# Manifest: AWS Row Segment Snapshot Signature Update

Task bucket: `reviews/task-85/018-aws-row-segment-signature-update/`
Head SHA: `e07b4be5ee28ae74d85a7b4a601340307f0bb413`
Timestamp: 2026-06-07

## Artifacts

- `suite-aws-row-segment-signature-update.json`
  - Suite config for the retained AWS signature update.
  - Uses `ecaz bench suite` raw steps; no ad hoc benchmark sweeper.

- `artifacts/suite-audit.log`
  - Command: `target/debug/ecaz bench suite audit --config reviews/task-85/018-aws-row-segment-signature-update/suite-aws-row-segment-signature-update.json`
  - Result: audit passed, 2 steps.

- `artifacts/cloud-install-append-only-signature.log`
  - Command: `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-85-spire-product-scale-pareto --skip-extension-recreate --timeout 3600 --log-file reviews/task-85/018-aws-row-segment-signature-update/artifacts/cloud-install-append-only-signature.log`
  - Result: installed current branch on retained AWS DB host.
  - Retained data/index: yes, `--skip-extension-recreate`.

- `artifacts/update-leaf-candidate-snapshot-signature.sql`
  - Manual SQL patch used by the suite raw step.
  - Drops the old extension-owned function, recreates it with append-only
    row-segment columns, and adds it back to the extension.

- `artifacts/aws-row-segment-signature-update/apply-leaf-candidate-snapshot-signature.log`
  - Result:
    - `BEGIN`
    - `ALTER EXTENSION`
    - `DROP FUNCTION`
    - `CREATE FUNCTION`
    - `ALTER EXTENSION`
    - `COMMIT`

- `artifacts/aws-row-segment-signature-update/verify-leaf-candidate-snapshot-segments.log`
  - Command: suite raw verification query against
    `aws_spire_1m_rabitq_t80_block16_tg256_idx`.
  - Key result: the appended `leaf_row_segment_read_count` and
    `leaf_row_segment_read_bytes` columns are selectable from SQL.

- `artifacts/cloud-status-before-signature-update.log`
  - Profile state before resume: paused.

- `artifacts/cloud-status-after-resume.log`
  - Profile state after resume: running.

- `artifacts/cloud-bench-signature-update.log`
  - Command: `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/018-aws-row-segment-signature-update/suite-aws-row-segment-signature-update.json --suite task85-aws-row-segment-signature-update --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/018-aws-row-segment-signature-update/artifacts/cloud-bench-signature-update.log`
  - Result: synced suite artifacts from S3.

Storage surface: retained AWS `1m`, isolated one-index surface
`aws_spire_1m_rabitq_t80_block16_tg256_idx`.

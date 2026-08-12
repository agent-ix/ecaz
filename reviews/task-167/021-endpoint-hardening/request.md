# Task 167 endpoint-hardening review request

Please independently review code checkpoint `48195c196`.

This checkpoint closes the FR-083 AC-5 security gap found during the Task 167
completion audit. The three physical DML endpoints
(`ec_distann_apply_physical_insert`, `ec_distann_apply_physical_backlink`, and
`ec_distann_apply_physical_tombstone`) now have `SECURITY DEFINER`, the fixed
`pg_catalog, @extschema@, pg_temp` search path, and `PUBLIC EXECUTE` revoked in
the extension SQL boundary. A focused PG18 pg_test exercises all three calls
as an unprivileged role and passes.

The acceptance matrix was updated to point FR-083 AC-4 and AC-6 through AC-9
at Task 167 packet 020, while AC-5 now records this endpoint-family evidence.
The packet 020 runtime review remains the primary behavioral evidence.

Evidence is in [`artifacts/schema-hardening.log`](artifacts/schema-hardening.log)
and [`artifacts/manifest.md`](artifacts/manifest.md). Disposition requested:
independent review of this code/security checkpoint. This packet remains
review-open pending an outside reviewer verdict.

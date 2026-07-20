# Verdict: retain the bounded remote transport

Retain the common interrupt-aware await wrapper and the two Userset timeout
controls introduced by `ceb15f73a`.

The implementation gives the transport two independent bounds: tokio-postgres
connection establishment uses `ec_distann.remote_connect_timeout_ms`, while a
live session applies `ec_distann.remote_statement_timeout_ms` server-side. The
client await adds a bounded five-second grace to the statement budget so that
PostgreSQL's remote cancellation error remains the primary diagnostic, with the
client deadline as a fallback.

Every foreground lifecycle, physical handoff, distributed scan, and identity
setup query now enters the common wrapper. PostgreSQL interrupts are checked on
both sides of each await. A pooled session records its applied statement budget
and refreshes it when the backend's Userset value changes.

The focused unit suite, live pooled-session timeout test, and existing
three-owner handoff regression all pass at the implementation SHA. Existing
SQLSTATE scan classification and conninfo redaction are retained.

This packet closes packet 025's transport P2 for outside review only. It does
not close Task 179 or supply the remaining real three-instance fault-window,
head-cap sensitivity, epoch-retry, or Task 172 benchmark/review evidence.

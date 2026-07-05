# Artifact Manifest

- packet: `734-c1-spire-customscan-coordinator-drop-index-lock`
- head SHA: `20c8837f`
- date: 2026-05-14
- measurement artifacts: none

This packet has no measurement artifacts. Validation is compile/static only;
the focused `cargo pgrx test pg18` runtime attempt failed at local loader
startup with `undefined symbol: pg_re_throw` before test execution.

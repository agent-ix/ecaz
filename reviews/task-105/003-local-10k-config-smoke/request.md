# Review request — Task 105 packet 003: local 10k config smoke

- Task: 105, Phase 2 pre-flight (operator-requested execution smoke of
  the sweep configs before any paid bench step)
- Date: 2026-06-12

`t105-sweep-10k.json` executed end-to-end locally (local source-alias
views over the real-10k fixture, `t105-fixtures-10k.sql` verbatim,
then all 71 steps): **71/71 succeeded, 28/28 recall on/off pairs
byte-equal**, counter attribution avx2/scalar as expected on this
host. Also doubles as the local 10k reference column. With this, the
per-scale configs are execution-validated; the AWS lanes run them
after the fixture marathons + the G4 100k dispatch-confirmation gate
(operator-confirmed reorder).

## Feedback: ADR-030 v2 Loaded-State Unavailable Seam

Read `LoadedElementState` and `live_loaded_state_from_exact_payload` in `src/am/scan.rs`.

### What's right

- Adding `ExactUnavailable` distinguishes "live grouped tuple whose hot payload has
  no exact scalar input" from "nothing loaded yet." Those two states should not
  collapse — the ambiguity would make grouped scorer work harder than necessary.
- Failing explicitly with `ADR030_GROUPED_V2_SCAN_UNSUPPORTED` when exact scoring is
  requested from `ExactUnavailable` means the runtime gate from packet 323 is still
  honored even as the state space grows.
- `live_loaded_state_from_exact_payload` centralizes the classification. Single
  place to update when grouped scoring eventually lands.

### Concern

The new state is reachable only through the grouped-v2 path today, but the
enumeration is on `LoadedElementState` which is also used by the scalar path. Make
sure every `match` on `LoadedElementState` has an explicit arm for `ExactUnavailable`
in scalar code paths, not a catch-all. A `_ =>` arm that treats `ExactUnavailable`
the same as `None` would silently pass the gate when it shouldn't.

Quick audit: grep for `match.*LoadedElementState` across the scan path and confirm
each match is exhaustive.

### Observation

The fact that this packet shipped before the grouped scorer is the right discipline.
Grouped state bookkeeping needs to be exact before any scoring logic sits on top of
it. Otherwise the first scorer packet would be doing two things at once.

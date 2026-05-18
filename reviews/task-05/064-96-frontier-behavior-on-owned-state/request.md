# Request: Frontier Behavior On Owned State

Commit: `5a5d620`

Summary:
- Moves visible-frontier behavior directly onto `VisibleCandidateFrontierState` in `src/am/scan.rs`.
- Removes the extra read/write wrapper structs that existed only to forward the same container operations.
- Keeps the owned frontier state and its public behavior the same, but simplifies where the behavior actually lives.

Files:
- `src/am/scan.rs`

Why this matters:
- The previous slice introduced an owned visible-frontier state type but still split container behavior across helper wrapper structs.
- That left the container boundary conceptually cleaner than before, but mechanically more layered than necessary.
- This slice makes the owned frontier state the direct home of length/iteration/slot/containment/best/remove behavior, which is a better base for the next move into stronger container ownership or shared search integration.

Review focus:
- Whether moving the behavior directly onto the owned state is the right simplification after the owned-frontier introduction
- Whether any lifecycle or empty-frontier edge cases changed unintentionally due to the new empty-state helper path
- Whether the next step should now be richer owned-frontier behavior in `scan.rs` or handing more of that authority to `search.rs`

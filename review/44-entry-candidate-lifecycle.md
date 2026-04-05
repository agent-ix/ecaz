# Review Request: Entry Candidate Lifecycle

Scope:
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- Added explicit lifecycle coverage for the scan-local entry candidate seeded during `amrescan`.
- The new debug helper verifies that the entry candidate stays valid and unchanged after partial bootstrap linear-scan progress, then clears only once the scan fully exhausts.
- This slice does not add graph traversal or candidate-queue advancement; it only sharpens the lifecycle contract of the existing seeded entry candidate.

Review focus:
- Whether keeping the seeded entry candidate stable during partial scan progress is the right groundwork for a future frontier
- Whether clearing only on exhaustion or next rescan is the right lifecycle rule for this pre-traversal state
- Whether the coverage now captures the intended semantics precisely enough before candidate queues or visited state arrive

Questions to answer:
- Should the entry candidate remain stable during bootstrap scan progress, or is there a stronger reason to consume or mutate it earlier?
- Is clearing on exhaustion the right behavior for this one-slot seed state?
- Are there any missing lifecycle edges around rescan-after-partial-progress that should be covered before the next candidate-state slice?

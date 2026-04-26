# Feedback: 644 Concurrent DSM Layout Reattach

## Verdict: Accept

Reconstructing layout offsets from the initialized DSM header is the correct
worker-attach contract. Workers receive only a base pointer via `shm_toc`; they
cannot access the leader's `EcHnswConcurrentDsmPreassemblyPlan`. The shared
header carries enough durable metadata for section pointer reconstruction.

Rejecting non-empty graphs without an entry node is the right invariant at
attach time — the entry node is set during initialization and its absence
indicates a corrupted or uninitialized image.

## No Issues

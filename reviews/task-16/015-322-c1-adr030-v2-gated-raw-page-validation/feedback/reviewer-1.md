## Feedback: ADR-030 v2 Gated Raw Page Validation

### What's right

- Validating raw pages at the persistence boundary means build-time bugs do not
  silently produce corrupt indexes. The fact that this validation is gated on v2
  means it does not impose overhead on v1 builds, while still catching v2-specific
  shape errors.
- Running validation behind the same gate as the build itself keeps experimental-path
  overhead out of production.

### What I'd like to see confirmed

1. **Does validation run on every v2 build, or only under an additional debug env
   var?** For an experimental format, I'd want it on every v2 build. The cost on a
   1024-row training sample is negligible and the value of catching a tuple-packing
   bug at build time is high.

2. **What exactly does "raw page validation" check?** If it re-reads each written
   tuple via `GraphTupleRef` and compares fields to the pre-write staged payload,
   that's a strong build-time invariant. If it just checks tag bytes are correct,
   that's weaker. Worth naming explicitly in this packet's outcome so the
   invariant is quoted by future packets that depend on it.

3. **Failure mode.** If validation fails on a raw page, does the build abort with a
   loud error, or does it log and continue? Abort is the right default for an
   experimental lane — silent validation failures in experimental code lead to
   confusing bug reports.

### Observation

The fact that validation is a separate packet from the build itself is correct
incremental discipline — it lets the validation be added and tested without being
tangled up in write-path logic.

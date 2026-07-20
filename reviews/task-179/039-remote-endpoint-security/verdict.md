# Verdict: retain the class-wide remote endpoint closure

Retain the class-wide endpoint privilege policy and the write isolation guard.

The two M2 signatures identified by the reviewer are no longer reachable
through PUBLIC EXECUTE. The remediation also closes the sibling remote write
endpoint, and the regression test audits the complete installed overload class:
eight functions, all SECURITY DEFINER, all fixed to the extension-safe search
path, none executable by a fresh unprivileged role. This converts the repeated
per-name repair pattern into a class invariant.

The remote write endpoint now checks READ COMMITTED before opening its index.
An invalid-OID Repeatable Read probe proves the ordering, while the existing
tombstone test proves normal writes remain functional.

This packet closes packet 033 P1-1 and P2-1 for outside review only. It does not
close P1-2's Cancelled-decision orphan-reclaim mismatch or the carried recovery,
abandonment, and disposition findings.

# Task 215 review request: production wiring

This packet records the normal-release candidate wiring after the release
contract in packet 001. Packet 003's complete A/B subsequently rejected this
candidate; the shipped defaults are restored by its rollback checkpoint.

The checkpoint changes only the production defaults to BW64/H8 and keeps the
Task 205 candidate heap GUC at L=32. Existing benchmark-control literals for
BW4/H100 remain in the suite/fixture driver so rollback and the release A/B
are explicit. No attribution-only GUC, feature selector, head format, or
index lifecycle behavior changed.

PG18 compile and normal release installation passed; see the packet-local
validation artifacts. Packet 003 contains the decision-bearing release A/B.

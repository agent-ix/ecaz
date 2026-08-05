# Task 207 correction notes

The physical activation marker is now persisted with the active head state as
`head_construction` and surfaced by `ec_distann_active_head_construction`.
This closes the prior marker gap in the implementation; the marker is not
claimed from a digest comparison alone.

The old release seed claim is withdrawn. Because the seed-control benchmark
GUCs are absent from the uninstrumented build, BW128 derives 256 effective
seeds at width 256. Packet 005 is superseded by this statement.

The owner-oracle table is withdrawn from the membership decision. Its scan is
head-independent and used top-k 32, so it cannot support the pre-registered
head-membership or overlap@k claim. The required head-sample IDs were not
captured in the existing run artifacts; no replacement numbers are inferred.

The phase-2 search result remains a diagnostic persisted-head/Vamana A/B.
The production state remains `training_landmarks_exact` when explicitly
selected, with no default change or promotion in this packet.

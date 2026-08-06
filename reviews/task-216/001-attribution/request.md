# Task 216 review request: attribution pre-registration

This packet pre-registers the owner-side expansion/serialization attribution
lane. It does not implement a candidate and does not combine with the Task 215
BW64/H8 release A/B.

The entry evidence is the Task 205 bounded-L disposition in
`reviews/task-205/005-attribution-closeout/` and the accepted Task 206 physical
attribution packets. Those establish that response-byte reduction is not, by
itself, an end-to-end latency result: owner compute, response assembly,
encoding, and materialization remain separate hypotheses.

The first measurement will use a fresh 100k conforming PG18 sharded-owner
generation and a checked-in `ecaz bench suite` configuration. At most three
candidate families will be named from the measured dominant stage; at most one
will advance to isolated A/B. No source change is requested in this packet.

The Task 215 release matrix is still active. Its results will not be mixed
with this attribution lane; any wide-beam diagnostic view will be labeled
secondary and non-decision evidence.

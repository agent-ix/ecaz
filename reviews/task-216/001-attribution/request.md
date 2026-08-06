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
generation and the checked-in
`artifacts/task216-100k-attribution.json` `ecaz bench suite` configuration.
The full-metrics run uses a release build with
`distann-head-attribution-benchmark` installed before the suite and
`skip_install=true`, so the suite cannot replace the diagnostic extension.
At most three candidate families are named from the measured dominant stage;
at most one will advance to isolated A/B. No source change is requested in
this packet.

Pre-registered candidates, before the fresh run:

- `MAT-15`: packed payload buffers with offsets and a null bitmap; predicted
  movement is in owner endpoint work/response materialization.
- `MAT-21`: typed/binary locators instead of textual locator formatting;
  predicted movement is in owner locator/serialization work.
- `TRAV-05`: packed expansion responses instead of row/array structures;
  predicted movement is in owner response assembly/encoding.

The fresh diagnostic may reject candidates whose named stage is not dominant
or whose stage movement cannot plausibly affect end-to-end latency. No
candidate is advanced by this pre-registration.

The Task 215 release matrix is still active. Its results will not be mixed
with this attribution lane; any wide-beam diagnostic view will be labeled
secondary and non-decision evidence.

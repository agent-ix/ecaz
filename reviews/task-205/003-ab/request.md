# Task 205 review request: A/B preregistration

This packet preregisters the required Algorithm 1 A/B matrix at fixed BW=4 and
H=100 across 10k/50k/100k. The owner-traversal path is the decision-bearing
control; the coordinator traversal-replica arm is deliberately excluded under
NFR-022. The candidate is the same distributed owner path built with the
pushdown implementation.

The suite config is the only matrix driver. It records separate external run
directories for each arm/scale and requests recall, latency, storage, request/
response bytes, and per-round transport wait.

Pre-registration passes the shape/control audit, but execution is blocked by the
missing staged corpus on this host. See `artifacts/benchmark-preflight.log` and
the manifest’s NFR-021 verdict. This packet records no invented results and
remains open until the matrix is run on a host with the staged inputs.

# Task 193 candidate audit manifest

- `src/am/ec_distann/remote_endpoint.rs:build_payload_sql` uses one
  `unnest($1::text[]) WITH ORDINALITY` query per owner window.
- `src/am/ec_distann/custom_scan.rs:fetch_remote_payloads` groups IDs by owner
  and sends one request per owner.
- Decision gate: do not implement duplicate MAT-23/MAT-24 behavior; measure
  MAT-19 separately or record STOP.

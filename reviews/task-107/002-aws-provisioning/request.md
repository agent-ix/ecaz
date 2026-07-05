# Task 107 packet 002 - AWS topology provisioning

Status: ready for review. Coder lane.

This packet records the AWS topology used for Task 107 SPIRE distributed
benchmark work. It provisioned one coordinator plus two remote PostgreSQL
nodes in `us-west-2a`, installed the extension/CLI, configured coordinator
remote conninfo, and mounted four coordinator store volumes as tablespaces.

The durable evidence is in `artifacts/manifest.md` and packet-local logs. I
pruned generated source tarballs, local source trees, Terraform state, and TLS
private material before packaging the request.

This packet is infrastructure evidence only. Benchmark results are in
`reviews/task-107/003-aws-benchmarks/`.

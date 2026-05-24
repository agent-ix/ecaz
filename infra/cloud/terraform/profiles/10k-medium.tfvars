profile              = "10k-medium"
db_instance_type     = "m8g.2xlarge"
db_volume_gb         = 100
loader_instance_type = "c8g.medium"
ecaz_git_ref         = "main"
region               = "us-west-2"
az                   = "us-west-2a"
enable_eice_ssh      = true
ssh_key_name         = "ecaz-bench"

# Notes on this profile vs `10k.tfvars`:
# - m8g.2xlarge: 8 vCPU / 32 GB (vs m8g.large: 2 vCPU / 8 GB)
# - Recommended default for 1M sidecar bench cycles. The sidecar
#   measurement harness fetches the real corpus and materializes sidecar
#   payloads in-process; 16 GB is not enough for the 990k x 1536 fixture.
#   32 GB leaves headroom without configuring swap, and the extra cores
#   keep the SSM agent responsive during long benchmark runs.
# - Uses the retained 100 GB data volume required by the preserved 1M
#   IVF/RaBitQ benchmark snapshot. The instance type changes without
#   reloading or rebuilding the preserved corpus/index.
# - Roughly 4x the per-hour cost of `10k.tfvars` (~$0.65/hr vs
#   ~$0.16/hr), but avoids repeated failed restore/install cycles for
#   memory-bound 1M sidecar gates.

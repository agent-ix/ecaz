profile              = "10k-medium"
db_instance_type     = "m8g.xlarge"
db_volume_gb         = 50
loader_instance_type = "c8g.medium"
ecaz_git_ref         = "main"
region               = "us-west-2"
az                   = "us-west-2a"
enable_eice_ssh      = true
ssh_key_name         = "ecaz-bench"

# Notes on this profile vs `10k.tfvars`:
# - m8g.xlarge: 4 vCPU / 16 GB (vs m8g.large: 2 vCPU / 8 GB)
# - Recommended default for bench cycles. The [profile.bench] build
#   (lto=fat, codegen-units=1, debug=true) needs > 8 GB to compile
#   without OOM; 16 GB gives headroom without configuring swap, and
#   the extra cores keep the SSM agent responsive during criterion +
#   perf-stat runs without taskset pinning.
# - Same EBS volume size as 10k. Snapshot/restore is interchangeable
#   between the two profiles -- the volume is identical, only the
#   instance type changes.
# - Roughly 2x the per-hour cost of `10k.tfvars` (~$0.32/hr vs
#   ~$0.16/hr), but cycle wall-time is roughly half, so total spend
#   is comparable.

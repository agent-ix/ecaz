profile              = "1b"
db_instance_type     = "r7g.8xlarge"
db_volume_gb         = 1024
db_volume_iops       = 12000
db_volume_throughput = 500
loader_instance_type = "c7g.4xlarge"
ecaz_git_ref         = "main"

# Sized for compressed indexes only: RaBitQ 1-bit (~200 GB) + PQ-fastscan
# rerank (~256 GB) + IVF/SPIRE metadata (~15 GB) + WAL/FS overhead (~50 GB).
# Do NOT use this profile to bench raw fp32 at 1B — wrong code path, 6 TB
# working set, ~10x the cost.

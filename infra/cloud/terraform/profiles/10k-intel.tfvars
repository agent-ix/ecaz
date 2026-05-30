profile               = "10k-intel"
db_instance_type      = "m7i.2xlarge"
db_volume_gb          = 100
loader_instance_type  = "c7i.large"
instance_architecture = "x86_64"
ecaz_git_ref          = "main"
region                = "us-west-2"
az                    = "us-west-2a"
enable_eice_ssh       = true
ssh_key_name          = "ecaz-bench"

# Intel x86_64 lane for Task 67 RaBitQ AVX-512 / AVX2 validation.
# m7i.2xlarge: 8 vCPU / 32 GB on Intel Sapphire Rapids, matching the
# memory shape of `10k-medium` while enabling x86 SIMD measurement.
# c7i.large is only a lightweight loader/utility host for this profile;
# Slice J suite execution runs on the DB host through `ecaz cloud bench`.

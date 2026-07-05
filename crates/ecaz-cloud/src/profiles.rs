//! Profile registry — single source of truth for instance/EBS sizing and
//! coarse cost projections used by `ecaz cloud status` and the
//! `--confirm-cost` gate (NFR-010).
//!
//! Costs are USD list price for `us-east-1` as of the spec checkpoint and
//! are intentionally rough — they exist to make a forgotten profile
//! visible, not to bill anyone.

use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum Profile {
    P10k,
    /// `10k-medium` = bigger 10k bench host (m8g.2xlarge / 8 vCPU /
    /// 32 GB). Recommended default for 1M sidecar bench cycles per
    /// `infra/cloud/terraform/profiles/10k-medium.tfvars`: the sidecar
    /// harness materializes the real corpus and sidecar payloads
    /// in-process, and 16 GB is not enough for the 990k x 1536 fixture.
    P10kMedium,
    /// `10k-intel` = x86_64 Intel lane for Task 67 RaBitQ AVX-512 /
    /// AVX2 validation. The selected instance families are Intel
    /// Sapphire Rapids; AVX feature availability is still verified at
    /// runtime by the benchmark packet before accepting results.
    P10kIntel,
    Dev,
    P1m,
    P10m,
    P100m,
    /// 1B vectors via RaBitQ / IVF / SPIRE (compressed indexes only). Sizing
    /// assumes ~500 GB total used (RaBitQ 1-bit + PQ-fastscan rerank +
    /// posting metadata), not raw fp32. fp32-raw at 1B is intentionally
    /// not a profile — it would be benchmarking the wrong code path.
    P1b,
}

impl Profile {
    pub fn parse(s: &str) -> Option<Profile> {
        match s {
            "10k" => Some(Profile::P10k),
            "10k-medium" => Some(Profile::P10kMedium),
            "10k-intel" => Some(Profile::P10kIntel),
            "dev" => Some(Profile::Dev),
            "1m" => Some(Profile::P1m),
            "10m" => Some(Profile::P10m),
            "100m" => Some(Profile::P100m),
            "1b" => Some(Profile::P1b),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Profile::P10k => "10k",
            Profile::P10kMedium => "10k-medium",
            Profile::P10kIntel => "10k-intel",
            Profile::Dev => "dev",
            Profile::P1m => "1m",
            Profile::P10m => "10m",
            Profile::P100m => "100m",
            Profile::P1b => "1b",
        }
    }

    pub fn db_instance_type(self) -> &'static str {
        match self {
            Profile::P10k | Profile::Dev => "m7g.large",
            Profile::P10kMedium => "m8g.2xlarge",
            Profile::P10kIntel => "m7i.2xlarge",
            Profile::P1m => "m7g.xlarge",
            Profile::P10m => "m7g.4xlarge",
            Profile::P100m => "r7g.4xlarge",
            Profile::P1b => "r7g.8xlarge",
        }
    }

    pub fn loader_instance_type(self) -> &'static str {
        match self {
            Profile::P10k | Profile::Dev => "c7g.large",
            Profile::P10kMedium => "c8g.medium",
            Profile::P10kIntel => "c7i.large",
            Profile::P1m | Profile::P10m => "c7g.2xlarge",
            Profile::P100m | Profile::P1b => "c7g.4xlarge",
        }
    }

    pub fn ebs_gb(self) -> u64 {
        match self {
            Profile::P10k => 20,
            Profile::Dev => 50,
            Profile::P10kMedium => 100,
            Profile::P10kIntel => 100,
            Profile::P1m => 100,
            Profile::P10m => 500,
            Profile::P100m => 2048,
            Profile::P1b => 1024,
        }
    }

    /// Estimated $/hr while the stack is running. Sum of DB + loader compute.
    pub fn estimated_hourly_usd(self) -> f64 {
        // Rough on-demand list price, us-east-1.
        let db = match self {
            Profile::P10k | Profile::Dev => 0.0816, // m7g.large
            Profile::P10kMedium => 0.3264,          // m8g.2xlarge
            Profile::P10kIntel => 0.4032,           // m7i.2xlarge
            Profile::P1m => 0.1632,                 // m7g.xlarge
            Profile::P10m => 0.6528,                // m7g.4xlarge
            Profile::P100m => 0.8568,               // r7g.4xlarge
            Profile::P1b => 1.7136,                 // r7g.8xlarge
        };
        let loader = match self {
            Profile::P10k | Profile::Dev => 0.0725, // c7g.large
            Profile::P10kMedium => 0.0181,          // c8g.medium
            Profile::P10kIntel => 0.0893,           // c7i.large
            Profile::P1m | Profile::P10m => 0.29,   // c7g.2xlarge
            Profile::P100m | Profile::P1b => 0.58,  // c7g.4xlarge
        };
        db + loader
    }

    /// Estimated $/mo of retained storage (EBS data volume only).
    /// gp3 is ~$0.08/GB-month in us-east-1.
    pub fn estimated_monthly_storage_usd(self) -> f64 {
        self.ebs_gb() as f64 * 0.08
    }

    /// Projected $/day if left running unattended.
    pub fn estimated_daily_usd(self) -> f64 {
        self.estimated_hourly_usd() * 24.0
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug)]
pub struct UnknownProfile(pub String);

impl std::fmt::Display for UnknownProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unknown profile {:?}; expected one of 10k/10k-medium/10k-intel/dev/1m/10m/100m/1b",
            self.0
        )
    }
}

impl std::error::Error for UnknownProfile {}

impl std::str::FromStr for Profile {
    type Err = UnknownProfile;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Profile::parse(s).ok_or_else(|| UnknownProfile(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::Profile;

    #[test]
    fn parses_named_profiles() {
        let cases = [
            ("10k", Profile::P10k),
            ("10k-medium", Profile::P10kMedium),
            ("10k-intel", Profile::P10kIntel),
            ("dev", Profile::Dev),
            ("1m", Profile::P1m),
            ("10m", Profile::P10m),
            ("100m", Profile::P100m),
            ("1b", Profile::P1b),
        ];

        for (name, profile) in cases {
            assert_eq!(Profile::parse(name), Some(profile));
            assert_eq!(profile.name(), name);
        }
    }

    #[test]
    fn intel_profile_uses_x86_hosts() {
        let profile = Profile::P10kIntel;

        assert_eq!(profile.db_instance_type(), "m7i.2xlarge");
        assert_eq!(profile.loader_instance_type(), "c7i.large");
        assert_eq!(profile.ebs_gb(), 100);
        assert!(profile.estimated_hourly_usd() > Profile::P10kMedium.estimated_hourly_usd());
    }
}

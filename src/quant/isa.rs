#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Isa {
    Scalar,
    Neon,
    Sve,
    Sve2,
    Avx2,
}

impl Isa {
    pub(crate) const ALL: [Self; 5] = [Self::Scalar, Self::Neon, Self::Sve, Self::Sve2, Self::Avx2];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Neon => "neon",
            Self::Sve => "sve",
            Self::Sve2 => "sve2",
            Self::Avx2 => "avx2",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Aarch64Features {
    pub(crate) neon: bool,
    pub(crate) sve: bool,
    pub(crate) sve2: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct X86Features {
    pub(crate) avx2: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct HostIsaFeatures {
    pub(crate) aarch64: Aarch64Features,
    pub(crate) x86: X86Features,
}

impl HostIsaFeatures {
    #[allow(dead_code)]
    pub(crate) fn detect() -> Self {
        Self {
            aarch64: detect_aarch64_features(),
            x86: detect_x86_features(),
        }
    }
}

pub(crate) fn select_highest_isa(features: HostIsaFeatures) -> Isa {
    if features.x86.avx2 {
        return Isa::Avx2;
    }
    if features.aarch64.sve2 {
        return Isa::Sve2;
    }
    if features.aarch64.sve {
        return Isa::Sve;
    }
    if features.aarch64.neon {
        return Isa::Neon;
    }
    Isa::Scalar
}

#[allow(dead_code)]
pub(crate) fn current_isa() -> Isa {
    select_highest_isa(HostIsaFeatures::detect())
}

#[cfg(target_arch = "aarch64")]
#[allow(dead_code)]
fn detect_aarch64_features() -> Aarch64Features {
    Aarch64Features {
        neon: std::arch::is_aarch64_feature_detected!("neon"),
        sve: std::arch::is_aarch64_feature_detected!("sve"),
        sve2: std::arch::is_aarch64_feature_detected!("sve2"),
    }
}

#[cfg(not(target_arch = "aarch64"))]
#[allow(dead_code)]
fn detect_aarch64_features() -> Aarch64Features {
    Aarch64Features::default()
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[allow(dead_code)]
fn detect_x86_features() -> X86Features {
    X86Features {
        avx2: std::arch::is_x86_feature_detected!("avx2"),
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[allow(dead_code)]
fn detect_x86_features() -> X86Features {
    X86Features::default()
}

#[cfg(test)]
mod tests {
    use super::{select_highest_isa, Aarch64Features, HostIsaFeatures, Isa, X86Features};

    #[test]
    fn select_highest_isa_prefers_graviton4_sve2() {
        let features = HostIsaFeatures {
            aarch64: Aarch64Features {
                neon: true,
                sve: true,
                sve2: true,
            },
            x86: X86Features::default(),
        };

        assert_eq!(select_highest_isa(features), Isa::Sve2);
    }

    #[test]
    fn select_highest_isa_distinguishes_base_sve() {
        let features = HostIsaFeatures {
            aarch64: Aarch64Features {
                neon: true,
                sve: true,
                sve2: false,
            },
            x86: X86Features::default(),
        };

        assert_eq!(select_highest_isa(features), Isa::Sve);
    }

    #[test]
    fn select_highest_isa_falls_back_to_neon_or_scalar() {
        let neon = HostIsaFeatures {
            aarch64: Aarch64Features {
                neon: true,
                sve: false,
                sve2: false,
            },
            x86: X86Features::default(),
        };
        let scalar = HostIsaFeatures::default();

        assert_eq!(select_highest_isa(neon), Isa::Neon);
        assert_eq!(select_highest_isa(scalar), Isa::Scalar);
    }

    #[test]
    fn select_highest_isa_prefers_x86_avx2_on_x86_features() {
        let features = HostIsaFeatures {
            aarch64: Aarch64Features {
                neon: true,
                sve: true,
                sve2: true,
            },
            x86: X86Features { avx2: true },
        };

        assert_eq!(select_highest_isa(features), Isa::Avx2);
    }
}

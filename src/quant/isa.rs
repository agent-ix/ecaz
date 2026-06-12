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

/// Dispatch-preference rank used by the ISA cap. The cap limits how high
/// the selector may climb; it never fakes an unavailable ISA. Capping
/// below the host's only SIMD tier therefore lands on `Scalar` (e.g.
/// `neon` on an x86 host).
fn cap_rank(isa: Isa) -> u8 {
    match isa {
        Isa::Scalar => 0,
        Isa::Neon => 1,
        Isa::Sve => 2,
        Isa::Sve2 => 3,
        Isa::Avx2 => 4,
    }
}

pub(crate) fn select_highest_isa_capped(features: HostIsaFeatures, cap: Option<Isa>) -> Isa {
    let selected = select_highest_isa(features);
    let Some(cap) = cap else {
        return selected;
    };
    if cap_rank(selected) <= cap_rank(cap) {
        return selected;
    }
    let mut candidates = [Isa::Scalar; 4];
    let mut count = 0usize;
    if features.aarch64.neon {
        candidates[count] = Isa::Neon;
        count += 1;
    }
    if features.aarch64.sve {
        candidates[count] = Isa::Sve;
        count += 1;
    }
    if features.aarch64.sve2 {
        candidates[count] = Isa::Sve2;
        count += 1;
    }
    if features.x86.avx2 {
        candidates[count] = Isa::Avx2;
        count += 1;
    }
    candidates[..count]
        .iter()
        .copied()
        .filter(|isa| cap_rank(*isa) <= cap_rank(cap))
        .max_by_key(|isa| cap_rank(*isa))
        .unwrap_or(Isa::Scalar)
}

/// Session-level ISA cap, synced from the `ecaz.isa_cap` GUC at the batch
/// driver entry (per-backend-process, hence per-session). Sentinel 255 =
/// no cap. Stored rank-coded per `cap_rank`.
static SESSION_ISA_CAP: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(CAP_UNSET);

const CAP_UNSET: u8 = u8::MAX;

#[allow(dead_code)] // the GUC sync caller is cfg(not(test))-gated
pub(crate) fn set_session_isa_cap(cap: Option<Isa>) {
    let encoded = cap.map_or(CAP_UNSET, cap_rank);
    SESSION_ISA_CAP.store(encoded, std::sync::atomic::Ordering::Relaxed);
}

fn session_isa_cap() -> Option<Isa> {
    decode_cap(SESSION_ISA_CAP.load(std::sync::atomic::Ordering::Relaxed))
}

fn decode_cap(encoded: u8) -> Option<Isa> {
    match encoded {
        0 => Some(Isa::Scalar),
        1 => Some(Isa::Neon),
        2 => Some(Isa::Sve),
        3 => Some(Isa::Sve2),
        4 => Some(Isa::Avx2),
        _ => None,
    }
}

/// Parses an `ECAZ_ISA_CAP` value. Empty/absent means no cap; an
/// unsupported value panics (fail-fast, mirroring the `ECAZ_SIMD`
/// precedent in `quant::simd`) so a mistyped bench configuration is
/// visible immediately instead of silently measuring the wrong column.
fn parse_isa_cap(value: &str) -> Option<Isa> {
    match value {
        "" => None,
        "scalar" => Some(Isa::Scalar),
        "neon" => Some(Isa::Neon),
        "sve" => Some(Isa::Sve),
        "sve2" => Some(Isa::Sve2),
        "avx2" => Some(Isa::Avx2),
        other => panic!("unsupported ECAZ_ISA_CAP value: {other}"),
    }
}

fn env_isa_cap() -> Option<Isa> {
    static ENV_CAP: std::sync::OnceLock<Option<Isa>> = std::sync::OnceLock::new();
    *ENV_CAP.get_or_init(|| {
        std::env::var("ECAZ_ISA_CAP")
            .ok()
            .and_then(|value| parse_isa_cap(&value))
    })
}

fn effective_isa_cap() -> Option<Isa> {
    session_isa_cap().or_else(env_isa_cap)
}

#[allow(dead_code)]
pub(crate) fn current_isa() -> Isa {
    select_highest_isa_capped(HostIsaFeatures::detect(), effective_isa_cap())
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
    use super::{
        parse_isa_cap, select_highest_isa, select_highest_isa_capped, Aarch64Features,
        HostIsaFeatures, Isa, X86Features,
    };

    fn graviton4() -> HostIsaFeatures {
        HostIsaFeatures {
            aarch64: Aarch64Features {
                neon: true,
                sve: true,
                sve2: true,
            },
            x86: X86Features::default(),
        }
    }

    fn intel_avx2() -> HostIsaFeatures {
        HostIsaFeatures {
            aarch64: Aarch64Features::default(),
            x86: X86Features { avx2: true },
        }
    }

    #[test]
    fn capped_selection_limits_but_never_fakes() {
        // No cap: unchanged.
        assert_eq!(select_highest_isa_capped(graviton4(), None), Isa::Sve2);
        // Caps below the selected tier walk down the available ladder.
        assert_eq!(
            select_highest_isa_capped(graviton4(), Some(Isa::Sve)),
            Isa::Sve
        );
        assert_eq!(
            select_highest_isa_capped(graviton4(), Some(Isa::Neon)),
            Isa::Neon
        );
        assert_eq!(
            select_highest_isa_capped(graviton4(), Some(Isa::Scalar)),
            Isa::Scalar
        );
        // A cap at or above the selected tier is a no-op.
        assert_eq!(
            select_highest_isa_capped(graviton4(), Some(Isa::Avx2)),
            Isa::Sve2
        );
        // Capping to an ISA the host lacks falls to scalar, never fakes.
        assert_eq!(
            select_highest_isa_capped(intel_avx2(), Some(Isa::Neon)),
            Isa::Scalar
        );
        assert_eq!(
            select_highest_isa_capped(HostIsaFeatures::default(), Some(Isa::Sve2)),
            Isa::Scalar
        );
    }

    // NOTE: no unit test mutates the SESSION_ISA_CAP global — per-family
    // parity tests dispatch through current_isa() concurrently, and a
    // transient cap would race them (the same isolation class the
    // CANDIDATE_BATCH_COUNTER_TEST_LOCK exists for). Cap semantics are
    // covered by the pure select_highest_isa_capped tests above; the
    // GUC-to-global sync is exercised end-to-end by the capped bench
    // cells (counter rows must report the capped ISA).
    #[test]
    fn cap_encoding_roundtrips() {
        for cap in [
            None,
            Some(Isa::Scalar),
            Some(Isa::Neon),
            Some(Isa::Sve),
            Some(Isa::Sve2),
            Some(Isa::Avx2),
        ] {
            assert_eq!(super::decode_cap(cap.map_or(super::CAP_UNSET, super::cap_rank)), cap);
        }
    }

    #[test]
    fn parse_isa_cap_accepts_known_values() {
        assert_eq!(parse_isa_cap(""), None);
        assert_eq!(parse_isa_cap("scalar"), Some(Isa::Scalar));
        assert_eq!(parse_isa_cap("neon"), Some(Isa::Neon));
        assert_eq!(parse_isa_cap("sve"), Some(Isa::Sve));
        assert_eq!(parse_isa_cap("sve2"), Some(Isa::Sve2));
        assert_eq!(parse_isa_cap("avx2"), Some(Isa::Avx2));
    }

    #[test]
    #[should_panic(expected = "unsupported ECAZ_ISA_CAP value")]
    fn parse_isa_cap_rejects_unknown_values() {
        let _ = parse_isa_cap("fast");
    }

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

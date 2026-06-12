//! `ecaz.isa_cap` — session cap on block-kernel ISA dispatch (Task 99).
//!
//! Diagnostic/bench switch: caps how high `quant::isa::current_isa()` may
//! climb so the same kernel families can be measured at a lower ISA tier
//! on hosts where a higher tier would always win dispatch (e.g. NEON
//! kernels on Graviton 4, where SVE2 is otherwise always selected at
//! equal 128-bit vector width). The cap limits, it never fakes: capping
//! to an ISA the host lacks lands on scalar. Counter attribution stays
//! truthful — kernel rows report the ISA that actually ran.
//!
//! The cap is synced into the quant layer at the batch-driver entry
//! (`candidate_batch::drivers::score_width_cascade`), so it applies to
//! block/octet/partial kernel dispatch. It does not affect the legacy
//! `quant::simd::SimdBackend` layer (`ECAZ_SIMD` covers that) or
//! non-batch one-off scoring.

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting, PostgresGucEnum};

use crate::quant::isa::Isa;

static ECAZ_ISA_CAP_GUC: GucSetting<IsaCapGuc> = GucSetting::<IsaCapGuc>::new(IsaCapGuc::None);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PostgresGucEnum)]
pub(crate) enum IsaCapGuc {
    #[name = c"none"]
    None,
    #[name = c"scalar"]
    Scalar,
    #[name = c"neon"]
    Neon,
    #[name = c"sve"]
    Sve,
    #[name = c"sve2"]
    Sve2,
    #[name = c"avx2"]
    Avx2,
}

impl IsaCapGuc {
    fn as_cap(self) -> Option<Isa> {
        match self {
            Self::None => None,
            Self::Scalar => Some(Isa::Scalar),
            Self::Neon => Some(Isa::Neon),
            Self::Sve => Some(Isa::Sve),
            Self::Sve2 => Some(Isa::Sve2),
            Self::Avx2 => Some(Isa::Avx2),
        }
    }
}

pub(crate) fn register_gucs() {
    GucRegistry::define_enum_guc(
        c"ecaz.isa_cap",
        c"Caps block-kernel ISA dispatch at the given tier.",
        c"Task 99 measurement switch for per-ISA A/B on one host (e.g. neon vs sve2 on Graviton 4). Caps dispatch preference; never selects an unavailable ISA (capping below the host's SIMD tiers falls back to scalar). Values: none, scalar, neon, sve, sve2, avx2.",
        &ECAZ_ISA_CAP_GUC,
        GucContext::Userset,
        GucFlags::default(),
    );
}

/// Syncs the GUC into the quant-layer session cap. Called at the batch
/// driver entry so a `SET ecaz.isa_cap = ...` takes effect on the next
/// flush without any registration-order or assign-hook coupling.
#[cfg(not(test))]
pub(crate) fn sync_session_isa_cap() {
    crate::quant::isa::set_session_isa_cap(ECAZ_ISA_CAP_GUC.get().as_cap());
}

#[cfg(test)]
pub(crate) fn sync_session_isa_cap() {}

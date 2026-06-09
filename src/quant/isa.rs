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

//! Typed expansion errors for the ec_distann seam (Task 164 M2, reviewer
//! 2026-07-08-01 P2). The coordinator must distinguish a **retriable** epoch
//! mismatch (FR-082: refresh epoch + restart the scan once) from
//! **non-retriable** placement / structural faults without parsing human text.
//!
//! Each variant maps to a distinct PostgreSQL SQLSTATE so a remote coordinator
//! reads `tokio_postgres::Error::code()` to classify, and also carries a stable
//! `[EC_*]` category token in the message as a secondary machine-readable
//! signal. FR-079's four outcomes each get their own class; ordinary bad input
//! and internal invariant violations are separated too.

use pgrx::PgSqlErrorCode;

#[derive(Debug, Clone)]
pub(crate) enum DistannExpandError {
    /// FR-082: caller epoch ≠ this node's active epoch. RETRIABLE.
    EpochMismatch(String),
    /// FR-079 case (b): vec_id not owned by this node under the epoch placement.
    Placement(String),
    /// FR-079 case (c): record owned but absent — structural fault.
    OwnedRecordMissing(String),
    /// FR-079 case (d): co-placed vector missing/unreadable — structural fault,
    /// distinct from (c).
    VectorMissing(String),
    /// Ordinary bad input (dimension mismatch, malformed fingerprint, …).
    BadInput(String),
    /// Internal invariant violation (should not happen behind validation).
    Internal(String),
}

impl DistannExpandError {
    /// Distinct SQLSTATE per class. Epoch mismatch uses the serialization-
    /// failure class (40001), the conventional retriable code.
    pub(crate) fn sqlstate(&self) -> PgSqlErrorCode {
        match self {
            Self::EpochMismatch(_) => PgSqlErrorCode::ERRCODE_T_R_SERIALIZATION_FAILURE,
            Self::Placement(_) => PgSqlErrorCode::ERRCODE_OBJECT_NOT_IN_PREREQUISITE_STATE,
            Self::OwnedRecordMissing(_) => PgSqlErrorCode::ERRCODE_DATA_CORRUPTED,
            Self::VectorMissing(_) => PgSqlErrorCode::ERRCODE_INDEX_CORRUPTED,
            Self::BadInput(_) => PgSqlErrorCode::ERRCODE_INVALID_PARAMETER_VALUE,
            Self::Internal(_) => PgSqlErrorCode::ERRCODE_INTERNAL_ERROR,
        }
    }

    /// The 5-character SQLSTATE string (for coordinator-side classification of a
    /// wire error and for tests).
    pub(crate) fn sqlstate_code(&self) -> &'static str {
        match self {
            Self::EpochMismatch(_) => "40001",
            Self::Placement(_) => "55000",
            Self::OwnedRecordMissing(_) => "XX001",
            Self::VectorMissing(_) => "XX002",
            Self::BadInput(_) => "22023",
            Self::Internal(_) => "XX000",
        }
    }

    /// True when the coordinator should refresh its epoch view and restart the
    /// scan (FR-082-AC-2). Only epoch mismatch is retriable.
    pub(crate) fn is_retriable(&self) -> bool {
        matches!(self, Self::EpochMismatch(_))
    }

    /// Stable machine-readable category, also embedded in the message.
    pub(crate) fn category(&self) -> &'static str {
        match self {
            Self::EpochMismatch(_) => "EC_EPOCH_MISMATCH",
            Self::Placement(_) => "EC_PLACEMENT",
            Self::OwnedRecordMissing(_) => "EC_RECORD_MISSING",
            Self::VectorMissing(_) => "EC_VECTOR_MISSING",
            Self::BadInput(_) => "EC_BAD_INPUT",
            Self::Internal(_) => "EC_INTERNAL",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::EpochMismatch(m)
            | Self::Placement(m)
            | Self::OwnedRecordMissing(m)
            | Self::VectorMissing(m)
            | Self::BadInput(m)
            | Self::Internal(m) => m,
        }
    }

    /// Raise as a PostgreSQL ERROR carrying the distinct SQLSTATE (endpoint
    /// side). Diverges.
    pub(crate) fn raise(&self) -> ! {
        pgrx::ereport!(
            ERROR,
            self.sqlstate(),
            format!("[{}] {}", self.category(), self.message())
        );
    }

    /// Classify a remote wire error by its SQLSTATE (coordinator side). Any
    /// error whose code is the retriable serialization-failure class is treated
    /// as an epoch mismatch; the message is carried through for diagnostics.
    pub(crate) fn from_wire_sqlstate(code: Option<&str>, message: String) -> Self {
        match code {
            Some("40001") => Self::EpochMismatch(message),
            Some("55000") => Self::Placement(message),
            Some("XX001") => Self::OwnedRecordMissing(message),
            Some("XX002") => Self::VectorMissing(message),
            Some("22023") => Self::BadInput(message),
            _ => Self::Internal(message),
        }
    }
}

impl std::fmt::Display for DistannExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.category(), self.message())
    }
}

impl std::error::Error for DistannExpandError {}

/// Setup/plumbing faults (relation opens, codec prep, metadata reads) that are
/// reported as plain strings collapse to the internal class — they are not one
/// of the FR-079 outcomes, which callers construct explicitly.
impl From<String> for DistannExpandError {
    fn from(message: String) -> Self {
        Self::Internal(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_have_distinct_sqlstates_and_categories() {
        let errors = [
            DistannExpandError::EpochMismatch("e".into()),
            DistannExpandError::Placement("e".into()),
            DistannExpandError::OwnedRecordMissing("e".into()),
            DistannExpandError::VectorMissing("e".into()),
            DistannExpandError::BadInput("e".into()),
            DistannExpandError::Internal("e".into()),
        ];
        let codes: Vec<&str> = errors.iter().map(|e| e.sqlstate_code()).collect();
        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        // All distinct.
        for i in 0..errors.len() {
            for j in (i + 1)..errors.len() {
                assert_ne!(codes[i], codes[j], "sqlstate collision {i}/{j}");
                assert_ne!(categories[i], categories[j], "category collision {i}/{j}");
            }
        }
        // Only epoch mismatch is retriable.
        assert!(errors[0].is_retriable());
        assert!(errors[1..].iter().all(|e| !e.is_retriable()));
    }

    #[test]
    fn wire_roundtrip_preserves_class() {
        for original in [
            DistannExpandError::EpochMismatch("x".into()),
            DistannExpandError::Placement("x".into()),
            DistannExpandError::OwnedRecordMissing("x".into()),
            DistannExpandError::VectorMissing("x".into()),
            DistannExpandError::BadInput("x".into()),
        ] {
            let reparsed =
                DistannExpandError::from_wire_sqlstate(Some(original.sqlstate_code()), "x".into());
            assert_eq!(reparsed.category(), original.category());
            assert_eq!(reparsed.is_retriable(), original.is_retriable());
        }
        // Unknown code => internal (non-retriable).
        let unknown = DistannExpandError::from_wire_sqlstate(None, "x".into());
        assert_eq!(unknown.category(), "EC_INTERNAL");
        assert!(!unknown.is_retriable());
    }
}

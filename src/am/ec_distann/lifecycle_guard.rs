use pgrx::pg_sys;

/// Distributed lifecycle operations rely on statement-to-statement catalog
/// visibility while retaining transaction-scoped governance locks.  Stronger
/// isolation levels can keep an obsolete snapshot across a recovery retry, so
/// reject them before an endpoint acquires any lock or opens a transport.
pub(crate) fn require_read_committed(operation: &str) -> Result<(), String> {
    let isolation = unsafe { pg_sys::XactIsoLevel };
    if isolation != pg_sys::XACT_READ_COMMITTED as i32 {
        return Err(format!(
            "EC_TRANSACTION_ISOLATION: {operation} requires READ COMMITTED"
        ));
    }
    Ok(())
}

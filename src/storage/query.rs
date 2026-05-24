//! PostgreSQL parser/analyzer helpers.

use pgrx::{ffi::CString, pg_sys};

pub(crate) struct AnalyzedQuery {
    query: *mut pg_sys::Query,
}

impl AnalyzedQuery {
    pub(crate) fn with_query_ptr<R>(self, f: impl FnOnce(*mut pg_sys::Query) -> R) -> R {
        f(self.query)
    }
}

pub(crate) fn analyze_single_query(sql: &str) -> Result<AnalyzedQuery, String> {
    let sql = CString::new(sql).map_err(|_| "SQL text contains an interior NUL byte".to_owned())?;
    // SAFETY: The CString is NUL-terminated and lives through the PostgreSQL
    // parser/analyzer calls. PostgreSQL owns the returned parser/analyzer Lists
    // in the current backend memory context, and this helper validates each
    // list contains exactly one element before reading index 0.
    unsafe {
        let raw_parses = pg_sys::pg_parse_query(sql.as_ptr());
        if raw_parses.is_null() {
            return Err("parser returned no statements".to_owned());
        }
        if pg_sys::list_length(raw_parses) != 1 {
            return Err("expected exactly one SQL statement".to_owned());
        }

        let raw_stmt = pg_sys::list_nth(raw_parses, 0).cast::<pg_sys::RawStmt>();
        let queries = pg_sys::pg_analyze_and_rewrite_fixedparams(
            raw_stmt,
            sql.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
        );
        if queries.is_null() {
            return Err("analyzer returned no query".to_owned());
        }
        if pg_sys::list_length(queries) != 1 {
            return Err("expected exactly one analyzed query".to_owned());
        }
        Ok(AnalyzedQuery {
            query: pg_sys::list_nth(queries, 0).cast::<pg_sys::Query>(),
        })
    }
}

//! Reloption parsing and SQL quoting for `CREATE INDEX ... WITH (...)`.
//!
//! Deliberately a leaf module: no dependencies on `profiles` or `psql` so
//! SQL-emission bugs can be tested in isolation and reused by the
//! inspect/load commands without import cycles.

use color_eyre::eyre::{eyre, Result};
use std::sync::OnceLock;

use regex::Regex;

fn ident_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").expect("static regex"))
}

fn numeric_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[+-]?(?:\d+(?:\.\d+)?|\.\d+)$").expect("static regex"))
}

/// Parse a `key=value` reloption spec into owned strings.
pub fn parse(raw: &str) -> Result<(String, String)> {
    let (key, value) = raw
        .split_once('=')
        .ok_or_else(|| eyre!("invalid reloption {:?}: expected key=value", raw))?;
    if key.is_empty() || value.is_empty() {
        return Err(eyre!("invalid reloption {:?}: expected key=value", raw));
    }
    if !ident_re().is_match(key) {
        return Err(eyre!(
            "invalid reloption key {:?}: must match [a-zA-Z_][a-zA-Z0-9_]*",
            key
        ));
    }
    Ok((key.to_owned(), value.to_owned()))
}

/// `clap` value parser adapter — same semantics as `parse` but returns a
/// string-formatted error suitable for clap's error chain.
pub fn parse_cli(raw: &str) -> std::result::Result<(String, String), String> {
    parse(raw).map_err(|e| e.to_string())
}

/// Quote `value` for a `WITH (...)` clause. Numbers and booleans pass
/// through unquoted; everything else becomes a single-quoted string with
/// `''`-escaping. Pre-quoted values are returned unchanged.
pub fn format_sql_value(value: &str) -> String {
    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        return value.to_owned();
    }
    let lowered = value.to_ascii_lowercase();
    if lowered == "true" || lowered == "false" || numeric_re().is_match(value) {
        return value.to_owned();
    }
    format!("'{}'", value.replace('\'', "''"))
}

/// Return the canonical `key=value` list used for equality checks against
/// `pg_class.reloptions`.
pub fn normalize_list(reloptions: &[(String, String)]) -> Vec<String> {
    reloptions.iter().map(|(k, v)| format!("{k}={v}")).collect()
}

/// Return the `WITH (...)` suffix for `CREATE INDEX`, or an empty string
/// when no reloptions are configured.
pub fn format_with_clause(reloptions: &[(String, String)]) -> String {
    if reloptions.is_empty() {
        return String::new();
    }
    let joined = reloptions
        .iter()
        .map(|(k, v)| format!("{k} = {}", format_sql_value(v)))
        .collect::<Vec<_>>()
        .join(", ");
    format!(" WITH ({joined})")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kv(k: &str, v: &str) -> (String, String) {
        (k.to_owned(), v.to_owned())
    }

    #[test]
    fn parse_accepts_simple_key_value() {
        assert_eq!(parse("m=8").unwrap(), kv("m", "8"));
        assert_eq!(parse("alpha=1.2").unwrap(), kv("alpha", "1.2"));
        assert_eq!(
            parse("storage_format=pq_fastscan").unwrap(),
            kv("storage_format", "pq_fastscan")
        );
    }

    #[test]
    fn parse_rejects_malformed_input() {
        assert!(parse("no_equals").is_err());
        assert!(parse("=missing_key").is_err());
        assert!(parse("missing_value=").is_err());
        assert!(parse("1_leading_digit=5").is_err());
        assert!(parse("has-dash=5").is_err());
    }

    #[test]
    fn format_sql_value_leaves_numbers_unquoted() {
        assert_eq!(format_sql_value("8"), "8");
        assert_eq!(format_sql_value("1.2"), "1.2");
        assert_eq!(format_sql_value("-0.5"), "-0.5");
        assert_eq!(format_sql_value("+42"), "+42");
    }

    #[test]
    fn format_sql_value_leaves_booleans_unquoted() {
        assert_eq!(format_sql_value("true"), "true");
        assert_eq!(format_sql_value("FALSE"), "FALSE");
    }

    #[test]
    fn format_sql_value_quotes_and_escapes_strings() {
        assert_eq!(format_sql_value("pq_fastscan"), "'pq_fastscan'");
        assert_eq!(format_sql_value("it's"), "'it''s'");
    }

    #[test]
    fn format_with_clause_empty_returns_empty_string() {
        assert_eq!(format_with_clause(&[]), "");
    }

    #[test]
    fn format_with_clause_emits_full_suffix() {
        let opts = vec![kv("m", "8"), kv("storage_format", "pq_fastscan")];
        assert_eq!(
            format_with_clause(&opts),
            " WITH (m = 8, storage_format = 'pq_fastscan')"
        );
    }

    #[test]
    fn normalize_list_produces_canonical_form() {
        let opts = vec![kv("m", "8"), kv("ef_construction", "128")];
        assert_eq!(normalize_list(&opts), vec!["m=8", "ef_construction=128"]);
    }
}

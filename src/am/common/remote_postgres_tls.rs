//! Shared PostgreSQL rustls connector and security-option parser.
//!
//! The parser deliberately removes certificate-path options before handing the
//! remaining descriptor to `postgres`/`tokio-postgres`. Callers choose an
//! explicit compatibility policy; ec_distann uses the fail-closed policy.

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use sha2::{Digest, Sha256};
use tokio_postgres::config::Host;
use tokio_postgres_rustls::MakeRustlsConnect;

pub(crate) const REMOTE_TLS_CONNINFO_PARSE_FAILED: &str = "conninfo_parse_failed";
pub(crate) const REMOTE_TLS_OPTION_UNSUPPORTED: &str = "tls_option_unsupported";
pub(crate) const REMOTE_TLS_PLAINTEXT_FORBIDDEN: &str = "plaintext_forbidden";
pub(crate) const REMOTE_TLS_CA_LOAD_FAILED: &str = "ca_load_failed";
pub(crate) const REMOTE_TLS_CLIENT_CERT_LOAD_FAILED: &str = "client_cert_load_failed";
pub(crate) const REMOTE_TLS_CLIENT_KEY_LOAD_FAILED: &str = "client_key_load_failed";
pub(crate) const REMOTE_TLS_CONNECT_FAILED: &str = "secure_connect_failed";
pub(crate) const REMOTE_TLS_SESSION_SETUP_FAILED: &str = "secure_session_setup_failed";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteTlsPolicy {
    /// Preserve SPIRE's current absent-sslmode and explicit-prefer behavior
    /// while both AMs migrate onto one connector implementation.
    SpireCompatibility,
    /// Require TLS when sslmode is absent and reject downgrade-capable modes.
    DistannSecure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteSslMode {
    Disable,
    Prefer,
    Require,
    VerifyFull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RemoteTlsError {
    category: &'static str,
    hint: &'static str,
}

impl RemoteTlsError {
    fn new(category: &'static str, hint: &'static str) -> Self {
        Self { category, hint }
    }

    pub(crate) fn category(&self) -> &'static str {
        self.category
    }
}

impl fmt::Display for RemoteTlsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.category, self.hint)
    }
}

#[derive(Clone)]
pub(crate) struct RemoteTlsConfig {
    sslmode: RemoteSslMode,
    sslrootcert: Option<String>,
    sslcert: Option<String>,
    sslkey: Option<String>,
}

impl fmt::Debug for RemoteTlsConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RemoteTlsConfig")
            .field("sslmode", &self.sslmode)
            .field(
                "sslrootcert",
                &self.sslrootcert.as_ref().map(|_| "[configured]"),
            )
            .field("sslcert", &self.sslcert.as_ref().map(|_| "[configured]"))
            .field("sslkey", &self.sslkey.as_ref().map(|_| "[configured]"))
            .finish()
    }
}

pub(crate) struct ParsedRemoteConninfo {
    base_conninfo: String,
    tls_config: RemoteTlsConfig,
    endpoint_fingerprint: [u8; 32],
    security_fingerprint: [u8; 32],
}

impl fmt::Debug for ParsedRemoteConninfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ParsedRemoteConninfo")
            .field("tls_config", &self.tls_config)
            .field(
                "endpoint_fingerprint",
                &hex::encode(self.endpoint_fingerprint),
            )
            .field(
                "security_fingerprint",
                &hex::encode(self.security_fingerprint),
            )
            .finish_non_exhaustive()
    }
}

impl RemoteTlsConfig {
    pub(crate) fn sslmode_name(&self) -> &'static str {
        match self.sslmode {
            RemoteSslMode::Disable => "disable",
            RemoteSslMode::Prefer => "prefer",
            RemoteSslMode::Require => "require",
            RemoteSslMode::VerifyFull => "verify-full",
        }
    }

    pub(crate) fn no_tls(&self) -> bool {
        self.sslmode == RemoteSslMode::Disable
    }

    pub(crate) fn connector(&self) -> Result<MakeRustlsConnect, RemoteTlsError> {
        let provider = rustls::crypto::ring::default_provider();
        let builder = ClientConfig::builder_with_provider(provider.clone().into())
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .map_err(|_| {
                RemoteTlsError::new(
                    REMOTE_TLS_OPTION_UNSUPPORTED,
                    "TLS protocol configuration is unavailable",
                )
            })?;
        let client_auth = remote_tls_client_auth(self)?;
        let config = match self.sslmode {
            RemoteSslMode::Disable => {
                return Err(RemoteTlsError::new(
                    REMOTE_TLS_OPTION_UNSUPPORTED,
                    "a TLS connector cannot be built for sslmode=disable",
                ));
            }
            RemoteSslMode::Prefer | RemoteSslMode::Require => {
                let builder = builder
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(AcceptAnyServerCertVerifier));
                match client_auth {
                    Some((certs, key)) => builder
                        .with_client_auth_cert(certs, key)
                        .map_err(|_| client_cert_error())?,
                    None => builder.with_no_client_auth(),
                }
            }
            RemoteSslMode::VerifyFull => {
                let roots = remote_tls_root_store(self.sslrootcert.as_deref())?;
                let builder = builder.with_root_certificates(roots);
                match client_auth {
                    Some((certs, key)) => builder
                        .with_client_auth_cert(certs, key)
                        .map_err(|_| client_cert_error())?,
                    None => builder.with_no_client_auth(),
                }
            }
        };
        Ok(MakeRustlsConnect::new(config))
    }
}

impl ParsedRemoteConninfo {
    pub(crate) fn base_conninfo(&self) -> &str {
        &self.base_conninfo
    }

    pub(crate) fn tls_config(&self) -> &RemoteTlsConfig {
        &self.tls_config
    }

    pub(crate) fn into_tls_config(self) -> RemoteTlsConfig {
        self.tls_config
    }

    pub(crate) fn security_fingerprint(&self) -> [u8; 32] {
        self.security_fingerprint
    }

    pub(crate) fn endpoint_fingerprint(&self) -> [u8; 32] {
        self.endpoint_fingerprint
    }
}

pub(crate) fn parse_remote_conninfo(
    conninfo: &str,
    policy: RemoteTlsPolicy,
) -> Result<ParsedRemoteConninfo, RemoteTlsError> {
    let parsed = if conninfo.trim_start().starts_with("postgres://")
        || conninfo.trim_start().starts_with("postgresql://")
    {
        parse_uri_conninfo(conninfo, policy)?
    } else {
        parse_keyword_conninfo(conninfo, policy)?
    };
    validate_policy(&parsed, policy)?;
    Ok(parsed)
}

pub(crate) fn remote_security_fingerprint(conninfo: &str, policy: RemoteTlsPolicy) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ecaz-remote-postgres-security-v1\0");
    hasher.update([policy as u8]);
    hasher.update(conninfo.as_bytes());
    hasher.finalize().into()
}

/// Open a blocking PostgreSQL connection under an explicit transport policy.
///
/// Error values are intentionally categorical: neither the descriptor nor a
/// driver error (which can contain endpoint or credential material) crosses
/// the boundary to callers.
pub(crate) fn connect_remote_postgres(
    conninfo: &str,
    policy: RemoteTlsPolicy,
    connect_timeout: std::time::Duration,
    statement_timeout_ms: u64,
) -> Result<postgres::Client, RemoteTlsError> {
    let parsed = parse_remote_conninfo(conninfo, policy)?;
    let mut config = parsed
        .base_conninfo()
        .parse::<postgres::Config>()
        .map_err(|_| conninfo_error())?;
    if !connect_timeout.is_zero() {
        config.connect_timeout(connect_timeout);
    }
    if statement_timeout_ms > 0 {
        // Synchronous transaction callbacks and operator recovery use this
        // blocking client. statement_timeout bounds server execution; the TCP
        // user timeout also bounds an acknowledged write whose response can no
        // longer make progress across a failed network path.
        config.tcp_user_timeout(std::time::Duration::from_millis(
            statement_timeout_ms.saturating_add(5_000),
        ));
    }
    let mut client = if parsed.tls_config().no_tls() {
        config
            .connect(postgres::NoTls)
            .map_err(|_| connect_error())?
    } else {
        let connector = parsed.tls_config().connector()?;
        config.connect(connector).map_err(|_| connect_error())?
    };
    if statement_timeout_ms > 0 {
        client
            .batch_execute(&format!("SET statement_timeout = {statement_timeout_ms}"))
            .map_err(|_| session_setup_error())?;
    }
    Ok(client)
}

/// The sole plaintext exception for PostgreSQL side transactions that dial
/// back into the current server. The caller must set an explicit host; an
/// omitted host is rejected even though libpq would normally imply a socket.
pub(crate) fn connect_loopback_postgres(
    config: postgres::Config,
) -> Result<postgres::Client, RemoteTlsError> {
    validate_loopback_plaintext_config(&config)?;
    config.connect(postgres::NoTls).map_err(|_| connect_error())
}

fn validate_loopback_plaintext_config(config: &postgres::Config) -> Result<(), RemoteTlsError> {
    if config.get_hosts().is_empty()
        || config
            .get_hosts()
            .iter()
            .any(|host| !is_loopback_host(host))
    {
        return Err(RemoteTlsError::new(
            REMOTE_TLS_PLAINTEXT_FORBIDDEN,
            "the plaintext side-transaction connector requires an explicit loopback endpoint",
        ));
    }
    Ok(())
}

fn default_tls_config(policy: RemoteTlsPolicy) -> RemoteTlsConfig {
    RemoteTlsConfig {
        sslmode: match policy {
            RemoteTlsPolicy::SpireCompatibility => RemoteSslMode::Disable,
            RemoteTlsPolicy::DistannSecure => RemoteSslMode::Require,
        },
        sslrootcert: None,
        sslcert: None,
        sslkey: None,
    }
}

fn parse_uri_conninfo(
    conninfo: &str,
    policy: RemoteTlsPolicy,
) -> Result<ParsedRemoteConninfo, RemoteTlsError> {
    let mut url = url::Url::parse(conninfo).map_err(|_| conninfo_error())?;
    let mut tls_config = default_tls_config(policy);
    let mut retained = Vec::new();
    let mut seen_security_options = HashSet::new();
    for (key, value) in url.query_pairs() {
        apply_conninfo_pair(
            &key,
            &value,
            policy,
            &mut tls_config,
            &mut retained,
            &mut seen_security_options,
        )?;
    }
    url.set_query(None);
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in retained {
            pairs.append_pair(&key, &value);
        }
        pairs.append_pair("sslmode", normalized_base_sslmode(tls_config.sslmode));
    }
    build_parsed(conninfo, url.to_string(), tls_config, policy)
}

fn parse_keyword_conninfo(
    conninfo: &str,
    policy: RemoteTlsPolicy,
) -> Result<ParsedRemoteConninfo, RemoteTlsError> {
    let pairs = keyword_pairs(conninfo)?;
    let mut tls_config = default_tls_config(policy);
    let mut retained = Vec::new();
    let mut seen_security_options = HashSet::new();
    for (key, value) in pairs {
        apply_conninfo_pair(
            &key,
            &value,
            policy,
            &mut tls_config,
            &mut retained,
            &mut seen_security_options,
        )?;
    }
    retained.push((
        "sslmode".to_owned(),
        normalized_base_sslmode(tls_config.sslmode).to_owned(),
    ));
    let base_conninfo = retained
        .iter()
        .map(|(key, value)| format!("{key}={}", quote_conninfo_value(value)))
        .collect::<Vec<_>>()
        .join(" ");
    build_parsed(conninfo, base_conninfo, tls_config, policy)
}

fn build_parsed(
    conninfo: &str,
    base_conninfo: String,
    tls_config: RemoteTlsConfig,
    policy: RemoteTlsPolicy,
) -> Result<ParsedRemoteConninfo, RemoteTlsError> {
    let config = base_conninfo
        .parse::<tokio_postgres::Config>()
        .map_err(|_| conninfo_error())?;
    let mut endpoint_hasher = Sha256::new();
    endpoint_hasher.update(b"ecaz-remote-postgres-endpoint-v1\0");
    for host in config.get_hosts() {
        endpoint_hasher.update(format!("{host:?}").as_bytes());
        endpoint_hasher.update([0]);
    }
    for port in config.get_ports() {
        endpoint_hasher.update(port.to_le_bytes());
    }
    if let Some(dbname) = config.get_dbname() {
        endpoint_hasher.update(dbname.as_bytes());
    }
    Ok(ParsedRemoteConninfo {
        base_conninfo,
        tls_config,
        endpoint_fingerprint: endpoint_hasher.finalize().into(),
        security_fingerprint: remote_security_fingerprint(conninfo, policy),
    })
}

fn apply_conninfo_pair(
    key: &str,
    value: &str,
    policy: RemoteTlsPolicy,
    tls_config: &mut RemoteTlsConfig,
    retained: &mut Vec<(String, String)>,
    seen_security_options: &mut HashSet<String>,
) -> Result<(), RemoteTlsError> {
    let normalized = key.to_ascii_lowercase();
    let security_option = matches!(
        normalized.as_str(),
        "sslmode" | "sslrootcert" | "sslcert" | "sslkey" | "sslpassword" | "channel_binding"
    ) || normalized.starts_with("ssl");
    if security_option && !seen_security_options.insert(normalized.clone()) {
        return Err(RemoteTlsError::new(
            REMOTE_TLS_OPTION_UNSUPPORTED,
            "duplicate TLS options are not allowed",
        ));
    }
    match normalized.as_str() {
        "sslmode" => tls_config.sslmode = parse_sslmode(value, policy)?,
        "sslrootcert" => tls_config.sslrootcert = Some(value.to_owned()),
        "sslcert" => tls_config.sslcert = Some(value.to_owned()),
        "sslkey" => tls_config.sslkey = Some(value.to_owned()),
        "channel_binding" => {
            if !matches!(value, "disable" | "prefer" | "require") {
                return Err(unsupported_option_error());
            }
            retained.push((normalized, value.to_owned()));
        }
        "sslpassword" => return Err(unsupported_option_error()),
        key if key.starts_with("ssl") => return Err(unsupported_option_error()),
        _ => retained.push((key.to_owned(), value.to_owned())),
    }
    Ok(())
}

fn parse_sslmode(value: &str, policy: RemoteTlsPolicy) -> Result<RemoteSslMode, RemoteTlsError> {
    match (policy, value) {
        (_, "disable") => Ok(RemoteSslMode::Disable),
        (RemoteTlsPolicy::SpireCompatibility, "allow" | "prefer") => Ok(RemoteSslMode::Prefer),
        (RemoteTlsPolicy::DistannSecure, "allow" | "prefer") => Err(unsupported_option_error()),
        (_, "require") => Ok(RemoteSslMode::Require),
        (_, "verify-ca") => Err(unsupported_option_error()),
        (_, "verify-full") => Ok(RemoteSslMode::VerifyFull),
        _ => Err(unsupported_option_error()),
    }
}

fn normalized_base_sslmode(sslmode: RemoteSslMode) -> &'static str {
    match sslmode {
        RemoteSslMode::Disable => "disable",
        RemoteSslMode::Prefer | RemoteSslMode::Require | RemoteSslMode::VerifyFull => "require",
    }
}

fn validate_policy(
    parsed: &ParsedRemoteConninfo,
    policy: RemoteTlsPolicy,
) -> Result<(), RemoteTlsError> {
    if parsed.tls_config.sslmode == RemoteSslMode::Disable {
        let config = parsed
            .base_conninfo
            .parse::<tokio_postgres::Config>()
            .map_err(|_| conninfo_error())?;
        if policy == RemoteTlsPolicy::DistannSecure
            && (config.get_hosts().is_empty()
                || config
                    .get_hosts()
                    .iter()
                    .any(|host| !is_loopback_host(host)))
        {
            return Err(RemoteTlsError::new(
                REMOTE_TLS_PLAINTEXT_FORBIDDEN,
                "sslmode=disable is restricted to explicit loopback endpoints",
            ));
        }
        if config.get_channel_binding() == tokio_postgres::config::ChannelBinding::Require {
            return Err(RemoteTlsError::new(
                REMOTE_TLS_PLAINTEXT_FORBIDDEN,
                "channel binding cannot be required with plaintext",
            ));
        }
    }
    if parsed.tls_config.sslmode != RemoteSslMode::VerifyFull
        && parsed.tls_config.sslrootcert.is_some()
    {
        return Err(RemoteTlsError::new(
            REMOTE_TLS_OPTION_UNSUPPORTED,
            "sslrootcert requires sslmode=verify-full",
        ));
    }
    Ok(())
}

fn is_loopback_host(host: &Host) -> bool {
    match host {
        Host::Unix(_) => true,
        Host::Tcp(host) if host.eq_ignore_ascii_case("localhost") => true,
        Host::Tcp(host) => host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback()),
    }
}

fn remote_tls_root_store(path: Option<&str>) -> Result<RootCertStore, RemoteTlsError> {
    let mut roots = RootCertStore::empty();
    if let Some(path) = path {
        let bytes = std::fs::read(path).map_err(|_| ca_error())?;
        let certs = CertificateDer::pem_slice_iter(&bytes)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ca_error())?;
        if certs.is_empty() {
            return Err(ca_error());
        }
        for cert in certs {
            roots.add(cert).map_err(|_| ca_error())?;
        }
    } else {
        roots
            .roots
            .extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }
    Ok(roots)
}

fn remote_tls_client_auth(
    tls_config: &RemoteTlsConfig,
) -> Result<Option<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>, RemoteTlsError> {
    match (&tls_config.sslcert, &tls_config.sslkey) {
        (None, None) => Ok(None),
        (Some(_), None) | (None, Some(_)) => Err(client_cert_error()),
        (Some(cert_path), Some(key_path)) => {
            validate_private_key_permissions(key_path)?;
            let cert_bytes = std::fs::read(cert_path).map_err(|_| client_cert_error())?;
            let certs = CertificateDer::pem_slice_iter(&cert_bytes)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| client_cert_error())?;
            if certs.is_empty() {
                return Err(client_cert_error());
            }
            let key_bytes = std::fs::read(key_path).map_err(|_| client_key_error())?;
            let key = PrivateKeyDer::from_pem_slice(&key_bytes).map_err(|_| client_key_error())?;
            Ok(Some((certs, key)))
        }
    }
}

#[cfg(unix)]
fn validate_private_key_permissions(path: &str) -> Result<(), RemoteTlsError> {
    use std::os::unix::fs::PermissionsExt;

    let mode = std::fs::metadata(path)
        .map_err(|_| client_key_error())?
        .permissions()
        .mode();
    if mode & 0o077 != 0 {
        return Err(RemoteTlsError::new(
            REMOTE_TLS_CLIENT_KEY_LOAD_FAILED,
            "the TLS private key grants group or other permissions",
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_key_permissions(_path: &str) -> Result<(), RemoteTlsError> {
    Ok(())
}

fn keyword_pairs(conninfo: &str) -> Result<Vec<(String, String)>, RemoteTlsError> {
    let bytes = conninfo.as_bytes();
    let mut pairs = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let key_start = i;
        while i < bytes.len() && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let key = std::str::from_utf8(&bytes[key_start..i])
            .map_err(|_| conninfo_error())?
            .to_owned();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if key.is_empty() || i >= bytes.len() || bytes[i] != b'=' {
            return Err(conninfo_error());
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let value = if i < bytes.len() && bytes[i] == b'\'' {
            i += 1;
            let mut value = String::new();
            loop {
                if i >= bytes.len() {
                    return Err(conninfo_error());
                }
                match bytes[i] {
                    b'\'' => {
                        i += 1;
                        break;
                    }
                    b'\\' => {
                        i += 1;
                        if i >= bytes.len() {
                            return Err(conninfo_error());
                        }
                        value.push(bytes[i] as char);
                        i += 1;
                    }
                    byte => {
                        value.push(byte as char);
                        i += 1;
                    }
                }
            }
            value
        } else {
            let value_start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            std::str::from_utf8(&bytes[value_start..i])
                .map_err(|_| conninfo_error())?
                .to_owned()
        };
        pairs.push((key, value));
    }
    Ok(pairs)
}

fn quote_conninfo_value(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('\'');
    for character in value.chars() {
        if character == '\\' || character == '\'' {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    quoted.push('\'');
    quoted
}

fn conninfo_error() -> RemoteTlsError {
    RemoteTlsError::new(
        REMOTE_TLS_CONNINFO_PARSE_FAILED,
        "the remote connection descriptor is malformed",
    )
}

fn unsupported_option_error() -> RemoteTlsError {
    RemoteTlsError::new(
        REMOTE_TLS_OPTION_UNSUPPORTED,
        "the requested TLS option is not supported by the secure connector",
    )
}

fn ca_error() -> RemoteTlsError {
    RemoteTlsError::new(
        REMOTE_TLS_CA_LOAD_FAILED,
        "the configured TLS trust roots could not be loaded",
    )
}

fn client_cert_error() -> RemoteTlsError {
    RemoteTlsError::new(
        REMOTE_TLS_CLIENT_CERT_LOAD_FAILED,
        "the TLS client certificate configuration is invalid",
    )
}

fn client_key_error() -> RemoteTlsError {
    RemoteTlsError::new(
        REMOTE_TLS_CLIENT_KEY_LOAD_FAILED,
        "the TLS client private key could not be loaded safely",
    )
}

fn connect_error() -> RemoteTlsError {
    RemoteTlsError::new(
        REMOTE_TLS_CONNECT_FAILED,
        "the secure remote PostgreSQL connection could not be established",
    )
}

fn session_setup_error() -> RemoteTlsError {
    RemoteTlsError::new(
        REMOTE_TLS_SESSION_SETUP_FAILED,
        "the secure remote PostgreSQL session could not be configured",
    )
}

#[derive(Debug)]
struct AcceptAnyServerCertVerifier;

impl ServerCertVerifier for AcceptAnyServerCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distann_default_requires_tls_and_preserves_channel_binding() {
        let parsed = parse_remote_conninfo(
            "host=db.example dbname=postgres channel_binding=require",
            RemoteTlsPolicy::DistannSecure,
        )
        .expect("secure conninfo should parse");
        assert_eq!(parsed.tls_config.sslmode, RemoteSslMode::Require);
        assert!(parsed.base_conninfo.contains("sslmode='require'"));
        assert!(parsed.base_conninfo.contains("channel_binding='require'"));
    }

    #[test]
    fn distann_rejects_downgrade_modes_and_remote_plaintext() {
        for mode in ["allow", "prefer"] {
            let error = parse_remote_conninfo(
                &format!("host=db.example sslmode={mode}"),
                RemoteTlsPolicy::DistannSecure,
            )
            .expect_err("downgrade mode should fail");
            assert_eq!(error.category(), REMOTE_TLS_OPTION_UNSUPPORTED);
        }
        let error = parse_remote_conninfo(
            "host=db.example sslmode=disable",
            RemoteTlsPolicy::DistannSecure,
        )
        .expect_err("remote plaintext should fail");
        assert_eq!(error.category(), REMOTE_TLS_PLAINTEXT_FORBIDDEN);
    }

    #[test]
    fn distann_allows_only_explicit_loopback_plaintext() {
        for conninfo in [
            "host=/var/run/postgresql sslmode=disable",
            "host=127.0.0.1 sslmode=disable",
            "host=::1 sslmode=disable",
            "host=localhost sslmode=disable",
        ] {
            let parsed = parse_remote_conninfo(conninfo, RemoteTlsPolicy::DistannSecure)
                .expect("explicit loopback plaintext should parse");
            assert!(parsed.tls_config.no_tls());
        }
    }

    #[test]
    fn parser_rejects_ambiguous_or_unsupported_security_options() {
        for conninfo in [
            "host=db.example sslmode=require sslmode=verify-full",
            "host=db.example sslmode=verify-ca",
            "host=db.example sslmode=require sslpassword=secret",
            "host=db.example sslmode=require sslnegotiation=direct",
        ] {
            let error = parse_remote_conninfo(conninfo, RemoteTlsPolicy::DistannSecure)
                .expect_err("unsupported security option should fail");
            assert_eq!(error.category(), REMOTE_TLS_OPTION_UNSUPPORTED);
            assert!(!error.to_string().contains("secret"));
        }
    }

    #[test]
    fn debug_and_parse_errors_do_not_expose_conninfo() {
        let raw = "host=db.example user=alice password=do-not-print sslmode=require";
        let parsed = parse_remote_conninfo(raw, RemoteTlsPolicy::DistannSecure)
            .expect("conninfo should parse");
        let debug = format!("{parsed:?}");
        assert!(!debug.contains("alice"));
        assert!(!debug.contains("do-not-print"));

        let error = parse_remote_conninfo(
            "host=db.example sslmode=require sslpassword=do-not-print",
            RemoteTlsPolicy::DistannSecure,
        )
        .expect_err("sslpassword should fail");
        assert!(!error.to_string().contains("do-not-print"));
    }

    #[test]
    fn spire_compatibility_keeps_existing_default() {
        let parsed = parse_remote_conninfo(
            "host=/tmp dbname=postgres",
            RemoteTlsPolicy::SpireCompatibility,
        )
        .expect("compatibility conninfo should parse");
        assert!(parsed.tls_config.no_tls());
    }

    #[test]
    fn security_fingerprint_changes_on_rotation_without_debug_exposure() {
        let first = parse_remote_conninfo(
            "host=db.example password=first sslmode=require",
            RemoteTlsPolicy::DistannSecure,
        )
        .expect("first conninfo should parse");
        let rotated = parse_remote_conninfo(
            "host=db.example password=second sslmode=require",
            RemoteTlsPolicy::DistannSecure,
        )
        .expect("rotated conninfo should parse");
        assert_eq!(first.endpoint_fingerprint(), rotated.endpoint_fingerprint());
        assert_ne!(first.security_fingerprint(), rotated.security_fingerprint());
    }

    #[test]
    fn plaintext_side_transaction_gate_requires_explicit_loopback() {
        let mut loopback = postgres::Config::new();
        loopback.host("127.0.0.1");
        validate_loopback_plaintext_config(&loopback).expect("loopback must be allowed");

        let mut remote = postgres::Config::new();
        remote.host("db.example");
        let error = validate_loopback_plaintext_config(&remote)
            .expect_err("remote plaintext must be rejected");
        assert_eq!(error.category(), REMOTE_TLS_PLAINTEXT_FORBIDDEN);

        let implicit = postgres::Config::new();
        assert!(validate_loopback_plaintext_config(&implicit).is_err());
    }
}

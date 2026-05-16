use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_postgres::tls::{ChannelBinding, MakeTlsConnect, TlsConnect, TlsStream};
use tokio_rustls::TlsConnector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpireRemoteSslMode {
    Disable,
    Prefer,
    Require,
    VerifyFull,
}

#[derive(Debug, Clone)]
pub(crate) struct SpireRemoteTlsConfig {
    sslmode: SpireRemoteSslMode,
    sslrootcert: Option<String>,
    sslcert: Option<String>,
    sslkey: Option<String>,
}

#[derive(Debug)]
pub(crate) struct SpireRemoteConnectError {
    pub(crate) category: &'static str,
    pub(crate) message: String,
}

pub(crate) struct SpireRemoteAsyncConnection {
    pub(crate) client: tokio_postgres::Client,
    pub(crate) connection_task: tokio::task::JoinHandle<()>,
    pub(crate) tls_config: SpireRemoteTlsConfig,
}

struct SpireRemoteParsedConninfo {
    base_conninfo: String,
    tls_config: SpireRemoteTlsConfig,
}

#[derive(Clone)]
struct SpireRemoteRustlsMakeTlsConnector {
    config: Arc<ClientConfig>,
}

struct SpireRemoteRustlsTlsConnector {
    connector: TlsConnector,
    domain: ServerName<'static>,
}

struct SpireRemoteRustlsTlsStream<S>(tokio_rustls::client::TlsStream<S>);

#[derive(Debug)]
struct SpireAcceptAnyServerCertVerifier;

impl SpireRemoteConnectError {
    fn conninfo_parse(message: String) -> Self {
        Self {
            category: SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNINFO_PARSE_FAILED,
            message,
        }
    }

    fn connect(message: String) -> Self {
        Self {
            category: SPIRE_REMOTE_PRODUCTION_TRANSPORT_CONNECT_FAILED,
            message,
        }
    }
}

impl SpireRemoteTlsConfig {
    fn sslmode_name(&self) -> &'static str {
        match self.sslmode {
            SpireRemoteSslMode::Disable => "disable",
            SpireRemoteSslMode::Prefer => "prefer",
            SpireRemoteSslMode::Require => "require",
            SpireRemoteSslMode::VerifyFull => "verify-full",
        }
    }

    fn no_tls(&self) -> bool {
        self.sslmode == SpireRemoteSslMode::Disable
    }

    fn connector(&self) -> Result<SpireRemoteRustlsMakeTlsConnector, String> {
        let provider = rustls::crypto::ring::default_provider();
        let builder = ClientConfig::builder_with_provider(provider.clone().into())
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
            .map_err(|error| format!("ec_spire remote TLS protocol setup failed: {error}"))?;
        let client_auth = spire_remote_tls_client_auth(self)?;
        let config = match self.sslmode {
            SpireRemoteSslMode::Disable => {
                return Err("ec_spire remote TLS connector requested for sslmode=disable".to_owned())
            }
            SpireRemoteSslMode::Prefer | SpireRemoteSslMode::Require => {
                let builder = builder
                    .dangerous()
                    .with_custom_certificate_verifier(Arc::new(SpireAcceptAnyServerCertVerifier));
                match client_auth {
                    Some((certs, key)) => builder
                        .with_client_auth_cert(certs, key)
                        .map_err(|error| {
                            format!("ec_spire remote TLS client certificate setup failed: {error}")
                        })?,
                    None => builder.with_no_client_auth(),
                }
            }
            SpireRemoteSslMode::VerifyFull => {
                let roots = spire_remote_tls_root_store(self.sslrootcert.as_deref())?;
                let builder = builder.with_root_certificates(roots);
                match client_auth {
                    Some((certs, key)) => builder
                        .with_client_auth_cert(certs, key)
                        .map_err(|error| {
                            format!("ec_spire remote TLS client certificate setup failed: {error}")
                        })?,
                    None => builder.with_no_client_auth(),
                }
            }
        };

        Ok(SpireRemoteRustlsMakeTlsConnector {
            config: Arc::new(config),
        })
    }
}

impl ServerCertVerifier for SpireAcceptAnyServerCertVerifier {
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

impl<S> MakeTlsConnect<S> for SpireRemoteRustlsMakeTlsConnector
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Stream = SpireRemoteRustlsTlsStream<S>;
    type TlsConnect = SpireRemoteRustlsTlsConnector;
    type Error = io::Error;

    fn make_tls_connect(&mut self, domain: &str) -> Result<Self::TlsConnect, Self::Error> {
        let domain = ServerName::try_from(domain.to_owned()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "ec_spire remote TLS invalid server name",
            )
        })?;
        Ok(SpireRemoteRustlsTlsConnector {
            connector: TlsConnector::from(self.config.clone()),
            domain,
        })
    }
}

impl<S> TlsConnect<S> for SpireRemoteRustlsTlsConnector
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    type Stream = SpireRemoteRustlsTlsStream<S>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Stream, Self::Error>> + Send>>;

    fn connect(self, stream: S) -> Self::Future {
        Box::pin(async move {
            self.connector
                .connect(self.domain, stream)
                .await
                .map(SpireRemoteRustlsTlsStream)
                .map_err(io::Error::other)
        })
    }
}

impl<S> AsyncRead for SpireRemoteRustlsTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl<S> AsyncWrite for SpireRemoteRustlsTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl<S> TlsStream for SpireRemoteRustlsTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn channel_binding(&self) -> ChannelBinding {
        ChannelBinding::none()
    }
}

pub(crate) fn remote_search_libpq_connect_with_session_timeouts(
    conninfo: &str,
    node_id: u32,
    context: &str,
) -> Result<postgres::Client, String> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let parsed = spire_remote_parse_conninfo(conninfo)
        .map_err(|error| format!("ec_spire {context} conninfo parse failed for node_id {node_id}: {error}"))?;
    let mut config = parsed
        .base_conninfo
        .parse::<postgres::Config>()
        .map_err(|_| format!("ec_spire {context} conninfo parse failed for node_id {node_id}"))?;
    if limits.connect_timeout_ms > 0 {
        config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
    }
    let mut client = if parsed.tls_config.no_tls() {
        config
            .connect(postgres::NoTls)
            .map_err(|_| format!("ec_spire {context} failed to open connection for node_id {node_id}"))?
    } else {
        let connector = parsed
            .tls_config
            .connector()
            .map_err(|error| format!("ec_spire {context} TLS setup failed for node_id {node_id}: {error}"))?;
        config
            .connect(connector)
            .map_err(|_| format!("ec_spire {context} failed to open connection for node_id {node_id}"))?
    };
    if limits.statement_timeout_ms > 0 {
        let sql = format!("SET statement_timeout = {}", limits.statement_timeout_ms);
        client.batch_execute(&sql).map_err(|_| {
            format!("ec_spire {context} failed to configure statement_timeout for node_id {node_id}")
        })?;
    }
    Ok(client)
}

pub(crate) async fn remote_search_libpq_connect_async_with_session_timeouts(
    conninfo: &str,
    node_id: u32,
    context: &str,
) -> Result<SpireRemoteAsyncConnection, SpireRemoteConnectError> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let parsed = spire_remote_parse_conninfo(conninfo).map_err(SpireRemoteConnectError::conninfo_parse)?;
    let mut config = parsed
        .base_conninfo
        .parse::<tokio_postgres::Config>()
        .map_err(|_| {
            SpireRemoteConnectError::conninfo_parse(format!(
                "ec_spire {context} conninfo parse failed for node_id {node_id}"
            ))
        })?;
    if limits.connect_timeout_ms > 0 {
        config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
    }

    if parsed.tls_config.no_tls() {
        let (client, connection) = config.connect(tokio_postgres::NoTls).await.map_err(|_| {
            SpireRemoteConnectError::connect(format!(
                "ec_spire {context} failed to open connection for node_id {node_id}"
            ))
        })?;
        let connection_task = tokio::spawn(async move {
            let _ = connection.await;
        });
        Ok(SpireRemoteAsyncConnection {
            client,
            connection_task,
            tls_config: parsed.tls_config,
        })
    } else {
        let connector = parsed.tls_config.connector().map_err(|error| {
            SpireRemoteConnectError::connect(format!(
                "ec_spire {context} TLS setup failed for node_id {node_id}: {error}"
            ))
        })?;
        let (client, connection) = config.connect(connector).await.map_err(|_| {
            SpireRemoteConnectError::connect(format!(
                "ec_spire {context} failed to open connection for node_id {node_id}"
            ))
        })?;
        let connection_task = tokio::spawn(async move {
            let _ = connection.await;
        });
        Ok(SpireRemoteAsyncConnection {
            client,
            connection_task,
            tls_config: parsed.tls_config,
        })
    }
}

pub(crate) async fn remote_search_libpq_cancel_query(
    cancel_token: tokio_postgres::CancelToken,
    tls_config: &SpireRemoteTlsConfig,
) {
    if tls_config.no_tls() {
        let _ = cancel_token.cancel_query(tokio_postgres::NoTls).await;
        return;
    }
    if let Ok(connector) = tls_config.connector() {
        let _ = cancel_token.cancel_query(connector).await;
    }
}

fn spire_remote_parse_conninfo(conninfo: &str) -> Result<SpireRemoteParsedConninfo, String> {
    let trimmed = conninfo.trim_start();
    if trimmed.starts_with("postgres://") || trimmed.starts_with("postgresql://") {
        spire_remote_parse_uri_conninfo(conninfo)
    } else {
        spire_remote_parse_keyword_conninfo(conninfo)
    }
}

fn spire_remote_default_tls_config() -> SpireRemoteTlsConfig {
    SpireRemoteTlsConfig {
        sslmode: SpireRemoteSslMode::Disable,
        sslrootcert: None,
        sslcert: None,
        sslkey: None,
    }
}

fn spire_remote_parse_uri_conninfo(conninfo: &str) -> Result<SpireRemoteParsedConninfo, String> {
    let mut url = url::Url::parse(conninfo)
        .map_err(|_| "ec_spire remote URI conninfo parse failed".to_owned())?;
    let mut tls_config = spire_remote_default_tls_config();
    let mut retained = Vec::new();
    for (key, value) in url.query_pairs() {
        spire_remote_apply_conninfo_pair(&key, &value, &mut tls_config, &mut retained)?;
    }
    url.set_query(None);
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in retained {
            pairs.append_pair(&key, &value);
        }
        if let Some(sslmode) = spire_remote_normalized_base_sslmode(tls_config.sslmode) {
            pairs.append_pair("sslmode", sslmode);
        }
    }
    Ok(SpireRemoteParsedConninfo {
        base_conninfo: url.to_string(),
        tls_config,
    })
}

fn spire_remote_parse_keyword_conninfo(conninfo: &str) -> Result<SpireRemoteParsedConninfo, String> {
    let pairs = spire_remote_keyword_pairs(conninfo)?;
    let mut tls_config = spire_remote_default_tls_config();
    let mut retained = Vec::new();
    for (key, value) in pairs {
        spire_remote_apply_conninfo_pair(&key, &value, &mut tls_config, &mut retained)?;
    }
    if let Some(sslmode) = spire_remote_normalized_base_sslmode(tls_config.sslmode) {
        retained.push(("sslmode".to_owned(), sslmode.to_owned()));
    }
    let base_conninfo = retained
        .iter()
        .map(|(key, value)| format!("{key}={}", spire_remote_quote_conninfo_value(value)))
        .collect::<Vec<_>>()
        .join(" ");
    Ok(SpireRemoteParsedConninfo {
        base_conninfo,
        tls_config,
    })
}

fn spire_remote_apply_conninfo_pair(
    key: &str,
    value: &str,
    tls_config: &mut SpireRemoteTlsConfig,
    retained: &mut Vec<(String, String)>,
) -> Result<(), String> {
    match key.to_ascii_lowercase().as_str() {
        "sslmode" => {
            tls_config.sslmode = spire_remote_parse_sslmode(value)?;
        }
        "sslrootcert" => tls_config.sslrootcert = Some(value.to_owned()),
        "sslcert" => tls_config.sslcert = Some(value.to_owned()),
        "sslkey" => tls_config.sslkey = Some(value.to_owned()),
        "sslpassword" => {
            return Err(
                "ec_spire remote TLS encrypted sslkey/sslpassword is not supported".to_owned(),
            )
        }
        _ => retained.push((key.to_owned(), value.to_owned())),
    }
    Ok(())
}

fn spire_remote_parse_sslmode(value: &str) -> Result<SpireRemoteSslMode, String> {
    match value {
        "disable" => Ok(SpireRemoteSslMode::Disable),
        "allow" | "prefer" => Ok(SpireRemoteSslMode::Prefer),
        "require" => Ok(SpireRemoteSslMode::Require),
        "verify-ca" => Err(
            "ec_spire remote conninfo sslmode=verify-ca is not supported; use sslmode=verify-full"
                .to_owned(),
        ),
        "verify-full" => Ok(SpireRemoteSslMode::VerifyFull),
        _ => Err("ec_spire remote conninfo has unsupported sslmode".to_owned()),
    }
}

fn spire_remote_normalized_base_sslmode(sslmode: SpireRemoteSslMode) -> Option<&'static str> {
    match sslmode {
        SpireRemoteSslMode::Disable => Some("disable"),
        SpireRemoteSslMode::Prefer => None,
        SpireRemoteSslMode::Require | SpireRemoteSslMode::VerifyFull => Some("require"),
    }
}

fn spire_remote_tls_root_store(sslrootcert: Option<&str>) -> Result<RootCertStore, String> {
    let mut roots = RootCertStore::empty();
    if let Some(path) = sslrootcert {
        let bytes = std::fs::read(path)
            .map_err(|_| "ec_spire remote TLS sslrootcert read failed".to_owned())?;
        let certs = CertificateDer::pem_slice_iter(&bytes)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "ec_spire remote TLS sslrootcert parse failed".to_owned())?;
        if certs.is_empty() {
            return Err("ec_spire remote TLS sslrootcert contained no certificates".to_owned());
        }
        for cert in certs {
            roots
                .add(cert)
                .map_err(|_| "ec_spire remote TLS sslrootcert trust anchor failed".to_owned())?;
        }
    } else {
        roots.roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }
    Ok(roots)
}

fn spire_remote_tls_client_auth(
    tls_config: &SpireRemoteTlsConfig,
) -> Result<Option<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>, String> {
    match (&tls_config.sslcert, &tls_config.sslkey) {
        (None, None) => Ok(None),
        (Some(_), None) | (None, Some(_)) => Err(
            "ec_spire remote TLS client certificate requires both sslcert and sslkey".to_owned(),
        ),
        (Some(cert_path), Some(key_path)) => {
            let cert_bytes = std::fs::read(cert_path)
                .map_err(|_| "ec_spire remote TLS sslcert read failed".to_owned())?;
            let certs = CertificateDer::pem_slice_iter(&cert_bytes)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "ec_spire remote TLS sslcert parse failed".to_owned())?;
            if certs.is_empty() {
                return Err("ec_spire remote TLS sslcert contained no certificates".to_owned());
            }
            let key_bytes = std::fs::read(key_path)
                .map_err(|_| "ec_spire remote TLS sslkey read failed".to_owned())?;
            let key = PrivateKeyDer::from_pem_slice(&key_bytes)
                .map_err(|_| "ec_spire remote TLS sslkey parse failed".to_owned())?;
            Ok(Some((certs, key)))
        }
    }
}

fn spire_remote_keyword_pairs(conninfo: &str) -> Result<Vec<(String, String)>, String> {
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
        if key_start == i {
            return Err("ec_spire remote conninfo has empty key".to_owned());
        }
        let key = std::str::from_utf8(&bytes[key_start..i])
            .map_err(|_| "ec_spire remote conninfo key is not UTF-8".to_owned())?
            .to_owned();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'=' {
            return Err("ec_spire remote conninfo key is missing '='".to_owned());
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
                    return Err("ec_spire remote conninfo quoted value is unterminated".to_owned());
                }
                match bytes[i] {
                    b'\'' => {
                        i += 1;
                        break;
                    }
                    b'\\' => {
                        i += 1;
                        if i >= bytes.len() {
                            return Err(
                                "ec_spire remote conninfo value has trailing escape".to_owned(),
                            );
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
                .map_err(|_| "ec_spire remote conninfo value is not UTF-8".to_owned())?
                .to_owned()
        };
        pairs.push((key, value));
    }
    Ok(pairs)
}

fn spire_remote_quote_conninfo_value(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('\'');
    for ch in value.chars() {
        if ch == '\\' || ch == '\'' {
            quoted.push('\\');
        }
        quoted.push(ch);
    }
    quoted.push('\'');
    quoted
}

#[cfg(test)]
mod spire_remote_tls_tests {
    use super::*;

    #[test]
    fn conninfo_parser_strips_tls_options_for_tokio_postgres() {
        let parsed = spire_remote_parse_conninfo(
            "host=example.com dbname=postgres sslmode=verify-full sslrootcert='/ca/root.pem' target_session_attrs=read-write",
        )
        .expect("conninfo should parse");

        assert!(parsed.base_conninfo.contains("sslmode='require'"));
        assert!(parsed.base_conninfo.contains("target_session_attrs='read-write'"));
        assert!(!parsed.base_conninfo.contains("sslrootcert"));
        assert_eq!(parsed.tls_config.sslmode, SpireRemoteSslMode::VerifyFull);
        assert_eq!(parsed.tls_config.sslrootcert.as_deref(), Some("/ca/root.pem"));
    }

    #[test]
    fn conninfo_parser_preserves_disable_for_local_non_tls() {
        let parsed = spire_remote_parse_conninfo("host=/tmp dbname=postgres sslmode=disable")
            .expect("conninfo should parse");

        assert!(parsed.tls_config.no_tls());
        assert!(parsed.base_conninfo.contains("sslmode='disable'"));
    }

    #[test]
    fn conninfo_parser_defaults_to_disable_for_unspecified_sslmode() {
        let parsed =
            spire_remote_parse_conninfo("host=/tmp dbname=postgres target_session_attrs=read-write")
                .expect("conninfo should parse");

        assert!(parsed.tls_config.no_tls());
        assert!(parsed.base_conninfo.contains("sslmode='disable'"));
        assert!(parsed.base_conninfo.contains("target_session_attrs='read-write'"));
    }
}

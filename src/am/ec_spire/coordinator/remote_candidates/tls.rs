use crate::am::common::remote_postgres_tls::{
    parse_remote_conninfo, RemoteTlsConfig, RemoteTlsPolicy,
};

type SpireRemoteTlsConfig = RemoteTlsConfig;

#[derive(Debug)]
pub(crate) struct SpireRemoteConnectError {
    pub(crate) category: &'static str,
    pub(crate) message: String,
}

pub(crate) struct SpireRemoteAsyncConnection {
    pub(crate) client: tokio_postgres::Client,
    pub(crate) connection_task: tokio::task::JoinHandle<()>,
    pub(crate) tls_config: RemoteTlsConfig,
}

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

pub(crate) fn remote_search_libpq_connect_with_session_timeouts(
    conninfo: &str,
    node_id: u32,
    context: &str,
) -> Result<postgres::Client, String> {
    let limits = SpireRemoteSearchLibpqExecutorBudgetLimits::from_session();
    let parsed = spire_remote_parse_conninfo(conninfo).map_err(|error| {
        format!("ec_spire {context} conninfo parse failed for node_id {node_id}: {error}")
    })?;
    let mut config = parsed
        .base_conninfo()
        .parse::<postgres::Config>()
        .map_err(|_| format!("ec_spire {context} conninfo parse failed for node_id {node_id}"))?;
    if limits.connect_timeout_ms > 0 {
        config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
    }
    let mut client = if parsed.tls_config().no_tls() {
        config.connect(postgres::NoTls).map_err(|_| {
            format!("ec_spire {context} failed to open connection for node_id {node_id}")
        })?
    } else {
        let connector = parsed.tls_config().connector().map_err(|error| {
            format!(
                "ec_spire {context} TLS setup failed for node_id {node_id}: {error}"
            )
        })?;
        config.connect(connector).map_err(|_| {
            format!("ec_spire {context} failed to open connection for node_id {node_id}")
        })?
    };
    if limits.statement_timeout_ms > 0 {
        let sql = format!("SET statement_timeout = {}", limits.statement_timeout_ms);
        client.batch_execute(&sql).map_err(|_| {
            format!(
                "ec_spire {context} failed to configure statement_timeout for node_id {node_id}"
            )
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
    let parsed = spire_remote_parse_conninfo(conninfo)
        .map_err(|error| SpireRemoteConnectError::conninfo_parse(error.to_string()))?;
    let mut config = parsed
        .base_conninfo()
        .parse::<tokio_postgres::Config>()
        .map_err(|_| {
            SpireRemoteConnectError::conninfo_parse(format!(
                "ec_spire {context} conninfo parse failed for node_id {node_id}"
            ))
        })?;
    if limits.connect_timeout_ms > 0 {
        config.connect_timeout(std::time::Duration::from_millis(limits.connect_timeout_ms));
    }

    if parsed.tls_config().no_tls() {
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
            tls_config: parsed.into_tls_config(),
        })
    } else {
        let connector = parsed.tls_config().connector().map_err(|error| {
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
            tls_config: parsed.into_tls_config(),
        })
    }
}

pub(crate) async fn remote_search_libpq_cancel_query(
    cancel_token: tokio_postgres::CancelToken,
    tls_config: &RemoteTlsConfig,
) {
    if tls_config.no_tls() {
        let _ = cancel_token.cancel_query(tokio_postgres::NoTls).await;
        return;
    }
    if let Ok(connector) = tls_config.connector() {
        let _ = cancel_token.cancel_query(connector).await;
    }
}

fn spire_remote_parse_conninfo(
    conninfo: &str,
) -> Result<crate::am::common::remote_postgres_tls::ParsedRemoteConninfo, String> {
    parse_remote_conninfo(conninfo, RemoteTlsPolicy::SpireCompatibility)
        .map_err(|error| error.to_string())
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

        assert!(parsed.base_conninfo().contains("sslmode='require'"));
        assert!(
            parsed
                .base_conninfo()
                .contains("target_session_attrs='read-write'")
        );
        assert!(!parsed.base_conninfo().contains("sslrootcert"));
        assert_eq!(parsed.tls_config().sslmode_name(), "verify-full");
    }

    #[test]
    fn conninfo_parser_preserves_disable_for_local_non_tls() {
        let parsed = spire_remote_parse_conninfo("host=/tmp dbname=postgres sslmode=disable")
            .expect("conninfo should parse");

        assert!(parsed.tls_config().no_tls());
        assert!(parsed.base_conninfo().contains("sslmode='disable'"));
    }

    #[test]
    fn conninfo_parser_defaults_to_disable_for_unspecified_sslmode() {
        let parsed = spire_remote_parse_conninfo(
            "host=/tmp dbname=postgres target_session_attrs=read-write",
        )
        .expect("conninfo should parse");

        assert!(parsed.tls_config().no_tls());
        assert!(parsed.base_conninfo().contains("sslmode='disable'"));
        assert!(
            parsed
                .base_conninfo()
                .contains("target_session_attrs='read-write'")
        );
    }
}

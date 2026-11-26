use std::sync::{Arc, OnceLock};

use rustls::crypto::aws_lc_rs;
use rustls::{ClientConfig, RootCertStore};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{self, handshake::client::Response};
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Client config pinned to the aws-lc-rs provider, so connecting never
/// depends on a process-default provider the application may not have set.
pub(crate) fn client_config() -> Result<Arc<ClientConfig>, tungstenite::Error> {
    static CONFIG: OnceLock<Result<Arc<ClientConfig>, String>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let mut roots = RootCertStore::empty();
            roots.add_parsable_certificates(rustls_native_certs::load_native_certs().certs);
            ClientConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider()))
                .with_safe_default_protocol_versions()
                .map(|builder| {
                    Arc::new(builder.with_root_certificates(roots).with_no_client_auth())
                })
                .map_err(|e| e.to_string())
        })
        .clone()
        .map_err(|e| tungstenite::Error::Io(std::io::Error::other(e)))
}

/// Open a WebSocket to `url` over TLS using [`client_config`].
pub(crate) async fn connect(url: &str) -> Result<(WsStream, Response), tungstenite::Error> {
    let connector = Connector::Rustls(client_config()?);
    tokio_tungstenite::connect_async_tls_with_config(url, None, false, Some(connector)).await
}

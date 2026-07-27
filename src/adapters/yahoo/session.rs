//! Process-wide, runtime-keyed Yahoo session cache.
//!
//! `reqwest::Client` spawns its connection-pool tasks on whichever tokio
//! runtime first drives it; if that runtime is dropped, later use from another
//! runtime fails with `DispatchGone`. Keying on the runtime id means a session
//! is only ever handed back to the runtime that created it.

use super::client::{ClientConfig, YahooClient};
use crate::error::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Maximum cached sessions before the map is cleared wholesale.
const SESSION_CAP: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SessionKey {
    runtime: tokio::runtime::Id,
    timeout: Duration,
    proxy: Option<String>,
    lang: String,
    region: String,
}

pub(crate) fn key_for(config: &ClientConfig) -> SessionKey {
    SessionKey {
        runtime: tokio::runtime::Handle::current().id(),
        timeout: config.timeout,
        proxy: config.proxy.clone(),
        lang: config.lang.clone(),
        region: config.region.clone(),
    }
}

fn sessions() -> &'static Mutex<HashMap<SessionKey, Arc<YahooClient>>> {
    static SESSIONS: OnceLock<Mutex<HashMap<SessionKey, Arc<YahooClient>>>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Return the cached session for this runtime and config, authenticating once
/// if there isn't one yet.
pub(crate) async fn get_or_auth(config: &ClientConfig) -> Result<Arc<YahooClient>> {
    let key = key_for(config);

    if let Some(client) = sessions().lock().unwrap().get(&key) {
        return Ok(Arc::clone(client));
    }

    let client = Arc::new(YahooClient::new(config.clone()).await?);

    let mut map = sessions().lock().unwrap();
    if map.len() >= SESSION_CAP {
        // Drop only sessions nobody else holds — clearing the whole map would
        // force every live runtime to redo the cookie + crumb handshake, which
        // is the cost this cache exists to avoid.
        map.retain(|_, c| Arc::strong_count(c) > 1);
        if map.len() >= SESSION_CAP {
            map.clear();
        }
    }
    // A concurrent caller may have inserted while we authenticated; prefer
    // theirs so every caller on this runtime shares one session.
    Ok(Arc::clone(
        map.entry(key).or_insert_with(|| Arc::clone(&client)),
    ))
}

/// Drop the cached session for this runtime and config.
pub(crate) fn invalidate(config: &ClientConfig) {
    sessions().lock().unwrap().remove(&key_for(config));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn key_includes_runtime_and_config() {
        let a = ClientConfig::default();
        let b = ClientConfig {
            lang: "ja-JP".to_string(),
            ..ClientConfig::default()
        };

        assert_eq!(key_for(&a), key_for(&a));
        assert_ne!(key_for(&a), key_for(&b));
    }

    #[test]
    fn keys_differ_across_runtimes() {
        let config = ClientConfig::default();
        let rt1 = tokio::runtime::Runtime::new().unwrap();
        let rt2 = tokio::runtime::Runtime::new().unwrap();
        let k1 = rt1.block_on(async { key_for(&config) });
        let k2 = rt2.block_on(async { key_for(&config) });
        assert_ne!(k1, k2);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn same_runtime_reuses_one_client() {
        let config = ClientConfig::default();
        let a = get_or_auth(&config).await.unwrap();
        let b = get_or_auth(&config).await.unwrap();
        assert!(Arc::ptr_eq(&a, &b));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn invalidate_forces_a_new_client() {
        let config = ClientConfig::default();
        let a = get_or_auth(&config).await.unwrap();
        invalidate(&config);
        let b = get_or_auth(&config).await.unwrap();
        assert!(!Arc::ptr_eq(&a, &b));
    }
}

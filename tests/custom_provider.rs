//! Registers a provider this crate does not build, the way a downstream crate
//! would: only `finance_query::` paths, no `async-trait` dependency of its own,
//! and no capability trait beyond what the provider actually serves.

use std::sync::Arc;

use finance_query::{
    Capability, Fetch, FinanceError, Provider, ProviderAdapter, ProviderCore, ProviderSet,
    Providers, Routes,
};

struct Canned;

impl ProviderCore for Canned {
    fn id(&self) -> Provider {
        Provider::custom("canned")
    }
}

#[finance_query::async_trait]
impl finance_query::FilingsProvider for Canned {
    async fn fetch_filings(
        &self,
        symbol: &str,
    ) -> finance_query::Result<finance_query::ProviderFilings> {
        serde_json::from_value(serde_json::json!({
            "symbol": symbol,
            "filings": [],
        }))
        .map_err(|e| FinanceError::ResponseStructureError {
            field: "filings".to_string(),
            context: e.to_string(),
        })
    }
}

impl ProviderAdapter for Canned {
    fn as_filings(&self) -> Option<&dyn finance_query::FilingsProvider> {
        Some(self)
    }
}

fn canned() -> Arc<dyn ProviderAdapter> {
    Arc::new(Canned)
}

#[tokio::test]
async fn a_registered_adapter_serves_its_routed_capability() {
    let providers = Providers::builder()
        .with_adapter(canned())
        .route(Capability::FILINGS, [Provider::custom("canned")])
        .build()
        .await
        .expect("builds");

    let filings = providers
        .filings("AAPL")
        .get()
        .await
        .expect("the custom adapter serves FILINGS");
    assert!(filings.filings.is_empty());
}

#[tokio::test]
async fn health_reports_the_custom_identity() {
    let providers = Providers::builder()
        .with_adapter(canned())
        .route(Capability::FILINGS, [Provider::custom("canned")])
        .build()
        .await
        .expect("builds");

    let ids: Vec<Provider> = providers.health().into_iter().map(|h| h.provider).collect();
    assert!(
        ids.contains(&Provider::custom("canned")),
        "custom provider should report under its own id, got {ids:?}"
    );
}

#[tokio::test]
async fn routing_to_an_unregistered_id_fails_at_build() {
    let err = Providers::builder()
        .route(Capability::FILINGS, [Provider::custom("absent")])
        .build()
        .await
        .expect_err("an unregistered route is a build error");
    assert!(
        matches!(err, FinanceError::ProviderNotRegistered { provider } if provider == Provider::custom("absent")),
        "got {err:?}"
    );
}

#[tokio::test]
async fn a_custom_id_may_not_shadow_a_built_in() {
    struct Impostor;
    impl ProviderCore for Impostor {
        fn id(&self) -> Provider {
            Provider::custom("yahoo")
        }
    }
    impl ProviderAdapter for Impostor {}

    let err = Providers::builder()
        .with_adapter(Arc::new(Impostor))
        .build()
        .await
        .expect_err("a colliding id is rejected");
    assert!(
        matches!(err, FinanceError::InvalidParameter { ref param, .. } if param == "adapter"),
        "got {err:?}"
    );
}

#[tokio::test]
async fn an_unserved_capability_names_the_custom_provider() {
    let providers = Providers::builder()
        .with_adapter(canned())
        .route(Capability::CHART, [Provider::custom("canned")])
        .build()
        .await
        .expect("builds");

    let err = providers
        .ticker("AAPL")
        .build()
        .await
        .expect("ticker builds")
        .chart(
            finance_query::Interval::OneDay,
            finance_query::TimeRange::OneMonth,
        )
        .await
        .expect_err("the canned provider serves no chart");
    assert!(
        matches!(err, FinanceError::NotSupported { provider, .. } if provider == Provider::custom("canned")),
        "got {err:?}"
    );
}

#[tokio::test]
async fn a_hand_built_set_reaches_a_domain_handle() {
    let set = ProviderSet::new(
        vec![canned()],
        Routes::new(Fetch::Sequential).route(Capability::FILINGS, [Provider::custom("canned")]),
    );
    let providers = Providers::from_set(Arc::new(set));
    assert_eq!(providers.provider_set().health().len(), 1);

    let filings = providers.filings("MSFT").get().await.expect("serves");
    assert!(filings.filings.is_empty());
}

#[test]
fn routes_report_what_they_carry() {
    let routes =
        Routes::new(Fetch::Parallel).route(Capability::FILINGS, [Provider::custom("canned")]);
    assert_eq!(routes.fetch_mode(), Fetch::Parallel);
    assert_eq!(
        routes.providers_for(Capability::FILINGS),
        Some(&[Provider::custom("canned")][..])
    );
    assert_eq!(routes.providers_for(Capability::QUOTE), None);
}

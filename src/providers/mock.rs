//! Counting provider adapter for offline cache tests.

use super::{Capability, Fetch, Provider, ProviderAdapter, ProviderSet, Routes};
use crate::error::Result;
use crate::models::chart::{Chart, ChartMeta};
use crate::models::corporate::news::News;
use crate::models::quote::QuoteSummaryResponse;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct CountingProvider {
    quote_calls: AtomicUsize,
    chart_calls: AtomicUsize,
    news_calls: AtomicUsize,
}

impl CountingProvider {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            quote_calls: AtomicUsize::new(0),
            chart_calls: AtomicUsize::new(0),
            news_calls: AtomicUsize::new(0),
        })
    }

    pub(crate) fn quotes(&self) -> usize {
        self.quote_calls.load(Ordering::SeqCst)
    }

    pub(crate) fn charts(&self) -> usize {
        self.chart_calls.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
    pub(crate) fn news(&self) -> usize {
        self.news_calls.load(Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for CountingProvider {
    fn id(&self) -> Provider {
        Provider::Yahoo
    }

    fn capabilities(&self) -> Capability {
        Capability::QUOTE | Capability::CHART | Capability::CORPORATE
    }

    async fn fetch_quote(&self, _: &str) -> Result<QuoteSummaryResponse> {
        self.quote_calls.fetch_add(1, Ordering::SeqCst);
        Ok(QuoteSummaryResponse::default())
    }

    async fn fetch_chart(
        &self,
        symbol: &str,
        _: crate::Interval,
        _: crate::TimeRange,
    ) -> Result<Chart> {
        self.chart_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Chart {
            symbol: symbol.to_string(),
            meta: ChartMeta::default(),
            candles: Vec::new(),
            interval: None,
            range: None,
            provider_id: Some(Provider::Yahoo),
        })
    }

    async fn fetch_news(&self, _: &str) -> Result<Vec<News>> {
        self.news_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    }
}

pub(crate) fn provider_set(provider: Arc<CountingProvider>) -> Arc<ProviderSet> {
    Arc::new(ProviderSet::new(
        vec![provider as Arc<dyn ProviderAdapter>],
        None,
        Routes::new(Fetch::Sequential),
    ))
}

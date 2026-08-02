//! Counting provider adapter for offline cache tests.

use super::{
    ChartProvider, CorporateProvider, Fetch, OptionsProvider, Provider, ProviderAdapter,
    ProviderCore, ProviderSet, QuoteProvider, Routes,
};
use crate::error::Result;
use crate::models::chart::events::ChartEvents;
use crate::models::chart::{Chart, ChartMeta};
use crate::models::corporate::news::News;
use crate::models::options::Options;
use crate::models::options::response::OptionChainContainer;
use crate::models::quote::QuoteSummaryResponse;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct CountingProvider {
    quote_calls: AtomicUsize,
    chart_calls: AtomicUsize,
    news_calls: AtomicUsize,
    events_calls: AtomicUsize,
    options_calls: AtomicUsize,
}

impl CountingProvider {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            quote_calls: AtomicUsize::new(0),
            chart_calls: AtomicUsize::new(0),
            news_calls: AtomicUsize::new(0),
            events_calls: AtomicUsize::new(0),
            options_calls: AtomicUsize::new(0),
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

    pub(crate) fn events(&self) -> usize {
        self.events_calls.load(Ordering::SeqCst)
    }

    pub(crate) fn options(&self) -> usize {
        self.options_calls.load(Ordering::SeqCst)
    }
}

impl ProviderCore for CountingProvider {
    fn id(&self) -> Provider {
        Provider::Yahoo
    }
}

#[async_trait::async_trait]
impl QuoteProvider for CountingProvider {
    async fn fetch_quote(&self, _: &str) -> Result<QuoteSummaryResponse> {
        self.quote_calls.fetch_add(1, Ordering::SeqCst);
        Ok(QuoteSummaryResponse::default())
    }
}

#[async_trait::async_trait]
impl ChartProvider for CountingProvider {
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
}

#[async_trait::async_trait]
impl CorporateProvider for CountingProvider {
    async fn fetch_news(&self, _: &str) -> Result<Vec<News>> {
        self.news_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Vec::new())
    }

    async fn fetch_events(&self, _: &str) -> Result<ChartEvents> {
        self.events_calls.fetch_add(1, Ordering::SeqCst);
        // A real network fetch always suspends; without a yield the mock resolves
        // inline and concurrent callers can never actually interleave.
        tokio::task::yield_now().await;
        Ok(ChartEvents::default())
    }
}

#[async_trait::async_trait]
impl OptionsProvider for CountingProvider {
    async fn fetch_options(&self, _: &str, _: Option<i64>) -> Result<Options> {
        self.options_calls.fetch_add(1, Ordering::SeqCst);
        Ok(Options {
            option_chain: OptionChainContainer {
                result: Vec::new(),
                error: None,
            },
            provider_id: Some(Provider::Yahoo),
        })
    }
}

#[async_trait::async_trait]
impl ProviderAdapter for CountingProvider {
    fn as_quote(&self) -> Option<&dyn QuoteProvider> {
        Some(self)
    }
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        Some(self)
    }
    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        Some(self)
    }
    fn as_options(&self) -> Option<&dyn OptionsProvider> {
        Some(self)
    }
}

pub(crate) fn provider_set(provider: Arc<CountingProvider>) -> Arc<ProviderSet> {
    Arc::new(ProviderSet::new(
        vec![provider as Arc<dyn ProviderAdapter>],
        None,
        Routes::new(Fetch::Sequential),
    ))
}

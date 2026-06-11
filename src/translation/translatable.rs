//! The [`Translatable`] trait and implementations for response models.
//!
//! Only human-readable natural-language fields are visited. Symbols,
//! exchange codes, currency codes, keys, URLs, addresses, and numeric or
//! date fields are never translated.

use crate::models::chart::spark::Spark;
use crate::models::chart::{CapitalGain, Chart, Dividend, Split};
use crate::models::corporate::news::News;
use crate::models::corporate::recommendation::Recommendation;
use crate::models::corporate::transcript::{Transcript, TranscriptWithMeta};
use crate::models::discovery::lookup::{LookupQuote, LookupResults};
use crate::models::discovery::search::{
    SearchNews, SearchNewsList, SearchQuote, SearchQuotes, SearchResults,
};
use crate::models::format::Format;
use crate::models::fundamentals::FinancialStatement;
use crate::models::market::industries::IndustryData;
use crate::models::market::market_summary::MarketSummaryQuote;
use crate::models::market::sectors::SectorData;
use crate::models::options::Options;
use crate::models::quote::{
    AssetProfile, CalendarEvents, CompanyOfficer, DefaultKeyStatistics, Earnings, EarningsHistory,
    EarningsTrend, EquityPerformance, FinancialData, FundOwnership, FundPerformance, FundProfile,
    IndexTrend, IndustryTrend, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, NetSharePurchaseActivity, Price, Quote, QuoteTypeData,
    RecommendationTrend, SecFilings, SectorTrend, SummaryDetail, SummaryProfile, TopHoldings,
    UpgradeDowngradeHistory,
};

/// A response type whose human-readable text fields can be translated.
///
/// The default implementation visits nothing, so types without
/// natural-language content implement this trait as a no-op marker.
///
/// `visit_translatable` must visit the same fields in the same order on
/// every call for an unmodified value: the translation pipeline performs one
/// pass to collect texts and a second pass to write translations back.
pub trait Translatable {
    /// Visit every translatable string field in a stable order.
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        let _ = visit;
    }

    /// Hook invoked after translations have been written back, for types
    /// that maintain derived text (e.g. a transcript's joined full text).
    fn after_translate(&mut self) {}
}

fn visit_opt(field: &mut Option<String>, visit: &mut dyn FnMut(&mut String)) {
    if let Some(value) = field {
        visit(value);
    }
}

impl<T: Translatable> Translatable for Option<T> {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        if let Some(value) = self {
            value.visit_translatable(visit);
        }
    }
    fn after_translate(&mut self) {
        if let Some(value) = self {
            value.after_translate();
        }
    }
}

impl<T: Translatable> Translatable for Vec<T> {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        for value in self {
            value.visit_translatable(visit);
        }
    }
    fn after_translate(&mut self) {
        for value in self {
            value.after_translate();
        }
    }
}

impl<F: Format> Translatable for Quote<F> {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.short_name, visit);
        visit_opt(&mut self.long_name, visit);
        visit_opt(&mut self.sector_disp, visit);
        visit_opt(&mut self.industry_disp, visit);
        visit_opt(&mut self.long_business_summary, visit);
    }
}

impl<F: Format> Translatable for Price<F> {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.short_name, visit);
        visit_opt(&mut self.long_name, visit);
    }
}

impl Translatable for CompanyOfficer {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.title, visit);
    }
}

impl Translatable for AssetProfile {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.sector_disp, visit);
        visit_opt(&mut self.industry_disp, visit);
        visit_opt(&mut self.long_business_summary, visit);
        self.company_officers.visit_translatable(visit);
    }
}

impl Translatable for SummaryProfile {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.sector_disp, visit);
        visit_opt(&mut self.industry_disp, visit);
        visit_opt(&mut self.long_business_summary, visit);
    }
}

impl Translatable for QuoteTypeData {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.short_name, visit);
        visit_opt(&mut self.long_name, visit);
    }
}

impl Translatable for News {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit(&mut self.title);
    }
}

impl Translatable for SearchNews {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.title, visit);
    }
}

impl Translatable for SearchQuote {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.short_name, visit);
        visit_opt(&mut self.long_name, visit);
        visit_opt(&mut self.type_disp, visit);
        visit_opt(&mut self.exch_disp, visit);
        visit_opt(&mut self.sector_disp, visit);
        visit_opt(&mut self.industry_disp, visit);
    }
}

impl Translatable for SearchQuotes {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        self.0.visit_translatable(visit);
    }
}

impl Translatable for SearchNewsList {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        self.0.visit_translatable(visit);
    }
}

impl Translatable for SearchResults {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        self.quotes.visit_translatable(visit);
        self.news.visit_translatable(visit);
    }
}

impl Translatable for LookupQuote {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.short_name, visit);
        visit_opt(&mut self.long_name, visit);
        visit_opt(&mut self.type_disp, visit);
        visit_opt(&mut self.exch_disp, visit);
        visit_opt(&mut self.sector, visit);
        visit_opt(&mut self.industry, visit);
    }
}

impl Translatable for LookupResults {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        self.quotes.visit_translatable(visit);
    }
}

impl Translatable for MarketSummaryQuote {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit_opt(&mut self.short_name, visit);
        visit_opt(&mut self.full_exchange_name, visit);
    }
}

impl Translatable for SectorData {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit(&mut self.name);
        if let Some(overview) = &mut self.overview {
            visit_opt(&mut overview.description, visit);
        }
        for industry in &mut self.industries {
            visit(&mut industry.name);
        }
    }
}

impl Translatable for IndustryData {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit(&mut self.name);
        visit_opt(&mut self.sector_name, visit);
        if let Some(overview) = &mut self.overview {
            visit_opt(&mut overview.description, visit);
        }
    }
}

impl Translatable for Transcript {
    /// Visits paragraph texts (not the joined `text`, to avoid translating the
    /// same content twice) plus speaker roles and the event title;
    /// [`after_translate`](Translatable::after_translate) rebuilds the joined
    /// text from the translated paragraphs. Per-sentence/word timing data is
    /// left untranslated. When no paragraphs are present, the full text is
    /// translated directly.
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit(&mut self.transcript_metadata.title);
        for mapping in &mut self.transcript_content.speaker_mapping {
            visit_opt(&mut mapping.speaker_data.role, visit);
        }
        if let Some(data) = &mut self.transcript_content.transcript {
            if data.paragraphs.is_empty() {
                visit(&mut data.text);
            } else {
                for paragraph in &mut data.paragraphs {
                    visit(&mut paragraph.text);
                }
            }
        }
    }

    fn after_translate(&mut self) {
        if let Some(data) = &mut self.transcript_content.transcript
            && !data.paragraphs.is_empty()
        {
            data.text = data
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
        }
    }
}

impl Translatable for TranscriptWithMeta {
    fn visit_translatable(&mut self, visit: &mut dyn FnMut(&mut String)) {
        visit(&mut self.title);
        self.transcript.visit_translatable(visit);
    }

    fn after_translate(&mut self) {
        self.transcript.after_translate();
    }
}

/// Marker implementations for response types without natural-language
/// content, so the `Ticker`/`Tickers` accessors can translate uniformly.
macro_rules! not_translatable {
    ($($ty:ty),* $(,)?) => {
        $(impl Translatable for $ty {})*
    };
}

not_translatable!(
    SummaryDetail,
    FinancialData,
    DefaultKeyStatistics,
    CalendarEvents,
    Earnings,
    EarningsTrend,
    EarningsHistory,
    RecommendationTrend,
    InsiderHolders,
    InsiderTransactions,
    InstitutionOwnership,
    FundOwnership,
    MajorHoldersBreakdown,
    NetSharePurchaseActivity,
    SecFilings,
    UpgradeDowngradeHistory,
    FundPerformance,
    FundProfile,
    TopHoldings,
    IndexTrend,
    IndustryTrend,
    SectorTrend,
    EquityPerformance,
    Chart,
    Spark,
    Dividend,
    Split,
    CapitalGain,
    FinancialStatement,
    Recommendation,
    Options,
);

#[cfg(feature = "indicators")]
not_translatable!(crate::indicators::IndicatorsSummary);

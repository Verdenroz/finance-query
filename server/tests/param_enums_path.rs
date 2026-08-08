//! Wire contract for the typed REST *path* parameters.
//!
//! Each of these was a `String` parsed with `FromStr` inside the handler, which
//! returned a hand-rolled 400 on a bad value. Now they are typed extractors, so
//! these tests pin two things: every spelling the old parser accepted is still
//! accepted, and a bad value is still a 400 (never a bare 422).

mod common;

use axum::http::StatusCode;
use finance_query::{Industry, Screener, Sector, StatementType};
use finance_query_server::params::{AnalysisType, HolderType};

use common::{get_status, path_route, symbol_kind_route};

// -- screener --------------------------------------------------------------

/// The 15 slugs the hand-written spec documented, verbatim.
const SCREENER_SLUGS: &[&str] = &[
    "aggressive-small-caps",
    "day-gainers",
    "day-losers",
    "growth-technology-stocks",
    "most-actives",
    "most-shorted-stocks",
    "small-cap-gainers",
    "undervalued-growth-stocks",
    "undervalued-large-caps",
    "conservative-foreign-funds",
    "high-yield-bond",
    "portfolio-anchors",
    "solid-large-growth-funds",
    "solid-midcap-growth-funds",
    "top-mutual-funds",
];

#[tokio::test]
async fn every_documented_screener_slug_is_accepted() {
    assert_eq!(SCREENER_SLUGS.len(), Screener::all().len());
    for slug in SCREENER_SLUGS {
        let (status, _) = get_status(
            path_route::<Screener>("/v2/screeners/{screener}"),
            &format!("/v2/screeners/{slug}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "screener slug {slug}");
    }
}

#[tokio::test]
async fn screener_shorthands_the_old_parser_took_are_still_accepted() {
    for (short, want) in [
        ("gainers", Screener::DayGainers),
        ("losers", Screener::DayLosers),
        ("growth-tech", Screener::GrowthTechnologyStocks),
        ("actives", Screener::MostActives),
        ("most-shorted", Screener::MostShortedStocks),
        ("undervalued-growth", Screener::UndervaluedGrowthStocks),
        ("undervalued-large", Screener::UndervaluedLargeCaps),
    ] {
        let (status, body) = get_status(
            path_route::<Screener>("/v2/screeners/{screener}"),
            &format!("/v2/screeners/{short}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "screener shorthand {short}");
        assert_eq!(body, format!("{want:?}"), "screener shorthand {short}");
    }
}

#[tokio::test]
async fn an_unknown_screener_is_still_a_400() {
    let (status, _) = get_status(
        path_route::<Screener>("/v2/screeners/{screener}"),
        "/v2/screeners/not-a-screener",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// -- sector ----------------------------------------------------------------

const SECTOR_SLUGS: &[&str] = &[
    "technology",
    "financial-services",
    "consumer-cyclical",
    "communication-services",
    "healthcare",
    "industrials",
    "consumer-defensive",
    "energy",
    "basic-materials",
    "real-estate",
    "utilities",
];

#[tokio::test]
async fn every_documented_sector_slug_is_accepted() {
    assert_eq!(SECTOR_SLUGS.len(), Sector::all().len());
    for slug in SECTOR_SLUGS {
        let (status, _) = get_status(
            path_route::<Sector>("/v2/sectors/{sector}"),
            &format!("/v2/sectors/{slug}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "sector slug {slug}");
    }
}

#[tokio::test]
async fn sector_shorthands_the_old_parser_took_are_still_accepted() {
    for (short, want) in [
        ("tech", Sector::Technology),
        ("financials", Sector::FinancialServices),
        ("financial", Sector::FinancialServices),
        ("communication", Sector::CommunicationServices),
        ("health", Sector::Healthcare),
        ("industrial", Sector::Industrials),
        ("materials", Sector::BasicMaterials),
        ("realestate", Sector::RealEstate),
        ("utility", Sector::Utilities),
    ] {
        let (status, body) = get_status(
            path_route::<Sector>("/v2/sectors/{sector}"),
            &format!("/v2/sectors/{short}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "sector shorthand {short}");
        assert_eq!(body, format!("{want:?}"), "sector shorthand {short}");
    }
}

#[tokio::test]
async fn an_unknown_sector_is_still_a_400() {
    let (status, _) = get_status(
        path_route::<Sector>("/v2/sectors/{sector}"),
        "/v2/sectors/not-a-sector",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// -- industry --------------------------------------------------------------

/// The 148 slugs the hand-written spec documented, verbatim.
const INDUSTRY_SLUGS: &[&str] = &[
    "aerospace-defense",
    "agricultural-inputs",
    "aluminum",
    "apparel-manufacturing",
    "apparel-retail",
    "asset-management",
    "auto-manufacturers",
    "auto-parts",
    "auto-truck-dealerships",
    "banks-diversified",
    "banks-regional",
    "beverages-brewers",
    "beverages-non-alcoholic",
    "beverages-wineries-distilleries",
    "biotechnology",
    "broadcasting",
    "building-materials",
    "building-products-equipment",
    "business-equipment-supplies",
    "capital-markets",
    "chemical-manufacturing",
    "chemicals",
    "closed-end-fund-debt",
    "closed-end-fund-equity",
    "closed-end-fund-foreign",
    "coal",
    "communication-equipment",
    "computer-hardware",
    "confectioners",
    "conglomerates",
    "consulting-services",
    "consumer-electronics",
    "copper",
    "credit-services",
    "data-analytics",
    "department-stores",
    "diagnostics-research",
    "discount-stores",
    "drug-manufacturers-general",
    "drug-manufacturers-specialty-generic",
    "electrical-equipment-parts",
    "electronic-components",
    "electronic-gaming-multimedia",
    "electronics-computer-distribution",
    "engineering-construction",
    "entertainment",
    "exchange-traded-fund",
    "farm-heavy-construction-machinery",
    "farm-products",
    "financial-data-stock-exchanges",
    "food-distribution",
    "footwear-accessories",
    "forest-products",
    "furnishings-fixtures-appliances",
    "gambling",
    "gold",
    "grocery-stores",
    "hardware-software-distribution",
    "health-information-services",
    "healthcare-plans",
    "home-improvement-retail",
    "household-personal-products",
    "industrial-distribution",
    "information-technology-services",
    "infrastructure-operations",
    "insurance-brokers",
    "insurance-diversified",
    "insurance-life",
    "insurance-property-casualty",
    "insurance-reinsurance",
    "insurance-specialty",
    "integrated-freight-logistics",
    "internet-content-information",
    "internet-retail",
    "leisure",
    "lodging",
    "lumber-wood-production",
    "luxury-goods",
    "manufacturing-diversified",
    "marine-ports-services",
    "marine-shipping",
    "medical-care-facilities",
    "medical-devices",
    "medical-distribution",
    "medical-instruments-supplies",
    "metal-fabrication",
    "mortgage-finance",
    "oil-gas-drilling",
    "oil-gas-ep",
    "oil-gas-equipment-services",
    "oil-gas-integrated",
    "oil-gas-midstream",
    "oil-gas-refining-marketing",
    "other-industrial-metals-mining",
    "other-precious-metals-mining",
    "packaged-foods",
    "paper-paper-products",
    "personal-services",
    "pharmaceutical-retailers",
    "pollution-treatment-controls",
    "publishing",
    "railroads",
    "real-estate-development",
    "real-estate-diversified",
    "real-estate-services",
    "reit-diversified",
    "reit-healthcare-facilities",
    "reit-hotel-motel",
    "reit-industrial",
    "reit-mortgage",
    "reit-office",
    "reit-residential",
    "reit-retail",
    "reit-specialty",
    "rental-leasing-services",
    "residential-construction",
    "resorts-casinos",
    "restaurants",
    "scientific-technical-instruments",
    "security-protection-services",
    "semiconductor-equipment-materials",
    "semiconductors",
    "shell-companies",
    "silver",
    "software-application",
    "software-infrastructure",
    "solar",
    "specialty-business-services",
    "specialty-chemicals",
    "specialty-industrial-machinery",
    "specialty-retail",
    "staffing-employment-services",
    "steel",
    "telecom-services",
    "textile-manufacturing",
    "thermal-coal",
    "tobacco",
    "tools-accessories",
    "travel-services",
    "trucking",
    "uranium",
    "utilities-diversified",
    "utilities-independent-power-producers",
    "utilities-regulated-electric",
    "utilities-regulated-gas",
    "utilities-regulated-water",
    "utilities-renewable",
    "waste-management",
];

#[tokio::test]
async fn every_documented_industry_slug_round_trips() {
    assert_eq!(INDUSTRY_SLUGS.len(), 148);
    let route = || {
        axum::Router::new().route(
            "/v2/industries/{industry}",
            axum::routing::get(
                |axum::extract::Path(i): axum::extract::Path<Industry>| async move { i.as_slug() },
            ),
        )
    };
    for slug in INDUSTRY_SLUGS {
        let (status, body) = get_status(route(), &format!("/v2/industries/{slug}")).await;
        assert_eq!(status, StatusCode::OK, "industry slug {slug}");
        // Deserializing and re-slugging must land on the same wire value, which
        // is what proves the serde renames still match `as_slug`.
        assert_eq!(&body, slug, "industry slug {slug}");
    }
}

#[tokio::test]
async fn an_unknown_industry_is_still_a_400() {
    let (status, _) = get_status(
        path_route::<Industry>("/v2/industries/{industry}"),
        "/v2/industries/not-an-industry",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// -- statement, holder type, analysis type ---------------------------------

#[tokio::test]
async fn every_statement_spelling_the_old_parser_took_is_accepted() {
    for (slug, want) in [
        ("income", StatementType::Income),
        ("balance", StatementType::Balance),
        ("cashflow", StatementType::CashFlow),
        ("income-statement", StatementType::Income),
        ("balance-sheet", StatementType::Balance),
        ("cash", StatementType::CashFlow),
        ("cash-flow", StatementType::CashFlow),
    ] {
        let (status, body) = get_status(
            symbol_kind_route::<StatementType>("/v2/financials/{symbol}/{statement}"),
            &format!("/v2/financials/AAPL/{slug}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "statement {slug}");
        assert_eq!(body, format!("{want:?}"), "statement {slug}");
    }
}

#[tokio::test]
async fn an_unknown_statement_is_still_a_400() {
    let (status, _) = get_status(
        symbol_kind_route::<StatementType>("/v2/financials/{symbol}/{statement}"),
        "/v2/financials/AAPL/nonsense",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn every_documented_holder_type_is_accepted() {
    for (slug, want) in [
        ("major", HolderType::Major),
        ("institutional", HolderType::Institutional),
        ("mutualfund", HolderType::MutualFund),
        ("insider-transactions", HolderType::InsiderTransactions),
        ("insider-purchases", HolderType::InsiderPurchases),
        ("insider-roster", HolderType::InsiderRoster),
    ] {
        let (status, body) = get_status(
            symbol_kind_route::<HolderType>("/v2/holders/{symbol}/{type}"),
            &format!("/v2/holders/AAPL/{slug}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "holder type {slug}");
        assert_eq!(body, format!("{want:?}"), "holder type {slug}");
        assert_eq!(want.as_str(), slug, "holder type {slug}");
    }
}

#[tokio::test]
async fn an_unknown_holder_type_is_still_a_400() {
    let (status, _) = get_status(
        symbol_kind_route::<HolderType>("/v2/holders/{symbol}/{type}"),
        "/v2/holders/AAPL/nonsense",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn every_documented_analysis_type_is_accepted() {
    for (slug, want) in [
        ("recommendations", AnalysisType::Recommendations),
        ("upgrades-downgrades", AnalysisType::UpgradesDowngrades),
        ("earnings-estimate", AnalysisType::EarningsEstimate),
        ("earnings-history", AnalysisType::EarningsHistory),
    ] {
        let (status, body) = get_status(
            symbol_kind_route::<AnalysisType>("/v2/analysis/{symbol}/{type}"),
            &format!("/v2/analysis/AAPL/{slug}"),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "analysis type {slug}");
        assert_eq!(body, format!("{want:?}"), "analysis type {slug}");
        assert_eq!(want.as_str(), slug, "analysis type {slug}");
    }
}

#[tokio::test]
async fn an_unknown_analysis_type_is_still_a_400() {
    let (status, _) = get_status(
        symbol_kind_route::<AnalysisType>("/v2/analysis/{symbol}/{type}"),
        "/v2/analysis/AAPL/nonsense",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

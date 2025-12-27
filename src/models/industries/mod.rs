//! Industry models.
//!
//! Types for Yahoo Finance industry data including overview, performance,
//! top companies, and research reports.

mod response;

pub use response::{
    BenchmarkPerformance, GrowthCompany, Industry, IndustryCompany, IndustryOverview,
    IndustryPerformance, PerformingCompany, ResearchReport,
};

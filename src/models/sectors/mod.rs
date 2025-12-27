//! Sector models.
//!
//! Types for Yahoo Finance sector data including overview, performance,
//! top companies, ETFs, mutual funds, and industries.

mod response;

pub use response::{
    ResearchReport, Sector, SectorCompany, SectorETF, SectorIndustry, SectorMutualFund,
    SectorOverview, SectorPerformance,
};

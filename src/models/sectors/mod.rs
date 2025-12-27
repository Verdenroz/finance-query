//! Sector models.
//!
//! Types for Yahoo Finance sector data including overview, performance,
//! top companies, ETFs, mutual funds, and industries.

mod response;

pub use response::{
    Industry, ResearchReport, SectorCompany, SectorETF, SectorMutualFund, SectorOverview,
    SectorPerformance, SectorResponse,
};

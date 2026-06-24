//! Sector models.

mod response;

pub use response::SectorData;

#[cfg(feature = "python")]
pub use response::{
    PyResearchReport, PySectorCompany, PySectorData, PySectorETF, PySectorIndustry,
    PySectorMutualFund, PySectorOverview, PySectorPerformance,
};

//! Python-facing mirrors of `finance_query::constants` enums.
//!
//! Each `Py*` is registered with PyO3 as a Python enum class. The
//! `impl_enum!` macro emits the mirror + `From` conversions in both
//! directions so Rust-side calls (e.g. `ticker.chart(interval.into(), ...)`)
//! work transparently.
#![cfg(feature = "python")]
#![allow(missing_docs)]

use crate::{
    ExchangeCode, Frequency, Industry, Interval, Region, Screener, Sector, StatementType,
    TimeRange, ValueFormat,
};
use pyo3::prelude::*;

/// Generates a #[pyclass(eq, eq_int)] enum mirroring a Rust enum, plus
/// `From` conversions both ways.
macro_rules! impl_enum {
    ($py_name:ident, $rust_name:ident, $py_str:literal, $($variant:ident),+ $(,)?) => {
        #[pyclass(eq, eq_int, hash, frozen, name = $py_str)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $py_name {
            $($variant,)+
        }

        impl ::core::convert::From<$py_name> for $rust_name {
            fn from(v: $py_name) -> Self {
                match v {
                    $($py_name::$variant => $rust_name::$variant,)+
                }
            }
        }

        impl ::core::convert::From<$rust_name> for $py_name {
            fn from(v: $rust_name) -> Self {
                match v {
                    $($rust_name::$variant => $py_name::$variant,)+
                }
            }
        }
    };
}

/// Same as `impl_enum!` but for source enums marked `#[non_exhaustive]`.
/// The `From<rust> for py` match needs a wildcard arm to satisfy the
/// compiler when the source enum is defined in another crate.
macro_rules! impl_enum_non_exhaustive {
    ($py_name:ident, $rust_name:ident, $py_str:literal, $($variant:ident),+ $(,)?) => {
        #[pyclass(eq, eq_int, hash, frozen, name = $py_str)]
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $py_name {
            $($variant,)+
        }

        impl ::core::convert::From<$py_name> for $rust_name {
            fn from(v: $py_name) -> Self {
                match v {
                    $($py_name::$variant => $rust_name::$variant,)+
                }
            }
        }

        impl ::core::convert::From<$rust_name> for $py_name {
            fn from(v: $rust_name) -> Self {
                match v {
                    $($rust_name::$variant => $py_name::$variant,)+
                    _ => unreachable!(
                        "non_exhaustive source enum gained a new variant — update the Python mirror"
                    ),
                }
            }
        }
    };
}

impl_enum!(
    PyInterval,
    Interval,
    "Interval",
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    OneDay,
    OneWeek,
    OneMonth,
    ThreeMonths,
);

impl_enum!(
    PyTimeRange,
    TimeRange,
    "TimeRange",
    OneDay,
    FiveDays,
    OneMonth,
    ThreeMonths,
    SixMonths,
    OneYear,
    TwoYears,
    FiveYears,
    TenYears,
    YearToDate,
    Max,
);

impl_enum!(PyFrequency, Frequency, "Frequency", Annual, Quarterly);

impl_enum!(
    PyStatementType,
    StatementType,
    "StatementType",
    Income,
    Balance,
    CashFlow,
);

impl_enum!(PyValueFormat, ValueFormat, "ValueFormat", Raw, Pretty, Both,);

impl_enum!(
    PyRegion,
    Region,
    "Region",
    Argentina,
    Australia,
    Brazil,
    Canada,
    China,
    Denmark,
    Finland,
    France,
    Germany,
    Greece,
    HongKong,
    India,
    Israel,
    Italy,
    Malaysia,
    NewZealand,
    Norway,
    Portugal,
    Russia,
    Singapore,
    Spain,
    Sweden,
    Taiwan,
    Thailand,
    Turkey,
    UnitedKingdom,
    UnitedStates,
    Vietnam,
);

impl_enum!(
    PySector,
    Sector,
    "Sector",
    Technology,
    FinancialServices,
    ConsumerCyclical,
    CommunicationServices,
    Healthcare,
    Industrials,
    ConsumerDefensive,
    Energy,
    BasicMaterials,
    RealEstate,
    Utilities,
);

impl_enum!(
    PyScreener,
    Screener,
    "Screener",
    AggressiveSmallCaps,
    DayGainers,
    DayLosers,
    GrowthTechnologyStocks,
    MostActives,
    MostShortedStocks,
    SmallCapGainers,
    UndervaluedGrowthStocks,
    UndervaluedLargeCaps,
    ConservativeForeignFunds,
    HighYieldBond,
    PortfolioAnchors,
    SolidLargeGrowthFunds,
    SolidMidcapGrowthFunds,
    TopMutualFunds,
);

/// Python-facing mirror of `providers::Provider`.
///
/// Feature-gated variants match the Rust `Provider` enum exactly so that
/// the same features must be enabled for both to compile.
#[pyclass(eq, eq_int, hash, frozen, name = "Provider")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PyProvider {
    Yahoo,
    #[cfg(feature = "polygon")]
    Polygon,
    #[cfg(feature = "fmp")]
    Fmp,
    #[cfg(feature = "alphavantage")]
    AlphaVantage,
    #[cfg(feature = "crypto")]
    CoinGecko,
    #[cfg(feature = "fred")]
    Fred,
    Edgar,
}
impl ::core::convert::From<crate::providers::Provider> for PyProvider {
    fn from(v: crate::providers::Provider) -> Self {
        use crate::providers::Provider as P;
        match v {
            P::Yahoo => PyProvider::Yahoo,
            #[cfg(feature = "polygon")]
            P::Polygon => PyProvider::Polygon,
            #[cfg(feature = "fmp")]
            P::Fmp => PyProvider::Fmp,
            #[cfg(feature = "alphavantage")]
            P::AlphaVantage => PyProvider::AlphaVantage,
            #[cfg(feature = "crypto")]
            P::CoinGecko => PyProvider::CoinGecko,
            #[cfg(feature = "fred")]
            P::Fred => PyProvider::Fred,
            P::Edgar => PyProvider::Edgar,
        }
    }
}
impl ::core::convert::From<PyProvider> for crate::providers::Provider {
    fn from(v: PyProvider) -> Self {
        use crate::providers::Provider as P;
        match v {
            PyProvider::Yahoo => P::Yahoo,
            #[cfg(feature = "polygon")]
            PyProvider::Polygon => P::Polygon,
            #[cfg(feature = "fmp")]
            PyProvider::Fmp => P::Fmp,
            #[cfg(feature = "alphavantage")]
            PyProvider::AlphaVantage => P::AlphaVantage,
            #[cfg(feature = "crypto")]
            PyProvider::CoinGecko => P::CoinGecko,
            #[cfg(feature = "fred")]
            PyProvider::Fred => P::Fred,
            PyProvider::Edgar => P::Edgar,
        }
    }
}

impl_enum_non_exhaustive!(
    PyExchangeCode,
    ExchangeCode,
    "ExchangeCode",
    Ase,
    Bts,
    Ncm,
    Ngm,
    Nms,
    Nyq,
    Pcx,
    Pnk,
    Nas,
    Asx,
    Bse,
    Hkg,
    Krx,
    Lse,
    Nsi,
    Shh,
    Shz,
    Tyo,
    Tor,
    Ger,
);

impl_enum_non_exhaustive!(
    PyIndustry,
    Industry,
    "Industry",
    // Agriculture / Raw Materials
    AgriculturalInputs,
    Aluminum,
    Coal,
    Copper,
    FarmProducts,
    ForestProducts,
    Gold,
    LumberAndWoodProduction,
    OtherIndustrialMetalsAndMining,
    OtherPreciousMetalsAndMining,
    Silver,
    Steel,
    ThermalCoal,
    Uranium,
    // Consumer
    ApparelManufacturing,
    ApparelRetail,
    AutoAndTruckDealerships,
    AutoManufacturers,
    AutoParts,
    BeveragesBrewers,
    BeveragesNonAlcoholic,
    BeveragesWineriesAndDistilleries,
    Confectioners,
    DepartmentStores,
    DiscountStores,
    ElectronicGamingAndMultimedia,
    FoodDistribution,
    FootwearAndAccessories,
    FurnishingsFixturesAndAppliances,
    Gambling,
    GroceryStores,
    HomeImprovementRetail,
    HouseholdAndPersonalProducts,
    InternetRetail,
    Leisure,
    Lodging,
    LuxuryGoods,
    PackagedFoods,
    PersonalServices,
    ResidentialConstruction,
    ResortsAndCasinos,
    Restaurants,
    SpecialtyRetail,
    TextileManufacturing,
    Tobacco,
    TravelServices,
    // Energy
    OilAndGasDrilling,
    OilAndGasEAndP,
    OilAndGasEquipmentAndServices,
    OilAndGasIntegrated,
    OilAndGasMidstream,
    OilAndGasRefiningAndMarketing,
    Solar,
    // Financial Services
    AssetManagement,
    BanksDiversified,
    BanksRegional,
    CapitalMarkets,
    CreditServices,
    FinancialDataAndStockExchanges,
    InsuranceBrokers,
    InsuranceDiversified,
    InsuranceLife,
    InsurancePropertyAndCasualty,
    InsuranceReinsurance,
    InsuranceSpecialty,
    MortgageFinance,
    ShellCompanies,
    // Healthcare
    Biotechnology,
    DiagnosticsAndResearch,
    DrugManufacturersGeneral,
    DrugManufacturersSpecialtyAndGeneric,
    HealthInformationServices,
    HealthcarePlans,
    MedicalCareFacilities,
    MedicalDevices,
    MedicalDistribution,
    MedicalInstrumentsAndSupplies,
    PharmaceuticalRetailers,
    // Industrials
    AerospaceAndDefense,
    BuildingMaterials,
    BuildingProductsAndEquipment,
    BusinessEquipmentAndSupplies,
    ChemicalManufacturing,
    Chemicals,
    Conglomerates,
    ConsultingServices,
    ElectricalEquipmentAndParts,
    EngineeringAndConstruction,
    FarmAndHeavyConstructionMachinery,
    IndustrialDistribution,
    InfrastructureOperations,
    IntegratedFreightAndLogistics,
    ManufacturingDiversified,
    MarinePortsAndServices,
    MarineShipping,
    MetalFabrication,
    PaperAndPaperProducts,
    PollutionAndTreatmentControls,
    Railroads,
    RentalAndLeasingServices,
    SecurityAndProtectionServices,
    SpecialtyBusinessServices,
    SpecialtyChemicals,
    SpecialtyIndustrialMachinery,
    StaffingAndEmploymentServices,
    ToolsAndAccessories,
    Trucking,
    WasteManagement,
    // Real Estate
    RealEstateDevelopment,
    RealEstateDiversified,
    RealEstateServices,
    ReitDiversified,
    ReitHealthcareFacilities,
    ReitHotelAndMotel,
    ReitIndustrial,
    ReitMortgage,
    ReitOffice,
    ReitResidential,
    ReitRetail,
    ReitSpecialty,
    // Technology
    CommunicationEquipment,
    ComputerHardware,
    ConsumerElectronics,
    DataAnalytics,
    ElectronicComponents,
    ElectronicsAndComputerDistribution,
    HardwareAndSoftwareDistribution,
    InformationTechnologyServices,
    InternetContentAndInformation,
    ScientificAndTechnicalInstruments,
    SemiconductorEquipmentAndMaterials,
    Semiconductors,
    SoftwareApplication,
    SoftwareInfrastructure,
    // Communication Services
    Broadcasting,
    Entertainment,
    Publishing,
    TelecomServices,
    // Utilities
    UtilitiesDiversified,
    UtilitiesIndependentPowerProducers,
    UtilitiesRegulatedElectric,
    UtilitiesRegulatedGas,
    UtilitiesRegulatedWater,
    UtilitiesRenewable,
    // Special
    ClosedEndFundDebt,
    ClosedEndFundEquity,
    ClosedEndFundForeign,
    ExchangeTradedFund,
);

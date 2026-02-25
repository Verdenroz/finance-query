//! Screener-specific typed value enums.
//!
//! For region, sector, industry, and exchange values — which are shared with other
//! parts of the API — use the types from `constants`:
//!
//! | Field | Type | Module |
//! |-------|------|--------|
//! | [`EquityField::Region`] | [`Region`](crate::Region) | `crate::Region` |
//! | [`EquityField::Sector`] | [`Sector`](crate::Sector) | `crate::Sector` |
//! | [`EquityField::Industry`] / `finance::industry()` | [`Industry`](crate::Industry) | `crate::Industry` |
//! | [`EquityField::Exchange`] / [`FundField::Exchange`] | [`ExchangeCode`](crate::ExchangeCode) | `crate::ExchangeCode` |
//!
//! This module contains only the screener-specific value enums that have no counterpart
//! elsewhere in the library:
//!
//! | Field | Type |
//! |-------|------|
//! | [`EquityField::PeerGroup`] | [`ScreenerPeerGroup`] |
//! | [`FundField::CategoryName`] | [`ScreenerFundCategory`] |
//!
//! [`EquityField::Region`]: super::fields::EquityField::Region
//! [`EquityField::Sector`]: super::fields::EquityField::Sector
//! [`EquityField::Industry`]: super::fields::EquityField::Industry
//! [`EquityField::Exchange`]: super::fields::EquityField::Exchange
//! [`EquityField::PeerGroup`]: super::fields::EquityField::PeerGroup
//! [`FundField::Exchange`]: super::fields::FundField::Exchange
//! [`FundField::CategoryName`]: super::fields::FundField::CategoryName
//!
//! # Example
//!
//! ```
//! use finance_query::{
//!     EquityField, EquityScreenerQuery, FundField, FundScreenerQuery, ScreenerFieldExt,
//!     Region, Sector, Industry, ExchangeCode,
//!     ScreenerPeerGroup, ScreenerFundCategory,
//! };
//!
//! let equity_query = EquityScreenerQuery::new()
//!     .add_condition(EquityField::Region.eq_str(Region::UnitedStates))
//!     .add_condition(EquityField::Sector.eq_str(Sector::Technology))
//!     .add_condition(EquityField::Exchange.eq_str(ExchangeCode::Nms))
//!     .add_condition(EquityField::Industry.eq_str(Industry::Semiconductors))
//!     .add_condition(EquityField::PeerGroup.eq_str(ScreenerPeerGroup::Semiconductors));
//!
//! let fund_query = FundScreenerQuery::new()
//!     .add_condition(FundField::Exchange.eq_str(ExchangeCode::Nas))
//!     .add_condition(FundField::CategoryName.eq_str(ScreenerFundCategory::UsLargeGrowth));
//! ```

// ============================================================================
// ScreenerPeerGroup
// ============================================================================

/// Equity peer group for screener queries.
///
/// Use with [`EquityField::PeerGroup`](super::fields::EquityField::PeerGroup) and
/// [`ScreenerFieldExt::eq_str`](super::condition::ScreenerFieldExt::eq_str).
///
/// These are Yahoo Finance's proprietary peer group classifications used to group
/// stocks for comparison purposes. Values are broader than [`Industry`](crate::Industry)
/// names.
///
/// # Example
///
/// ```
/// use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt, ScreenerPeerGroup};
///
/// let query = EquityScreenerQuery::new()
///     .add_condition(EquityField::PeerGroup.eq_str(ScreenerPeerGroup::Semiconductors));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenerPeerGroup {
    // ── Technology ───────────────────────────────────────────────────────────
    /// "Semiconductors"
    Semiconductors,
    /// "Semiconductor Equipment"
    SemiconductorEquipment,
    /// "Software & Services"
    SoftwareAndServices,
    /// "Technology Hardware & Equipment"
    TechnologyHardwareAndEquipment,
    /// "Internet Companies"
    InternetCompanies,
    /// "Electronic Equipment"
    ElectronicEquipment,

    // ── Financials ───────────────────────────────────────────────────────────
    /// "Banks"
    Banks,
    /// "Diversified Banks"
    DiversifiedBanks,
    /// "Regional Banks"
    RegionalBanks,
    /// "Capital Markets"
    CapitalMarkets,
    /// "Insurance"
    Insurance,
    /// "Life & Health Insurance"
    LifeAndHealthInsurance,
    /// "Property & Casualty Insurance"
    PropertyAndCasualtyInsurance,
    /// "Asset Management"
    AssetManagement,
    /// "Consumer Finance"
    ConsumerFinance,
    /// "Mortgage Finance"
    MortgageFinance,
    /// "Specialty Finance"
    SpecialtyFinance,
    /// "Diversified Financial Services"
    DiversifiedFinancialServices,
    /// "Investment Banking & Brokerage"
    InvestmentBankingAndBrokerage,

    // ── Healthcare ───────────────────────────────────────────────────────────
    /// "Pharmaceuticals"
    Pharmaceuticals,
    /// "Biotechnology"
    Biotechnology,
    /// "Health Care Equipment"
    HealthCareEquipment,
    /// "Health Care Services"
    HealthCareServices,
    /// "Health Care Facilities"
    HealthCareFacilities,
    /// "Managed Healthcare"
    ManagedHealthcare,

    // ── Consumer ─────────────────────────────────────────────────────────────
    /// "Automobiles"
    Automobiles,
    /// "Auto Components"
    AutoComponents,
    /// "Food Products"
    FoodProducts,
    /// "Packaged Foods & Meats"
    PackagedFoodsAndMeats,
    /// "Beverages"
    Beverages,
    /// "Household Products"
    HouseholdProducts,
    /// "Household Durables"
    HouseholdDurables,
    /// "Personal Products"
    PersonalProducts,
    /// "Apparel & Luxury Goods"
    ApparelAndLuxuryGoods,
    /// "Retailers"
    Retailers,
    /// "Food & Drug Retailing"
    FoodAndDrugRetailing,
    /// "General Merchandise Stores"
    GeneralMerchandiseStores,
    /// "Specialty Retail"
    SpecialtyRetail,
    /// "Leisure Products"
    LeisureProducts,
    /// "Hotels"
    Hotels,
    /// "Restaurants"
    Restaurants,
    /// "Media"
    Media,
    /// "Drug Retail"
    DrugRetail,

    // ── Energy ───────────────────────────────────────────────────────────────
    /// "Integrated Oil & Gas"
    IntegratedOilAndGas,
    /// "Oil & Gas Exploration & Production"
    OilAndGasExplorationAndProduction,
    /// "Oil & Gas Refining & Marketing"
    OilAndGasRefiningAndMarketing,
    /// "Oil & Gas Storage & Transportation"
    OilAndGasStorageAndTransportation,
    /// "Energy Equipment & Services"
    EnergyEquipmentAndServices,

    // ── Industrials ──────────────────────────────────────────────────────────
    /// "Aerospace & Defense"
    AerospaceAndDefense,
    /// "Airlines"
    Airlines,
    /// "Industrial Machinery"
    IndustrialMachinery,
    /// "Industrial Conglomerates"
    IndustrialConglomerates,
    /// "Industrial Gases"
    IndustrialGases,
    /// "Building Products"
    BuildingProducts,
    /// "Chemicals"
    Chemicals,
    /// "Specialty Chemicals"
    SpecialtyChemicals,
    /// "Metals & Mining"
    MetalsAndMining,
    /// "Paper & Forest Products"
    PaperAndForestProducts,
    /// "Containers & Packaging"
    ContainersAndPackaging,
    /// "Commercial Services & Supplies"
    CommercialServicesAndSupplies,
    /// "Railroads"
    Railroads,
    /// "Trucking"
    Trucking,
    /// "Research & Consulting Services"
    ResearchAndConsultingServices,
    /// "Human Resource & Employment Services"
    HumanResourceAndEmploymentServices,
    /// "Data Processing & Outsourced Services"
    DataProcessingAndOutsourcedServices,
    /// "IT Consulting & Other Services"
    ItConsultingAndOtherServices,

    // ── Real Estate / Utilities ───────────────────────────────────────────────
    /// "Real Estate"
    RealEstate,
    /// "Retail REITs"
    RetailReits,
    /// "Electric Utilities"
    ElectricUtilities,
    /// "Gas Utilities"
    GasUtilities,
    /// "Water Utilities"
    WaterUtilities,
    /// "Multi-Utilities"
    MultiUtilities,
    /// "Renewable Electricity"
    RenewableElectricity,

    // ── Telecom ───────────────────────────────────────────────────────────────
    /// "Telecom Services"
    TelecomServices,
    /// "Wireless Telecommunication Services"
    WirelessTelecommunicationServices,

    // ── Commodities ───────────────────────────────────────────────────────────
    /// "Steel"
    Steel,
    /// "Gold"
    Gold,
    /// "Oil & Gas"
    OilAndGas,

    // ── Other ─────────────────────────────────────────────────────────────────
    /// "Waste Management"
    WasteManagement,
    /// "Tobacco"
    Tobacco,
    /// "Office Electronics"
    OfficeElectronics,
    /// "Communications Equipment"
    CommunicationsEquipment,
    /// "Home Building"
    HomeBuilding,
    /// "Home Furnishings"
    HomeFurnishings,
    /// "Household Appliances"
    HouseholdAppliances,
    /// "Publishing"
    Publishing,
}

impl ScreenerPeerGroup {
    /// Returns the peer group name used by Yahoo Finance.
    pub fn as_str(self) -> &'static str {
        match self {
            ScreenerPeerGroup::Semiconductors => "Semiconductors",
            ScreenerPeerGroup::SemiconductorEquipment => "Semiconductor Equipment",
            ScreenerPeerGroup::SoftwareAndServices => "Software & Services",
            ScreenerPeerGroup::TechnologyHardwareAndEquipment => "Technology Hardware & Equipment",
            ScreenerPeerGroup::InternetCompanies => "Internet Companies",
            ScreenerPeerGroup::ElectronicEquipment => "Electronic Equipment",
            ScreenerPeerGroup::Banks => "Banks",
            ScreenerPeerGroup::DiversifiedBanks => "Diversified Banks",
            ScreenerPeerGroup::RegionalBanks => "Regional Banks",
            ScreenerPeerGroup::CapitalMarkets => "Capital Markets",
            ScreenerPeerGroup::Insurance => "Insurance",
            ScreenerPeerGroup::LifeAndHealthInsurance => "Life & Health Insurance",
            ScreenerPeerGroup::PropertyAndCasualtyInsurance => "Property & Casualty Insurance",
            ScreenerPeerGroup::AssetManagement => "Asset Management",
            ScreenerPeerGroup::ConsumerFinance => "Consumer Finance",
            ScreenerPeerGroup::MortgageFinance => "Mortgage Finance",
            ScreenerPeerGroup::SpecialtyFinance => "Specialty Finance",
            ScreenerPeerGroup::DiversifiedFinancialServices => "Diversified Financial Services",
            ScreenerPeerGroup::InvestmentBankingAndBrokerage => "Investment Banking & Brokerage",
            ScreenerPeerGroup::Pharmaceuticals => "Pharmaceuticals",
            ScreenerPeerGroup::Biotechnology => "Biotechnology",
            ScreenerPeerGroup::HealthCareEquipment => "Health Care Equipment",
            ScreenerPeerGroup::HealthCareServices => "Health Care Services",
            ScreenerPeerGroup::HealthCareFacilities => "Health Care Facilities",
            ScreenerPeerGroup::ManagedHealthcare => "Managed Healthcare",
            ScreenerPeerGroup::Automobiles => "Automobiles",
            ScreenerPeerGroup::AutoComponents => "Auto Components",
            ScreenerPeerGroup::FoodProducts => "Food Products",
            ScreenerPeerGroup::PackagedFoodsAndMeats => "Packaged Foods & Meats",
            ScreenerPeerGroup::Beverages => "Beverages",
            ScreenerPeerGroup::HouseholdProducts => "Household Products",
            ScreenerPeerGroup::HouseholdDurables => "Household Durables",
            ScreenerPeerGroup::PersonalProducts => "Personal Products",
            ScreenerPeerGroup::ApparelAndLuxuryGoods => "Apparel & Luxury Goods",
            ScreenerPeerGroup::Retailers => "Retailers",
            ScreenerPeerGroup::FoodAndDrugRetailing => "Food & Drug Retailing",
            ScreenerPeerGroup::GeneralMerchandiseStores => "General Merchandise Stores",
            ScreenerPeerGroup::SpecialtyRetail => "Specialty Retail",
            ScreenerPeerGroup::LeisureProducts => "Leisure Products",
            ScreenerPeerGroup::Hotels => "Hotels",
            ScreenerPeerGroup::Restaurants => "Restaurants",
            ScreenerPeerGroup::Media => "Media",
            ScreenerPeerGroup::DrugRetail => "Drug Retail",
            ScreenerPeerGroup::IntegratedOilAndGas => "Integrated Oil & Gas",
            ScreenerPeerGroup::OilAndGasExplorationAndProduction => {
                "Oil & Gas Exploration & Production"
            }
            ScreenerPeerGroup::OilAndGasRefiningAndMarketing => "Oil & Gas Refining & Marketing",
            ScreenerPeerGroup::OilAndGasStorageAndTransportation => {
                "Oil & Gas Storage & Transportation"
            }
            ScreenerPeerGroup::EnergyEquipmentAndServices => "Energy Equipment & Services",
            ScreenerPeerGroup::AerospaceAndDefense => "Aerospace & Defense",
            ScreenerPeerGroup::Airlines => "Airlines",
            ScreenerPeerGroup::IndustrialMachinery => "Industrial Machinery",
            ScreenerPeerGroup::IndustrialConglomerates => "Industrial Conglomerates",
            ScreenerPeerGroup::IndustrialGases => "Industrial Gases",
            ScreenerPeerGroup::BuildingProducts => "Building Products",
            ScreenerPeerGroup::Chemicals => "Chemicals",
            ScreenerPeerGroup::SpecialtyChemicals => "Specialty Chemicals",
            ScreenerPeerGroup::MetalsAndMining => "Metals & Mining",
            ScreenerPeerGroup::PaperAndForestProducts => "Paper & Forest Products",
            ScreenerPeerGroup::ContainersAndPackaging => "Containers & Packaging",
            ScreenerPeerGroup::CommercialServicesAndSupplies => "Commercial Services & Supplies",
            ScreenerPeerGroup::Railroads => "Railroads",
            ScreenerPeerGroup::Trucking => "Trucking",
            ScreenerPeerGroup::ResearchAndConsultingServices => "Research & Consulting Services",
            ScreenerPeerGroup::HumanResourceAndEmploymentServices => {
                "Human Resource & Employment Services"
            }
            ScreenerPeerGroup::DataProcessingAndOutsourcedServices => {
                "Data Processing & Outsourced Services"
            }
            ScreenerPeerGroup::ItConsultingAndOtherServices => "IT Consulting & Other Services",
            ScreenerPeerGroup::RealEstate => "Real Estate",
            ScreenerPeerGroup::RetailReits => "Retail REITs",
            ScreenerPeerGroup::ElectricUtilities => "Electric Utilities",
            ScreenerPeerGroup::GasUtilities => "Gas Utilities",
            ScreenerPeerGroup::WaterUtilities => "Water Utilities",
            ScreenerPeerGroup::MultiUtilities => "Multi-Utilities",
            ScreenerPeerGroup::RenewableElectricity => "Renewable Electricity",
            ScreenerPeerGroup::TelecomServices => "Telecom Services",
            ScreenerPeerGroup::WirelessTelecommunicationServices => {
                "Wireless Telecommunication Services"
            }
            ScreenerPeerGroup::Steel => "Steel",
            ScreenerPeerGroup::Gold => "Gold",
            ScreenerPeerGroup::OilAndGas => "Oil & Gas",
            ScreenerPeerGroup::WasteManagement => "Waste Management",
            ScreenerPeerGroup::Tobacco => "Tobacco",
            ScreenerPeerGroup::OfficeElectronics => "Office Electronics",
            ScreenerPeerGroup::CommunicationsEquipment => "Communications Equipment",
            ScreenerPeerGroup::HomeBuilding => "Home Building",
            ScreenerPeerGroup::HomeFurnishings => "Home Furnishings",
            ScreenerPeerGroup::HouseholdAppliances => "Household Appliances",
            ScreenerPeerGroup::Publishing => "Publishing",
        }
    }
}

impl From<ScreenerPeerGroup> for String {
    fn from(v: ScreenerPeerGroup) -> Self {
        v.as_str().to_string()
    }
}

// ============================================================================
// ScreenerFundCategory
// ============================================================================

/// Morningstar fund category for mutual fund screener queries.
///
/// Use with [`FundField::CategoryName`](super::fields::FundField::CategoryName) and
/// [`ScreenerFieldExt::eq_str`](super::condition::ScreenerFieldExt::eq_str).
///
/// # Example
///
/// ```
/// use finance_query::{FundField, FundScreenerQuery, ScreenerFieldExt, ScreenerFundCategory};
///
/// let query = FundScreenerQuery::new()
///     .add_condition(FundField::CategoryName.eq_str(ScreenerFundCategory::UsLargeGrowth));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenerFundCategory {
    // ── US Equity ─────────────────────────────────────────────────────────────
    /// "US Fund Large Blend"
    UsLargeBlend,
    /// "US Fund Large Growth"
    UsLargeGrowth,
    /// "US Fund Large Value"
    UsLargeValue,
    /// "US Fund Mid-Cap Blend"
    UsMidCapBlend,
    /// "US Fund Mid-Cap Growth"
    UsMidCapGrowth,
    /// "US Fund Mid-Cap Value"
    UsMidCapValue,
    /// "US Fund Small Blend"
    UsSmallBlend,
    /// "US Fund Small Growth"
    UsSmallGrowth,
    /// "US Fund Small Value"
    UsSmallValue,
    /// "US Fund Diversified Emerging Markets"
    UsDiversifiedEmergingMarkets,
    /// "US Fund Foreign Large Blend"
    UsForeignLargeBlend,
    /// "US Fund Foreign Large Growth"
    UsForeignLargeGrowth,
    /// "US Fund Foreign Large Value"
    UsForeignLargeValue,
    /// "US Fund Foreign Small/Mid Blend"
    UsForeignSmallMidBlend,
    /// "US Fund Foreign Small/Mid Growth"
    UsForeignSmallMidGrowth,
    /// "US Fund Foreign Small/Mid Value"
    UsForeignSmallMidValue,
    /// "US Fund World Large-Stock Blend"
    UsWorldLargeStockBlend,
    /// "US Fund World Large-Stock Growth"
    UsWorldLargeStockGrowth,
    /// "US Fund World Small/Mid Stock"
    UsWorldSmallMidStock,

    // ── US Fixed Income ───────────────────────────────────────────────────────
    /// "US Fund Intermediate Core Bond"
    UsIntermediateCoreBond,
    /// "US Fund Intermediate Core-Plus Bond"
    UsIntermediateCorePlusBond,
    /// "US Fund Short-Term Bond"
    UsShortTermBond,
    /// "US Fund Long-Term Bond"
    UsLongTermBond,
    /// "US Fund Corporate Bond"
    UsCorporateBond,
    /// "US Fund High Yield Bond"
    UsHighYieldBond,
    /// "US Fund Ultrashort Bond"
    UsUltrashortBond,
    /// "US Fund Inflation-Protected Bond"
    UsInflationProtectedBond,
    /// "US Fund Multisector Bond"
    UsMultisectorBond,
    /// "US Fund Short Government"
    UsShortGovernment,
    /// "US Fund Intermediate Government"
    UsIntermediateGovernment,
    /// "US Fund Long Government"
    UsLongGovernment,
    /// "US Fund Muni National Interm"
    UsMuniNationalInterm,
    /// "US Fund Muni National Short"
    UsMuniNationalShort,
    /// "US Fund Muni National Long"
    UsMuniNationalLong,
    /// "US Fund Money Market Taxable"
    UsMoneyMarketTaxable,
    /// "US Fund Money Market Tax-Free"
    UsMoneyMarketTaxFree,
    /// "US Fund Bank Loan"
    UsBankLoan,
    /// "US Fund Emerging Markets Bond"
    UsEmergingMarketsBond,

    // ── US Allocation ─────────────────────────────────────────────────────────
    /// "US Fund Allocation--15% to 30% Equity"
    UsAllocation15To30Equity,
    /// "US Fund Allocation--30% to 50% Equity"
    UsAllocation30To50Equity,
    /// "US Fund Allocation--50% to 70% Equity"
    UsAllocation50To70Equity,
    /// "US Fund Allocation--70% to 85% Equity"
    UsAllocation70To85Equity,
    /// "US Fund Allocation--85%+ Equity"
    UsAllocation85PlusEquity,

    // ── US Sector Equity ──────────────────────────────────────────────────────
    /// "US Fund Real Estate"
    UsRealEstate,
    /// "US Fund Health"
    UsHealth,
    /// "US Fund Technology"
    UsTechnology,
    /// "US Fund Natural Resources"
    UsNaturalResources,
    /// "US Fund Utilities"
    UsUtilities,
    /// "US Fund Communications"
    UsCommunications,
    /// "US Fund Industrials"
    UsIndustrials,
    /// "US Fund Financial"
    UsFinancial,
    /// "US Fund Consumer Defensive"
    UsConsumerDefensive,
    /// "US Fund Consumer Cyclical"
    UsConsumerCyclical,
    /// "US Fund Energy Limited Partnership"
    UsEnergyLimitedPartnership,
    /// "US Fund Miscellaneous Sector"
    UsMiscellaneousSector,
    /// "US Fund Infrastructure"
    UsInfrastructure,

    // ── US Target-Date ────────────────────────────────────────────────────────
    /// "US Fund Target-Date 2025"
    UsTargetDate2025,
    /// "US Fund Target-Date 2030"
    UsTargetDate2030,
    /// "US Fund Target-Date 2035"
    UsTargetDate2035,
    /// "US Fund Target-Date 2040"
    UsTargetDate2040,
    /// "US Fund Target-Date 2045"
    UsTargetDate2045,
    /// "US Fund Target-Date 2050"
    UsTargetDate2050,
    /// "US Fund Target-Date 2055"
    UsTargetDate2055,
    /// "US Fund Target-Date 2060"
    UsTargetDate2060,
    /// "US Fund Target-Date 2065+"
    UsTargetDate2065Plus,
    /// "US Fund Target-Date Retirement"
    UsTargetDateRetirement,

    // ── US Other ──────────────────────────────────────────────────────────────
    /// "US Fund Convertibles"
    UsConvertibles,
    /// "US Fund Preferred Stock"
    UsPreferredStock,
    /// "US Fund Multialternative"
    UsMultialternative,
    /// "US Fund Options-based"
    UsOptionsBased,

    // ── EAA Fund Categories ───────────────────────────────────────────────────
    /// "EAA Fund Europe Large-Cap Blend Equity"
    EaaEuropeLargeCapBlendEquity,
    /// "EAA Fund Europe Large-Cap Growth Equity"
    EaaEuropeLargeCapGrowthEquity,
    /// "EAA Fund Europe Large-Cap Value Equity"
    EaaEuropeLargeCapValueEquity,
    /// "EAA Fund Global Large-Cap Blend Equity"
    EaaGlobalLargeCapBlendEquity,
    /// "EAA Fund Global Large-Cap Growth Equity"
    EaaGlobalLargeCapGrowthEquity,
    /// "EAA Fund EUR Diversified Bond"
    EaaEurDiversifiedBond,
    /// "EAA Fund EUR Corporate Bond"
    EaaEurCorporateBond,
    /// "EAA Fund USD Corporate Bond"
    EaaUsdCorporateBond,
    /// "EAA Fund USD High Yield Bond"
    EaaUsdHighYieldBond,
    /// "EAA Fund Global Bond"
    EaaGlobalBond,
    /// "EAA Fund USD Money Market"
    EaaUsdMoneyMarket,
    /// "EAA Fund EUR Money Market"
    EaaEurMoneyMarket,
}

impl ScreenerFundCategory {
    /// Returns the fund category name used by Yahoo Finance / Morningstar.
    pub fn as_str(self) -> &'static str {
        match self {
            ScreenerFundCategory::UsLargeBlend => "US Fund Large Blend",
            ScreenerFundCategory::UsLargeGrowth => "US Fund Large Growth",
            ScreenerFundCategory::UsLargeValue => "US Fund Large Value",
            ScreenerFundCategory::UsMidCapBlend => "US Fund Mid-Cap Blend",
            ScreenerFundCategory::UsMidCapGrowth => "US Fund Mid-Cap Growth",
            ScreenerFundCategory::UsMidCapValue => "US Fund Mid-Cap Value",
            ScreenerFundCategory::UsSmallBlend => "US Fund Small Blend",
            ScreenerFundCategory::UsSmallGrowth => "US Fund Small Growth",
            ScreenerFundCategory::UsSmallValue => "US Fund Small Value",
            ScreenerFundCategory::UsDiversifiedEmergingMarkets => {
                "US Fund Diversified Emerging Markets"
            }
            ScreenerFundCategory::UsForeignLargeBlend => "US Fund Foreign Large Blend",
            ScreenerFundCategory::UsForeignLargeGrowth => "US Fund Foreign Large Growth",
            ScreenerFundCategory::UsForeignLargeValue => "US Fund Foreign Large Value",
            ScreenerFundCategory::UsForeignSmallMidBlend => "US Fund Foreign Small/Mid Blend",
            ScreenerFundCategory::UsForeignSmallMidGrowth => "US Fund Foreign Small/Mid Growth",
            ScreenerFundCategory::UsForeignSmallMidValue => "US Fund Foreign Small/Mid Value",
            ScreenerFundCategory::UsWorldLargeStockBlend => "US Fund World Large-Stock Blend",
            ScreenerFundCategory::UsWorldLargeStockGrowth => "US Fund World Large-Stock Growth",
            ScreenerFundCategory::UsWorldSmallMidStock => "US Fund World Small/Mid Stock",
            ScreenerFundCategory::UsIntermediateCoreBond => "US Fund Intermediate Core Bond",
            ScreenerFundCategory::UsIntermediateCorePlusBond => {
                "US Fund Intermediate Core-Plus Bond"
            }
            ScreenerFundCategory::UsShortTermBond => "US Fund Short-Term Bond",
            ScreenerFundCategory::UsLongTermBond => "US Fund Long-Term Bond",
            ScreenerFundCategory::UsCorporateBond => "US Fund Corporate Bond",
            ScreenerFundCategory::UsHighYieldBond => "US Fund High Yield Bond",
            ScreenerFundCategory::UsUltrashortBond => "US Fund Ultrashort Bond",
            ScreenerFundCategory::UsInflationProtectedBond => "US Fund Inflation-Protected Bond",
            ScreenerFundCategory::UsMultisectorBond => "US Fund Multisector Bond",
            ScreenerFundCategory::UsShortGovernment => "US Fund Short Government",
            ScreenerFundCategory::UsIntermediateGovernment => "US Fund Intermediate Government",
            ScreenerFundCategory::UsLongGovernment => "US Fund Long Government",
            ScreenerFundCategory::UsMuniNationalInterm => "US Fund Muni National Interm",
            ScreenerFundCategory::UsMuniNationalShort => "US Fund Muni National Short",
            ScreenerFundCategory::UsMuniNationalLong => "US Fund Muni National Long",
            ScreenerFundCategory::UsMoneyMarketTaxable => "US Fund Money Market Taxable",
            ScreenerFundCategory::UsMoneyMarketTaxFree => "US Fund Money Market Tax-Free",
            ScreenerFundCategory::UsBankLoan => "US Fund Bank Loan",
            ScreenerFundCategory::UsEmergingMarketsBond => "US Fund Emerging Markets Bond",
            ScreenerFundCategory::UsAllocation15To30Equity => {
                "US Fund Allocation--15% to 30% Equity"
            }
            ScreenerFundCategory::UsAllocation30To50Equity => {
                "US Fund Allocation--30% to 50% Equity"
            }
            ScreenerFundCategory::UsAllocation50To70Equity => {
                "US Fund Allocation--50% to 70% Equity"
            }
            ScreenerFundCategory::UsAllocation70To85Equity => {
                "US Fund Allocation--70% to 85% Equity"
            }
            ScreenerFundCategory::UsAllocation85PlusEquity => "US Fund Allocation--85%+ Equity",
            ScreenerFundCategory::UsRealEstate => "US Fund Real Estate",
            ScreenerFundCategory::UsHealth => "US Fund Health",
            ScreenerFundCategory::UsTechnology => "US Fund Technology",
            ScreenerFundCategory::UsNaturalResources => "US Fund Natural Resources",
            ScreenerFundCategory::UsUtilities => "US Fund Utilities",
            ScreenerFundCategory::UsCommunications => "US Fund Communications",
            ScreenerFundCategory::UsIndustrials => "US Fund Industrials",
            ScreenerFundCategory::UsFinancial => "US Fund Financial",
            ScreenerFundCategory::UsConsumerDefensive => "US Fund Consumer Defensive",
            ScreenerFundCategory::UsConsumerCyclical => "US Fund Consumer Cyclical",
            ScreenerFundCategory::UsEnergyLimitedPartnership => {
                "US Fund Energy Limited Partnership"
            }
            ScreenerFundCategory::UsMiscellaneousSector => "US Fund Miscellaneous Sector",
            ScreenerFundCategory::UsInfrastructure => "US Fund Infrastructure",
            ScreenerFundCategory::UsTargetDate2025 => "US Fund Target-Date 2025",
            ScreenerFundCategory::UsTargetDate2030 => "US Fund Target-Date 2030",
            ScreenerFundCategory::UsTargetDate2035 => "US Fund Target-Date 2035",
            ScreenerFundCategory::UsTargetDate2040 => "US Fund Target-Date 2040",
            ScreenerFundCategory::UsTargetDate2045 => "US Fund Target-Date 2045",
            ScreenerFundCategory::UsTargetDate2050 => "US Fund Target-Date 2050",
            ScreenerFundCategory::UsTargetDate2055 => "US Fund Target-Date 2055",
            ScreenerFundCategory::UsTargetDate2060 => "US Fund Target-Date 2060",
            ScreenerFundCategory::UsTargetDate2065Plus => "US Fund Target-Date 2065+",
            ScreenerFundCategory::UsTargetDateRetirement => "US Fund Target-Date Retirement",
            ScreenerFundCategory::UsConvertibles => "US Fund Convertibles",
            ScreenerFundCategory::UsPreferredStock => "US Fund Preferred Stock",
            ScreenerFundCategory::UsMultialternative => "US Fund Multialternative",
            ScreenerFundCategory::UsOptionsBased => "US Fund Options-based",
            ScreenerFundCategory::EaaEuropeLargeCapBlendEquity => {
                "EAA Fund Europe Large-Cap Blend Equity"
            }
            ScreenerFundCategory::EaaEuropeLargeCapGrowthEquity => {
                "EAA Fund Europe Large-Cap Growth Equity"
            }
            ScreenerFundCategory::EaaEuropeLargeCapValueEquity => {
                "EAA Fund Europe Large-Cap Value Equity"
            }
            ScreenerFundCategory::EaaGlobalLargeCapBlendEquity => {
                "EAA Fund Global Large-Cap Blend Equity"
            }
            ScreenerFundCategory::EaaGlobalLargeCapGrowthEquity => {
                "EAA Fund Global Large-Cap Growth Equity"
            }
            ScreenerFundCategory::EaaEurDiversifiedBond => "EAA Fund EUR Diversified Bond",
            ScreenerFundCategory::EaaEurCorporateBond => "EAA Fund EUR Corporate Bond",
            ScreenerFundCategory::EaaUsdCorporateBond => "EAA Fund USD Corporate Bond",
            ScreenerFundCategory::EaaUsdHighYieldBond => "EAA Fund USD High Yield Bond",
            ScreenerFundCategory::EaaGlobalBond => "EAA Fund Global Bond",
            ScreenerFundCategory::EaaUsdMoneyMarket => "EAA Fund USD Money Market",
            ScreenerFundCategory::EaaEurMoneyMarket => "EAA Fund EUR Money Market",
        }
    }
}

impl From<ScreenerFundCategory> for String {
    fn from(v: ScreenerFundCategory) -> Self {
        v.as_str().to_string()
    }
}

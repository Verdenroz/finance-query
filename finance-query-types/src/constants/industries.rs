/// Typed industry identifier for the industry endpoint and custom screener queries.
///
/// See the module-level doc for usage.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Industry {
    // ── Agriculture / Raw Materials ──────────────────────────────────────
    /// Agricultural inputs, fertilizers, and crop chemicals
    #[serde(rename = "agricultural-inputs")]
    AgriculturalInputs,
    /// Aluminum production and processing companies
    #[serde(rename = "aluminum")]
    Aluminum,
    /// Coal mining and processing companies
    #[serde(rename = "coal")]
    Coal,
    /// Copper mining and processing companies
    #[serde(rename = "copper")]
    Copper,
    /// Farm products including grains, livestock, and produce
    #[serde(rename = "farm-products")]
    FarmProducts,
    /// Forest products including timber and paper pulp
    #[serde(rename = "forest-products")]
    ForestProducts,
    /// Gold mining and royalty companies
    #[serde(rename = "gold")]
    Gold,
    /// Lumber and wood production companies
    #[serde(rename = "lumber-wood-production")]
    LumberAndWoodProduction,
    /// Other industrial metals and mining (zinc, nickel, etc.)
    #[serde(rename = "other-industrial-metals-mining")]
    OtherIndustrialMetalsAndMining,
    /// Other precious metals and mining (platinum, palladium, etc.)
    #[serde(rename = "other-precious-metals-mining")]
    OtherPreciousMetalsAndMining,
    /// Silver mining and streaming companies
    #[serde(rename = "silver")]
    Silver,
    /// Steel production and processing companies
    #[serde(rename = "steel")]
    Steel,
    /// Thermal coal mining for electricity generation
    #[serde(rename = "thermal-coal")]
    ThermalCoal,
    /// Uranium mining companies
    #[serde(rename = "uranium")]
    Uranium,
    // ── Consumer ─────────────────────────────────────────────────────────
    /// Clothing and apparel manufacturing companies
    #[serde(rename = "apparel-manufacturing")]
    ApparelManufacturing,
    /// Clothing and apparel retail chains
    #[serde(rename = "apparel-retail")]
    ApparelRetail,
    /// Automotive and truck dealerships
    #[serde(rename = "auto-truck-dealerships")]
    AutoAndTruckDealerships,
    /// Automobile manufacturers and assemblers
    #[serde(rename = "auto-manufacturers")]
    AutoManufacturers,
    /// Automotive parts manufacturers and distributors
    #[serde(rename = "auto-parts")]
    AutoParts,
    /// Beer brewing and distribution companies
    #[serde(rename = "beverages-brewers")]
    BeveragesBrewers,
    /// Non-alcoholic beverages including soft drinks and juices
    #[serde(rename = "beverages-non-alcoholic")]
    BeveragesNonAlcoholic,
    /// Wineries, distilleries, and spirits producers
    #[serde(rename = "beverages-wineries-distilleries")]
    BeveragesWineriesAndDistilleries,
    /// Candy, chocolate, and confectionery makers
    #[serde(rename = "confectioners")]
    Confectioners,
    /// Traditional department store retailers
    #[serde(rename = "department-stores")]
    DepartmentStores,
    /// Discount and value retail stores
    #[serde(rename = "discount-stores")]
    DiscountStores,
    /// Electronic gaming software and multimedia entertainment
    #[serde(rename = "electronic-gaming-multimedia")]
    ElectronicGamingAndMultimedia,
    /// Wholesale food distribution companies
    #[serde(rename = "food-distribution")]
    FoodDistribution,
    /// Footwear, handbags, and fashion accessories
    #[serde(rename = "footwear-accessories")]
    FootwearAndAccessories,
    /// Furniture, fixtures, and household appliances
    #[serde(rename = "furnishings-fixtures-appliances")]
    FurnishingsFixturesAndAppliances,
    /// Casinos, online gambling, and gaming operators
    #[serde(rename = "gambling")]
    Gambling,
    /// Supermarkets and grocery retail chains
    #[serde(rename = "grocery-stores")]
    GroceryStores,
    /// Home improvement retail stores
    #[serde(rename = "home-improvement-retail")]
    HomeImprovementRetail,
    /// Household cleaning products and personal care items
    #[serde(rename = "household-personal-products")]
    HouseholdAndPersonalProducts,
    /// Online retail and e-commerce marketplaces
    #[serde(rename = "internet-retail")]
    InternetRetail,
    /// Leisure, recreation, and entertainment companies
    #[serde(rename = "leisure")]
    Leisure,
    /// Hotels and lodging companies
    #[serde(rename = "lodging")]
    Lodging,
    /// Luxury goods, fashion, and premium consumer brands
    #[serde(rename = "luxury-goods")]
    LuxuryGoods,
    /// Packaged and processed food manufacturers
    #[serde(rename = "packaged-foods")]
    PackagedFoods,
    /// Personal care, laundry, and household services
    #[serde(rename = "personal-services")]
    PersonalServices,
    /// Home builders and residential construction
    #[serde(rename = "residential-construction")]
    ResidentialConstruction,
    /// Resorts, integrated casinos, and hotel-casinos
    #[serde(rename = "resorts-casinos")]
    ResortsAndCasinos,
    /// Restaurant chains and food service operators
    #[serde(rename = "restaurants")]
    Restaurants,
    /// Specialty retail stores (pets, books, electronics, etc.)
    #[serde(rename = "specialty-retail")]
    SpecialtyRetail,
    /// Textile and fabric manufacturers
    #[serde(rename = "textile-manufacturing")]
    TextileManufacturing,
    /// Tobacco product manufacturers
    #[serde(rename = "tobacco")]
    Tobacco,
    /// Travel agencies, booking platforms, and tour operators
    #[serde(rename = "travel-services")]
    TravelServices,
    // ── Energy ───────────────────────────────────────────────────────────
    /// Oil and gas contract drilling services
    #[serde(rename = "oil-gas-drilling")]
    OilAndGasDrilling,
    /// Oil and gas exploration and production companies
    #[serde(rename = "oil-gas-ep")]
    OilAndGasEAndP,
    /// Oil field equipment, services, and engineering
    #[serde(rename = "oil-gas-equipment-services")]
    OilAndGasEquipmentAndServices,
    /// Vertically integrated oil and gas majors
    #[serde(rename = "oil-gas-integrated")]
    OilAndGasIntegrated,
    /// Oil and gas pipelines, storage, and transportation
    #[serde(rename = "oil-gas-midstream")]
    OilAndGasMidstream,
    /// Oil refining, wholesale fuel, and marketing
    #[serde(rename = "oil-gas-refining-marketing")]
    OilAndGasRefiningAndMarketing,
    /// Solar panel manufacturers and solar energy producers
    #[serde(rename = "solar")]
    Solar,
    // ── Financial Services ───────────────────────────────────────────────
    /// Asset managers, fund sponsors, and investment advisors
    #[serde(rename = "asset-management")]
    AssetManagement,
    /// Large diversified national and international banks
    #[serde(rename = "banks-diversified")]
    BanksDiversified,
    /// Regional and community banks
    #[serde(rename = "banks-regional")]
    BanksRegional,
    /// Investment banks, brokers, and financial exchanges
    #[serde(rename = "capital-markets")]
    CapitalMarkets,
    /// Credit card issuers and consumer credit services
    #[serde(rename = "credit-services")]
    CreditServices,
    /// Financial data, analytics, and stock exchange operators
    #[serde(rename = "financial-data-stock-exchanges")]
    FinancialDataAndStockExchanges,
    /// Insurance brokers and agencies
    #[serde(rename = "insurance-brokers")]
    InsuranceBrokers,
    /// Diversified multi-line insurance companies
    #[serde(rename = "insurance-diversified")]
    InsuranceDiversified,
    /// Life insurance and annuity providers
    #[serde(rename = "insurance-life")]
    InsuranceLife,
    /// Property and casualty insurance companies
    #[serde(rename = "insurance-property-casualty")]
    InsurancePropertyAndCasualty,
    /// Reinsurance companies
    #[serde(rename = "insurance-reinsurance")]
    InsuranceReinsurance,
    /// Specialty insurance lines (title, mortgage, etc.)
    #[serde(rename = "insurance-specialty")]
    InsuranceSpecialty,
    /// Mortgage banking and loan origination
    #[serde(rename = "mortgage-finance")]
    MortgageFinance,
    /// Blank-check and shell holding companies
    #[serde(rename = "shell-companies")]
    ShellCompanies,
    // ── Healthcare ───────────────────────────────────────────────────────
    /// Biotechnology drug development companies
    #[serde(rename = "biotechnology")]
    Biotechnology,
    /// Medical diagnostics labs and clinical research
    #[serde(rename = "diagnostics-research")]
    DiagnosticsAndResearch,
    /// Large branded pharmaceutical manufacturers
    #[serde(rename = "drug-manufacturers-general")]
    DrugManufacturersGeneral,
    /// Specialty drugs, generics, and biosimilars
    #[serde(rename = "drug-manufacturers-specialty-generic")]
    DrugManufacturersSpecialtyAndGeneric,
    /// Healthcare IT, EHR, and health data services
    #[serde(rename = "health-information-services")]
    HealthInformationServices,
    /// Managed care organizations and health insurers
    #[serde(rename = "healthcare-plans")]
    HealthcarePlans,
    /// Hospitals, clinics, and outpatient care facilities
    #[serde(rename = "medical-care-facilities")]
    MedicalCareFacilities,
    /// Medical device manufacturers (implants, diagnostics equipment)
    #[serde(rename = "medical-devices")]
    MedicalDevices,
    /// Medical product wholesalers and distributors
    #[serde(rename = "medical-distribution")]
    MedicalDistribution,
    /// Surgical instruments, disposables, and medical supplies
    #[serde(rename = "medical-instruments-supplies")]
    MedicalInstrumentsAndSupplies,
    /// Retail pharmacies and drug store chains
    #[serde(rename = "pharmaceutical-retailers")]
    PharmaceuticalRetailers,
    // ── Industrials ──────────────────────────────────────────────────────
    /// Defense contractors, aircraft, and space systems
    #[serde(rename = "aerospace-defense")]
    AerospaceAndDefense,
    /// Construction aggregates, cement, and building materials
    #[serde(rename = "building-materials")]
    BuildingMaterials,
    /// HVAC, plumbing, windows, and building equipment
    #[serde(rename = "building-products-equipment")]
    BuildingProductsAndEquipment,
    /// Office supplies, commercial equipment, and printers
    #[serde(rename = "business-equipment-supplies")]
    BusinessEquipmentAndSupplies,
    /// Specialty chemical manufacturing for industrial use
    #[serde(rename = "chemical-manufacturing")]
    ChemicalManufacturing,
    /// Diversified commodity chemicals producers
    #[serde(rename = "chemicals")]
    Chemicals,
    /// Diversified industrial holding companies
    #[serde(rename = "conglomerates")]
    Conglomerates,
    /// Management consulting and professional advisory services
    #[serde(rename = "consulting-services")]
    ConsultingServices,
    /// Electrical components, motors, and power equipment
    #[serde(rename = "electrical-equipment-parts")]
    ElectricalEquipmentAndParts,
    /// Civil engineering, construction, and infrastructure projects
    #[serde(rename = "engineering-construction")]
    EngineeringAndConstruction,
    /// Agricultural equipment and heavy construction machinery
    #[serde(rename = "farm-heavy-construction-machinery")]
    FarmAndHeavyConstructionMachinery,
    /// Industrial goods wholesalers and distributors
    #[serde(rename = "industrial-distribution")]
    IndustrialDistribution,
    /// Toll roads, airports, and infrastructure operators
    #[serde(rename = "infrastructure-operations")]
    InfrastructureOperations,
    /// Third-party logistics and supply chain management
    #[serde(rename = "integrated-freight-logistics")]
    IntegratedFreightAndLogistics,
    /// Diversified manufacturers across multiple industrial segments
    #[serde(rename = "manufacturing-diversified")]
    ManufacturingDiversified,
    /// Port operators and marine terminal services
    #[serde(rename = "marine-ports-services")]
    MarinePortsAndServices,
    /// Bulk cargo and tanker shipping companies
    #[serde(rename = "marine-shipping")]
    MarineShipping,
    /// Custom metal fabrication and machined components
    #[serde(rename = "metal-fabrication")]
    MetalFabrication,
    /// Paper, packaging, and pulp product manufacturers
    #[serde(rename = "paper-paper-products")]
    PaperAndPaperProducts,
    /// Environmental controls, water treatment, and remediation
    #[serde(rename = "pollution-treatment-controls")]
    PollutionAndTreatmentControls,
    /// Rail freight carriers and passenger rail operators
    #[serde(rename = "railroads")]
    Railroads,
    /// Equipment rental, leasing, and fleet management
    #[serde(rename = "rental-leasing-services")]
    RentalAndLeasingServices,
    /// Security systems, guards, and monitoring services
    #[serde(rename = "security-protection-services")]
    SecurityAndProtectionServices,
    /// Outsourced business services and BPO companies
    #[serde(rename = "specialty-business-services")]
    SpecialtyBusinessServices,
    /// High-value specialty chemicals and advanced materials
    #[serde(rename = "specialty-chemicals")]
    SpecialtyChemicals,
    /// Specialized industrial machinery and equipment makers
    #[serde(rename = "specialty-industrial-machinery")]
    SpecialtyIndustrialMachinery,
    /// Staffing agencies and employment service providers
    #[serde(rename = "staffing-employment-services")]
    StaffingAndEmploymentServices,
    /// Hand tools, power tools, and hardware accessories
    #[serde(rename = "tools-accessories")]
    ToolsAndAccessories,
    /// Freight trucking and less-than-truckload carriers
    #[serde(rename = "trucking")]
    Trucking,
    /// Waste collection, recycling, and disposal services
    #[serde(rename = "waste-management")]
    WasteManagement,
    // ── Real Estate ──────────────────────────────────────────────────────
    /// Real estate developers and homebuilders
    #[serde(rename = "real-estate-development")]
    RealEstateDevelopment,
    /// Diversified real estate companies with mixed portfolios
    #[serde(rename = "real-estate-diversified")]
    RealEstateDiversified,
    /// Real estate brokers, agents, and property managers
    #[serde(rename = "real-estate-services")]
    RealEstateServices,
    /// Diversified REITs across multiple property types
    #[serde(rename = "reit-diversified")]
    ReitDiversified,
    /// Healthcare and senior living facility REITs
    #[serde(rename = "reit-healthcare-facilities")]
    ReitHealthcareFacilities,
    /// Hotel and motel property REITs
    #[serde(rename = "reit-hotel-motel")]
    ReitHotelAndMotel,
    /// Industrial, warehouse, and logistics property REITs
    #[serde(rename = "reit-industrial")]
    ReitIndustrial,
    /// Mortgage REITs investing in real estate debt
    #[serde(rename = "reit-mortgage")]
    ReitMortgage,
    /// Office building and commercial property REITs
    #[serde(rename = "reit-office")]
    ReitOffice,
    /// Apartment, multifamily, and residential property REITs
    #[serde(rename = "reit-residential")]
    ReitResidential,
    /// Shopping center and retail property REITs
    #[serde(rename = "reit-retail")]
    ReitRetail,
    /// Specialty REITs (data centers, cell towers, self-storage)
    #[serde(rename = "reit-specialty")]
    ReitSpecialty,
    // ── Technology ───────────────────────────────────────────────────────
    /// Networking hardware, routers, and communication equipment
    #[serde(rename = "communication-equipment")]
    CommunicationEquipment,
    /// PCs, servers, and computer hardware manufacturers
    #[serde(rename = "computer-hardware")]
    ComputerHardware,
    /// Smartphones, TVs, and consumer electronic devices
    #[serde(rename = "consumer-electronics")]
    ConsumerElectronics,
    /// Data analytics, business intelligence, and AI platforms
    #[serde(rename = "data-analytics")]
    DataAnalytics,
    /// Passive electronic components and circuit boards
    #[serde(rename = "electronic-components")]
    ElectronicComponents,
    /// Distributors of electronics and computer products
    #[serde(rename = "electronics-computer-distribution")]
    ElectronicsAndComputerDistribution,
    /// Value-added resellers and software/hardware distributors
    #[serde(rename = "hardware-software-distribution")]
    HardwareAndSoftwareDistribution,
    /// IT services, outsourcing, and technology consulting
    #[serde(rename = "information-technology-services")]
    InformationTechnologyServices,
    /// Online media, search engines, and digital content platforms
    #[serde(rename = "internet-content-information")]
    InternetContentAndInformation,
    /// Precision instruments, sensors, and test equipment
    #[serde(rename = "scientific-technical-instruments")]
    ScientificAndTechnicalInstruments,
    /// Semiconductor manufacturing equipment and materials
    #[serde(rename = "semiconductor-equipment-materials")]
    SemiconductorEquipmentAndMaterials,
    /// Integrated circuit and chip designers and manufacturers
    #[serde(rename = "semiconductors")]
    Semiconductors,
    /// Business application software companies
    #[serde(rename = "software-application")]
    SoftwareApplication,
    /// Operating systems, middleware, and infrastructure software
    #[serde(rename = "software-infrastructure")]
    SoftwareInfrastructure,
    // ── Communication Services ───────────────────────────────────────────
    /// Television, radio, and broadcast media companies
    #[serde(rename = "broadcasting")]
    Broadcasting,
    /// Film studios, streaming, and live entertainment
    #[serde(rename = "entertainment")]
    Entertainment,
    /// Book, magazine, newspaper, and digital media publishers
    #[serde(rename = "publishing")]
    Publishing,
    /// Wireless carriers and wireline telephone companies
    #[serde(rename = "telecom-services")]
    TelecomServices,
    // ── Utilities ────────────────────────────────────────────────────────
    /// Multi-utility companies serving electricity, gas, and water
    #[serde(rename = "utilities-diversified")]
    UtilitiesDiversified,
    /// Independent power producers and energy traders
    #[serde(rename = "utilities-independent-power-producers")]
    UtilitiesIndependentPowerProducers,
    /// Regulated electric utility companies
    #[serde(rename = "utilities-regulated-electric")]
    UtilitiesRegulatedElectric,
    /// Regulated natural gas distribution utilities
    #[serde(rename = "utilities-regulated-gas")]
    UtilitiesRegulatedGas,
    /// Regulated water and wastewater utilities
    #[serde(rename = "utilities-regulated-water")]
    UtilitiesRegulatedWater,
    /// Renewable energy generation companies (wind, solar, hydro)
    #[serde(rename = "utilities-renewable")]
    UtilitiesRenewable,
    // ── Special ──────────────────────────────────────────────────────────
    /// Closed-end funds investing in debt instruments
    #[serde(rename = "closed-end-fund-debt")]
    ClosedEndFundDebt,
    /// Closed-end funds investing in equities
    #[serde(rename = "closed-end-fund-equity")]
    ClosedEndFundEquity,
    /// Closed-end funds investing in foreign securities
    #[serde(rename = "closed-end-fund-foreign")]
    ClosedEndFundForeign,
    /// Exchange-traded fund products
    #[serde(rename = "exchange-traded-fund")]
    ExchangeTradedFund,
}

impl Industry {
    /// Returns the lowercase hyphenated slug used by `finance::industry()`.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Industry;
    /// assert_eq!(Industry::Semiconductors.as_slug(), "semiconductors");
    /// assert_eq!(Industry::SoftwareApplication.as_slug(), "software-application");
    /// ```
    pub fn as_slug(self) -> &'static str {
        match self {
            Industry::AgriculturalInputs => "agricultural-inputs",
            Industry::Aluminum => "aluminum",
            Industry::Coal => "coal",
            Industry::Copper => "copper",
            Industry::FarmProducts => "farm-products",
            Industry::ForestProducts => "forest-products",
            Industry::Gold => "gold",
            Industry::LumberAndWoodProduction => "lumber-wood-production",
            Industry::OtherIndustrialMetalsAndMining => "other-industrial-metals-mining",
            Industry::OtherPreciousMetalsAndMining => "other-precious-metals-mining",
            Industry::Silver => "silver",
            Industry::Steel => "steel",
            Industry::ThermalCoal => "thermal-coal",
            Industry::Uranium => "uranium",
            Industry::ApparelManufacturing => "apparel-manufacturing",
            Industry::ApparelRetail => "apparel-retail",
            Industry::AutoAndTruckDealerships => "auto-truck-dealerships",
            Industry::AutoManufacturers => "auto-manufacturers",
            Industry::AutoParts => "auto-parts",
            Industry::BeveragesBrewers => "beverages-brewers",
            Industry::BeveragesNonAlcoholic => "beverages-non-alcoholic",
            Industry::BeveragesWineriesAndDistilleries => "beverages-wineries-distilleries",
            Industry::Confectioners => "confectioners",
            Industry::DepartmentStores => "department-stores",
            Industry::DiscountStores => "discount-stores",
            Industry::ElectronicGamingAndMultimedia => "electronic-gaming-multimedia",
            Industry::FoodDistribution => "food-distribution",
            Industry::FootwearAndAccessories => "footwear-accessories",
            Industry::FurnishingsFixturesAndAppliances => "furnishings-fixtures-appliances",
            Industry::Gambling => "gambling",
            Industry::GroceryStores => "grocery-stores",
            Industry::HomeImprovementRetail => "home-improvement-retail",
            Industry::HouseholdAndPersonalProducts => "household-personal-products",
            Industry::InternetRetail => "internet-retail",
            Industry::Leisure => "leisure",
            Industry::Lodging => "lodging",
            Industry::LuxuryGoods => "luxury-goods",
            Industry::PackagedFoods => "packaged-foods",
            Industry::PersonalServices => "personal-services",
            Industry::ResidentialConstruction => "residential-construction",
            Industry::ResortsAndCasinos => "resorts-casinos",
            Industry::Restaurants => "restaurants",
            Industry::SpecialtyRetail => "specialty-retail",
            Industry::TextileManufacturing => "textile-manufacturing",
            Industry::Tobacco => "tobacco",
            Industry::TravelServices => "travel-services",
            Industry::OilAndGasDrilling => "oil-gas-drilling",
            Industry::OilAndGasEAndP => "oil-gas-ep",
            Industry::OilAndGasEquipmentAndServices => "oil-gas-equipment-services",
            Industry::OilAndGasIntegrated => "oil-gas-integrated",
            Industry::OilAndGasMidstream => "oil-gas-midstream",
            Industry::OilAndGasRefiningAndMarketing => "oil-gas-refining-marketing",
            Industry::Solar => "solar",
            Industry::AssetManagement => "asset-management",
            Industry::BanksDiversified => "banks-diversified",
            Industry::BanksRegional => "banks-regional",
            Industry::CapitalMarkets => "capital-markets",
            Industry::CreditServices => "credit-services",
            Industry::FinancialDataAndStockExchanges => "financial-data-stock-exchanges",
            Industry::InsuranceBrokers => "insurance-brokers",
            Industry::InsuranceDiversified => "insurance-diversified",
            Industry::InsuranceLife => "insurance-life",
            Industry::InsurancePropertyAndCasualty => "insurance-property-casualty",
            Industry::InsuranceReinsurance => "insurance-reinsurance",
            Industry::InsuranceSpecialty => "insurance-specialty",
            Industry::MortgageFinance => "mortgage-finance",
            Industry::ShellCompanies => "shell-companies",
            Industry::Biotechnology => "biotechnology",
            Industry::DiagnosticsAndResearch => "diagnostics-research",
            Industry::DrugManufacturersGeneral => "drug-manufacturers-general",
            Industry::DrugManufacturersSpecialtyAndGeneric => {
                "drug-manufacturers-specialty-generic"
            }
            Industry::HealthInformationServices => "health-information-services",
            Industry::HealthcarePlans => "healthcare-plans",
            Industry::MedicalCareFacilities => "medical-care-facilities",
            Industry::MedicalDevices => "medical-devices",
            Industry::MedicalDistribution => "medical-distribution",
            Industry::MedicalInstrumentsAndSupplies => "medical-instruments-supplies",
            Industry::PharmaceuticalRetailers => "pharmaceutical-retailers",
            Industry::AerospaceAndDefense => "aerospace-defense",
            Industry::BuildingMaterials => "building-materials",
            Industry::BuildingProductsAndEquipment => "building-products-equipment",
            Industry::BusinessEquipmentAndSupplies => "business-equipment-supplies",
            Industry::ChemicalManufacturing => "chemical-manufacturing",
            Industry::Chemicals => "chemicals",
            Industry::Conglomerates => "conglomerates",
            Industry::ConsultingServices => "consulting-services",
            Industry::ElectricalEquipmentAndParts => "electrical-equipment-parts",
            Industry::EngineeringAndConstruction => "engineering-construction",
            Industry::FarmAndHeavyConstructionMachinery => "farm-heavy-construction-machinery",
            Industry::IndustrialDistribution => "industrial-distribution",
            Industry::InfrastructureOperations => "infrastructure-operations",
            Industry::IntegratedFreightAndLogistics => "integrated-freight-logistics",
            Industry::ManufacturingDiversified => "manufacturing-diversified",
            Industry::MarinePortsAndServices => "marine-ports-services",
            Industry::MarineShipping => "marine-shipping",
            Industry::MetalFabrication => "metal-fabrication",
            Industry::PaperAndPaperProducts => "paper-paper-products",
            Industry::PollutionAndTreatmentControls => "pollution-treatment-controls",
            Industry::Railroads => "railroads",
            Industry::RentalAndLeasingServices => "rental-leasing-services",
            Industry::SecurityAndProtectionServices => "security-protection-services",
            Industry::SpecialtyBusinessServices => "specialty-business-services",
            Industry::SpecialtyChemicals => "specialty-chemicals",
            Industry::SpecialtyIndustrialMachinery => "specialty-industrial-machinery",
            Industry::StaffingAndEmploymentServices => "staffing-employment-services",
            Industry::ToolsAndAccessories => "tools-accessories",
            Industry::Trucking => "trucking",
            Industry::WasteManagement => "waste-management",
            Industry::RealEstateDevelopment => "real-estate-development",
            Industry::RealEstateDiversified => "real-estate-diversified",
            Industry::RealEstateServices => "real-estate-services",
            Industry::ReitDiversified => "reit-diversified",
            Industry::ReitHealthcareFacilities => "reit-healthcare-facilities",
            Industry::ReitHotelAndMotel => "reit-hotel-motel",
            Industry::ReitIndustrial => "reit-industrial",
            Industry::ReitMortgage => "reit-mortgage",
            Industry::ReitOffice => "reit-office",
            Industry::ReitResidential => "reit-residential",
            Industry::ReitRetail => "reit-retail",
            Industry::ReitSpecialty => "reit-specialty",
            Industry::CommunicationEquipment => "communication-equipment",
            Industry::ComputerHardware => "computer-hardware",
            Industry::ConsumerElectronics => "consumer-electronics",
            Industry::DataAnalytics => "data-analytics",
            Industry::ElectronicComponents => "electronic-components",
            Industry::ElectronicsAndComputerDistribution => "electronics-computer-distribution",
            Industry::HardwareAndSoftwareDistribution => "hardware-software-distribution",
            Industry::InformationTechnologyServices => "information-technology-services",
            Industry::InternetContentAndInformation => "internet-content-information",
            Industry::ScientificAndTechnicalInstruments => "scientific-technical-instruments",
            Industry::SemiconductorEquipmentAndMaterials => "semiconductor-equipment-materials",
            Industry::Semiconductors => "semiconductors",
            Industry::SoftwareApplication => "software-application",
            Industry::SoftwareInfrastructure => "software-infrastructure",
            Industry::Broadcasting => "broadcasting",
            Industry::Entertainment => "entertainment",
            Industry::Publishing => "publishing",
            Industry::TelecomServices => "telecom-services",
            Industry::UtilitiesDiversified => "utilities-diversified",
            Industry::UtilitiesIndependentPowerProducers => "utilities-independent-power-producers",
            Industry::UtilitiesRegulatedElectric => "utilities-regulated-electric",
            Industry::UtilitiesRegulatedGas => "utilities-regulated-gas",
            Industry::UtilitiesRegulatedWater => "utilities-regulated-water",
            Industry::UtilitiesRenewable => "utilities-renewable",
            Industry::ClosedEndFundDebt => "closed-end-fund-debt",
            Industry::ClosedEndFundEquity => "closed-end-fund-equity",
            Industry::ClosedEndFundForeign => "closed-end-fund-foreign",
            Industry::ExchangeTradedFund => "exchange-traded-fund",
        }
    }

    /// Returns the display name used by the Yahoo Finance screener.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::Industry;
    /// assert_eq!(Industry::Semiconductors.screener_value(), "Semiconductors");
    /// assert_eq!(Industry::OilAndGasDrilling.screener_value(), "Oil & Gas Drilling");
    /// ```
    pub fn screener_value(self) -> &'static str {
        match self {
            Industry::AgriculturalInputs => "Agricultural Inputs",
            Industry::Aluminum => "Aluminum",
            Industry::Coal => "Coal",
            Industry::Copper => "Copper",
            Industry::FarmProducts => "Farm Products",
            Industry::ForestProducts => "Forest Products",
            Industry::Gold => "Gold",
            Industry::LumberAndWoodProduction => "Lumber & Wood Production",
            Industry::OtherIndustrialMetalsAndMining => "Other Industrial Metals & Mining",
            Industry::OtherPreciousMetalsAndMining => "Other Precious Metals & Mining",
            Industry::Silver => "Silver",
            Industry::Steel => "Steel",
            Industry::ThermalCoal => "Thermal Coal",
            Industry::Uranium => "Uranium",
            Industry::ApparelManufacturing => "Apparel Manufacturing",
            Industry::ApparelRetail => "Apparel Retail",
            Industry::AutoAndTruckDealerships => "Auto & Truck Dealerships",
            Industry::AutoManufacturers => "Auto Manufacturers",
            Industry::AutoParts => "Auto Parts",
            Industry::BeveragesBrewers => "Beverages - Brewers",
            Industry::BeveragesNonAlcoholic => "Beverages - Non-Alcoholic",
            Industry::BeveragesWineriesAndDistilleries => "Beverages - Wineries & Distilleries",
            Industry::Confectioners => "Confectioners",
            Industry::DepartmentStores => "Department Stores",
            Industry::DiscountStores => "Discount Stores",
            Industry::ElectronicGamingAndMultimedia => "Electronic Gaming & Multimedia",
            Industry::FoodDistribution => "Food Distribution",
            Industry::FootwearAndAccessories => "Footwear & Accessories",
            Industry::FurnishingsFixturesAndAppliances => "Furnishings, Fixtures & Appliances",
            Industry::Gambling => "Gambling",
            Industry::GroceryStores => "Grocery Stores",
            Industry::HomeImprovementRetail => "Home Improvement Retail",
            Industry::HouseholdAndPersonalProducts => "Household & Personal Products",
            Industry::InternetRetail => "Internet Retail",
            Industry::Leisure => "Leisure",
            Industry::Lodging => "Lodging",
            Industry::LuxuryGoods => "Luxury Goods",
            Industry::PackagedFoods => "Packaged Foods",
            Industry::PersonalServices => "Personal Services",
            Industry::ResidentialConstruction => "Residential Construction",
            Industry::ResortsAndCasinos => "Resorts & Casinos",
            Industry::Restaurants => "Restaurants",
            Industry::SpecialtyRetail => "Specialty Retail",
            Industry::TextileManufacturing => "Textile Manufacturing",
            Industry::Tobacco => "Tobacco",
            Industry::TravelServices => "Travel Services",
            Industry::OilAndGasDrilling => "Oil & Gas Drilling",
            Industry::OilAndGasEAndP => "Oil & Gas E&P",
            Industry::OilAndGasEquipmentAndServices => "Oil & Gas Equipment & Services",
            Industry::OilAndGasIntegrated => "Oil & Gas Integrated",
            Industry::OilAndGasMidstream => "Oil & Gas Midstream",
            Industry::OilAndGasRefiningAndMarketing => "Oil & Gas Refining & Marketing",
            Industry::Solar => "Solar",
            Industry::AssetManagement => "Asset Management",
            Industry::BanksDiversified => "Banks - Diversified",
            Industry::BanksRegional => "Banks - Regional",
            Industry::CapitalMarkets => "Capital Markets",
            Industry::CreditServices => "Credit Services",
            Industry::FinancialDataAndStockExchanges => "Financial Data & Stock Exchanges",
            Industry::InsuranceBrokers => "Insurance Brokers",
            Industry::InsuranceDiversified => "Insurance - Diversified",
            Industry::InsuranceLife => "Insurance - Life",
            Industry::InsurancePropertyAndCasualty => "Insurance - Property & Casualty",
            Industry::InsuranceReinsurance => "Insurance - Reinsurance",
            Industry::InsuranceSpecialty => "Insurance - Specialty",
            Industry::MortgageFinance => "Mortgage Finance",
            Industry::ShellCompanies => "Shell Companies",
            Industry::Biotechnology => "Biotechnology",
            Industry::DiagnosticsAndResearch => "Diagnostics & Research",
            Industry::DrugManufacturersGeneral => "Drug Manufacturers - General",
            Industry::DrugManufacturersSpecialtyAndGeneric => {
                "Drug Manufacturers - Specialty & Generic"
            }
            Industry::HealthInformationServices => "Health Information Services",
            Industry::HealthcarePlans => "Healthcare Plans",
            Industry::MedicalCareFacilities => "Medical Care Facilities",
            Industry::MedicalDevices => "Medical Devices",
            Industry::MedicalDistribution => "Medical Distribution",
            Industry::MedicalInstrumentsAndSupplies => "Medical Instruments & Supplies",
            Industry::PharmaceuticalRetailers => "Pharmaceutical Retailers",
            Industry::AerospaceAndDefense => "Aerospace & Defense",
            Industry::BuildingMaterials => "Building Materials",
            Industry::BuildingProductsAndEquipment => "Building Products & Equipment",
            Industry::BusinessEquipmentAndSupplies => "Business Equipment & Supplies",
            Industry::ChemicalManufacturing => "Chemical Manufacturing",
            Industry::Chemicals => "Chemicals",
            Industry::Conglomerates => "Conglomerates",
            Industry::ConsultingServices => "Consulting Services",
            Industry::ElectricalEquipmentAndParts => "Electrical Equipment & Parts",
            Industry::EngineeringAndConstruction => "Engineering & Construction",
            Industry::FarmAndHeavyConstructionMachinery => "Farm & Heavy Construction Machinery",
            Industry::IndustrialDistribution => "Industrial Distribution",
            Industry::InfrastructureOperations => "Infrastructure Operations",
            Industry::IntegratedFreightAndLogistics => "Integrated Freight & Logistics",
            Industry::ManufacturingDiversified => "Manufacturing - Diversified",
            Industry::MarinePortsAndServices => "Marine Ports & Services",
            Industry::MarineShipping => "Marine Shipping",
            Industry::MetalFabrication => "Metal Fabrication",
            Industry::PaperAndPaperProducts => "Paper & Paper Products",
            Industry::PollutionAndTreatmentControls => "Pollution & Treatment Controls",
            Industry::Railroads => "Railroads",
            Industry::RentalAndLeasingServices => "Rental & Leasing Services",
            Industry::SecurityAndProtectionServices => "Security & Protection Services",
            Industry::SpecialtyBusinessServices => "Specialty Business Services",
            Industry::SpecialtyChemicals => "Specialty Chemicals",
            Industry::SpecialtyIndustrialMachinery => "Specialty Industrial Machinery",
            Industry::StaffingAndEmploymentServices => "Staffing & Employment Services",
            Industry::ToolsAndAccessories => "Tools & Accessories",
            Industry::Trucking => "Trucking",
            Industry::WasteManagement => "Waste Management",
            Industry::RealEstateDevelopment => "Real Estate - Development",
            Industry::RealEstateDiversified => "Real Estate - Diversified",
            Industry::RealEstateServices => "Real Estate Services",
            Industry::ReitDiversified => "REIT - Diversified",
            Industry::ReitHealthcareFacilities => "REIT - Healthcare Facilities",
            Industry::ReitHotelAndMotel => "REIT - Hotel & Motel",
            Industry::ReitIndustrial => "REIT - Industrial",
            Industry::ReitMortgage => "REIT - Mortgage",
            Industry::ReitOffice => "REIT - Office",
            Industry::ReitResidential => "REIT - Residential",
            Industry::ReitRetail => "REIT - Retail",
            Industry::ReitSpecialty => "REIT - Specialty",
            Industry::CommunicationEquipment => "Communication Equipment",
            Industry::ComputerHardware => "Computer Hardware",
            Industry::ConsumerElectronics => "Consumer Electronics",
            Industry::DataAnalytics => "Data Analytics",
            Industry::ElectronicComponents => "Electronic Components",
            Industry::ElectronicsAndComputerDistribution => "Electronics & Computer Distribution",
            Industry::HardwareAndSoftwareDistribution => "Hardware & Software Distribution",
            Industry::InformationTechnologyServices => "Information Technology Services",
            Industry::InternetContentAndInformation => "Internet Content & Information",
            Industry::ScientificAndTechnicalInstruments => "Scientific & Technical Instruments",
            Industry::SemiconductorEquipmentAndMaterials => "Semiconductor Equipment & Materials",
            Industry::Semiconductors => "Semiconductors",
            Industry::SoftwareApplication => "Software - Application",
            Industry::SoftwareInfrastructure => "Software - Infrastructure",
            Industry::Broadcasting => "Broadcasting",
            Industry::Entertainment => "Entertainment",
            Industry::Publishing => "Publishing",
            Industry::TelecomServices => "Telecom Services",
            Industry::UtilitiesDiversified => "Utilities - Diversified",
            Industry::UtilitiesIndependentPowerProducers => {
                "Utilities - Independent Power Producers"
            }
            Industry::UtilitiesRegulatedElectric => "Utilities - Regulated Electric",
            Industry::UtilitiesRegulatedGas => "Utilities - Regulated Gas",
            Industry::UtilitiesRegulatedWater => "Utilities - Regulated Water",
            Industry::UtilitiesRenewable => "Utilities - Renewable",
            Industry::ClosedEndFundDebt => "Closed-End Fund - Debt",
            Industry::ClosedEndFundEquity => "Closed-End Fund - Equity",
            Industry::ClosedEndFundForeign => "Closed-End Fund - Foreign",
            Industry::ExchangeTradedFund => "Exchange Traded Fund",
        }
    }
}

impl std::str::FromStr for Industry {
    type Err = ();

    /// Parses the same kebab-case slug returned by [`Industry::as_slug`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "agricultural-inputs" => Ok(Industry::AgriculturalInputs),
            "aluminum" => Ok(Industry::Aluminum),
            "coal" => Ok(Industry::Coal),
            "copper" => Ok(Industry::Copper),
            "farm-products" => Ok(Industry::FarmProducts),
            "forest-products" => Ok(Industry::ForestProducts),
            "gold" => Ok(Industry::Gold),
            "lumber-wood-production" => Ok(Industry::LumberAndWoodProduction),
            "other-industrial-metals-mining" => Ok(Industry::OtherIndustrialMetalsAndMining),
            "other-precious-metals-mining" => Ok(Industry::OtherPreciousMetalsAndMining),
            "silver" => Ok(Industry::Silver),
            "steel" => Ok(Industry::Steel),
            "thermal-coal" => Ok(Industry::ThermalCoal),
            "uranium" => Ok(Industry::Uranium),
            "apparel-manufacturing" => Ok(Industry::ApparelManufacturing),
            "apparel-retail" => Ok(Industry::ApparelRetail),
            "auto-truck-dealerships" => Ok(Industry::AutoAndTruckDealerships),
            "auto-manufacturers" => Ok(Industry::AutoManufacturers),
            "auto-parts" => Ok(Industry::AutoParts),
            "beverages-brewers" => Ok(Industry::BeveragesBrewers),
            "beverages-non-alcoholic" => Ok(Industry::BeveragesNonAlcoholic),
            "beverages-wineries-distilleries" => Ok(Industry::BeveragesWineriesAndDistilleries),
            "confectioners" => Ok(Industry::Confectioners),
            "department-stores" => Ok(Industry::DepartmentStores),
            "discount-stores" => Ok(Industry::DiscountStores),
            "electronic-gaming-multimedia" => Ok(Industry::ElectronicGamingAndMultimedia),
            "food-distribution" => Ok(Industry::FoodDistribution),
            "footwear-accessories" => Ok(Industry::FootwearAndAccessories),
            "furnishings-fixtures-appliances" => Ok(Industry::FurnishingsFixturesAndAppliances),
            "gambling" => Ok(Industry::Gambling),
            "grocery-stores" => Ok(Industry::GroceryStores),
            "home-improvement-retail" => Ok(Industry::HomeImprovementRetail),
            "household-personal-products" => Ok(Industry::HouseholdAndPersonalProducts),
            "internet-retail" => Ok(Industry::InternetRetail),
            "leisure" => Ok(Industry::Leisure),
            "lodging" => Ok(Industry::Lodging),
            "luxury-goods" => Ok(Industry::LuxuryGoods),
            "packaged-foods" => Ok(Industry::PackagedFoods),
            "personal-services" => Ok(Industry::PersonalServices),
            "residential-construction" => Ok(Industry::ResidentialConstruction),
            "resorts-casinos" => Ok(Industry::ResortsAndCasinos),
            "restaurants" => Ok(Industry::Restaurants),
            "specialty-retail" => Ok(Industry::SpecialtyRetail),
            "textile-manufacturing" => Ok(Industry::TextileManufacturing),
            "tobacco" => Ok(Industry::Tobacco),
            "travel-services" => Ok(Industry::TravelServices),
            "oil-gas-drilling" => Ok(Industry::OilAndGasDrilling),
            "oil-gas-ep" => Ok(Industry::OilAndGasEAndP),
            "oil-gas-equipment-services" => Ok(Industry::OilAndGasEquipmentAndServices),
            "oil-gas-integrated" => Ok(Industry::OilAndGasIntegrated),
            "oil-gas-midstream" => Ok(Industry::OilAndGasMidstream),
            "oil-gas-refining-marketing" => Ok(Industry::OilAndGasRefiningAndMarketing),
            "solar" => Ok(Industry::Solar),
            "asset-management" => Ok(Industry::AssetManagement),
            "banks-diversified" => Ok(Industry::BanksDiversified),
            "banks-regional" => Ok(Industry::BanksRegional),
            "capital-markets" => Ok(Industry::CapitalMarkets),
            "credit-services" => Ok(Industry::CreditServices),
            "financial-data-stock-exchanges" => Ok(Industry::FinancialDataAndStockExchanges),
            "insurance-brokers" => Ok(Industry::InsuranceBrokers),
            "insurance-diversified" => Ok(Industry::InsuranceDiversified),
            "insurance-life" => Ok(Industry::InsuranceLife),
            "insurance-property-casualty" => Ok(Industry::InsurancePropertyAndCasualty),
            "insurance-reinsurance" => Ok(Industry::InsuranceReinsurance),
            "insurance-specialty" => Ok(Industry::InsuranceSpecialty),
            "mortgage-finance" => Ok(Industry::MortgageFinance),
            "shell-companies" => Ok(Industry::ShellCompanies),
            "biotechnology" => Ok(Industry::Biotechnology),
            "diagnostics-research" => Ok(Industry::DiagnosticsAndResearch),
            "drug-manufacturers-general" => Ok(Industry::DrugManufacturersGeneral),
            "drug-manufacturers-specialty-generic" => {
                Ok(Industry::DrugManufacturersSpecialtyAndGeneric)
            }
            "health-information-services" => Ok(Industry::HealthInformationServices),
            "healthcare-plans" => Ok(Industry::HealthcarePlans),
            "medical-care-facilities" => Ok(Industry::MedicalCareFacilities),
            "medical-devices" => Ok(Industry::MedicalDevices),
            "medical-distribution" => Ok(Industry::MedicalDistribution),
            "medical-instruments-supplies" => Ok(Industry::MedicalInstrumentsAndSupplies),
            "pharmaceutical-retailers" => Ok(Industry::PharmaceuticalRetailers),
            "aerospace-defense" => Ok(Industry::AerospaceAndDefense),
            "building-materials" => Ok(Industry::BuildingMaterials),
            "building-products-equipment" => Ok(Industry::BuildingProductsAndEquipment),
            "business-equipment-supplies" => Ok(Industry::BusinessEquipmentAndSupplies),
            "chemical-manufacturing" => Ok(Industry::ChemicalManufacturing),
            "chemicals" => Ok(Industry::Chemicals),
            "conglomerates" => Ok(Industry::Conglomerates),
            "consulting-services" => Ok(Industry::ConsultingServices),
            "electrical-equipment-parts" => Ok(Industry::ElectricalEquipmentAndParts),
            "engineering-construction" => Ok(Industry::EngineeringAndConstruction),
            "farm-heavy-construction-machinery" => Ok(Industry::FarmAndHeavyConstructionMachinery),
            "industrial-distribution" => Ok(Industry::IndustrialDistribution),
            "infrastructure-operations" => Ok(Industry::InfrastructureOperations),
            "integrated-freight-logistics" => Ok(Industry::IntegratedFreightAndLogistics),
            "manufacturing-diversified" => Ok(Industry::ManufacturingDiversified),
            "marine-ports-services" => Ok(Industry::MarinePortsAndServices),
            "marine-shipping" => Ok(Industry::MarineShipping),
            "metal-fabrication" => Ok(Industry::MetalFabrication),
            "paper-paper-products" => Ok(Industry::PaperAndPaperProducts),
            "pollution-treatment-controls" => Ok(Industry::PollutionAndTreatmentControls),
            "railroads" => Ok(Industry::Railroads),
            "rental-leasing-services" => Ok(Industry::RentalAndLeasingServices),
            "security-protection-services" => Ok(Industry::SecurityAndProtectionServices),
            "specialty-business-services" => Ok(Industry::SpecialtyBusinessServices),
            "specialty-chemicals" => Ok(Industry::SpecialtyChemicals),
            "specialty-industrial-machinery" => Ok(Industry::SpecialtyIndustrialMachinery),
            "staffing-employment-services" => Ok(Industry::StaffingAndEmploymentServices),
            "tools-accessories" => Ok(Industry::ToolsAndAccessories),
            "trucking" => Ok(Industry::Trucking),
            "waste-management" => Ok(Industry::WasteManagement),
            "real-estate-development" => Ok(Industry::RealEstateDevelopment),
            "real-estate-diversified" => Ok(Industry::RealEstateDiversified),
            "real-estate-services" => Ok(Industry::RealEstateServices),
            "reit-diversified" => Ok(Industry::ReitDiversified),
            "reit-healthcare-facilities" => Ok(Industry::ReitHealthcareFacilities),
            "reit-hotel-motel" => Ok(Industry::ReitHotelAndMotel),
            "reit-industrial" => Ok(Industry::ReitIndustrial),
            "reit-mortgage" => Ok(Industry::ReitMortgage),
            "reit-office" => Ok(Industry::ReitOffice),
            "reit-residential" => Ok(Industry::ReitResidential),
            "reit-retail" => Ok(Industry::ReitRetail),
            "reit-specialty" => Ok(Industry::ReitSpecialty),
            "communication-equipment" => Ok(Industry::CommunicationEquipment),
            "computer-hardware" => Ok(Industry::ComputerHardware),
            "consumer-electronics" => Ok(Industry::ConsumerElectronics),
            "data-analytics" => Ok(Industry::DataAnalytics),
            "electronic-components" => Ok(Industry::ElectronicComponents),
            "electronics-computer-distribution" => Ok(Industry::ElectronicsAndComputerDistribution),
            "hardware-software-distribution" => Ok(Industry::HardwareAndSoftwareDistribution),
            "information-technology-services" => Ok(Industry::InformationTechnologyServices),
            "internet-content-information" => Ok(Industry::InternetContentAndInformation),
            "scientific-technical-instruments" => Ok(Industry::ScientificAndTechnicalInstruments),
            "semiconductor-equipment-materials" => Ok(Industry::SemiconductorEquipmentAndMaterials),
            "semiconductors" => Ok(Industry::Semiconductors),
            "software-application" => Ok(Industry::SoftwareApplication),
            "software-infrastructure" => Ok(Industry::SoftwareInfrastructure),
            "broadcasting" => Ok(Industry::Broadcasting),
            "entertainment" => Ok(Industry::Entertainment),
            "publishing" => Ok(Industry::Publishing),
            "telecom-services" => Ok(Industry::TelecomServices),
            "utilities-diversified" => Ok(Industry::UtilitiesDiversified),
            "utilities-independent-power-producers" => {
                Ok(Industry::UtilitiesIndependentPowerProducers)
            }
            "utilities-regulated-electric" => Ok(Industry::UtilitiesRegulatedElectric),
            "utilities-regulated-gas" => Ok(Industry::UtilitiesRegulatedGas),
            "utilities-regulated-water" => Ok(Industry::UtilitiesRegulatedWater),
            "utilities-renewable" => Ok(Industry::UtilitiesRenewable),
            "closed-end-fund-debt" => Ok(Industry::ClosedEndFundDebt),
            "closed-end-fund-equity" => Ok(Industry::ClosedEndFundEquity),
            "closed-end-fund-foreign" => Ok(Industry::ClosedEndFundForeign),
            "exchange-traded-fund" => Ok(Industry::ExchangeTradedFund),
            _ => Err(()),
        }
    }
}

impl AsRef<str> for Industry {
    /// Returns the slug, enabling `finance::industry(Industry::Semiconductors)`.
    fn as_ref(&self) -> &str {
        self.as_slug()
    }
}

impl From<Industry> for String {
    /// Returns the screener display name, enabling `EquityField::Industry.eq_str(Industry::Semiconductors)`.
    fn from(v: Industry) -> Self {
        v.screener_value().to_string()
    }
}

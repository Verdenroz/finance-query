/// Typed industry identifier for the industry endpoint and custom screener queries.
///
/// See the module-level doc for usage.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Industry {
    // ── Agriculture / Raw Materials ──────────────────────────────────────
    /// Agricultural inputs, fertilizers, and crop chemicals
    AgriculturalInputs,
    /// Aluminum production and processing companies
    Aluminum,
    /// Coal mining and processing companies
    Coal,
    /// Copper mining and processing companies
    Copper,
    /// Farm products including grains, livestock, and produce
    FarmProducts,
    /// Forest products including timber and paper pulp
    ForestProducts,
    /// Gold mining and royalty companies
    Gold,
    /// Lumber and wood production companies
    LumberAndWoodProduction,
    /// Other industrial metals and mining (zinc, nickel, etc.)
    OtherIndustrialMetalsAndMining,
    /// Other precious metals and mining (platinum, palladium, etc.)
    OtherPreciousMetalsAndMining,
    /// Silver mining and streaming companies
    Silver,
    /// Steel production and processing companies
    Steel,
    /// Thermal coal mining for electricity generation
    ThermalCoal,
    /// Uranium mining companies
    Uranium,
    // ── Consumer ─────────────────────────────────────────────────────────
    /// Clothing and apparel manufacturing companies
    ApparelManufacturing,
    /// Clothing and apparel retail chains
    ApparelRetail,
    /// Automotive and truck dealerships
    AutoAndTruckDealerships,
    /// Automobile manufacturers and assemblers
    AutoManufacturers,
    /// Automotive parts manufacturers and distributors
    AutoParts,
    /// Beer brewing and distribution companies
    BeveragesBrewers,
    /// Non-alcoholic beverages including soft drinks and juices
    BeveragesNonAlcoholic,
    /// Wineries, distilleries, and spirits producers
    BeveragesWineriesAndDistilleries,
    /// Candy, chocolate, and confectionery makers
    Confectioners,
    /// Traditional department store retailers
    DepartmentStores,
    /// Discount and value retail stores
    DiscountStores,
    /// Electronic gaming software and multimedia entertainment
    ElectronicGamingAndMultimedia,
    /// Wholesale food distribution companies
    FoodDistribution,
    /// Footwear, handbags, and fashion accessories
    FootwearAndAccessories,
    /// Furniture, fixtures, and household appliances
    FurnishingsFixturesAndAppliances,
    /// Casinos, online gambling, and gaming operators
    Gambling,
    /// Supermarkets and grocery retail chains
    GroceryStores,
    /// Home improvement retail stores
    HomeImprovementRetail,
    /// Household cleaning products and personal care items
    HouseholdAndPersonalProducts,
    /// Online retail and e-commerce marketplaces
    InternetRetail,
    /// Leisure, recreation, and entertainment companies
    Leisure,
    /// Hotels and lodging companies
    Lodging,
    /// Luxury goods, fashion, and premium consumer brands
    LuxuryGoods,
    /// Packaged and processed food manufacturers
    PackagedFoods,
    /// Personal care, laundry, and household services
    PersonalServices,
    /// Home builders and residential construction
    ResidentialConstruction,
    /// Resorts, integrated casinos, and hotel-casinos
    ResortsAndCasinos,
    /// Restaurant chains and food service operators
    Restaurants,
    /// Specialty retail stores (pets, books, electronics, etc.)
    SpecialtyRetail,
    /// Textile and fabric manufacturers
    TextileManufacturing,
    /// Tobacco product manufacturers
    Tobacco,
    /// Travel agencies, booking platforms, and tour operators
    TravelServices,
    // ── Energy ───────────────────────────────────────────────────────────
    /// Oil and gas contract drilling services
    OilAndGasDrilling,
    /// Oil and gas exploration and production companies
    OilAndGasEAndP,
    /// Oil field equipment, services, and engineering
    OilAndGasEquipmentAndServices,
    /// Vertically integrated oil and gas majors
    OilAndGasIntegrated,
    /// Oil and gas pipelines, storage, and transportation
    OilAndGasMidstream,
    /// Oil refining, wholesale fuel, and marketing
    OilAndGasRefiningAndMarketing,
    /// Solar panel manufacturers and solar energy producers
    Solar,
    // ── Financial Services ───────────────────────────────────────────────
    /// Asset managers, fund sponsors, and investment advisors
    AssetManagement,
    /// Large diversified national and international banks
    BanksDiversified,
    /// Regional and community banks
    BanksRegional,
    /// Investment banks, brokers, and financial exchanges
    CapitalMarkets,
    /// Credit card issuers and consumer credit services
    CreditServices,
    /// Financial data, analytics, and stock exchange operators
    FinancialDataAndStockExchanges,
    /// Insurance brokers and agencies
    InsuranceBrokers,
    /// Diversified multi-line insurance companies
    InsuranceDiversified,
    /// Life insurance and annuity providers
    InsuranceLife,
    /// Property and casualty insurance companies
    InsurancePropertyAndCasualty,
    /// Reinsurance companies
    InsuranceReinsurance,
    /// Specialty insurance lines (title, mortgage, etc.)
    InsuranceSpecialty,
    /// Mortgage banking and loan origination
    MortgageFinance,
    /// Blank-check and shell holding companies
    ShellCompanies,
    // ── Healthcare ───────────────────────────────────────────────────────
    /// Biotechnology drug development companies
    Biotechnology,
    /// Medical diagnostics labs and clinical research
    DiagnosticsAndResearch,
    /// Large branded pharmaceutical manufacturers
    DrugManufacturersGeneral,
    /// Specialty drugs, generics, and biosimilars
    DrugManufacturersSpecialtyAndGeneric,
    /// Healthcare IT, EHR, and health data services
    HealthInformationServices,
    /// Managed care organizations and health insurers
    HealthcarePlans,
    /// Hospitals, clinics, and outpatient care facilities
    MedicalCareFacilities,
    /// Medical device manufacturers (implants, diagnostics equipment)
    MedicalDevices,
    /// Medical product wholesalers and distributors
    MedicalDistribution,
    /// Surgical instruments, disposables, and medical supplies
    MedicalInstrumentsAndSupplies,
    /// Retail pharmacies and drug store chains
    PharmaceuticalRetailers,
    // ── Industrials ──────────────────────────────────────────────────────
    /// Defense contractors, aircraft, and space systems
    AerospaceAndDefense,
    /// Construction aggregates, cement, and building materials
    BuildingMaterials,
    /// HVAC, plumbing, windows, and building equipment
    BuildingProductsAndEquipment,
    /// Office supplies, commercial equipment, and printers
    BusinessEquipmentAndSupplies,
    /// Specialty chemical manufacturing for industrial use
    ChemicalManufacturing,
    /// Diversified commodity chemicals producers
    Chemicals,
    /// Diversified industrial holding companies
    Conglomerates,
    /// Management consulting and professional advisory services
    ConsultingServices,
    /// Electrical components, motors, and power equipment
    ElectricalEquipmentAndParts,
    /// Civil engineering, construction, and infrastructure projects
    EngineeringAndConstruction,
    /// Agricultural equipment and heavy construction machinery
    FarmAndHeavyConstructionMachinery,
    /// Industrial goods wholesalers and distributors
    IndustrialDistribution,
    /// Toll roads, airports, and infrastructure operators
    InfrastructureOperations,
    /// Third-party logistics and supply chain management
    IntegratedFreightAndLogistics,
    /// Diversified manufacturers across multiple industrial segments
    ManufacturingDiversified,
    /// Port operators and marine terminal services
    MarinePortsAndServices,
    /// Bulk cargo and tanker shipping companies
    MarineShipping,
    /// Custom metal fabrication and machined components
    MetalFabrication,
    /// Paper, packaging, and pulp product manufacturers
    PaperAndPaperProducts,
    /// Environmental controls, water treatment, and remediation
    PollutionAndTreatmentControls,
    /// Rail freight carriers and passenger rail operators
    Railroads,
    /// Equipment rental, leasing, and fleet management
    RentalAndLeasingServices,
    /// Security systems, guards, and monitoring services
    SecurityAndProtectionServices,
    /// Outsourced business services and BPO companies
    SpecialtyBusinessServices,
    /// High-value specialty chemicals and advanced materials
    SpecialtyChemicals,
    /// Specialized industrial machinery and equipment makers
    SpecialtyIndustrialMachinery,
    /// Staffing agencies and employment service providers
    StaffingAndEmploymentServices,
    /// Hand tools, power tools, and hardware accessories
    ToolsAndAccessories,
    /// Freight trucking and less-than-truckload carriers
    Trucking,
    /// Waste collection, recycling, and disposal services
    WasteManagement,
    // ── Real Estate ──────────────────────────────────────────────────────
    /// Real estate developers and homebuilders
    RealEstateDevelopment,
    /// Diversified real estate companies with mixed portfolios
    RealEstateDiversified,
    /// Real estate brokers, agents, and property managers
    RealEstateServices,
    /// Diversified REITs across multiple property types
    ReitDiversified,
    /// Healthcare and senior living facility REITs
    ReitHealthcareFacilities,
    /// Hotel and motel property REITs
    ReitHotelAndMotel,
    /// Industrial, warehouse, and logistics property REITs
    ReitIndustrial,
    /// Mortgage REITs investing in real estate debt
    ReitMortgage,
    /// Office building and commercial property REITs
    ReitOffice,
    /// Apartment, multifamily, and residential property REITs
    ReitResidential,
    /// Shopping center and retail property REITs
    ReitRetail,
    /// Specialty REITs (data centers, cell towers, self-storage)
    ReitSpecialty,
    // ── Technology ───────────────────────────────────────────────────────
    /// Networking hardware, routers, and communication equipment
    CommunicationEquipment,
    /// PCs, servers, and computer hardware manufacturers
    ComputerHardware,
    /// Smartphones, TVs, and consumer electronic devices
    ConsumerElectronics,
    /// Data analytics, business intelligence, and AI platforms
    DataAnalytics,
    /// Passive electronic components and circuit boards
    ElectronicComponents,
    /// Distributors of electronics and computer products
    ElectronicsAndComputerDistribution,
    /// Value-added resellers and software/hardware distributors
    HardwareAndSoftwareDistribution,
    /// IT services, outsourcing, and technology consulting
    InformationTechnologyServices,
    /// Online media, search engines, and digital content platforms
    InternetContentAndInformation,
    /// Precision instruments, sensors, and test equipment
    ScientificAndTechnicalInstruments,
    /// Semiconductor manufacturing equipment and materials
    SemiconductorEquipmentAndMaterials,
    /// Integrated circuit and chip designers and manufacturers
    Semiconductors,
    /// Business application software companies
    SoftwareApplication,
    /// Operating systems, middleware, and infrastructure software
    SoftwareInfrastructure,
    // ── Communication Services ───────────────────────────────────────────
    /// Television, radio, and broadcast media companies
    Broadcasting,
    /// Film studios, streaming, and live entertainment
    Entertainment,
    /// Book, magazine, newspaper, and digital media publishers
    Publishing,
    /// Wireless carriers and wireline telephone companies
    TelecomServices,
    // ── Utilities ────────────────────────────────────────────────────────
    /// Multi-utility companies serving electricity, gas, and water
    UtilitiesDiversified,
    /// Independent power producers and energy traders
    UtilitiesIndependentPowerProducers,
    /// Regulated electric utility companies
    UtilitiesRegulatedElectric,
    /// Regulated natural gas distribution utilities
    UtilitiesRegulatedGas,
    /// Regulated water and wastewater utilities
    UtilitiesRegulatedWater,
    /// Renewable energy generation companies (wind, solar, hydro)
    UtilitiesRenewable,
    // ── Special ──────────────────────────────────────────────────────────
    /// Closed-end funds investing in debt instruments
    ClosedEndFundDebt,
    /// Closed-end funds investing in equities
    ClosedEndFundEquity,
    /// Closed-end funds investing in foreign securities
    ClosedEndFundForeign,
    /// Exchange-traded fund products
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

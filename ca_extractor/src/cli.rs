use clap::{Parser, ValueEnum};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ServiceType {
    /// Qualified certificate for website authentication
    QWAC,
    /// Qualified certificate for electronic seal
    QSealC,
}

#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "Extract CA certificates from eIDAS Trusted List", 
    long_about = "A tool to extract CA certificates from XML files available through eIDAS Trusted List"
)]
pub struct Args {
    /// Type of service to retrieve certificate for
    #[arg(value_enum)]
    pub service: ServiceType,

    /// ISO 3166-1 alpha-2 country code (only EEA countries are supported)
    pub country: String,

    /// Target folder to save certificate files in
    #[arg(long, default_value = ".")]
    pub target_folder: String,

    /// Enable verbose logging
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

impl From<ServiceType> for String {
    fn from(service_type: ServiceType) -> Self {
        match service_type {
            ServiceType::QWAC => "QWAC".to_string(),
            ServiceType::QSealC => "QSealC".to_string(),
        }
    }
}

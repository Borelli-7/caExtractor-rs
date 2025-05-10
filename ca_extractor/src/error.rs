use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaExtractorError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("XML parsing error: {0}")]
    XmlError(#[from] xml::reader::Error),
    
    #[error("Invalid service type: {0}")]
    InvalidServiceType(String),
    
    #[error("Certificate extraction error: {0}")]
    CertificateExtractionError(String),
    
    #[error("Invalid country code: {0}")]
    InvalidCountryCode(String),
    
    #[error("No certificates found for country {country} and service {service}")]
    NoCertificatesFound { country: String, service: String },

    #[error("Invalid certificate format: {0}")]
    InvalidCertificateFormat(String),
}

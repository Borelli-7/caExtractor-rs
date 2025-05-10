use std::collections::HashMap;
use std::io::Read;

use reqwest;
use xml::reader::{EventReader, XmlEvent};

use crate::error::CaExtractorError;

pub struct NamespaceContext {
    namespaces: HashMap<&'static str, &'static str>,
}

impl NamespaceContext {
    pub fn new() -> Self {
        let namespaces = HashMap::from([("tsl", "http://uri.etsi.org/02231/v2#")]);
        NamespaceContext { namespaces }
    }

    pub fn get_namespace_uri(&self, prefix: &str) -> Option<&&str> {
        self.namespaces.get(prefix)
    }
}

use crate::cli::ServiceType;

pub struct CertificateExtractor {
    country: String,
    service_uri: String,
    service_type: String,
}

impl CertificateExtractor {
    pub fn new(service: ServiceType, country: &str) -> Result<Self, CaExtractorError> {
        // Validate country code (simple check for now)
        if country.len() != 2 {
            return Err(CaExtractorError::InvalidCountryCode(
                format!("Invalid country code '{}'. Must be a 2-letter ISO country code.", country)
            ));
        }
        
        let service_uri = match service {
            ServiceType::QWAC => "http://uri.etsi.org/TrstSvc/TrustedList/SvcInfoExt/ForWebSiteAuthentication",
            ServiceType::QSealC => "http://uri.etsi.org/TrstSvc/TrustedList/SvcInfoExt/ForeSeals",
        };

        Ok(CertificateExtractor {
            country: country.to_string(),
            service_uri: service_uri.to_string(),
            service_type: service.into(),
        })
    }
    
    pub fn fetch_xml_content(&self) -> Result<String, CaExtractorError> {
        let url = format!(
            "https://eidas.ec.europa.eu/efda/tl-browser/api/v1/browser/download/{}",
            self.country
        );

        let mut response = reqwest::blocking::get(&url)?;
        let mut xml_content = String::new();
        response.read_to_string(&mut xml_content)?;
        
        Ok(xml_content)
    }
    
    pub fn extract_certificates(&self, xml_content: &str) -> Result<Vec<String>, CaExtractorError> {
        let parser = EventReader::new(xml_content.as_bytes());
        let mut current_element = String::new();
        let mut elements = Vec::new();

        for event in parser {
            match event? {
                XmlEvent::StartElement { name, .. } => {
                    current_element = name.local_name;
                }
                XmlEvent::Characters(content) if current_element == "TSPService" => {
                    elements.push(content);
                }
                XmlEvent::EndElement { .. } => {
                    current_element.clear();
                }
                _ => {}
            }
        }
        
        let mut certificates = Vec::new();
        
        for element in elements {
            if let Some(cert_start) = element.find("<tsl:X509Certificate>") {
                if let Some(cert_end) = element.find("</tsl:X509Certificate>") {
                    let cert_content = &element[cert_start + 19..cert_end].trim().replace([' ', '\n'], "");

                    let wrapped_cert_str = format!(
                        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
                        cert_content
                            .as_bytes()
                            .chunks(64)
                            .map(|chunk| std::str::from_utf8(chunk).unwrap())
                            .collect::<Vec<&str>>()
                            .join("\n")
                    );
                    
                    certificates.push(wrapped_cert_str);
                }
            }
        }
        
        if certificates.is_empty() {
            return Err(CaExtractorError::CertificateExtractionError(
                "No certificates found in the XML content".to_string()));
        }
        
        Ok(certificates)
    }
}

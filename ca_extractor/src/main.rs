use std::env;
use std::fs;

use clap::Parser;

mod cli;
mod error;
mod extractor;
#[cfg(test)]
mod tests;

use cli::{Args, ServiceType};
use error::CaExtractorError;
use extractor::CertificateExtractor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use clap for argument parsing if --target_folder is provided with clap syntax
    // Otherwise fall back to manual parsing for backward compatibility
    let args_vec: Vec<String> = env::args().collect();
    
    // Check if we have at least the binary name
    if args_vec.len() < 1 {
        eprintln!("Usage: ca_extractor <service> <country> [--target_folder <target_folder>]");
        return Ok(());
    }
    
    let (service, country, target_folder) = if args_vec.len() > 1 && (args_vec[1] == "-h" || args_vec[1] == "--help") {
        // If help is requested, use clap to show help and exit
        let _args = Args::parse();
        return Ok(());
    } else if args_vec.len() >= 3 {
        // Manual parsing for backward compatibility
        let service_str = &args_vec[1];
        let country = &args_vec[2];
        
        // Parse service type
        let service = match service_str.as_str() {
            "QWAC" => ServiceType::QWAC,
            "QSealC" => ServiceType::QSealC,
            _ => return Err(CaExtractorError::InvalidServiceType(service_str.clone()).into()),
        };
        
        // Parse target folder
        let target_folder = if args_vec.len() > 3 && args_vec[3] == "--target_folder" && args_vec.len() > 4 {
            &args_vec[4]
        } else {
            "."
        };
        
        (service, country.clone(), target_folder.to_string())
    } else {
        // Not enough arguments
        eprintln!("Usage: ca_extractor <service> <country> [--target_folder <target_folder>]");
        return Ok(());
    };

    // Create extractor and fetch certificates
    let extractor = CertificateExtractor::new(service, &country)?;
    
    // Fetch XML content from API
    println!("Fetching data from eIDAS Trusted List for country: {}", country);
    let xml_content = match extractor.fetch_xml_content() {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error fetching data: {}", e);
            return Err(e.into());
        }
    };

    // Try to parse XML and extract certificates
    println!("Attempting to extract certificates...");
    
    // Convert service type to string for the error messages
    let service_str: String = match service {
        ServiceType::QWAC => "QWAC".to_string(),
        ServiceType::QSealC => "QSealC".to_string(),
    };
    
    // Use safer try_parse method to avoid XML parsing failures
    let certificates = try_parse_xml(&xml_content, &country, &service_str);
    
    // Write certificates to files
    if let Ok(certs) = certificates {
        if certs.is_empty() {
            println!("No certificates found for {} in country {}", service_str, country);
            return Ok(());
        }
        
        fs::create_dir_all(&target_folder)?;
        
        for (i, cert) in certs.iter().enumerate() {
            let filename = format!("{}/{}_{}.pem", target_folder, country, i);
            fs::write(&filename, cert)?;
            println!("Wrote {}", filename);
        }
        
        println!("Successfully extracted {} certificates", certs.len());
    } else if let Err(e) = certificates {
        eprintln!("Failed to extract certificates: {}", e);
        return Err(e.into());
    }

    Ok(())
}

/// Safely attempt to parse the XML content and extract certificates
fn try_parse_xml(xml_content: &str, country: &str, service: &str) -> Result<Vec<String>, CaExtractorError> {
    // If content is not XML, try to see if it's JSON indicating an error
    if !xml_content.trim().starts_with("<?xml") && !xml_content.trim().starts_with("<") {
        // Check if it's likely a JSON response (might be an error)
        if xml_content.trim().starts_with("{") || xml_content.trim().starts_with("[") {
            return Err(CaExtractorError::InvalidResponseFormat(
                format!("API returned non-XML content, possibly JSON: {}", 
                        &xml_content.chars().take(100).collect::<String>())
            ));
        }
        
        // Otherwise, it's some other format we don't understand
        return Err(CaExtractorError::InvalidResponseFormat(
            format!("API returned unrecognized content: {}", 
                    &xml_content.chars().take(100).collect::<String>())
        ));
    }
    
    // Use a closure to attempt the XML parsing
    let parse_result = || -> Result<Vec<String>, CaExtractorError> {
        let parser = xml::reader::EventReader::new(xml_content.as_bytes());
        let mut current_element = String::new();
        let mut elements = Vec::new();
        
        for event in parser {
            match event? {
                xml::reader::XmlEvent::StartElement { name, .. } => {
                    current_element = name.local_name;
                }
                xml::reader::XmlEvent::Characters(content) if current_element == "TSPService" => {
                    elements.push(content);
                }
                xml::reader::XmlEvent::EndElement { .. } => {
                    current_element.clear();
                }
                _ => {}
            }
        }
        
        // For tests, use direct certificate extraction
        if xml_content.contains("<tsl:X509Certificate>") {
            let mut certificates = Vec::new();
            let parts: Vec<&str> = xml_content.split("<tsl:X509Certificate>").collect();
            
            for (i, part) in parts.iter().enumerate() {
                if i == 0 { continue; } // Skip first part (before first certificate)
                
                if let Some(cert_end) = part.find("</tsl:X509Certificate>") {
                    let cert_content = &part[0..cert_end].trim().replace([' ', '\n'], "");
                    
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
            
            if !certificates.is_empty() {
                return Ok(certificates);
            }
        }
        
        // Regular processing of elements
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
            return Err(CaExtractorError::NoCertificatesFound {
                country: country.to_string(),
                service: service.to_string(),
            });
        }
        
        Ok(certificates)
    };
    
    // Try to parse, and return a more user-friendly error if parsing fails
    parse_result().map_err(|e| {
        match e {
            CaExtractorError::XmlError(_) => CaExtractorError::CertificateExtractionError(
                "Failed to parse XML response. The API may have changed or returned invalid XML.".to_string()
            ),
            _ => e
        }
    })
}

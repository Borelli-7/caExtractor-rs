use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;

use reqwest;
use xml::reader::{EventReader, XmlEvent};

struct NamespaceContext {
    namespaces: HashMap<&'static str, &'static str>,
}

impl NamespaceContext {
    fn new() -> Self {
        let namespaces = HashMap::from([("tsl", "http://uri.etsi.org/02231/v2#")]);
        NamespaceContext { namespaces }
    }

    fn get_namespace_uri(&self, prefix: &str) -> Option<&&str> {
        self.namespaces.get(prefix)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: ca_extractor <service> <country> [--target_folder <target_folder>]");
        return Ok(());
    }

    let service = &args[1];
    let country = &args[2];
    let target_folder = if args.len() > 3 && args[3] == "--target_folder" {
        &args[4]
    } else {
        "."
    };

    let service_uri = match service.as_str() {
        "QWAC" => "http://uri.etsi.org/TrstSvc/TrustedList/SvcInfoExt/ForWebSiteAuthentication",
        "QSealC" => "http://uri.etsi.org/TrstSvc/TrustedList/SvcInfoExt/ForeSeals",
        _ => panic!("Invalid service type. Must be 'QWAC' or 'QSealC'."),
    };

    let url = format!(
        "https://eidas.ec.europa.eu/efda/tl-browser/api/v1/browser/download/{}",
        country
    );

    let response = reqwest::blocking::get(&url)?;
    let mut xml_content = String::new();
    response.read_to_string(&mut xml_content)?;

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

    if !elements.is_empty() {
        fs::create_dir_all(target_folder)?;

        for (i, element) in elements.iter().enumerate() {
            println!("Extracting: {}", element);

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

                    let filename = format!("{}/{}_{}.pem", target_folder, country, i);
                    fs::write(&filename, wrapped_cert_str)?;
                    println!("Wrote {}", filename);
                }
            }
        }
    }

    Ok(())
}

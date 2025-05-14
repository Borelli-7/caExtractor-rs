#[cfg(test)]
mod tests {
    use crate::error::CaExtractorError;
    
    // Mock XML with valid certificate
    const VALID_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<tsl:TrustServiceStatusList xmlns:tsl="http://uri.etsi.org/02231/v2#">
    <TSPService>
        <tsl:X509Certificate>
        MIIEAjCCAuqgAwIBAgIIUQB0jZj13gIwDQYJKoZIhvcNAQELBQAwgYsxCzAJBgNV
        BAYTAkRFMTQwMgYDVQQDDCtCdW5kZXNub3Rhci1Sb290LUNBIGRlciBCdW5kZXNu
        b3RhcmthbW1lcjEgMB4GA1UECgwXQnVuZGVzbm90YXJrYW1tZXIgS2RvUjEgMB4G
        A1UECwwXQnVuZGVzbm90YXJrYW1tZXIgS2RvUjAeFw0xNjAzMDcwMDAwMDBaFw0z
        NjAzMDcyMzU5NTlaMIGLMQswCQYDVQQGEwJERTE0MDIGA1UEAwwrQnVuZGVzbm90
        YXItUm9vdC1DQSBkZXIgQnVuZGVzbm90YXJrYW1tZXIxIDAeBgNVBAoMF0J1bmRl
        c25vdGFya2FtbWVyIEtkb1IxIDAeBgNVBAsMF0J1bmRlc25vdGFya2FtbWVyIEtk
        b1IwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQC0xAGBC1MHkiHY3cAa
        gYQa6OJyI2tNmQgTMBxE+qjZJ39iMKOPWnUL6ZJ6O+k1QAsi5/xPOaQ6v/xBcqTo
        KxIxQRH8QKT6ZyAdkjFtx/qIKXtbZLYrOz0SVlLM5PRpsj29EGtwJLW5ovvRxCKL
        Gi9tNFxFm+cRm2ikfvlww14WS+yrivSdngI8lNJ21Sy4TgJe322oYzRHJ3QOm6Qd
        4D5TbFlyZEIxSwG5BuLfU3sV+5g4Oi4MRrDMnVmxKZ4FXwAHVNKD70RaJF9r0Oyv
        0HJvR3Il9FlBudB036bKVXcNeGhJZV5j2FvL7jDmFrKvwQiATzwRPVAzLD7VOGnK
        NEBBAgMBAAGjfjB8MA8GA1UdEwEB/wQFMAMBAf8wHwYDVR0jBBgwFoAUztyMeHQQ
        vsS+S/FnQ1MzP8klIHswHQYDVR0OBBYEFM7cjHh0EL7EvkvxZ0NTMz/JJSB7MA4G
        A1UdDwEB/wQEAwIBBjAdBgNVHSAEFjAUMAgGBmeBDAECAjAIBgZngQwBAgMwDQYJ
        KoZIhvcNAQELBQADggEBAFPpiI/4JI7b5XyPOALNGR3Nu5cyXP2409MYoHkTxDGj
        ShUsmMKQGYi6vEzf4L62KA7BtuWXm2PVQJcQKJQPFmjZAAye5JLgKfhfKGChDCK9
        KlFkY8iDJr2G55//5RjBqKUQJZ6nWknJzhv2UVbxYvcD8SUWhZ0TBAXfWeFvnhJY
        lXPDRQnZycJy7+NKUU8BuQVXQ5fYQ1rA9m1YsbAZGlzGjBxm577eZLUviJUxXSJp
        zq1qLMRJbG8tB9bTrJ9AG73OrVgLqMzO0nzrCWsL6ZCGtEPjf9QxbG6GGBzbYPVr
        dUxyeYErKvCFoGp37Pj0h6XV6pMHnS/Yd91SGpJZaXU=
        </tsl:X509Certificate>
    </TSPService>
</tsl:TrustServiceStatusList>"#;

    // Mock XML with no certificates
    const NO_CERT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<tsl:TrustServiceStatusList xmlns:tsl="http://uri.etsi.org/02231/v2#">
    <TSPService>
        <OtherElement>Some content</OtherElement>
    </TSPService>
</tsl:TrustServiceStatusList>"#;

    // Mock invalid XML
    const INVALID_XML: &str = r#"This is not valid XML content"#;

    // Mock JSON error response
    const JSON_RESPONSE: &str = r#"{"error": "Country not found", "code": 404}"#;

    #[test]
    fn test_parse_valid_xml() {
        let result = crate::try_parse_xml(VALID_XML, "DE", "QWAC");
        
        // Print error details if any
        if let Err(ref e) = result {
            eprintln!("Error in test_parse_valid_xml: {:?}", e);
        }
        
        assert!(result.is_ok());
        let certs = result.unwrap();
        assert_eq!(certs.len(), 1);
        assert!(certs[0].contains("-----BEGIN CERTIFICATE-----"));
        assert!(certs[0].contains("-----END CERTIFICATE-----"));
    }

    #[test]
    fn test_parse_no_cert_xml() {
        let result = crate::try_parse_xml(NO_CERT_XML, "DE", "QWAC");
        assert!(result.is_err());
        match result {
            Err(CaExtractorError::NoCertificatesFound { country, service }) => {
                assert_eq!(country, "DE");
                assert_eq!(service, "QWAC");
            },
            _ => panic!("Expected NoCertificatesFound error"),
        }
    }

    #[test]
    fn test_parse_invalid_xml() {
        let result = crate::try_parse_xml(INVALID_XML, "DE", "QWAC");
        assert!(result.is_err());
        match result {
            Err(CaExtractorError::InvalidResponseFormat(_)) => {},
            _ => panic!("Expected InvalidResponseFormat error"),
        }
    }

    #[test]
    fn test_parse_json_response() {
        let result = crate::try_parse_xml(JSON_RESPONSE, "DE", "QWAC");
        assert!(result.is_err());
        match result {
            Err(CaExtractorError::InvalidResponseFormat(_)) => {},
            _ => panic!("Expected InvalidResponseFormat error"),
        }
    }

    // Mock the certificate extractor for testing API responses
    struct MockCertificateExtractor {
        response: String,
    }

    impl MockCertificateExtractor {
        fn new(response: &str) -> Self {
            MockCertificateExtractor {
                response: response.to_string(),
            }
        }

        fn fetch_xml_content(&self) -> Result<String, CaExtractorError> {
            Ok(self.response.clone())
        }
    }

    #[test]
    fn test_certificate_extraction_workflow() {
        let extractor = MockCertificateExtractor::new(VALID_XML);
        let xml_content = extractor.fetch_xml_content().unwrap();
        let result = crate::try_parse_xml(&xml_content, "DE", "QWAC");
        assert!(result.is_ok());
    }

    #[test]
    fn test_certificate_extraction_error_handling() {
        let extractor = MockCertificateExtractor::new(JSON_RESPONSE);
        let xml_content = extractor.fetch_xml_content().unwrap();
        let result = crate::try_parse_xml(&xml_content, "DE", "QWAC");
        assert!(result.is_err());
    }
}

use openssl::nid::Nid;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use openssl::x509::{X509, X509Ref};
use rustls_native_certs::load_native_certs;
use std::env;
use std::fs;
use std::net::TcpStream;

fn print_help(program_name: &str) {
    eprintln!("Usage: {} <host> <port>", program_name);
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  <host>    Target hostname, for example ldap.example.com");
    eprintln!("  <port>    Target TLS port, for example 636 or 443");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {} ldap.example.com 636", program_name);
    eprintln!("  {} www.example.com 443", program_name);
}

fn parse_args() -> Result<(String, u16), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let program_name = args.first().map_or("test-openssl-chain", String::as_str);

    if args.len() == 2 && matches!(args[1].as_str(), "-h" | "--help") {
        print_help(program_name);
        std::process::exit(0);
    }

    if args.len() < 3 {
        print_help(program_name);
        return Err("missing required arguments: host and port".into());
    }

    let target_host = args[1].clone();
    let target_port = args[2]
        .parse::<u16>()
        .map_err(|_| format!("invalid port: {}", args[2]))?;

    Ok((target_host, target_port))
}

fn name_entry_by_nid(certificate: &X509Ref, nid: Nid) -> Option<String> {
    certificate
        .subject_name()
        .entries_by_nid(nid)
        .next()
        .and_then(|entry| entry.data().as_utf8().ok())
        .map(|text| text.trim().replace(['\r', '\n'], " "))
        .filter(|text| !text.is_empty())
}

fn issuer_entry_by_nid(certificate: &X509Ref, nid: Nid) -> Option<String> {
    certificate
        .issuer_name()
        .entries_by_nid(nid)
        .next()
        .and_then(|entry| entry.data().as_utf8().ok())
        .map(|text| text.trim().replace(['\r', '\n'], " "))
        .filter(|text| !text.is_empty())
}

fn certificate_label(certificate: &X509Ref, fallback_index: usize) -> String {
    for nid in [Nid::COMMONNAME, Nid::ORGANIZATIONNAME] {
        if let Some(text) = name_entry_by_nid(certificate, nid) {
            return text;
        }
    }

    format!("certificate_{:03}", fallback_index)
}

fn load_native_root_certificates() -> Vec<X509> {
    let native_certificates = load_native_certs();

    for error in &native_certificates.errors {
        eprintln!(
            "Warning: could not load some native certificates: {}",
            error
        );
    }

    native_certificates
        .certs
        .into_iter()
        .filter_map(|certificate| X509::from_der(certificate.as_ref()).ok())
        .collect()
}

fn find_missing_root_certificate(chain: &[X509], root_certificates: &[X509]) -> Option<X509> {
    let last_certificate = chain.last()?;
    let issuer_common_name = issuer_entry_by_nid(last_certificate, Nid::COMMONNAME)?;

    let already_present = chain.iter().any(|certificate| {
        name_entry_by_nid(certificate, Nid::COMMONNAME).as_deref()
            == Some(issuer_common_name.as_str())
    });

    if already_present {
        return None;
    }

    root_certificates.iter().find_map(|certificate| {
        let subject_common_name = name_entry_by_nid(certificate, Nid::COMMONNAME)?;

        if subject_common_name == issuer_common_name {
            Some(certificate.clone())
        } else {
            None
        }
    })
}

fn annotate_pem(label: &str, pem: &[u8]) -> Vec<u8> {
    let mut annotated_pem = Vec::new();
    annotated_pem.extend_from_slice(format!("# {}\n", label).as_bytes());
    annotated_pem.extend_from_slice(pem);

    if !pem.ends_with(b"\n") {
        annotated_pem.push(b'\n');
    }

    annotated_pem
}

fn annotate_bundle_pem(label: &str, pem: &[u8]) -> Vec<u8> {
    let mut annotated_pem = annotate_pem(label, pem);
    annotated_pem.push(b'\n');
    annotated_pem
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (target_host, target_port) = parse_args()?;
    let target_address = format!("{}:{}", target_host, target_port);
    let bundle_file_name = "ca-certificates.crt";
    let native_root_certificates = load_native_root_certificates();

    // Configure the connector to build a verified chain using the default
    // trust store, but keep the handshake permissive so certificates can
    // still be inspected even when verification is not fully successful.
    let mut builder = SslConnector::builder(SslMethod::tls())?;

    builder.set_default_verify_paths()?;
    for certificate in &native_root_certificates {
        if let Err(error) = builder.cert_store_mut().add_cert(certificate.clone()) {
            let error_message = error.to_string();

            if !error_message.contains("cert already in hash table") {
                eprintln!(
                    "Warning: could not add a native root certificate to the trust store: {}",
                    error
                );
            }
        }
    }
    builder.set_verify_callback(SslVerifyMode::PEER, |_, _| true);

    let connector = builder.build();

    // Open the TCP connection.
    let tcp_stream = TcpStream::connect(&target_address)?;

    // Start the TLS handshake.
    let ssl_stream = connector
        .connect(&target_host, tcp_stream)
        .map_err(|error| format!("TLS handshake error: {}", error))?;

    let verify_result = ssl_stream.ssl().verify_result();
    let mut chain: Vec<X509> = ssl_stream
        .ssl()
        .verified_chain()
        .or_else(|| ssl_stream.ssl().peer_cert_chain())
        .map(|stack| {
            stack
                .iter()
                .map(|certificate| certificate.to_owned())
                .collect()
        })
        .unwrap_or_default();

    if let Some(root_certificate) = find_missing_root_certificate(&chain, &native_root_certificates)
    {
        chain.push(root_certificate);
    }

    // Extract the certificate chain from the peer.
    if !chain.is_empty() {
        println!("Verification result: {:?}", verify_result);
        println!("--- Found {} certificates in the chain ---\n", chain.len());
        let mut bundle_content = Vec::new();

        for (index, certificate) in chain.iter().enumerate() {
            println!("Certificate [{}]:", index);
            println!("Subject: {:?}", certificate.subject_name());
            println!("Issuer:  {:?}", certificate.issuer_name());

            // Convert the certificate to PEM so it can be saved or inspected.
            let pem = certificate.to_pem()?;
            let certificate_name = certificate_label(certificate, index + 1);
            let annotated_pem = annotate_pem(&certificate_name, &pem);
            let bundle_pem = annotate_bundle_pem(&certificate_name, &pem);
            let file_name = format!("cert_{:03}.crt", index + 1);

            fs::write(&file_name, &annotated_pem)?;
            bundle_content.extend_from_slice(&bundle_pem);

            println!("{}", String::from_utf8(pem)?);
            println!("Label: {}", certificate_name);
            println!("Saved to: {}", file_name);
            println!("-------------------------------------------\n");
        }

        fs::write(bundle_file_name, &bundle_content)?;
        println!("Bundle saved to: {}", bundle_file_name);
    } else {
        println!("No certificates received from the server.");
    }

    Ok(())
}

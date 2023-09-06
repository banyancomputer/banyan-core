pub fn pretty_fingerprint(pem: &str) -> String {
    openssl::sha::sha1(pem.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join(":")
}

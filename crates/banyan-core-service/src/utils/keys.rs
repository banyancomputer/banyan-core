
pub fn pretty_fingerprint(pem: &str) -> String {
    pem
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join(":")
}
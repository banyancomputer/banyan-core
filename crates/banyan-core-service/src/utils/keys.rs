use jwt_simple::prelude::*;
use sha1::Sha1;
use sha2::{Digest, Sha256};

pub fn sha1_fingerprint_publickey(public_key: &ES384PublicKey) -> String {
    let compressed_point = public_key.public_key().as_ref().to_encoded_point(true);

    let mut hasher = Sha1::new();
    hasher.update(compressed_point);
    let hashed_bytes = hasher.finalize();

    format_fingerprint_bytes(&hashed_bytes)
}

#[allow(dead_code)]
pub fn sha256_fingerprint_publickey(public_key: &ES384PublicKey) -> String {
    let compressed_point = public_key.public_key().as_ref().to_encoded_point(true);

    let mut hasher = Sha256::new();
    hasher.update(compressed_point);
    let hashed_bytes = hasher.finalize();

    format_fingerprint_bytes(&hashed_bytes)
}

fn format_fingerprint_bytes(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |chain, byte| format!("{chain}{byte:02x}"))
}

use jwt_simple::prelude::*;
use blake3::Hasher;

/// Number of bytes present in an unformatted fingerprint.
pub const FINGERPRINT_SIZE: usize = 20;

/// Compute the blake3 fingerprint of a public key.
pub fn fingerprint_public_key(public_key: &ES384PublicKey) -> String {
    let compressed_point = public_key.public_key().as_ref().to_encoded_point(true);
    let compressed_point = compressed_point.as_bytes();
    let mut hasher = Hasher::new();
    hasher.update(compressed_point);
    let mut output = [0u8; FINGERPRINT_SIZE];
    let mut output_reader = hasher.finalize_xof();
    output_reader.fill(&mut output);
    output
        .iter()
        .fold(String::new(), |chain, byte| format!("{chain}{byte:02x}"))
}
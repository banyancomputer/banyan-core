use std::ops::Deref;
use std::sync::Arc;

use blake3::Hasher;
use jwt_simple::algorithms::{ES384KeyPair, ES384PublicKey};
use jwt_simple::prelude::*;

/// Number of bytes present in an unformatted fingerprint.
pub const FINGERPRINT_SIZE: usize = 20;

/// Verification key for verifying singnature of JWTs.
#[derive(Clone)]
pub struct VerificationKey(pub Arc<ES384PublicKey>);

impl Deref for VerificationKey {
    type Target = Arc<ES384PublicKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

/// Compute the blake3 fingerprint of a key pair.
pub fn fingerprint_key_pair(keys: &ES384KeyPair) -> String {
    let key_pair = keys.key_pair();
    let public_key = key_pair.public_key();
    let compressed_point = public_key.as_ref().to_encoded_point(true);
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

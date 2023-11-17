use sha1::{Digest, Sha1};
use std::ops::Deref;
use std::sync::Arc;

use jwt_simple::prelude::*;

/// Verification key for verifying singnature of JWTs.
#[derive(Clone)]
pub struct VerificationKey(Arc<ES384PublicKey>);

impl VerificationKey {
    pub fn new(key: ES384PublicKey) -> Self {
        Self(Arc::new(key))
    }
}

impl Deref for VerificationKey {
    type Target = Arc<ES384PublicKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Key pair for signing JWTs.
#[derive(Clone)]
pub struct SigningKey(Arc<ES384KeyPair>);

impl SigningKey {
    pub fn new(key: ES384KeyPair) -> Self {
        Self(Arc::new(key))
    }

    pub fn verifier(&self) -> VerificationKey {
        let key_pair = self.0.clone();
        VerificationKey::new(key_pair.public_key())
    }
}

impl Deref for SigningKey {
    type Target = Arc<ES384KeyPair>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn sha1_fingerprint_publickey(public_key: &ES384PublicKey) -> String {
    let compressed_point = public_key.public_key().as_ref().to_encoded_point(true);

    let mut hasher = Sha1::new();
    hasher.update(compressed_point);
    let hashed_bytes = hasher.finalize();

    format_fingerprint_bytes(&hashed_bytes)
}

fn format_fingerprint_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::new(), |chain, byte| format!("{chain}{byte:02x}"))
}

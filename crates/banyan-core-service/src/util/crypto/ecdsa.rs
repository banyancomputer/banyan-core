use pem::Pem;
use ring::pkcs8::Document;
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};

use crate::util::crypto::CryptoError;

pub fn decode_private_key(data: &[u8]) -> Result<EcdsaKeyPair, CryptoError> {
    let pem_data = pem::parse(data).map_err(CryptoError::invalid_pem)?;

    if pem_data.tag != "ECDSA PRIVATE KEY" {
        return Err(CryptoError::not_private_key());
    }

    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pem_data.contents.as_ref())
        .map_err(CryptoError::invalid_key_type)
}

pub fn decode_public_key(data: &[u8]) -> Result<(), CryptoError> {
    todo!()
}

pub fn encode_private_key(private_key: &Document) -> Result<String, CryptoError> {
    let pem = Pem {
        tag: "ECDSA PRIVATE KEY".to_string(),
        contents: private_key.as_ref().to_vec(),
    };

    Ok(pem::encode(&pem))
}

pub fn encode_public_key(data: &[u8]) -> Result<(), CryptoError> {
    todo!()
}

pub fn generate() -> Result<Document, CryptoError> {
    let rng = SystemRandom::new();
    EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
        .map_err(|_| CryptoError::key_gen_failed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_lifecycle() {
        let private_key = generate().expect("generation to succeed");

        let encoded_private_key =
            encode_private_key(&private_key).expect("private key encoding to succeed");
        let decoded_private_key = decode_private_key(encoded_private_key.as_ref())
            .expect("private key decoding to succeed");
        //assert_eq!(private_key.as_ref(), decoded_private_key.as_ref(), "decoded private key should match the original");
    }
}

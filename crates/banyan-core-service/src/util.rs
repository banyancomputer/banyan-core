#![allow(dead_code)]
use std::error::Error;

use pem::Pem;
use ring::agreement::{EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519};
use ring::pkcs8::Document;
use ring::rand::SystemRandom;
use ring::signature::{ECDSA_P256_SHA256_FIXED_SIGNING, EcdsaKeyPair};

pub fn collect_error_messages(base_error: impl Error) -> Vec<String> {
    let mut errors = vec![base_error.to_string()];
    let mut source = base_error.source();

    while let Some(err) = source {
        errors.push(err.to_string());
        source = err.source();
    }

    errors
}

#[derive(Debug)]
#[non_exhaustive]
pub struct CryptoError {
    kind: CryptoErrorKind,
}

impl CryptoError {
    fn incorrect_algorithm() -> Self {
        Self {
            kind: CryptoErrorKind::IncorrectAlgorithm,
        }
    }

    fn invalid_key_type(err: ring::error::KeyRejected) -> Self {
        Self {
            kind: CryptoErrorKind::InvalidKeyType(err),
        }
    }

    fn invalid_pem(err: pem::PemError) -> Self {
        Self {
            kind: CryptoErrorKind::InvalidPem(err),
        }
    }

    fn key_gen_failed(err: ring::error::Unspecified) -> Self {
        Self {
            kind: CryptoErrorKind::KeyGenerationFailed(err),
        }
    }

    fn key_parse_failed(err: ring::error::KeyRejected) -> Self {
        Self {
            kind: CryptoErrorKind::KeyParseFailed(err),
        }
    }

    fn not_private_key() -> Self {
        Self {
            kind: CryptoErrorKind::NotPrivateKey,
        }
    }

    fn not_public_key() -> Self {
        Self {
            kind: CryptoErrorKind::NotPublicKey,
        }
    }

    fn public_key_decode_failed(err: asn1::ParseError) -> Self {
        Self {
            kind: CryptoErrorKind::PublicKeyDecodeFailed(err),
        }
    }

    fn public_key_encode_failed(err: asn1::WriteError) -> Self {
        Self {
            kind: CryptoErrorKind::PublicKeyEncodeFailed(err),
        }
    }
}

#[derive(Debug)]
enum CryptoErrorKind {
    PublicKeyDecodeFailed(asn1::ParseError),
    PublicKeyEncodeFailed(asn1::WriteError),
    IncorrectAlgorithm,
    InvalidKeyType(ring::error::KeyRejected),
    InvalidPem(pem::PemError),
    KeyGenerationFailed(ring::error::Unspecified),
    KeyParseFailed(ring::error::KeyRejected),
    NotPrivateKey,
    NotPublicKey,
}

pub fn decode_ecdh_public_key(data: &[u8]) -> Result<UnparsedPublicKey<Vec<u8>>, CryptoError> {
    let pem_data = pem::parse(data).map_err(CryptoError::invalid_pem)?;

    if pem_data.tag != "ECDH PUBLIC KEY" {
        return Err(CryptoError::not_public_key());
    }

    let der_encoding = pem_data.contents;
    let decoding_result: asn1::ParseResult<_> = asn1::parse(&der_encoding, |d| {
        let oid = d.read_element::<asn1::Sequence>()?.parse(|d| {
            let oid = d.read_element::<asn1::ObjectIdentifier>()?;
            d.read_element::<asn1::Null>()?;
            Ok(oid)
        })?;

        let read_bytes = d.read_element::<asn1::BitString>()?;

        Ok((oid, read_bytes.as_bytes().to_vec()))
    });

    let (oid, read_bytes) = decoding_result.map_err(CryptoError::public_key_decode_failed)?;

    if oid != asn1::oid!(1, 3, 101, 110) {
        return Err(CryptoError::incorrect_algorithm());
    }

    Ok(UnparsedPublicKey::new(&X25519, read_bytes))
}

pub fn encode_ecdh_public_key(public_key: &PublicKey) -> Result<String, CryptoError> {
    let public_key_bytes = public_key.as_ref().to_vec();

    // We're building up a modified version of PKCS1 here with the only difference being the OID
    // for the public key's algorithm to made the one allocated for X25519 and the encoding of the
    // public key itself which is just a bitstring of the computed public key.
    let encoding_result = asn1::write(|w| {
        w.write_element(&asn1::SequenceWriter::new(&|w| {
            w.write_element(&asn1::oid!(1, 3, 101, 110))?;
            w.write_element(&()) // This is the same as asn1::Null, but I can't use that here
        }))?;

        w.write_element(&asn1::BitString::new(public_key_bytes.as_ref(), 0))
    });

    let der_bytes = encoding_result.map_err(CryptoError::public_key_encode_failed)?;

    let pem = Pem {
        tag: "ECDH PUBLIC KEY".to_string(),
        contents: der_bytes,
    };

    Ok(pem::encode(&pem))
}

pub fn encode_ecdsa_private_key(private_key: &Document) -> Result<String, CryptoError> {
    let pem = Pem {
        tag: "ECDSA PRIVATE KEY".to_string(),
        contents: private_key.as_ref().to_vec(),
    };

    Ok(pem::encode(&pem))
}

pub fn decode_ecdsa_private_key(data: &[u8]) -> Result<EcdsaKeyPair, CryptoError> {
    let pem_data = pem::parse(data).map_err(CryptoError::invalid_pem)?;

    if pem_data.tag != "ECDSA PRIVATE KEY" {
        return Err(CryptoError::not_private_key());
    }

    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pem_data.contents.as_ref())
        .map_err(CryptoError::invalid_key_type)
}

pub fn generate_ecdh_key_pair() -> Result<EphemeralPrivateKey, CryptoError> {
    let rng = SystemRandom::new();
    EphemeralPrivateKey::generate(&X25519, &rng).map_err(CryptoError::key_gen_failed)
}

pub fn generate_ecdsa_key_pair() -> Result<Document, CryptoError> {
    let rng = SystemRandom::new();
    EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).map_err(CryptoError::key_gen_failed)
}

//pub fn extract_private_key(pkcs8_doc: Document) -> Result<EcdsaKeyPair, CryptoError> {
//    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8_doc.as_ref())
//        .map_err(CryptoError::key_parse_failed)
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecdh_key_lifecycle() {
        let private_key = generate_ecdh_key_pair().expect("generation to succeed");
        let public_key = private_key.compute_public_key().expect("to generate public key");

        let encoded_public_key = encode_ecdh_public_key(&public_key).expect("public key encoding to succeed");
        let decoded_public_key = decode_ecdh_public_key(&encoded_public_key.as_ref()).expect("public key decoding to succeed");
        assert_eq!(public_key.as_ref(), decoded_public_key.bytes(), "decoded public key should match the original");
    }

    #[test]
    fn test_ecdsa_key_lifecycle() {
        let private_key = generate_ecdsa_key_pair().expect("generation to succeed");

        let encoded_private_key = encode_ecdsa_private_key(&private_key).expect("private key encoding to succeed");
        let _decoded_private_key = decode_ecdsa_private_key(&encoded_private_key.as_ref()).expect("private key decoding to succeed");
        //assert_eq!(private_key.as_ref(), encoded_private_key.as_ref(), "decoded private key should match the original");
    }
}

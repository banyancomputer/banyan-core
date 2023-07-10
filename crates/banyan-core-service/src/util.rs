#![allow(dead_code)]
use std::error::Error;

use pem::Pem;
use ring::agreement::{EphemeralPrivateKey, PublicKey, X25519};
use ring::rand::SystemRandom;
use simple_asn1::{ASN1Block, ASN1EncodeErr};
//use spki::SubjectPublicKeyInfo;

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

    fn public_key_encode_failed(err: ASN1EncodeErr) -> Self {
        Self {
            kind: CryptoErrorKind::PublicKeyEncodeFailed(err),
        }
    }
}

#[derive(Debug)]
enum CryptoErrorKind {
    PublicKeyEncodeFailed(ASN1EncodeErr),
    InvalidKeyType(ring::error::KeyRejected),
    InvalidPem(pem::PemError),
    KeyGenerationFailed(ring::error::Unspecified),
    KeyParseFailed(ring::error::KeyRejected),
    NotPrivateKey,
    NotPublicKey,
}

//pub fn public_from_pem(data: Vec<u8>) -> Result<String, CryptoError> {
//    let pem_data = pem::parse(data).map_err(CryptoError::invalid_pem)?;
//
//    if pem_data.tag != "EC PUBLIC KEY" {
//        return Err(CryptoError::not_public_key());
//    }
//
//    //UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED_SIGNING, pem_data.contents) ???
//
//    todo!()
//}

pub fn encode_public_key(public_key: PublicKey) -> Result<String, CryptoError> {
    let alg_id = ASN1Block::Sequence(
        0,
        vec![
            ASN1Block::ObjectIdentifier(0, simple_asn1::oid!(1, 3, 101, 110)), // OID for X25519
            ASN1Block::Null(0),
        ],
    );

    let public_key_bytes = public_key.as_ref().to_vec();
    let asn_key = ASN1Block::BitString(0, public_key_bytes.len(), public_key_bytes);
    let raw_spki = ASN1Block::Sequence(0, vec![alg_id, asn_key]);

    let der_encoding =
        simple_asn1::to_der(&raw_spki).map_err(CryptoError::public_key_encode_failed)?;

    let pem = Pem {
        tag: "EC PUBLIC KEY".to_string(),
        contents: der_encoding,
    };

    Ok(pem::encode(&pem))
}

//pub fn encode_private_key(private_key: Document) -> Result<String, CryptoError> {
//    let pem = Pem {
//        tag: "EC PRIVATE KEY".to_string(),
//        contents: private_key.as_ref().to_vec(),
//    };
//
//    Ok(pem::encode(&pem))
//}

//pub fn private_from_pem(data: Vec<u8>) -> Result<EcdsaKeyPair, CryptoError> {
//    let pem_data = pem::parse(data).map_err(CryptoError::invalid_pem)?;
//
//    if pem_data.tag != "EC PRIVATE KEY" {
//        return Err(CryptoError::not_private_key());
//    }
//
//    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pem_data.contents.as_ref())
//        .map_err(CryptoError::invalid_key_type)
//}

pub fn generate_ecdsa_key_pair() -> Result<EphemeralPrivateKey, CryptoError> {
    let rng = SystemRandom::new();
    EphemeralPrivateKey::generate(&X25519, &rng).map_err(CryptoError::key_gen_failed)
}

//pub fn extract_private_key(pkcs8_doc: Document) -> Result<EcdsaKeyPair, CryptoError> {
//    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8_doc.as_ref())
//        .map_err(CryptoError::key_parse_failed)
//}

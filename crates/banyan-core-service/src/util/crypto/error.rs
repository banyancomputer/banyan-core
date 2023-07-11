use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
#[non_exhaustive]
pub struct CryptoError {
    kind: CryptoErrorKind,
}

impl CryptoError {
    pub(crate) fn incorrect_algorithm() -> Self {
        Self {
            kind: CryptoErrorKind::IncorrectAlgorithm,
        }
    }

    pub(crate) fn invalid_key_type(err: ring::error::KeyRejected) -> Self {
        Self {
            kind: CryptoErrorKind::InvalidKeyType(err),
        }
    }

    pub(crate) fn invalid_pem(err: pem::PemError) -> Self {
        Self {
            kind: CryptoErrorKind::InvalidPem(err),
        }
    }

    pub(crate) fn key_gen_failed() -> Self {
        Self {
            kind: CryptoErrorKind::KeyGenerationFailed,
        }
    }

    pub(crate) fn key_parse_failed(err: ring::error::KeyRejected) -> Self {
        Self {
            kind: CryptoErrorKind::KeyParseFailed(err),
        }
    }

    pub(crate) fn not_private_key() -> Self {
        Self {
            kind: CryptoErrorKind::NotPrivateKey,
        }
    }

    pub(crate) fn not_public_key() -> Self {
        Self {
            kind: CryptoErrorKind::NotPublicKey,
        }
    }

    pub(crate) fn public_key_decode_failed(err: asn1::ParseError) -> Self {
        Self {
            kind: CryptoErrorKind::PublicKeyDecodeFailed(err),
        }
    }

    pub(crate) fn public_key_encode_failed(err: asn1::WriteError) -> Self {
        Self {
            kind: CryptoErrorKind::PublicKeyEncodeFailed(err),
        }
    }
}

#[derive(Debug)]
enum CryptoErrorKind {
    PublicKeyDecodeFailed(asn1::ParseError),
    PublicKeyEncodeFailed(asn1::WriteError),

    InvalidPem(pem::PemError),
    KeyGenerationFailed,

    InvalidKeyType(ring::error::KeyRejected),
    KeyParseFailed(ring::error::KeyRejected),

    IncorrectAlgorithm,
    NotPrivateKey,
    NotPublicKey,
}

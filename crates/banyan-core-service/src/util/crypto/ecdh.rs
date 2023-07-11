use pem::Pem;
use ring::agreement::{EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519};
use ring::rand::SystemRandom;

use crate::util::crypto::CryptoError;

pub fn decode_public_key(data: &[u8]) -> Result<UnparsedPublicKey<Vec<u8>>, CryptoError> {
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

pub fn encode_public_key(public_key: &PublicKey) -> Result<String, CryptoError> {
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

pub fn generate() -> Result<EphemeralPrivateKey, CryptoError> {
    let rng = SystemRandom::new();
    EphemeralPrivateKey::generate(&X25519, &rng).map_err(|_| { CryptoError::key_gen_failed() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_lifecycle() {
        let private_key = generate().expect("generation to succeed");
        let public_key = private_key.compute_public_key().expect("to generate public key");

        let encoded_public_key = encode_public_key(&public_key).expect("public key encoding to succeed");
        let decoded_public_key = decode_public_key(&encoded_public_key.as_ref()).expect("public key decoding to succeed");
        assert_eq!(public_key.as_ref(), decoded_public_key.bytes(), "decoded public key should match the original");
    }
}

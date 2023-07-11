use std::error::Error;

pub mod crypto;

pub fn collect_error_messages(base_error: impl Error) -> Vec<String> {
    let mut errors = vec![base_error.to_string()];
    let mut source = base_error.source();

    while let Some(err) = source {
        errors.push(err.to_string());
        source = err.source();
    }

    errors
}

//pub fn extract_private_key(pkcs8_doc: Document) -> Result<EcdsaKeyPair, CryptoError> {
//    EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8_doc.as_ref())
//        .map_err(CryptoError::key_parse_failed)
//}

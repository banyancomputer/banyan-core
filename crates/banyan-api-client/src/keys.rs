use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private, Public};

pub fn create_private_ec_pem() -> String {
    let private_key: PKey<Private> = {
        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let ec_key = EcKey::generate(&ec_group).unwrap();
        ec_key.try_into().unwrap()
    };

    String::from_utf8(private_key.private_key_to_pem_pkcs8().unwrap()).unwrap()
}

pub fn fingerprint_public_pem(public_pem: &str) -> String {
    let public_key = PKey::public_key_from_pem(public_pem.as_bytes()).unwrap();

    let fingerprint_bytes = {
        use openssl::bn::BigNumContext;
        use openssl::ec::PointConversionForm;

        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let mut big_num_ctx = BigNumContext::new().unwrap();

        let ec_pub_key = public_key.ec_key().unwrap();
        let compressed_key = ec_pub_key
            .public_key()
            .to_bytes(&ec_group, PointConversionForm::COMPRESSED, &mut big_num_ctx)
            .unwrap();

        openssl::sha::sha1(&compressed_key)
    };

    fingerprint_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join(":")
}

pub fn public_from_private(private_pem: &str) -> String {
    let private_key = PKey::private_key_from_pem(private_pem.as_bytes()).unwrap();

    let public_key: PKey<Public> = {
        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let priv_ec_key = private_key.ec_key().unwrap();
        let pub_ec_key: EcKey<Public> =
            EcKey::from_public_key(&ec_group, priv_ec_key.public_key()).unwrap();

        PKey::from_ec_key(pub_ec_key).unwrap()
    };

    String::from_utf8(public_key.public_key_to_pem().unwrap()).unwrap()
}

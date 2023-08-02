use openssl::bn::BigNumContext;
use openssl::ec::{EcGroup, EcKey, PointConversionForm};
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private, Public};
use reqwest::{Client, RequestBuilder, Url};
use serde::Deserialize;
use uuid::Uuid;

use crate::requests::{ApiRequest, InfallibleError};

pub fn create_private_ec_pem() -> String {
    let private_key: PKey<Private> = {
        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let ec_key = EcKey::generate(&ec_group).unwrap();
        ec_key.try_into().unwrap()
    };

    String::from_utf8(private_key.private_key_to_pem_pkcs8().unwrap()).unwrap()
}

#[derive(Debug)]
pub struct RegisterFakeAccount;

impl ApiRequest for RegisterFakeAccount {
    type ResponseType = RegisterFakeAccountResponse;
    type ErrorType = InfallibleError;

    fn build_request(&self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url.join("/api/v1/auth/create_fake_account").unwrap();
        client.get(full_url)
    }

    fn requires_authentication(&self) -> bool {
        false
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterFakeAccountResponse {
    pub id: Uuid,
}

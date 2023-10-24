use oauth2::{AuthUrl, RevocationUrl, TokenUrl};

pub struct ProviderConfig {
    auth_url: &'static str,
    token_url: Option<&'static str>,
    revocation_url: Option<&'static str>,
    scopes: &'static [&'static str],
}

impl ProviderConfig {
    pub fn auth_url(&self) -> AuthUrl {
        AuthUrl::new(self.auth_url.to_string()).expect("static auth url to be valid")
    }

    pub const fn new(
        auth_url: &'static str,
        token_url: Option<&'static str>,
        revocation_url: Option<&'static str>,
        scopes: &'static [&'static str],
    ) -> Self {
        Self {
            auth_url,
            token_url,
            revocation_url,
            scopes,
        }
    }

    pub fn revocation_url(&self) -> Option<RevocationUrl> {
        self.revocation_url.map(|ru| {
            RevocationUrl::new(ru.to_string()).expect("static revocation url to be valid")
        })
    }

    pub fn scopes(&self) -> &'static [&'static str] {
        self.scopes
    }

    pub fn token_url(&self) -> Option<TokenUrl> {
        self.token_url
            .map(|tu| TokenUrl::new(tu.to_string()).expect("static token url to be valid"))
    }
}

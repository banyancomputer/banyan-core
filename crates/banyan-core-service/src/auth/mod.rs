use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use oauth2::basic::BasicClient;
use oauth2::RedirectUrl;
use url::Url;

use crate::app::{AppState, Secrets};

mod authentication_error;
//mod login;
//mod logout;
//mod oauth_callback;
mod provider_config;

use authentication_error::AuthenticationError;
use provider_config::ProviderConfig;

pub static CALLBACK_PATH_TEMPLATE: &str = "/auth/callback/{}";

pub static LOGIN_PATH: &str = "/auth/login";

pub const NEW_USER_COOKIE_NAME: &'static str = "_is_new_user";

pub static PROVIDER_CONFIGS: phf::Map<&'static str, ProviderConfig> = phf::phf_map! {
    "google" => ProviderConfig::new(
        "https://accounts.google.com/o/oauth2/v2/auth",
        Some("https://www.googleapis.com/oauth2/v3/token"),
        Some("https://oauth2.googleapis.com/revoke"),
        &[
            "https://www.googleapis.com/auth/userinfo.email",
            "https://www.googleapis.com/auth/userinfo.profile"
        ],
    ),
};

pub static SESSION_COOKIE_NAME: &str = "_session_id";

pub const SESSION_TTL: u64 = 28 * 24 * 60 * 60;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        //.route("/callback/:provider", get(oauth_callback::handler))
        //.route("/login/:provider", get(login::handler))
        //.route("/logout", get(logout::handler))
        .with_state(state)
}

fn oauth_client(
    config_id: &str,
    hostname: Url,
    secrets: Secrets,
) -> Result<BasicClient, AuthenticationError> {
    let provider_config = PROVIDER_CONFIGS
        .get(config_id)
        .ok_or(AuthenticationError::UnknownProvider)?;
    let provider_credentials = secrets.provider_credential(config_id).ok_or(
        AuthenticationError::ProviderNotConfigured(config_id.to_string()),
    )?;

    let auth_url = provider_config.auth_url();
    let token_url = provider_config.token_url();

    let mut redirect_url = hostname;
    redirect_url.set_path(&CALLBACK_PATH_TEMPLATE.replace("{}", config_id));
    let redirect_url = RedirectUrl::from_url(redirect_url);

    let mut client = BasicClient::new(
        provider_credentials.id(),
        Some(provider_credentials.secret()),
        auth_url,
        token_url,
    )
    .set_redirect_uri(redirect_url);

    if let Some(ru) = provider_config.revocation_url() {
        client = client.set_revocation_uri(ru);
    }

    Ok(client)
}

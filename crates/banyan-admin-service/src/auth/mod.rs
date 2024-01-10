use authentication_error::AuthenticationError;
use axum::body::HttpBody;
use axum::routing::get;
use axum::Router;
use oauth2::basic::BasicClient;
use oauth2::RedirectUrl;
use provider_config::ProviderConfig;
use url::Url;

use crate::app::{AppState, Secrets};

mod authentication_error;
mod login;
mod logout;
mod oauth_callback;
mod provider_config;

pub static CALLBACK_PATH_TEMPLATE: &str = "/auth/callback/{}";

pub static LOGIN_PATH: &str = "/auth/login";

pub const NEW_USER_COOKIE_NAME: &str = "_is_new_user";

pub static GOOGLE_PROVIDER_CONFIG: ProviderConfig = ProviderConfig::new(
    "https://accounts.google.com/o/oauth2/v2/auth",
    Some("https://www.googleapis.com/oauth2/v3/token"),
    Some("https://oauth2.googleapis.com/revoke"),
    &[
        "https://www.googleapis.com/auth/userinfo.email",
        "https://www.googleapis.com/auth/userinfo.profile",
    ],
);

/// Name of the cookie used to store the session identifier
pub static SESSION_COOKIE_NAME: &str = "_session_id";

/// Name of the cookie used to store user related data
pub static USER_DATA_COOKIE_NAME: &str = "_user_data";

pub const LOCAL_KEY_COOKIE_NAME: &str = "_local_key";

pub const SESSION_TTL: u64 = 28 * 24 * 60 * 60;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
{
    Router::new()
        .route("/callback/:provider", get(oauth_callback::handler))
        .route("/login/:provider", get(login::handler))
        .route("/logout", get(logout::handler))
        .with_state(state)
}

fn oauth_client(
    config_id: &str,
    hostname: Url,
    secrets: Secrets,
) -> Result<BasicClient, AuthenticationError> {
    if config_id != "google" {
        return Err(AuthenticationError::ProviderNotConfigured(
            config_id.to_string(),
        ));
    }

    let provider_credentials = secrets.provider_credential(config_id).ok_or(
        AuthenticationError::ProviderNotConfigured(config_id.to_string()),
    )?;

    let auth_url = GOOGLE_PROVIDER_CONFIG.auth_url();
    let token_url = GOOGLE_PROVIDER_CONFIG.token_url();

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

    if let Some(ru) = GOOGLE_PROVIDER_CONFIG.revocation_url() {
        client = client.set_revocation_uri(ru);
    }

    Ok(client)
}

use axum::extract::{Host, Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use http::StatusCode;
use oauth2::basic::BasicClient;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse};
use serde::Deserialize;
use url::Url;

use crate::app::{Secrets, State as AppState};
use crate::database::Database;
use crate::extractors::{SessionIdentity, LOGIN_PATH, SESSION_COOKIE_NAME};

mod authentication_error;
mod provider_config;

use authentication_error::AuthenticationError;
use provider_config::ProviderConfig;

static CALLBACK_PATH_TEMPLATE: &str = "/auth/callback/{}";

static PROVIDER_CONFIGS: phf::Map<&'static str, ProviderConfig> = phf::phf_map! {
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

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/callback/:provider", get(oauth_callback))
        .route("/login/:provider", get(login_handler))
        .route("/logout", get(logout_handler))
        .with_state(state)
}

#[axum::debug_handler]
pub async fn login_handler(
    session: Option<SessionIdentity>,
    State(state): State<AppState>,
    Host(hostname): Host,
    Path(provider): Path<String>,
    Query(params): Query<LoginParams>,
) -> Response {
    if session.is_some() {
        return Redirect::to(&params.next_url.unwrap_or("/".to_string())).into_response();
    }

    let provider_config = match PROVIDER_CONFIGS.get(&provider) {
        Some(pc) => pc,
        None => {
            tracing::error!("attempted to login using unknown provider '{provider}'");
            let response = serde_json::json!({"msg": "provider is not recognized on this server"});
            return (StatusCode::NOT_FOUND, Json(response)).into_response();
        }
    };

    // todo: should return an error here
    let hostname = Url::parse(&hostname).expect("host to be valid");
    let oauth_client = match oauth_client(&provider, hostname, state.secrets()) {
        Ok(oc) => oc,
        Err(err) => {
            tracing::error!("failed to build oauth client: {err}");
            let response = serde_json::json!({"msg": "unable to use login services"});
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut auth_request = oauth_client.authorize_url(CsrfToken::new_random);

    for scope in provider_config.scopes() {
        auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
    }

    let (authorize_url, csrf_state) = auth_request.set_pkce_challenge(pkce_code_challenge).url();

    let csrf_secret = csrf_state.secret();
    let pkce_verifier_secret = pkce_code_verifier.secret();

    let query = sqlx::query!(
        r#"INSERT INTO oauth_state (csrf_secret, pkce_verifier_secret, next_url)
                   VALUES (?, ?, ?);"#,
        csrf_secret,
        pkce_verifier_secret,
        params.next_url,
    );

    if let Err(err) = query.execute(state.database()).await {
        tracing::error!("failed to create oauth session handle: {err}");
        let response = serde_json::json!({"msg": "unable to use login services"});
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
    };

    Redirect::to(authorize_url.as_str()).into_response()
}

pub async fn logout_handler(
    session: Option<SessionIdentity>,
    database: Database,
    mut cookie_jar: CookieJar,
) -> Response {
    if let Some(sid) = session {
        let session_id = sid.session_id();

        // todo: revoke token?

        let query = sqlx::query!("DELETE FROM sessions WHERE id = ?;", session_id);
        if let Err(err) = query.execute(&database).await {
            tracing::error!("failed to remove session from the db: {err}");
        }
    }

    cookie_jar = cookie_jar.remove(Cookie::named(SESSION_COOKIE_NAME));
    (cookie_jar, Redirect::to(LOGIN_PATH)).into_response()
}

pub async fn oauth_callback(
    database: Database,
    mut cookie_jar: CookieJar,
    State(state): State<AppState>,
    Host(hostname): Host,
    Path(provider): Path<String>,
    Query(params): Query<CallbackParameters>,
) -> Result<Response, AuthenticationError> {
    let csrf_secret = CsrfToken::new(params.state);
    let exchange_code = AuthorizationCode::new(params.code);

    let query_secret = csrf_secret.secret();
    let oauth_state_query: (String, Option<String>) = sqlx::query_as(
            "SELECT pkce_verifier_secret,next_url FROM oauth_state WHERE csrf_secret = ?;"
        )
        .bind(query_secret)
        .fetch_one(&database)
        .await
        .map_err(AuthenticationError::MissingCallbackState)?;

    sqlx::query!("DELETE FROM oauth_state WHERE csrf_secret = ?;", query_secret)
        .execute(&database)
        .await
        .map_err(|_| AuthenticationError::CleanupFailed)?;

    let (pkce_verifier_secret, next_url) = oauth_state_query;
    let pkce_code_verifier = PkceCodeVerifier::new(pkce_verifier_secret);

    let hostname = Url::parse(&hostname).expect("host to be valid");
    let oauth_client = oauth_client(&provider, hostname, state.secrets())?;

    let token_response = tokio::task::spawn_blocking(move || {
        oauth_client
            .exchange_code(exchange_code)
            .set_pkce_verifier(pkce_code_verifier)
            .request(oauth2::reqwest::http_client)
    })
    .await
    .map_err(AuthenticationError::SpawnFailure)?
    .map_err(|err| AuthenticationError::ExchangeCodeFailure(err.to_string()))?;

    let access_token = token_response.access_token().secret();

    let user_info_url = Url::parse_with_params(
        "https://www.googleapis.com/oauth2/v2/userinfo",
        &[("oauth_token", access_token)]
    ).expect("fixed format to be valid");

    let user_info: GoogleUserProfile = reqwest::get(user_info_url)
        .await
        .expect("building a fixed format request to succeed")
        .json()
        .await
        .map_err(AuthenticationError::ProfileUnavailable)?;

    if !user_info.verified_email {
        return Err(AuthenticationError::UnverifiedEmail);
    }

    // We're back in provider specific land for getting information about the authenticated user,
    // todo: allow for providers other than Google here...

    // todo:
    //  * reject with a nice error if the email isn't verified
    //  * find or create a new user account for the email
    //  * create a new session for the user
    //    * record it in the database
    //    * build and sign an appropriate cookie for it

    let redirect_url = next_url.unwrap_or("/".to_string());
    Ok((cookie_jar, Redirect::to(&redirect_url)).into_response())
}

fn oauth_client(
    config_id: &str,
    hostname: Url,
    secrets: &Secrets,
) -> Result<BasicClient, AuthenticationError> {
    let provider_config = PROVIDER_CONFIGS
        .get(config_id)
        .ok_or(AuthenticationError::UnknownProvider)?;
    let provider_credentials = secrets
        .provider_credential(config_id)
        .ok_or(AuthenticationError::ProviderNotConfigured(config_id.to_string()))?;

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

#[derive(Deserialize)]
pub struct CallbackParameters {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct LoginParams {
    next_url: Option<String>,
}

#[derive(Deserialize)]
pub struct GoogleUserProfile {
    email: String,
    verified_email: bool,
}

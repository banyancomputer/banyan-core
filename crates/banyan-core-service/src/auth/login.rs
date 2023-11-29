use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use http::StatusCode;
use oauth2::{CsrfToken, PkceCodeChallenge, Scope};
use serde::Deserialize;

use crate::app::AppState;
use crate::auth::{oauth_client, PROVIDER_CONFIGS};
use crate::extractors::{ServerBase, SessionIdentity};

pub async fn handler(
    session: Option<SessionIdentity>,
    State(state): State<AppState>,
    ServerBase(hostname): ServerBase,
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
    let oauth_client =
        match oauth_client(&provider, hostname, state.secrets()) {
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
        r#"INSERT INTO oauth_state (provider, csrf_secret, pkce_verifier_secret, next_url)
                   VALUES ($1, $2, $3, $4);"#,
        provider,
        csrf_secret,
        pkce_verifier_secret,
        params.next_url,
    );

    if let Err(err) = query.execute(&state.database()).await {
        tracing::error!("failed to create oauth session handle: {err}");
        let response = serde_json::json!({"msg": "unable to use login services"});
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
    };

    Redirect::to(authorize_url.as_str()).into_response()
}

#[derive(Deserialize)]
pub struct LoginParams {
    next_url: Option<String>,
}

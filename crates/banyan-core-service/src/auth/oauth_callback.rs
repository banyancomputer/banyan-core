use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine;
use ecdsa::signature::RandomizedDigestSigner;
use jwt_simple::algorithms::ECDSAP384KeyPairLike;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeVerifier, TokenResponse};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::app::AppState;

use crate::auth::{
    oauth_client, AuthenticationError,
    NEW_USER_COOKIE_NAME, SESSION_COOKIE_NAME, SESSION_TTL, USER_DATA_COOKIE_NAME,
};
use crate::api::models::{ApiUser, ApiEscrowedKeyMaterial};
use crate::extractors::ServerBase;

// Data returned in the User Data Cookie
#[derive(Serialize)]
struct UserData {
    user: ApiUser,
    escrowed_key_material: Option<ApiEscrowedKeyMaterial>,
}

pub async fn handler(
    mut cookie_jar: CookieJar,
    State(state): State<AppState>,
    ServerBase(hostname): ServerBase,
    Path(provider): Path<String>,
    Query(params): Query<CallbackParameters>,
) -> Result<Response, AuthenticationError> {
    let csrf_secret = CsrfToken::new(params.state);
    let exchange_code = AuthorizationCode::new(params.code);
    let database = state.database();

    let query_secret = csrf_secret.secret();
    let oauth_state_query: (String, Option<String>) = sqlx::query_as(
        r#"SELECT pkce_verifier_secret, next_url
            FROM oauth_state
            WHERE provider = $1 AND csrf_secret = $2;"#,
    )
    .bind(provider.clone())
    .bind(query_secret)
    .fetch_one(&database)
    .await
    .map_err(AuthenticationError::MissingCallbackState)?;

    sqlx::query!(
        r#"DELETE FROM oauth_state
            WHERE provider = $1 AND csrf_secret = $2;"#,
        provider,
        query_secret,
    )
    .execute(&database)
    .await
    .map_err(|_| AuthenticationError::CleanupFailed)?;

    let (pkce_verifier_secret, next_url) = oauth_state_query;
    let pkce_code_verifier = PkceCodeVerifier::new(pkce_verifier_secret);

    let oauth_client = oauth_client(&provider, hostname.clone(), state.secrets())?;

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
    let access_expires_at = token_response
        .expires_in()
        .map(|secs| OffsetDateTime::now_utc() + secs);
    let refresh_token = token_response.refresh_token().map(|rt| rt.secret());

    let user_info_url = Url::parse_with_params(
        "https://www.googleapis.com/oauth2/v2/userinfo",
        &[("oauth_token", access_token)],
    )
    .expect("fixed format to be valid");

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
    // TODO: allow for providers other than Google here, deprecate hard-coded parameters

    // Attempt to look up the user in the database, if they don't exist, create them
    let user_row = sqlx::query!(
        "SELECT id FROM users WHERE email = LOWER($1);",
        user_info.email
    )
    .fetch_optional(&database)
    .await
    .map_err(AuthenticationError::LookupFailed)?;

    let cookie_domain = hostname
        .host_str()
        .expect("built from a hostname")
        .to_string();
    let cookie_secure = hostname.scheme() == "https";

    let user_id = match user_row {
        Some(u) => Uuid::parse_str(&u.id.to_string()).expect("db ids to be valid"),
        None => {
            let mut transcation = database
                .begin()
                .await
                .map_err(AuthenticationError::CreationFailed)?;

            // Try creating the top level User record
            let new_user_id = sqlx::query_scalar!(
                r#"INSERT 
                    INTO users (email, verified_email, display_name, locale, profile_image)
                    VALUES (LOWER($1), $2, $3, $4, $5)
                RETURNING id;"#,
                user_info.email,
                user_info.verified_email,
                user_info.name,
                user_info.locale,
                user_info.picture,
            )
            .fetch_one(&mut *transcation)
            .await
            .map_err(AuthenticationError::CreationFailed)?;

            // TODO: rm hardcoded provider
            // Try creating the provider account record
            sqlx::query!(
                r#"INSERT 
                    INTO oauth_provider_accounts (user_id, provider, provider_id)
                    VALUES ($1, 'google', $2);"#,
                new_user_id,
                user_info.id,
            )
            .execute(&mut *transcation)
            .await
            .map_err(AuthenticationError::CreationFailed)?;

            transcation
                .commit()
                .await
                .map_err(AuthenticationError::CreationFailed)?;

            cookie_jar = cookie_jar.add(
                Cookie::build(NEW_USER_COOKIE_NAME, "yes")
                    .http_only(false)
                    .expires(None)
                    .same_site(SameSite::Lax)
                    .domain(cookie_domain.clone())
                    .secure(cookie_secure)
                    .finish(),
            );

            Uuid::parse_str(&new_user_id).expect("db ids to be valid")
        }
    };

    let expires_at = time::OffsetDateTime::now_utc() + Duration::from_secs(SESSION_TTL);
    let db_uid = user_id.clone().to_string();

    // Lookup User Data to attach include in the CookieJar
    let user = sqlx::query_as!(
        ApiUser,
        r#"SELECT 
            id, email, verified_email, display_name, locale, profile_image
        FROM users
        WHERE id = $1;"#,
        db_uid
    )
    .fetch_one(&database)
    .await
    .map_err(AuthenticationError::UserDataLookupFailed)?;

    let escrowed_key_material = sqlx::query_as!(
        ApiEscrowedKeyMaterial,
        r#"SELECT
            api_public_key_pem, encryption_public_key_pem, encrypted_private_key_material, pass_key_salt
        FROM escrowed_devices
        WHERE user_id = $1;"#,
        db_uid
    )
    .fetch_optional(&database)
    .await
    .map_err(AuthenticationError::UserDataLookupFailed)?;

    let user_data = UserData {
        user,
        escrowed_key_material,
    };

    // Create a Session to record in the database and attach to the CookieJar
    let new_sid_row = sqlx::query!(
        "INSERT INTO sessions
            (user_id, provider, access_token, access_expires_at, refresh_token, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id;",
        db_uid,
        provider,
        access_token,
        access_expires_at,
        refresh_token,
        expires_at,
    )
    .fetch_one(&database)
    .await
    .map_err(AuthenticationError::SessionSaveFailed)?;

    let session_id = Uuid::parse_str(&new_sid_row.id.to_string()).expect("db ids to be valid");

    let session_enc = B64.encode(session_id.to_bytes_le());
    let mut digest = hmac_sha512::sha384::Hash::new();
    digest.update(session_enc.as_bytes());
    let mut rng = rand::thread_rng();

    let service_signing_key = state.secrets().service_signing_key();
    let signature: ecdsa::Signature<p384::NistP384> = service_signing_key
        .key_pair()
        .as_ref()
        .sign_digest_with_rng(&mut rng, digest);

    let auth_tag = B64.encode(signature.to_vec());

    let session_value = format!("{session_enc}{auth_tag}");
    let user_data_value = serde_json::to_string(&user_data).expect("user data to serialize");

    // Populate the CookieJar
    cookie_jar = cookie_jar.add(
        Cookie::build(SESSION_COOKIE_NAME, session_value)
            .http_only(false)
            .expires(expires_at)
            .same_site(SameSite::Lax)
            .path("/")
            .domain(cookie_domain.clone())
            .secure(cookie_secure)
            .finish(),
    );
    cookie_jar = cookie_jar.add(
        Cookie::build(USER_DATA_COOKIE_NAME, user_data_value)
            .http_only(false)
            .expires(expires_at)
            .same_site(SameSite::Lax)
            .path("/")
            .domain(cookie_domain)
            .secure(cookie_secure)
            .finish(),
    );

    // TODO: make this point at the root url
    let redirect_url = next_url.unwrap_or("/dist".to_string());
    Ok((cookie_jar, Redirect::to(&redirect_url)).into_response())
}

#[derive(Deserialize)]
pub struct CallbackParameters {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct GoogleUserProfile {
    id: String,
    name: String,
    email: String,
    verified_email: bool,

    picture: String,
    locale: String,
}

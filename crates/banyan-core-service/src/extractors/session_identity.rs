#![allow(dead_code)]

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts, OriginalUri};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::CookieJar;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use base64::Engine;
use ecdsa::signature::DigestVerifier;
use http::request::Parts;
use jwt_simple::prelude::*;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app::ServiceVerificationKey;
use crate::auth::{LOGIN_PATH, SESSION_COOKIE_NAME};
use crate::database::Database;

/// Extracted identity from a request made with a server-signed JWT
pub struct SessionIdentity {
    /// The session ID of the session
    session_id: Uuid,
    /// The email of the user who owns the session
    email: String,
    /// The user id of the user who owns the session
    user_id: Uuid,

    created_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

impl SessionIdentity {
    pub fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionIdentity
where
    Database: FromRef<S>,
    ServiceVerificationKey: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = SessionIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookie_jar: CookieJar = CookieJar::from_headers(&parts.headers);

        let session_cookie = match cookie_jar.get(SESSION_COOKIE_NAME) {
            Some(st) => st,
            None => {
                let OriginalUri(uri) = OriginalUri::from_request_parts(parts, state)
                    .await
                    .expect("infallible conversion");
                return Err(SessionIdentityError::NoSession(uri.to_string()));
            }
        };

        let raw_cookie_val = session_cookie.value();
        if raw_cookie_val.len() != 150 {
            return Err(SessionIdentityError::EncodingError);
        }

        let (session_id_b64, authentication_tag_b64) = raw_cookie_val.split_at(22);

        let authentication_tag_bytes = B64
            .decode(authentication_tag_b64)
            .map_err(|_| SessionIdentityError::EncodingError)?;

        let ecdsa_signature = ecdsa::Signature::try_from(authentication_tag_bytes.as_slice())
            .map_err(SessionIdentityError::InvalidSignatureBytes)?;
        let mut digest = hmac_sha512::sha384::Hash::new();
        digest.update(session_id_b64);

        let verification_key = ServiceVerificationKey::from_ref(state);
        verification_key
            .public_key()
            .as_ref()
            .verify_digest(digest, &ecdsa_signature)
            .map_err(SessionIdentityError::BadSignature)?;

        // We now know these are good bytes, decode them, turn them into a valid session ID and
        // check the DB for them...

        let session_id_bytes = B64
            .decode(session_id_b64)
            .map_err(|_| SessionIdentityError::EncodingError)?;

        let session_id_bytes: [u8; 16] = session_id_bytes
            .try_into()
            .expect("signed session ID to be valid byte slice");
        let session_id = Uuid::from_bytes_le(session_id_bytes);

        let database = Database::from_ref(state);

        let db_sid = session_id.to_string();
        let db_session = sqlx::query_as!(
            DatabaseSession,
            r#"SELECT ss.id, ss.user_id, us.email, ss.created_at, ss.expires_at
                FROM sessions AS ss
                 JOIN users as us on us.id = ss.user_id
                WHERE ss.id =  $1;"#,
            db_sid,
        )
        .fetch_one(&database)
        .await
        .map_err(SessionIdentityError::LookupFailed)?;

        // todo: check session against client IP address and user agent

        if db_session.expires_at <= OffsetDateTime::now_utc() {
            return Err(SessionIdentityError::SessionExpired);
        }

        let session_id =
            Uuid::parse_str(&db_session.id).map_err(SessionIdentityError::CorruptDatabaseId)?;
        let user_id = Uuid::parse_str(&db_session.user_id)
            .map_err(SessionIdentityError::CorruptDatabaseId)?;

        Ok(SessionIdentity {
            session_id,
            user_id,
            email: db_session.email,

            created_at: db_session.created_at,
            expires_at: db_session.expires_at,
        })
    }
}

#[derive(sqlx::FromRow)]
struct DatabaseSession {
    id: String,
    user_id: String,
    email: String,

    created_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionIdentityError {
    #[error("signature did not match digest, tampering likely: {0}")]
    BadSignature(ecdsa::Error),

    #[error("a UUID in the database was corrupted and can not be parsed")]
    CorruptDatabaseId(uuid::Error),

    #[error("cookie was not encoded into the correct format")]
    EncodingError,

    #[error("authenicated signature was in a valid format: {0}")]
    InvalidSignatureBytes(ecdsa::Error),

    #[error("unable to lookup session in database: {0}")]
    LookupFailed(sqlx::Error),

    #[error("user didn't have an existing session")]
    NoSession(String),

    #[error("session was expired")]
    SessionExpired,

    #[error("not admin")]
    NotAdmin(String),
}

impl IntoResponse for SessionIdentityError {
    fn into_response(self) -> Response {
        use SessionIdentityError as SIE;

        // todo: Need to drop this intoresponse as it can't properly handle return urls or cleaning
        // out bad cookie states

        match self {
            SIE::NoSession(_orig_uri) => {
                tracing::debug!("request had no session when trying to access protected path");
            }
            SIE::NotAdmin(user) => {
                tracing::warn!("user not an admin {user}");
            }
            err => tracing::warn!("session validation error: {err}"),
        }

        Redirect::to(LOGIN_PATH).into_response()
    }
}

#[cfg(test)]
pub mod tests {
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::*;
    use crate::database::test_helpers;

    pub struct SessionIdentityBuilder {
        pub session_id: Uuid,
        pub email: String,
        pub user_id: Uuid,
        pub created_at: OffsetDateTime,
        pub expires_at: OffsetDateTime,
    }

    impl SessionIdentityBuilder {
        fn new() -> Self {
            Self {
                session_id: Uuid::new_v4(),
                email: String::from("test@example.com"),
                user_id: Uuid::new_v4(),
                created_at: OffsetDateTime::now_utc(),
                expires_at: OffsetDateTime::now_utc() + time::Duration::days(1),
            }
        }

        pub fn build(self) -> SessionIdentity {
            SessionIdentity {
                session_id: self.session_id,
                email: self.email,
                user_id: self.user_id,
                created_at: self.created_at,
                expires_at: self.expires_at,
            }
        }
    }

    #[tokio::test]
    async fn test_session_identity_builder() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "test@example.com").await;
        let session_identity = test_helpers::get_or_create_session(&mut conn, &user_id).await;

        assert_eq!(session_identity.user_id().to_string(), user_id);
        assert_eq!(session_identity.email(), "test@example.com");
    }
}

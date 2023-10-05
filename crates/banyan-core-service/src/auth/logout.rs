use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;

use crate::auth::{NEW_USER_COOKIE_NAME, SESSION_COOKIE_NAME};
use crate::database::Database;
use crate::extractors::SessionIdentity;

pub async fn handler(
    session: Option<SessionIdentity>,
    database: Database,
    mut cookie_jar: CookieJar,
) -> Response {
    if let Some(sid) = session {
        let session_id = sid.session_id();

        // todo: revoke token?

        let query = sqlx::query!("DELETE FROM sessions WHERE id = $1;", session_id);
        if let Err(err) = query.execute(&database).await {
            tracing::error!("failed to remove session from the db: {err}");
        }
    }

    cookie_jar = remove_cookie(NEW_USER_COOKIE_NAME, cookie_jar);
    cookie_jar = remove_cookie(SESSION_COOKIE_NAME, cookie_jar);

    (cookie_jar, Redirect::to("/")).into_response()
}

fn remove_cookie(name: &'static str, mut cookie_jar: CookieJar) -> CookieJar {
    cookie_jar = cookie_jar.remove(Cookie::named(name));
    cookie_jar.add(
        Cookie::build(name, "")
            .path("/")
            .http_only(false)
            .expires(time::OffsetDateTime::UNIX_EPOCH)
            .finish(),
    )
}

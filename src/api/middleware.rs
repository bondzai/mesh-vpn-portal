use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Redirect, Response, IntoResponse},
};
use axum_extra::extract::cookie::SignedCookieJar;

pub async fn auth(
    jar: SignedCookieJar,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if jar.get("auth_token").is_some() {
         return Ok(next.run(req).await);
    }

    Ok(Redirect::to("/login").into_response())
}

use axum::{
    response::{IntoResponse, Redirect},
    extract::Form,
};
use axum_extra::extract::cookie::{Cookie, SameSite, SignedCookieJar};
use askama::Template;
use serde::Deserialize;
use std::env;
use crate::api::htmx::HtmlTemplate;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

pub async fn login_page(jar: SignedCookieJar) -> impl IntoResponse {
    // If already logged in, redirect to admin
    if jar.get("auth_token").is_some() {
        return Redirect::to("/admin").into_response();
    }
    HtmlTemplate(LoginTemplate { error: None }).into_response()
}

pub async fn login_submit(
    jar: SignedCookieJar,
    Form(payload): Form<LoginPayload>,
) -> impl IntoResponse {
    let expected_username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let expected_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

    if payload.username == expected_username && payload.password == expected_password {
        let cookie = Cookie::build(("auth_token", "true"))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(false) // Set to true in prod with HTTPS
            .build();

        return (jar.add(cookie), Redirect::to("/admin")).into_response();
    }

    HtmlTemplate(LoginTemplate {
        error: Some("Invalid username or password".to_string()),
    })
    .into_response()
}

pub async fn logout(jar: SignedCookieJar) -> impl IntoResponse {
    let cookie = Cookie::build(("auth_token", ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();
    
    (jar.remove(cookie), Redirect::to("/login"))
}

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose, Engine as _};
use std::env;

pub async fn auth(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(auth_header) if auth_header.starts_with("Basic ") => {
            let credentials = auth_header.trim_start_matches("Basic ");
            match general_purpose::STANDARD.decode(credentials) {
                Ok(decoded) => {
                    let decoded_str = String::from_utf8(decoded).map_err(|_| StatusCode::BAD_REQUEST)?;
                    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let _username = parts[0]; // We don't check username currently
                        let password = parts[1];
                        let expected_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".to_string());

                        if password == expected_password {
                            return Ok(next.run(req).await);
                        }
                    }
                }
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            }
        }
        _ => {}
    }

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::UNAUTHORIZED;
    response.headers_mut().insert(
        "WWW-Authenticate",
        "Basic realm=\"Admin Area\"".parse().unwrap(),
    );
    Ok(response)
}

use google_authenticator::GoogleAuthenticator;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::{distributions::Alphanumeric, Rng};
use rocket::serde::json::{serde_json::json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::State;

/// Creates a URL for adding a new service to Google Authenticator.
/// Stolen from the `google_authenticator` package, since that one is private.
pub fn create_scheme(name: &str, secret: &str, title: &str) -> String {
    let name = utf8_percent_encode(name, NON_ALPHANUMERIC);
    let title = utf8_percent_encode(title, NON_ALPHANUMERIC);
    format!("otpauth://totp/{}?secret={}&issuer={}", name, secret, title)
}

#[derive(Deserialize)]
pub struct AuthCode {
    code: String,
}

#[derive(Serialize)]
pub struct AuthToken {
    token: String,
}

#[derive(Responder)]
pub enum AuthResult {
    #[response(status = 200)]
    Ok(Value),
    #[response(status = 401)]
    Unauthorized(()),
}

#[post("/login", data = "<code>")]
pub fn login(
    code: Json<AuthCode>,
    tokens: &State<crate::guards::ValidTokens>,
    config: &State<crate::Config>,
) -> AuthResult {
	if config.no_auth {
		return AuthResult::Ok(json!({ "token": "NO AUTH" }));
	}
    let auth = GoogleAuthenticator::new();
    let secret = &config.auth_secret;

    if auth.verify_code(secret, &code.code, 1, 0) {
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        tokens.lock().unwrap().insert(token.clone());
        return AuthResult::Ok(json!({ "token": token }));
    }

    AuthResult::Unauthorized(())
}

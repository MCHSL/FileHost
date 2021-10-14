use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::State;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub type ValidTokens = Arc<Mutex<HashSet<String>>>;

pub struct TokenChecker;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TokenChecker {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(tokens) = req.guard::<&State<ValidTokens>>().await.succeeded() {
            let tokens = tokens.lock().unwrap();
            if let Some(auth) = req.headers().get_one("Authorization") {
                let mut parts = auth.split_ascii_whitespace();
                if let Some(_kind) = parts.next() {
                    if let Some(token) = parts.next() {
                        if tokens.contains(token) {
                            return Outcome::Success(Self);
                        }
                    }
                }
            }
            return Outcome::Failure((Status::Unauthorized, "Missing or invalid token"));
        }

        Outcome::Failure((Status::InternalServerError, "Could not check token"))
    }
}

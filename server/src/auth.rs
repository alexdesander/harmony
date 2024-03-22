use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use common::token::ApiToken;
use tracing::debug;

pub struct TokenManager {
    secret: String,
    tokens: Mutex<HashMap<ApiToken, Instant>>,
}

impl TokenManager {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
            tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_token(&self) -> ApiToken {
        let api_token = ApiToken::new();
        self.tokens
            .lock()
            .unwrap()
            .insert(api_token.clone(), Instant::now());
        api_token
    }

    pub fn token_valid(&self, token: &ApiToken) -> bool {
        let mut tokens = self.tokens.lock().unwrap();
        if let Some(time) = tokens.get_mut(token) {
            if time.elapsed().as_secs() > 60 * 60 {
                tokens.remove(token);
                return false;
            }
            *time = Instant::now();
            return true;
        }
        false
    }
}

pub async fn use_secret(token_manager: Arc<TokenManager>, body: String) -> Response {
    debug!("use_secret fired with body: {body}");
    if body != token_manager.secret {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let token = token_manager.new_token();
    debug!("Created token: {:?}", token);
    (
        [(header::CONTENT_TYPE, "application/octet-stream")],
        bitcode::encode(&token),
    )
        .into_response()
}

pub async fn auth_middleware(
    token_manager: Arc<TokenManager>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match request.headers().get("API_TOKEN") {
        Some(api_token) => {
            if let Ok(api_token) = api_token.to_str() {
                if !token_manager.token_valid(&ApiToken::from_string(api_token)) {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let response = next.run(request).await;
    Ok(response)
}

use std::env;
use std::sync::{Arc, LazyLock};

use axum::Json;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::error::Error;
use common::user::{LoginParams, User};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::AppState;
use crate::db_ops::{DatabaseOps, get_user_by_username};

#[derive(Debug)]
pub struct AuthUser(User);

impl std::ops::Deref for AuthUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub const COOKIE_NAME: &str = "__Host-typst-translation-login";

static JWT_SECRET: LazyLock<String> =
    LazyLock::new(|| env::var("JWT_SIGNING_KEY").expect("JWT_SIGNING_KEY must be set"));

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub login_epoch: i64,
    pub exp: usize, // Expiration time (as UTC timestamp)
}

#[instrument(skip_all)]
pub fn generate_jwt(user_id: i64, login_epoch: i64) -> String {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        user_id,
        login_epoch,
        exp: expiration as usize,
    };
    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .expect("Failed to encode JWT")
}

#[instrument(skip_all)]
pub fn add_cookie(cookies: CookieJar, user_id: i64, login_epoch: i64) -> CookieJar {
    let token = generate_jwt(user_id, login_epoch);
    cookies.add(
        Cookie::build((COOKIE_NAME, token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(true)
            .build(),
    )
}

#[instrument(skip_all)]
pub fn remove_cookie(cookies: CookieJar) -> CookieJar {
    cookies.add(
        Cookie::build((COOKIE_NAME, ""))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(true)
            .max_age(time::Duration::seconds(0))
            .build(),
    )
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = (CookieJar, Error);

    #[instrument(skip_all)] // Removed err(level = "error")
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let cookies = CookieJar::from_request_parts(parts, state).await.unwrap();
        let Some(cookie) = cookies.get(COOKIE_NAME) else {
            return Err((cookies, Error::LoginRequired));
        };

        if cookie.value().is_empty() {
            return Err((cookies, Error::LoginRequired));
        }

        let jwt = cookie.value();
        let decoding_key = DecodingKey::from_secret(JWT_SECRET.as_bytes());
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

        let token_data = match decode::<Claims>(jwt, &decoding_key, &validation) {
            Ok(data) => data,
            Err(e) => {
                error!("JWT decoding error: {:?}", e);
                return Err((remove_cookie(cookies), Error::LoginInvalidated));
            }
        };

        let pool = state.db(); // Use state.db() here
        let user = User::get_by_id(pool, token_data.claims.user_id)
            .await
            .map_err(|e| {
                error!("Failed to fetch user from DB: {:?}", e);
                (remove_cookie(cookies.clone()), Error::InternalServerError)
            })?;

        let Some(user) = user else {
            return Err((remove_cookie(cookies), Error::LoginInvalidated));
        };

        if user.login_epoch != token_data.claims.login_epoch {
            return Err((remove_cookie(cookies), Error::LoginInvalidated));
        }

        Ok(AuthUser(user))
    }
}

#[instrument(skip_all)]
pub async fn login(
    State(state): State<AppState>,
    mut cookies: CookieJar,
    Json(login_data): Json<LoginParams>,
) -> Result<CookieJar, Error> {
    let pool = state.db();

    let user = get_user_by_username(pool, &login_data.username).await?;

    let user = match user {
        Some(user) => user,
        None => return Err(Error::Forbidden),
    };

    // Compare passwords directly (as per user instruction)
    if user.password != login_data.password {
        return Err(Error::Forbidden);
    }

    // Do not update login_epoch, it is meant for password changes.

    cookies = add_cookie(cookies, user.id, user.login_epoch);

    Ok(cookies)
}

pub async fn whoami(current_user: Option<AuthUser>) -> Result<Json<User>, Error> {
    tracing::info!(user = ?current_user, "whoami");
    let Some(current_user) = current_user else {
        return Err(Error::Forbidden);
    };
    Ok(Json(current_user.0))
}

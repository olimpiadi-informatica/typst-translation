use axum::Json;
use axum::extract::{FromRequestParts, OptionalFromRequestParts, State};
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::error::Error;
use common::user::{ExtUser, LoginParams, User, WhoAmIResponse};
use derive_more::{Deref, DerefMut};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::AppState;
use crate::db_ops::user_db;

pub const COOKIE_NAME: &str = "__Host-typst-translation-login";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Option<i64>,
    pub login_epoch: i64,
    pub admin: bool,
    pub staff: bool,
    /// Expiration time (as UTC timestamp)
    pub exp: i64,
}

impl Claims {
    fn from_auth_any(a: &Option<AuthAny>) -> Self {
        Claims {
            user_id: a.as_ref().and_then(|x| x.user.as_ref().map(|x| x.id)),
            login_epoch: a
                .as_ref()
                .and_then(|x| x.user.as_ref().map(|x| x.login_epoch))
                .unwrap_or_default(),
            admin: a.as_ref().is_some_and(|x| x.is_admin),
            staff: a.as_ref().is_some_and(|x| x.is_staff),
            exp: 0,
        }
    }
}

#[instrument(skip_all)]
pub fn generate_jwt(mut claims: Claims, jwt_signing_key: &str) -> String {
    claims.exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_signing_key.as_bytes()),
    )
    .expect("Failed to encode JWT")
}

#[instrument(skip_all)]
pub fn add_cookie(cookies: CookieJar, claims: Claims, state: &AppState) -> CookieJar {
    let token = generate_jwt(claims, &state.config.jwt_signing_key);
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

#[instrument(skip_all)]
pub async fn login(
    State(state): State<AppState>,
    mut cookies: CookieJar,
    current_user: Option<AuthAny>,
    Json(login_data): Json<LoginParams>,
) -> Result<CookieJar, Error> {
    let pool = state.db();

    let user = user_db::get_user_by_username(pool, &login_data.username).await?;

    let user = match user {
        Some(user) => user,
        None => return Err(Error::Forbidden),
    };

    // Compare passwords directly (as per user instruction)
    if user.password != login_data.password {
        return Err(Error::Forbidden);
    }

    // Do not update login_epoch, it is meant for password changes.

    cookies = add_cookie(
        cookies,
        Claims {
            user_id: Some(user.id),
            login_epoch: user.login_epoch,
            ..Claims::from_auth_any(&current_user)
        },
        &state,
    );

    Ok(cookies)
}

#[instrument(skip_all)]
pub async fn admin_login(
    State(state): State<AppState>,
    cookies: CookieJar,
    current_user: Option<AuthAny>,
    Json(password): Json<String>,
) -> Result<CookieJar, Error> {
    if password != state.config.admin_password {
        return Err(Error::Forbidden);
    }
    Ok(add_cookie(
        cookies,
        Claims {
            admin: true,
            ..Claims::from_auth_any(&current_user)
        },
        &state,
    ))
}

#[instrument(skip_all)]
pub async fn staff_login(
    State(state): State<AppState>,
    cookies: CookieJar,
    current_user: Option<AuthAny>,
    Json(password): Json<String>,
) -> Result<CookieJar, Error> {
    if password != state.config.staff_password {
        return Err(Error::Forbidden);
    }

    Ok(add_cookie(
        cookies,
        Claims {
            staff: true,
            ..Claims::from_auth_any(&current_user)
        },
        &state,
    ))
}

#[instrument(skip_all)]
pub async fn whoami(current_user: Option<AuthAny>) -> Json<WhoAmIResponse> {
    tracing::info!(user = ?current_user, "whoami");
    Json(current_user.map(|u| u.0))
}

#[instrument(skip_all)]
pub async fn logout(cookies: CookieJar) -> Result<CookieJar, Error> {
    Ok(remove_cookie(cookies))
}

#[derive(Debug, Deref, DerefMut)]
pub struct AuthUser(User);

#[derive(Debug)]
pub struct AuthAdmin;

#[derive(Debug)]
pub struct AuthStaff;

#[derive(Debug, Deref, DerefMut)]
pub struct AuthAny(ExtUser);

impl FromRequestParts<AppState> for AuthStaff {
    type Rejection = (CookieJar, Error);

    #[instrument(skip_all)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = <AuthAny as FromRequestParts<_>>::from_request_parts(parts, state).await?;
        if user.is_admin || user.is_staff {
            Ok(AuthStaff)
        } else {
            let cookies = CookieJar::from_request_parts(parts, state).await.unwrap();
            Err((cookies, Error::Forbidden))
        }
    }
}

impl FromRequestParts<AppState> for AuthAdmin {
    type Rejection = (CookieJar, Error);

    #[instrument(skip_all)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = <AuthAny as FromRequestParts<_>>::from_request_parts(parts, state).await?;
        if user.is_admin {
            Ok(AuthAdmin)
        } else {
            let cookies = CookieJar::from_request_parts(parts, state).await.unwrap();
            Err((cookies, Error::Forbidden))
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (CookieJar, Error);

    #[instrument(skip_all)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user =
            <AuthAny as OptionalFromRequestParts<AppState>>::from_request_parts(parts, state)
                .await?;
        match user {
            Some(AuthAny(ExtUser { user: Some(u), .. })) => Ok(AuthUser(u)),
            _ => {
                let cookies = CookieJar::from_request_parts(parts, state).await.unwrap();
                Err((cookies, Error::Forbidden))
            }
        }
    }
}

impl FromRequestParts<AppState> for AuthAny {
    type Rejection = (CookieJar, Error);

    #[instrument(skip_all)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user =
            <Self as OptionalFromRequestParts<AppState>>::from_request_parts(parts, state).await?;
        match user {
            Some(u) => Ok(u),
            None => {
                let cookies = CookieJar::from_request_parts(parts, state).await.unwrap();
                Err((remove_cookie(cookies), Error::LoginInvalidated))
            }
        }
    }
}

impl OptionalFromRequestParts<AppState> for AuthAny {
    type Rejection = (CookieJar, Error);

    #[instrument(skip_all)]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Option<Self>, Self::Rejection> {
        let cookies = CookieJar::from_request_parts(parts, state).await.unwrap();
        let Some(cookie) = cookies.get(COOKIE_NAME) else {
            return Ok(None);
        };

        if cookie.value().is_empty() {
            return Ok(None);
        }

        let jwt = cookie.value();
        let decoding_key = DecodingKey::from_secret(state.config.jwt_signing_key.as_bytes());
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

        let token_data = match decode::<Claims>(jwt, &decoding_key, &validation) {
            Ok(data) => data,
            Err(e) => {
                error!("JWT decoding error: {:?}", e);
                return Err((remove_cookie(cookies), Error::LoginInvalidated));
            }
        };

        let mut user = ExtUser {
            user: None,
            is_admin: token_data.claims.admin,
            is_staff: token_data.claims.staff,
        };

        if let Some(user_id) = token_data.claims.user_id {
            let pool = state.db();
            user.user = user_db::get_by_id(pool, user_id).await.map_err(|e| {
                error!("Failed to fetch user from DB: {e}");
                (
                    remove_cookie(cookies.clone()),
                    Error::InternalServerError(format!("Failed to fetch user from DB: {e}")),
                )
            })?;

            if user
                .user
                .as_ref()
                .is_none_or(|x| x.login_epoch != token_data.claims.login_epoch)
            {
                return Err((remove_cookie(cookies), Error::LoginInvalidated));
            };
        };

        if user.user.is_none() && !user.is_admin && !user.is_staff {
            Ok(None)
        } else {
            Ok(Some(AuthAny(user)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_generation_and_decoding() {
        let claims = Claims {
            user_id: Some(1),
            login_epoch: 12345,
            admin: true,
            staff: false,
            exp: 0,
        };
        let secret = "secret_test_key_12345";
        let token = generate_jwt(claims, secret);
        assert!(!token.is_empty());

        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        let token_data = decode::<Claims>(&token, &decoding_key, &validation).unwrap();
        assert_eq!(token_data.claims.user_id, Some(1));
        assert_eq!(token_data.claims.login_epoch, 12345);
        assert!(token_data.claims.admin);
        assert!(!token_data.claims.staff);
    }
}

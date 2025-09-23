use axum::Json;
use axum::extract::{FromRequestParts, OptionalFromRequestParts, State};
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use common::error::Error;
use common::user::{LoginParams, User, WhoAmIResponse};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::AppState;
use crate::db_ops::{DatabaseOps, get_user_by_username};

#[derive(Debug)]
pub enum AuthUser {
    RegularUser(User),
    AdminUser,
    StaffUser,
}

impl AuthUser {
    pub fn as_user(&self) -> Option<&User> {
        match self {
            AuthUser::RegularUser(user) => Some(user),
            AuthUser::AdminUser => None,
            AuthUser::StaffUser => None,
        }
    }
}

pub const COOKIE_NAME: &str = "__Host-typst-translation-login";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub subj: AuthSubject,
    /// Expiration time (as UTC timestamp)
    pub exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthSubject {
    User { user_id: i64, login_epoch: i64 },
    Admin,
    Staff,
}

#[instrument(skip_all)]
pub fn generate_jwt(subject: AuthSubject, jwt_signing_key: &str) -> String {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        subj: subject,
        exp: expiration,
    };
    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_signing_key.as_bytes()),
    )
    .expect("Failed to encode JWT")
}

#[instrument(skip_all)]
pub fn add_cookie(cookies: CookieJar, subject: AuthSubject, state: &AppState) -> CookieJar {
    let token = generate_jwt(subject, &state.config.jwt_signing_key);
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

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (CookieJar, Error);

    #[instrument(skip_all)] // Removed err(level = "error")
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = CookieJar::from_request_parts(parts, state).await.unwrap();
        let Some(cookie) = cookies.get(COOKIE_NAME) else {
            return Err((cookies, Error::LoginRequired));
        };

        if cookie.value().is_empty() {
            return Err((cookies, Error::LoginRequired));
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

        match token_data.claims.subj {
            AuthSubject::Admin => Ok(AuthUser::AdminUser),
            AuthSubject::Staff => Ok(AuthUser::StaffUser),
            AuthSubject::User {
                user_id,
                login_epoch,
            } => {
                let pool = state.db(); // Use state.db() here
                let user = User::get_by_id(pool, user_id).await.map_err(|e| {
                    error!("Failed to fetch user from DB: {e}");
                    (
                        remove_cookie(cookies.clone()),
                        Error::InternalServerError(format!("Failed to fetch user from DB: {e}")),
                    )
                })?;

                let Some(user) = user else {
                    return Err((remove_cookie(cookies), Error::LoginInvalidated));
                };

                if user.login_epoch != login_epoch {
                    return Err((remove_cookie(cookies), Error::LoginInvalidated));
                }

                Ok(AuthUser::RegularUser(user))
            }
        }
    }
}

impl OptionalFromRequestParts<AppState> for AuthUser {
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

        match token_data.claims.subj {
            AuthSubject::Admin => Ok(Some(AuthUser::AdminUser)),
            AuthSubject::Staff => Ok(Some(AuthUser::StaffUser)),
            AuthSubject::User {
                user_id,
                login_epoch,
            } => {
                let pool = state.db(); // Use state.db() here
                let user = User::get_by_id(pool, user_id).await.map_err(|e| {
                    error!("Failed to fetch user from DB: {e}");
                    (
                        remove_cookie(cookies.clone()),
                        Error::InternalServerError(format!("Failed to fetch user from DB: {e}")),
                    )
                })?;

                let Some(user) = user else {
                    return Err((remove_cookie(cookies), Error::LoginInvalidated));
                };

                if user.login_epoch != login_epoch {
                    return Err((remove_cookie(cookies), Error::LoginInvalidated));
                }

                Ok(Some(AuthUser::RegularUser(user)))
            }
        }
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

    cookies = add_cookie(
        cookies,
        AuthSubject::User {
            user_id: user.id,
            login_epoch: user.login_epoch,
        },
        &state,
    );

    Ok(cookies)
}

#[instrument(skip_all)]
pub async fn admin_login(
    State(state): State<AppState>,
    cookies: CookieJar,
    Json(password): Json<String>,
) -> Result<CookieJar, Error> {
    if password != state.config.admin_password {
        return Err(Error::Forbidden);
    }
    Ok(add_cookie(cookies, AuthSubject::Admin, &state))
}

#[instrument(skip_all)]
pub async fn staff_login(
    State(state): State<AppState>,
    cookies: CookieJar,
    Json(password): Json<String>,
) -> Result<CookieJar, Error> {
    if password != state.config.staff_password {
        return Err(Error::Forbidden);
    }

    Ok(add_cookie(cookies, AuthSubject::Staff, &state))
}

#[axum::debug_handler(state = AppState)]
pub async fn whoami(current_user: Option<AuthUser>) -> Result<Json<WhoAmIResponse>, Error> {
    tracing::info!(user = ?current_user, "whoami");
    let Some(current_user) = current_user else {
        return Ok(Json(WhoAmIResponse::Nobody));
    };
    match current_user {
        AuthUser::RegularUser(user) => Ok(Json(WhoAmIResponse::RegularUser(user))),
        AuthUser::AdminUser => Ok(Json(WhoAmIResponse::AdminUser)),
        AuthUser::StaffUser => Ok(Json(WhoAmIResponse::StaffUser)),
    }
}

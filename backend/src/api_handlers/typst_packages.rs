use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use common::typst_packages::TypstPackagePayload;
use common::error::Error;
use reqwest::header::CONTENT_ENCODING;

use crate::{auth::AuthUser, AppState};


pub async fn get_typst_package(
    State(app_state): State<AppState>,
    _current_user: AuthUser,
    Json(payload): Json<TypstPackagePayload>
) -> Result<impl IntoResponse, Error> {
    if !app_state.typst_packages.contains_key(&payload) {
        let url = format!("https://packages.typst.org/{}/{}-{}.tar.gz", payload.namespace, payload.name, payload.version);
        let package = app_state.reqwest
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        app_state.typst_packages.insert(payload.clone(), package.to_vec());
    }

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_ENCODING, "gz".parse().unwrap());
    let gz_data = app_state.typst_packages.get(&payload).unwrap().to_vec();
    Ok((headers, gz_data))
}

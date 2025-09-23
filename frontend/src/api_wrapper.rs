use std::any::{Any, TypeId};

use common::error::Error;
use gloo_net::http::Request;
use serde::Serialize;
use serde::de::DeserializeOwned;

async fn request<T: DeserializeOwned + Any>(request: Request) -> Result<T, Error> {
    let response = request.send().await?;
    if response.status() >= 200 && response.status() < 300 {
        // workaround resp.json() not successfully decoding an empty response as ().
        let resp = if TypeId::of::<T>() == TypeId::of::<()>() {
            serde_json::from_str("null").unwrap()
        } else {
            response.json::<T>().await?
        };
        Ok(resp)
    } else {
        Err(response.json().await?)
    }
}

fn get_url(relative_url: &str) -> String {
    format!(
        "{}{relative_url}",
        leptos::prelude::window()
            .location()
            .origin()
            .expect("invalid origin")
            .as_str()
    )
}

pub async fn api_get<T: DeserializeOwned + Any>(url: &str) -> Result<T, Error> {
    request(Request::get(&get_url(url)).build()?).await
}

pub async fn api_post<Req: Serialize, Resp: DeserializeOwned + Any>(
    url: &str,
    req: &Req,
) -> Result<Resp, Error> {
    let json_body = serde_json::to_string(req)?;
    request(
        Request::post(&get_url(url))
            .header("Content-Type", "application/json")
            .body(json_body)?,
    )
    .await
}

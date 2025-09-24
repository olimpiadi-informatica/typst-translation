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

pub async fn api_get<T: DeserializeOwned + Any>(url: &str) -> Result<T, Error> {
    request(Request::get(url).build()?).await
}

pub async fn api_post<Req: Serialize, Resp: DeserializeOwned + Any>(
    url: &str,
    req: &Req,
) -> Result<Resp, Error> {
    request(Request::post(url).json(req)?).await
}

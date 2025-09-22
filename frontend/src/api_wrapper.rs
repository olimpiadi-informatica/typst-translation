use std::any::{Any, TypeId};

// Assuming common::error is available in your new project
// use common::error::Error;
use gloo_net::http::Request;
use serde::Serialize;
use serde::de::DeserializeOwned;
// `thaw::LoadingBarInjection` is specific to the 'thaw' UI library and would need replacement
// use thaw::LoadingBarInjection;

// `show_error!` macro is specific to the current project's error handling and would need replacement
// use crate::show_error;

async fn request<T: DeserializeOwned + Any>(request: Request) -> Result<T, String> {
    // Changed Error to String for simplicity
    // let lb = LoadingBarInjection::expect_context(); // Specific to 'thaw'
    let resp = async move {
        // lb.start(); // Specific to 'thaw'
        let response = request.send().await.map_err(|e| e.to_string())?; // Convert gloo_net::Error to String
        if response.status() >= 200 && response.status() < 300 {
            let resp = if TypeId::of::<T>() == TypeId::of::<()>() {
                serde_json::from_str("null").unwrap()
            } else {
                response.json::<T>().await.map_err(|e| e.to_string())? // Convert gloo_net::Error to String
            };
            Ok::<_, String>(Ok(resp))
        } else {
            let resp = response.json().await.map_err(|e| e.to_string())?; // Convert gloo_net::Error to String
            // lb.error(); // Specific to 'thaw'
            Ok(Err(resp))
        }
    };
    match resp.await {
        Ok(r) => r,
        Err(e) => {
            // lb.error(); // Specific to 'thaw'
            // show_error!("Network error: {e}"); // Specific to project's error handling
            Err(format!("Network error: {e}")) // Simplified error handling
        }
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

pub async fn api_get<T: DeserializeOwned + Any>(url: &str) -> Result<T, String> {
    // Changed Error to String
    request(
        Request::get(&get_url(url))
            .build()
            .map_err(|e| e.to_string())?,
    )
    .await // Convert gloo_net::Error to String
}

pub async fn api_post<Req: Serialize, Resp: DeserializeOwned + Any>(
    url: &str,
    req: &Req,
) -> Result<Resp, String> {
    // Changed Error to String
    let json_body = serde_json::to_string(req).map_err(|e| e.to_string())?; // Convert serde_json::Error to String
    request(
        Request::post(&get_url(url))
            .header("Content-Type", "application/json")
            .body(json_body)
            .map_err(|e| e.to_string())?, // Convert gloo_net::Error to String
    )
    .await
}

pub async fn api_delete(url: &str, id: i64) -> Result<(), String> {
    // Changed Error to String
    request(
        Request::delete(&get_url(&format!("{}/{}", url, id)))
            .build()
            .map_err(|e| e.to_string())?,
    )
    .await // Convert gloo_net::Error to String
}

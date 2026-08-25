use std::time::Instant;

use axum::extract;
use axum::middleware::Next;
use axum::response::Response;
use tracing::level_filters::LevelFilter;
use tracing::{info, warn};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub async fn trace_requests(request: extract::Request, next: Next) -> Response {
    let start = Instant::now();
    let uri = request.uri().path().to_string();
    let method = request.method().clone();

    let response = next.run(request).await;

    let status = response.status();
    let latency = Instant::now().duration_since(start);
    let latency_ms = latency.as_secs_f32() * 1000.0;

    if response.status().is_success() || response.status().is_redirection() {
        info!(uri, ?method, ?latency, ?status, latency_ms, "HTTP");
    } else {
        warn!(uri, ?method, ?latency, ?status, latency_ms, "HTTP");
    }

    response
}

pub fn init_logging() {
    let registry = tracing_subscriber::registry()
        .with(tracing_error::ErrorLayer::default())
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            ),
        );

    registry.init();
}

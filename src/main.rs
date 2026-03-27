mod result;

use core::net::SocketAddr;
use std::sync::Arc;

use aide::axum::{ApiRouter, IntoApiResponse, routing::get};
use aide::openapi::OpenApi;
use aide::transform::TransformOpenApi;
use anyhow::{Context, Result};
use axum::{Extension, Json, Router, ServiceExt, extract::Request, response::IntoResponse};
use axum_extra::response::ErasedJson;
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::catch_panic::CatchPanicLayer;

use crate::result::ApiResult;

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Test Integration").description("Good stuff.")
}

async fn metadata() -> ApiResult<Json<Value>> {
    Ok(Json(json!({
        "protocol_version": 1,
        "test": "value"
    })))
}

async fn serve_api_spec(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    ErasedJson::new(api.as_ref()).into_response()
}

pub async fn create_router() -> Router {
    let mut api = OpenApi::default();
    // Force aide to use OpenAPI 3 style
    aide::generate::in_context(|ctx| {
        ctx.schema = schemars::generate::SchemaSettings::openapi3().into()
    });
    let routes = ApiRouter::new()
        .route("/v1/api.json", get(serve_api_spec))
        .api_route("/v1", get(metadata))
        .finish_api_with(&mut api, api_docs)
        .layer(Extension(Arc::new(api)));
    Router::new().merge(routes).layer(CatchPanicLayer::new())
}

#[tokio::main]
async fn main() -> Result<()> {
    let router = create_router().await;
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .context("failed to bind to HTTP API address")?;
    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(router),
    )
    .await
    .context("HTTP server error")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::{Value, json};

    #[tokio::test]
    async fn test_metadata() {
        let server = TestServer::new(create_router().await);
        let resp = server.get("/v1").await;
        assert_eq!(resp.status_code(), 200);
        let body: Value = resp.json();
        assert_eq!(
            body,
            json!({
                "protocol_version": 1,
                "test": "value"
            })
        );
    }

    #[tokio::test]
    async fn test_can_parse_api_spec() {
        let server = TestServer::new(create_router().await);
        let resp = server.get("/v1/api.json").await;
        assert_eq!(resp.status_code(), 200);
        let body: Value = resp.json();
        let _openapi: OpenApi =
            serde_json::from_str(&body.to_string()).expect("failed to parse OpenAPI spec");
    }
}

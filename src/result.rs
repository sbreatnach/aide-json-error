use aide::{OperationOutput, generate::GenContext, openapi::Operation};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type ApiResult<T> = Result<T, InternalServerError>;

pub struct InternalServerError(anyhow::Error);

impl From<anyhow::Error> for InternalServerError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for InternalServerError {
    fn into_response(self) -> Response {
        tracing::error!("Internal server error {}", self.0);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl OperationOutput for InternalServerError {
    type Inner = Self;

    fn operation_response(
        _ctx: &mut GenContext,
        _operation: &mut Operation,
    ) -> Option<aide::openapi::Response> {
        Some(aide::openapi::Response {
            description: "Internal server error".to_string(),
            ..Default::default()
        })
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        vec![(
            Some(StatusCode::INTERNAL_SERVER_ERROR.into()),
            Self::operation_response(ctx, operation).expect("operation_response is always Some"),
        )]
    }
}

//! API error model: every failure maps to a JSON body
//! `{"error": {"code": ..., "message": ...}}` with an appropriate HTTP status.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::safe::CoolPropError;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorEnvelope {
    pub error: ErrorDetail,
}

#[derive(Debug)]
pub enum ApiError {
    /// CoolProp rejected the input (invalid fluid, parameter, state, ...).
    CoolProp(String),
    /// No AbstractState with the given handle exists on this server.
    UnknownHandle(i64),
    /// Request could not be carried out for reasons not reported by CoolProp.
    BadInput(String),
    /// Unexpected internal failure.
    Internal(String),
}

impl ApiError {
    fn parts(&self) -> (StatusCode, i32, String) {
        match self {
            ApiError::CoolProp(msg) => (StatusCode::BAD_REQUEST, 400, msg.clone()),
            ApiError::UnknownHandle(h) => (
                StatusCode::NOT_FOUND,
                404,
                format!("no AbstractState with handle {h} on this server"),
            ),
            ApiError::BadInput(msg) => (StatusCode::BAD_REQUEST, 400, msg.clone()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 500, msg.clone()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();
        let body = ErrorEnvelope {
            error: ErrorDetail { code, message },
        };
        (status, Json(body)).into_response()
    }
}

impl From<CoolPropError> for ApiError {
    fn from(e: CoolPropError) -> Self {
        ApiError::CoolProp(e.0)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

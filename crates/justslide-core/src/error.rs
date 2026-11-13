use serde::Serialize;

/// Error codes are the closed set declared in the protocol registry
/// (`definitions.Error.code`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidArgument,
    UnknownCommand,
    Unauthenticated,
    PermissionDenied,
    NotFound,
    PreconditionFailed,
    Conflict,
    UserInteractionRequired,
    RateLimited,
    PayloadTooLarge,
    Cancelled,
    Timeout,
    Unavailable,
    Unsupported,
    Internal,
}

#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[error("{code:?}: {message}")]
pub struct CoreError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_request_id: Option<String>,
}

impl CoreError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let mut m: String = message.into();
        m.truncate(1024);
        CoreError { code, message: m, retryable: matches!(code, ErrorCode::RateLimited | ErrorCode::Timeout | ErrorCode::Unavailable), pending_request_id: None }
    }
    pub fn invalid(m: impl Into<String>) -> Self { Self::new(ErrorCode::InvalidArgument, m) }
    pub fn not_found(m: impl Into<String>) -> Self { Self::new(ErrorCode::NotFound, m) }
    pub fn denied(m: impl Into<String>) -> Self { Self::new(ErrorCode::PermissionDenied, m) }
    pub fn unsupported(m: impl Into<String>) -> Self { Self::new(ErrorCode::Unsupported, m) }
    pub fn internal(m: impl Into<String>) -> Self { Self::new(ErrorCode::Internal, m) }
    pub fn needs_user(m: impl Into<String>, pending: Option<String>) -> Self {
        let mut e = Self::new(ErrorCode::UserInteractionRequired, m);
        e.pending_request_id = pending;
        e
    }
}

impl From<rusqlite::Error> for CoreError {
    fn from(e: rusqlite::Error) -> Self {
        tracing::error!("storage error: {e}");
        CoreError::internal("storage error")
    }
}

pub type CoreResult<T> = Result<T, CoreError>;

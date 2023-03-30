use axum::{response::IntoResponse, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug, PartialEq, Error)]
#[serde(tag = "error", content = "details")]
pub enum AuthError {
    /// This occurs if a request is made without any authentication details.
    #[error("unauthenticated")]
    Unauthenticated,

    /// This occurs if the credentials provided are not allowed to do the specified action.
    #[error("unauthorized")]
    Unauthorized { action: Option<String> },
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            AuthError::Unauthenticated => StatusCode::UNAUTHORIZED,
            AuthError::Unauthorized { .. } => StatusCode::FORBIDDEN,
        };

        (status_code, Json(self)).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct User {
    pub username: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct NewUser {
    pub username: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Error)]
#[serde(tag = "error", content = "details")]
pub enum GetUserError {
    #[error("user was not found: {username}")]
    UserNotFound { username: String },

    #[error(transparent)]
    AuthError(AuthError),
}

impl From<AuthError> for GetUserError {
    fn from(auth_error: AuthError) -> Self {
        Self::AuthError(auth_error)
    }
}

impl IntoResponse for GetUserError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            GetUserError::UserNotFound { .. } => StatusCode::NOT_FOUND,
            GetUserError::AuthError(auth_err) => return auth_err.into_response(),
        };

        (status_code, Json(self)).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Error)]
#[serde(tag = "error", content = "details")]
pub enum CreateUserError {
    #[error("username already exists")]
    UsernameAlreadyExists,

    #[error("invalid username: {0:?}")]
    InvalidUsername(InvalidUsernameReason),

    #[error("invalid name: {0:?}")]
    InvalidName(InvalidNameReason),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InvalidNewUserReason {
    pub field: String,
    pub reason: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum InvalidUsernameReason {
    TooShort,
    TooLong,
    InvalidCharacters,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum InvalidNameReason {
    TooShort,
    TooLong,
    InvalidCharacters,
}

impl IntoResponse for CreateUserError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

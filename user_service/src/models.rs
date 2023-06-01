use axum::response::IntoResponse;
use axum::Json;
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
    Unauthorized,
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
pub enum HandlerError<E> {
    /// This occurs if a request is made without any authentication details.
    #[error("unauthenticated")]
    Unauthenticated,

    /// This occurs if the credentials provided are not allowed to do the specified action.
    #[error("unauthorized")]
    Unauthorized,

    #[error(transparent)]
    ServiceError(E),
}

impl<E> HandlerError<E> {
    pub fn service_error(err: E) -> Self {
        HandlerError::ServiceError(err)
    }
}

impl<E> From<AuthError> for HandlerError<E> {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::Unauthenticated => HandlerError::Unauthenticated,
            AuthError::Unauthorized => HandlerError::Unauthorized,
        }
    }
}

impl<E> IntoResponse for HandlerError<E>
where
    E: IntoResponse + Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            HandlerError::Unauthenticated => StatusCode::UNAUTHORIZED,
            HandlerError::Unauthorized => StatusCode::FORBIDDEN,
            HandlerError::ServiceError(service_err) => return service_err.into_response(),
        };

        (status_code, Json(self)).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Error)]
#[serde(tag = "error", content = "details")]
pub enum GetUserError {
    #[error("user was not found: {username}")]
    UserNotFound { username: String },
}

impl IntoResponse for GetUserError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            GetUserError::UserNotFound { .. } => StatusCode::NOT_FOUND,
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

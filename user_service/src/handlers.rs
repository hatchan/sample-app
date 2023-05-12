use crate::models::{
    self, CreateUserError, GetUserError, HandlerError, InvalidNameReason, InvalidUsernameReason,
    User,
};
use axum::extract::Path;
use axum::Json;
use tracing::{debug, instrument};

#[instrument(err)]
pub async fn get_user(
    Path(username): Path<String>,
) -> Result<Json<User>, HandlerError<GetUserError>> {
    check_auth(&username)?;

    if username == "not_found" {
        return Err(HandlerError::service_error(GetUserError::UserNotFound {
            username,
        }));
    }

    Ok(Json(models::User {
        username,
        name: "John Doe".to_string(),
    }))
}

#[instrument(err)]
pub async fn create_user(
    Json(new_user): Json<models::NewUser>,
) -> Result<Json<models::User>, HandlerError<CreateUserError>> {
    debug!("creating user: {:?}", new_user);
    if new_user.username.is_empty() {
        return Err(HandlerError::service_error(
            CreateUserError::InvalidUsername(InvalidUsernameReason::TooShort),
        ));
    } else if new_user.username.len() > 20 {
        return Err(HandlerError::service_error(
            CreateUserError::InvalidUsername(InvalidUsernameReason::TooLong),
        ));
    } else if new_user.username == "invalid" {
        return Err(HandlerError::service_error(
            CreateUserError::InvalidUsername(InvalidUsernameReason::InvalidCharacters),
        ));
    } else if new_user.username == "taken" {
        return Err(HandlerError::service_error(
            CreateUserError::UsernameAlreadyExists,
        ));
    }

    if new_user.name.is_empty() {
        return Err(HandlerError::service_error(CreateUserError::InvalidName(
            InvalidNameReason::TooShort,
        )));
    } else if new_user.name.len() > 20 {
        return Err(HandlerError::service_error(CreateUserError::InvalidName(
            InvalidNameReason::TooLong,
        )));
    } else if new_user.name == "invalid" {
        return Err(HandlerError::service_error(CreateUserError::InvalidName(
            InvalidNameReason::InvalidCharacters,
        )));
    }

    Ok(Json(models::User {
        username: new_user.username,
        name: new_user.name,
    }))
}

/// Just a fake auth check, this can force a specific error by supplying a
/// specific username.
pub fn check_auth(username: &str) -> Result<(), models::AuthError> {
    match username {
        "unauthenticated" => Err(models::AuthError::Unauthenticated),
        "unauthorized" => Err(models::AuthError::Unauthorized),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::response::IntoResponse;
    use http::StatusCode;

    #[test]
    fn get_user_error_into_response() {
        let tests = vec![(
            models::GetUserError::UserNotFound {
                username: "not_found".to_string(),
            },
            StatusCode::NOT_FOUND,
        )];

        for (err, expected_status_code) in tests {
            let response = err.into_response();
            let actual_status_code = response.status();

            assert_eq!(expected_status_code, actual_status_code);
        }
    }

    #[tokio::test]
    async fn get_user_not_found() {
        let path = "not_found".to_string();
        let err = get_user(Path(path)).await.expect_err("expected an error");

        match err {
            models::GetUserError::UserNotFound { username } if username == "not_found" => {}
            _ => panic!("expected UserNotFound error"),
        }
    }
}

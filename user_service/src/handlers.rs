use crate::models;
use axum::extract::Path;
use axum::Json;

pub async fn get_user(
    Path(username): Path<String>,
) -> Result<Json<models::User>, models::GetUserError> {
    check_auth(&username)?;

    if username == "not_found" {
        return Err(models::GetUserError::UserNotFound { username });
    }

    Ok(Json(models::User {
        username,
        name: "John Doe".to_string(),
    }))
}

pub async fn create_user(
    Json(new_user): Json<models::NewUser>,
) -> Result<Json<models::User>, models::CreateUserError> {
    if new_user.username.is_empty() {
        return Err(models::CreateUserError::InvalidUsername(
            models::InvalidUsernameReason::TooShort,
        ));
    } else if new_user.username.len() > 20 {
        return Err(models::CreateUserError::InvalidUsername(
            models::InvalidUsernameReason::TooLong,
        ));
    } else if new_user.username == "invalid" {
        return Err(models::CreateUserError::InvalidUsername(
            models::InvalidUsernameReason::InvalidCharacters,
        ));
    } else if new_user.username == "taken" {
        return Err(models::CreateUserError::UsernameAlreadyExists);
    }

    if new_user.name.is_empty() {
        return Err(models::CreateUserError::InvalidName(
            models::InvalidNameReason::TooShort,
        ));
    } else if new_user.name.len() > 20 {
        return Err(models::CreateUserError::InvalidName(
            models::InvalidNameReason::TooLong,
        ));
    } else if new_user.name == "invalid" {
        return Err(models::CreateUserError::InvalidName(
            models::InvalidNameReason::InvalidCharacters,
        ));
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
        "unauthorized" => Err(models::AuthError::Unauthorized {
            action: Some("get_user".to_string()),
        }),
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

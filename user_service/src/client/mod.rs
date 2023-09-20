use crate::models::{CreateUserError, GetUserError, NewUser, User};
use http::{header::CONTENT_TYPE, Method, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

pub struct Client {
    base_url: url::Url,
    client: reqwest::Client,
}

impl Client {
    pub fn new(base_url: url::Url) -> Self {
        let client = reqwest::Client::new();
        Self { base_url, client }
    }

    async fn do_req<T, E>(
        &self,
        method: Method,
        path: impl AsRef<str>,
        query: Option<String>, // Probably needs a different type
        payload: Option<Vec<u8>>,
    ) -> Result<T, ClientError<E>>
    where
        T: DeserializeOwned,
        E: DeserializeOwned,
    {
        // Make request -> DNS lookup, TCP connection, TLS invalid, timeout
        // Get response -> unauthorized, unauthenticated, invalid json result
        let url = self.base_url.join(path.as_ref()).unwrap();

        let mut request = self.client.request(method, url);

        if let Some(query) = query {
            request = request.query(&query);
        }

        if let Some(payload) = payload {
            request = request
                .header(CONTENT_TYPE, "application/json")
                .body(payload);
        }

        let response = request.send().await.map_err(map_to_client_err)?;
        let status_code = response.status();
        if !status_code.is_success() {
            if status_code == StatusCode::UNAUTHORIZED {
                return Err(ClientError::Unauthenticated);
            }

            if status_code == StatusCode::FORBIDDEN {
                return Err(ClientError::Unauthorized);
            }

            // looks like an execeptional status was returned. The body _could_
            // contain a service error.
            if let Ok(err) = response.json::<E>().await {
                return Err(ClientError::ServiceError(err));
            }

            return Err(ClientError::UnknownError);
        }

        let response = response.json().await.map_err(map_to_client_err)?;

        Ok(response)
    }

    pub async fn get_user(
        &self,
        username: impl AsRef<str>,
    ) -> Result<User, ClientError<GetUserError>> {
        self.do_req(
            Method::GET,
            format!("users/{username}", username = username.as_ref()),
            None,
            None,
        )
        .await
    }

    pub async fn create_user(
        &self,
        new_user: NewUser,
    ) -> Result<User, ClientError<CreateUserError>> {
        let payload = serde_json::to_vec(&new_user).unwrap();
        self.do_req(Method::POST, "users", None, Some(payload))
            .await
    }
}

fn map_to_client_err<E>(err: reqwest::Error) -> ClientError<E> {
    if err.is_connect() {
        ClientError::ConnectionError
    } else if err.is_timeout() {
        ClientError::TimeoutError
    } else if err.is_decode() {
        ClientError::DeserializationError
    } else {
        ClientError::UnknownError
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Error)]
#[serde(tag = "error", content = "details")]
pub enum ClientError<E> {
    ConnectionError,
    TimeoutError,
    UnknownError,
    DeserializationError,
    ServiceError(E),
}

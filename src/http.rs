use crate::{Error as OpaError, Input, OpenPolicyAgentClient, Output};
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error as JsonError;
use url::{ParseError, Url};

pub const PATH_PREFIX: &str = "/v1/data/";

#[derive(Debug)]
pub enum Error {
    JsonError(JsonError),
    UrlError(ParseError),
    HttpError,
}

impl From<JsonError> for Error {
    fn from(inner: JsonError) -> Self {
        Self::JsonError(inner)
    }
}

impl From<ParseError> for Error {
    fn from(inner: ParseError) -> Self {
        Self::UrlError(inner)
    }
}

impl From<::reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Self::HttpError
    }
}

/// Client to communicate and interact with an OpenPolicyAgent (OPA) server
/// over HTTP(S).
pub struct OpenPolicyAgentHttpClient {
    client: Client,
    url: Url,
}

impl OpenPolicyAgentHttpClient {
    /// Construct a new client given an endpoint.
    pub fn new(url: Url) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }
}

#[async_trait(?Send)]
impl OpenPolicyAgentClient for OpenPolicyAgentHttpClient {
    //impl OpenPolicyAgentHttpClient {
    async fn query<I: Serialize, O: DeserializeOwned>(
        &self,
        policy: &str,
        input: &I,
    ) -> Result<Option<O>, OpaError> {
        let policy = policy.strip_prefix('/').unwrap_or(policy);
        let path = self
            .url
            .join(PATH_PREFIX)
            .map_err(|_| OpaError::PolicyError)?
            .join(policy)
            .map_err(|_| OpaError::JsonError)?;

        let input = Input { input };

        let req = self.client.post(path).json(&input);
        let response = req.send().await.map_err(|_| OpaError::PolicyError)?;
        let output: Output<O> = response.json().await.map_err(|_| OpaError::PolicyError)?;

        Ok(output.result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct MyInput {
        user: String,
        groups: Vec<String>,
    }

    #[test]
    fn input_serialization() -> Result<(), Error> {
        let input = MyInput {
            user: "bob".to_string(),
            groups: vec!["tall".to_string(), "virginia".to_string()],
        };

        let input = Input { input };

        let json = serde_json::to_string(&input)?;

        assert_eq!(
            json,
            "{\"input\":{\"user\":\"bob\",\"groups\":[\"tall\",\"virginia\"]}}"
        );

        Ok(())
    }
}

use crate::{Input, OpaClientError, OpenPolicyAgentClient, Output};
use async_trait::async_trait;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use url::{ParseError, Url};

pub const PATH_PREFIX: &str = "/v1/data/";

impl From<ParseError> for OpaClientError {
    fn from(_inner: ParseError) -> Self {
        Self::ParseError
    }
}

impl From<::reqwest::Error> for OpaClientError {
    fn from(_: reqwest::Error) -> Self {
        Self::PolicyError
    }
}

/// Client to communicate and interact with an OpenPolicyAgent (OPA) server
/// over HTTP(S).
#[derive(Clone)]
pub struct OpenPolicyAgentHttpClient {
    client: Client,
    url: Url,
}

#[async_trait(?Send)]
impl OpenPolicyAgentClient for OpenPolicyAgentHttpClient {
    /// Construct a new client given an endpoint.
    fn new(bytes: &[u8]) -> Result<Self, OpaClientError> {
        let url = Url::parse(std::str::from_utf8(bytes)?)?;

        Ok(Self {
            client: Client::new(),
            url,
        })
    }

    //impl OpenPolicyAgentHttpClient {
    async fn query<I: Serialize, D: Serialize, O: DeserializeOwned>(
        &mut self,
        policy: &str,
        input: &I,
        _data: &D,
    ) -> Result<Option<O>, OpaClientError> {
        let policy = policy.strip_prefix('/').unwrap_or(policy);
        let path = self
            .url
            .join(PATH_PREFIX)
            .map_err(|_| OpaClientError::PolicyError)?
            .join(policy)
            .map_err(|_| OpaClientError::JsonError)?;

        let input = Input { input };

        let req = self.client.post(path).json(&input);
        let response = req.send().await.map_err(|_| OpaClientError::PolicyError)?;
        let output: Output<O> = response
            .json()
            .await
            .map_err(|_| OpaClientError::PolicyError)?;

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
    fn input_serialization() -> Result<(), OpaClientError> {
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

use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};
use serde_json::Error as JsonError;

pub const PATH_PREFIX: &str = "/v1/data";

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

#[derive(Serialize)]
struct Input<I: Serialize> {
    input: I,
}

#[derive(Deserialize)]
struct Output<O> {
    result: Option<O>,
}

pub struct OpenPolicyAgentClient {
    client: Client,
    url: Url,
}

impl OpenPolicyAgentClient {
    pub fn new(url: Url) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }

    pub async fn query<I: Serialize, O: DeserializeOwned>(
        &self,
        policy: &str,
        input: &I,
    ) -> Result<Option<O>, Error> {
        let policy = policy.replace(".", "/");
        let path = self
            .url
            .join(PATH_PREFIX )?
            .join(&policy)?;

        let input = Input { input };

        let req = self.client.get(path).json(&input);
        let response = req.send().await?;
        let output: Output<O> = response.json().await?;

        Ok(output.result)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn input_serialization() -> Result<(), Error>{
        #[derive(Serialize)]
        struct MyInput {
            user: String,
            groups: Vec<String>,
        }

        let input = MyInput {
            user: "bob".to_string(),
            groups: vec![
                "tall".to_string(),
                "virginia".to_string()
            ]
        };

        let input = Input {
            input,
        };

        let json = serde_json::to_string(&input)?;

        println!("{}", json);

        assert_eq!( json, "{\"input\":{\"user\":\"bob\",\"groups\":[\"tall\",\"virginia\"]}}");

        Ok(())
    }

}

use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
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
        //let policy = policy.replace(".", "/");
        let policy = if policy.starts_with("/") {
            policy[1..].to_string()
        } else {
            policy.into()
        };

        let path = self.url.join(PATH_PREFIX)?.join(&policy)?;

        let input = Input { input };

        let req = self.client.post(path).json(&input);
        let response = req.send().await?;
        let output: Output<O> = response.json().await?;

        Ok(output.result)
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use bollard::{API_DEFAULT_VERSION, Docker};
    use bollard_stubs::models::BollardDate;
    use super::*;
    use serde::{Deserialize, Serialize};

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

    use testcontainers::clients::Http;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;
    use testcontainers::{Image, RunnableImage};
    use tokio;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query() {
        env_logger::init();
        if let Ok(opa_server_url) = Url::parse("http://localhost:8181/") {
            let socket = env::var("DOCKER_SOCKET").unwrap_or( "unix:///var/run/docker.sock".into() );
            let docker = Docker::connect_with_socket(&socket, 120, API_DEFAULT_VERSION).unwrap();
            let container_client = Http::new(docker);
            let opa = container_client.run(opa_server()).await;
            let client = OpenPolicyAgentClient::new(opa_server_url);

            let input = MyInput {
                user: "bob".to_string(),
                groups: vec!["tall".to_string(), "virginia".to_string()],
            };

            let result: Result<Option<bool>, Error> = client.query("/basic/allow", &input).await;
            assert_eq!(true, result.unwrap().unwrap());

            let input = MyInput {
                user: "melissa".to_string(),
                groups: vec!["short".to_string(), "virginia".to_string()],
            };

            let result: Result<Option<bool>, Error> = client.query("/basic/allow", &input).await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }
    }

    fn opa_server() -> RunnableImage<GenericImage> {
        let image = GenericImage::new("openpolicyagent/opa", "latest")
            .with_volume(
                "/Users/bob/repos/seedwing-io/opa-client/example/",
                "/example",
            )
            .with_exposed_port(8181)
            .with_wait_for(WaitFor::message_on_stderr("Server initialized"));

        let args = vec![
            "run".into(),
            "--server".into(),
            "--log-level=debug".into(),
            "/example".into(),
        ];

        RunnableImage::from((image, args)).with_mapped_port((8181, 8181))
    }
}

use crate::images::simple_opa_server;

use bollard::{Docker, API_DEFAULT_VERSION};
use opa_client::http::OpenPolicyAgentHttpClient;
use opa_client::{OpaClientError, OpenPolicyAgentClient};
use serde::Serialize;
use std::env;
use testcontainers::clients::Http;
use tokio;
use url::Url;

mod images;

#[derive(Serialize)]
struct MyInput {
    user: String,
    groups: Vec<String>,
}

#[derive(Serialize)]
struct MyData {}

#[tokio::test(flavor = "multi_thread")]
async fn test_query() {
    env_logger::try_init().ok();

    if let Ok(opa_server_url) = Url::parse("http://localhost:8181/") {
        let socket = env::var("DOCKER_SOCKET").unwrap_or("unix:///var/run/docker.sock".into());
        let docker = Docker::connect_with_socket(&socket, 120, API_DEFAULT_VERSION).unwrap();
        let container_client = Http::new(docker);
        let _opa = container_client.run(simple_opa_server()).await;
        let mut client =
            OpenPolicyAgentHttpClient::new(opa_server_url.as_str().as_bytes()).unwrap();

        let input = MyInput {
            user: "bob".to_string(),
            groups: vec!["tall".to_string(), "virginia".to_string()],
        };

        let data = MyData {};

        let result: Result<Option<bool>, OpaClientError> =
            client.query("/basic/allow", &input, &data).await;
        assert_eq!(true, result.unwrap().unwrap());

        let input = MyInput {
            user: "melissa".to_string(),
            groups: vec!["short".to_string(), "virginia".to_string()],
        };

        let result: Result<Option<bool>, OpaClientError> =
            client.query("/basic/allow", &input, &data).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}

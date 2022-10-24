use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod http;

#[derive(Serialize)]
pub struct Input<I: Serialize> {
    input: I,
}

#[derive(Deserialize)]
pub struct Output<O> {
    result: Option<O>,
}

#[derive(Debug)]
pub enum Error {
    PolicyError,
    JsonError,
}

#[async_trait(?Send)]
pub trait OpenPolicyAgentClient {
    /// Query a policy given `input` data and a policy path.
    async fn query<I: Serialize, O: DeserializeOwned>(
        &self,
        policy: &str,
        input: &I,
    ) -> Result<Option<O>, Error>;
}

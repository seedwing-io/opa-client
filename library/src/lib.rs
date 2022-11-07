use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use thiserror::Error;

pub mod http;
pub mod local_wasm;

#[derive(Serialize, Debug)]
pub struct Input<I: Serialize> {
    input: I,
}

#[derive(Serialize, Debug)]
pub struct Data<I: Serialize> {
    data: I,
}

#[derive(Deserialize)]
pub struct Output<O> {
    result: Option<O>,
}

#[derive(Error, Debug)]
pub enum OpaClientError {
    #[error("OPA Policy Error")]
    PolicyError,
    #[error("OPA Parse Error")]
    ParseError,
    #[error("OPA JSON Error")]
    JsonError,
}

impl From<JsonError> for OpaClientError {
    fn from(_inner: JsonError) -> Self {
        Self::JsonError
    }
}

#[async_trait(?Send)]
pub trait OpenPolicyAgentClient<'a> {
    /// Query a policy given `input` data and a policy path.
    async fn query<I: Serialize, D: Serialize, O: DeserializeOwned>(
        &mut self,
        rule: &'a str,
        input: &I,
        data: &D,
    ) -> Result<Option<O>, OpaClientError>;
}

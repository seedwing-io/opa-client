use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use std::str::Utf8Error;
use thiserror::Error;

pub mod http;
pub mod local_wasm;

#[derive(Serialize, Debug)]
pub struct Input<I: Serialize> {
    input: I,
}

#[derive(Serialize, Debug)]
pub struct Data<I: Serialize> {
    pub data: I,
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
    #[error("Utf8 Error")]
    Utf8Error,
}

impl From<JsonError> for OpaClientError {
    fn from(_inner: JsonError) -> Self {
        Self::JsonError
    }
}

impl From<Utf8Error> for OpaClientError {
    fn from(_inner: Utf8Error) -> Self {
        Self::Utf8Error
    }
}

#[async_trait(?Send)]
pub trait OpenPolicyAgentClient {
    /// Instantiate a new instance of a struct implementing this trait.
    fn new(bytes: &[u8]) -> Result<Self, OpaClientError>
    where
        Self: Sized;

    /// Query a policy given `input` data and a policy path.
    async fn query<I: Serialize, D: Serialize, O: DeserializeOwned>(
        &mut self,
        rule: &str,
        input: &I,
        data: &D,
    ) -> Result<Option<O>, OpaClientError>;
}

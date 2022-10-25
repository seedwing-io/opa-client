use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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

#[derive(Debug)]
pub enum Error {
    PolicyError,
    JsonError,
}

#[async_trait(?Send)]
pub trait OpenPolicyAgentClient {
    /// Query a policy given `input` data and a policy path.
    async fn query<I: Serialize, D: Serialize, O: DeserializeOwned>(
        &mut self,
        input: &I,
        data: &D,
    ) -> Result<Option<O>, Error>;
}

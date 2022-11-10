use crate::{OpaClientError, OpenPolicyAgentClient};
use async_trait::async_trait;
use policy_evaluator::burrego::Evaluator;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

pub struct OpenPolicyAgentWasmClient {
    evaluator: Evaluator,
}

impl OpenPolicyAgentWasmClient {
    pub fn entrypoints(&mut self) -> Result<HashMap<String, i32>, OpaClientError> {
        self.evaluator
            .entrypoints()
            .map_err(|_| OpaClientError::PolicyError)
    }
}

#[async_trait(?Send)]
impl OpenPolicyAgentClient for OpenPolicyAgentWasmClient {
    fn new(wasm: &[u8]) -> Result<Self, OpaClientError> {
        Ok(Self {
            evaluator: Evaluator::new(wasm, Default::default()).unwrap(),
        })
    }

    async fn query<I, D, O>(
        &mut self,
        policy: &str,
        input: &I,
        data: &D,
    ) -> Result<Option<O>, OpaClientError>
    where
        I: Serialize,
        D: Serialize,
        O: DeserializeOwned,
    {
        let entrypoint_id = self
            .evaluator
            .entrypoint_id(policy)
            .map_err(|_| OpaClientError::PolicyError)?;

        let r: serde_json::Value = self
            .evaluator
            .evaluate(
                entrypoint_id,
                &serde_json::value::to_value(input)?,
                &serde_json::value::to_value(data)?,
            )
            .map_err(|_| OpaClientError::PolicyError)?;
        let val = &r[0];
        Ok(Some(<O as Deserialize>::deserialize(val).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use std::env;
    use std::fs;
    use std::path::Path;

    #[tokio::test(flavor = "multi_thread")]
    async fn local_wasm_query_test() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let example_path = Path::new(&manifest_dir).join("example");
        let wasm_path = Path::new(&example_path).join("license.wasm");
        let wasm: [u8; 131650] = to_array(fs::read(wasm_path).unwrap());
        let mut client = OpenPolicyAgentWasmClient::new(&wasm).unwrap();

        let input_str =
            fs::read_to_string(Path::new(&example_path).join("licenses-input.txt")).unwrap();
        let input: serde_json::Value = serde_json::from_str(&input_str).unwrap();
        println!("Input: {}", input);
        let data: serde_json::Value =
            serde_json::from_str("{}").expect("data json does not have correct format.");

        let result: Result<Option<serde_json::Value>, OpaClientError> =
            client.query("license/allow", &input, &data).await;
        assert_eq!(r#"{"result":true}"#, result.unwrap().unwrap().to_string());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn entrypoints() {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let example_path = Path::new(&manifest_dir).join("example");
        let wasm_path = Path::new(&example_path).join("license.wasm");
        let wasm: [u8; 131650] = to_array(fs::read(wasm_path).unwrap());
        let mut client = OpenPolicyAgentWasmClient::new(&wasm).unwrap();
        assert_eq!(
            client.entrypoints().unwrap().contains_key("license/allow"),
            true
        );
    }

    fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
        v.try_into().unwrap_or_else(|v: Vec<T>| {
            panic!("Incorrect vector length: {}, expected: {}", N, v.len())
        })
    }
}

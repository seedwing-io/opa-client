use clap::Parser;
use opa_client::local_wasm::OpenPolicyAgentWasmClient;
use opa_client::{OpaClientError, OpenPolicyAgentClient};
use std::fs;
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
#[command(author,
    version,
    long_about = None)]
/// opa-client-cli is a tool to execute OPA wasm policy modules.
struct Args {
    #[arg(short, long, help = "The OPA policy wasm module")]
    wasm: String,

    #[arg(short, long, help = "The entry_point/rule to be executed")]
    entry_point: String,

    #[arg(short, long, help = "The input file in json format (optional)")]
    input: Option<String>,

    #[arg(short, long, help = "The data file in json format (optional)")]
    data: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let wasm = fs::read(args.wasm).unwrap();
    let mut client = OpenPolicyAgentWasmClient::new(&wasm, &args.entry_point);

    let input_str = args
        .input
        .map_or_else(|| read_from_stdin(), |f| read_from_file(f));
    let input: serde_json::Value =
        serde_json::from_str(&input_str).expect("data json does not have correct format.");

    let data_str = read_from_file_arg(args.data);
    let data: serde_json::Value =
        serde_json::from_str(&data_str).expect("data json does not have correct format.");

    let result: Result<Option<serde_json::Value>, OpaClientError> =
        client.query(&input, &data).await;
    if result.is_ok() {
        let ret = result.unwrap().unwrap();
        println!("{}", ret.to_string());
    } else {
        println!("{{}}");
    }
}

fn read_from_file_arg(s: Option<String>) -> String {
    s.map_or(String::from("{}"), |p| read_from_file(p))
}

fn read_from_file(s: String) -> String {
    fs::read_to_string(s).unwrap()
}

fn read_from_stdin() -> String {
    let mut lines = io::stdin().lock().lines();
    let mut input = String::new();
    while let Some(line) = lines.next() {
        let last_input = line.unwrap();
        if last_input.len() == 0 {
            break;
        }
        if input.len() > 0 {
            input.push_str("\n");
        }
        input.push_str(&last_input);
    }
    input
}

use clap::Parser;
use opa_client::wasm::OpenPolicyAgentWasmClient;
use opa_client::{OpaClientError, OpenPolicyAgentClient};
use std::fs;
use std::io::{self, BufRead};
use std::process::exit;

#[derive(Parser, Debug)]
#[command(author,
    version,
    long_about = None)]
/// opa-client-cli is a tool to execute OPA wasm policy modules.
struct Args {
    #[arg(short, long, help = "The OPA policy wasm module")]
    wasm: String,

    #[arg(
        short,
        long,
        help = "The entry_point/rule to be executed",
        required_unless_present("print_entrypoints")
    )]
    entry_point: Option<String>,

    #[arg(short, long, help = "The input file in json format (optional)")]
    input: Option<String>,

    #[arg(short, long, help = "The data file in json format (optional)")]
    data: Option<String>,

    #[arg(short, long, help = "Print the entrypoints (rules) in the wasm module")]
    print_entrypoints: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let wasm = fs::read(&args.wasm).unwrap();
    let mut client = OpenPolicyAgentWasmClient::new(&wasm).unwrap();

    if args.print_entrypoints {
        let entrypoints = client.entrypoints();
        if let Ok(e) = entrypoints {
            println!("entrypoints:");
            for (key, value) in &e {
                println!("name: {}, index: {}", key, value);
            }
        } else {
            println!("No entrypoints were found for {}", &args.wasm);
        }
        exit(1);
    }

    let input_str = args.input.map_or_else(read_from_stdin, read_from_file);
    let input: serde_json::Value =
        serde_json::from_str(&input_str).expect("data json does not have correct format.");

    let data_str = read_from_file_arg(args.data);
    let data: serde_json::Value =
        serde_json::from_str(&data_str).expect("data json does not have correct format.");

    let result: Result<Option<serde_json::Value>, OpaClientError> = client
        .query(&args.entry_point.unwrap(), &input, &data)
        .await;
    if let Ok(r) = result {
        println!("{}", r.unwrap());
    } else {
        println!("{{}}");
    }
}

fn read_from_file_arg(s: Option<String>) -> String {
    s.map_or(String::from("{}"), read_from_file)
}

fn read_from_file(s: String) -> String {
    fs::read_to_string(s).unwrap()
}

fn read_from_stdin() -> String {
    let lines = io::stdin().lock().lines();
    let mut input = String::new();
    for line in lines {
        let last_input = line.unwrap();
        if last_input.is_empty() {
            break;
        }
        if !input.is_empty() {
            input.push('\n');
        }
        input.push_str(&last_input);
    }
    input
}

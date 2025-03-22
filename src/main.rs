mod anthropic_client;

use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::env;
use anthropic_client::AnthropicClient;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("aish")
        .version("0.1.0")
        .author("Daniel Echlin")
        .about("Call LLM from command line and get shell commands")
        .subcommand(
            SubCommand::with_name("ask")
                .about("Ask Claude a question")
                .arg(
                    Arg::with_name("query")
                        .help("The question to ask Claude")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("ask", ask_matches)) => {
            let query = ask_matches.value_of("query").unwrap();
            let response = call_claude(query)?;
            println!("{}", response);
            Ok(())
        }
        _ => {
            println!("Try using the 'ask' subcommand followed by your question");
            Ok(())
        }
    }
}

fn call_claude(query: &str) -> Result<String, Box<dyn Error>> {
    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    // Create the client
    let client = AnthropicClient::new(api_key);

    // Make the API call
    let response = client.send_message(
        "claude-3-7-sonnet-20250219",
        1000,
        query
    )?;

    Ok(response)
}
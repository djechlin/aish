use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::env;

// Define structures for JSON serialization/deserialization
#[derive(serde::Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(serde::Serialize)]
struct Message {
    role: String,
    content: Vec<ContentItem>,
}

#[derive(serde::Serialize)]
struct ContentItem {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(serde::Deserialize)]
struct AnthropicResponse {
    content: Vec<ResponseContent>,
}

#[derive(serde::Deserialize)]
struct ResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

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
    println!("Asking Claude: {}", query);

    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    // Prepare the request
    let client = reqwest::blocking::Client::new();

    let request = AnthropicRequest {
        model: "claude-3-7-sonnet-20250219".to_string(),
        max_tokens: 1000,
        messages: vec![
            Message {
                role: "user".to_string(),
                content: vec![
                    ContentItem {
                        content_type: "text".to_string(),
                        text: query.to_string(),
                    },
                ],
            },
        ],
    };

    // Make the API call
    println!("Waiting for Claude's response...");

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()?;

    // Process the response
    let response_json: AnthropicResponse = response.json()?;

    // Extract and return the text content
    let response_text = response_json.content
        .iter()
        .filter(|content| content.content_type == "text")
        .map(|content| content.text.clone())
        .collect::<Vec<String>>()
        .join("\n");

    Ok(response_text)
}
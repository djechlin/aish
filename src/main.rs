use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::env;

// Define structures for JSON serialization/deserialization
#[derive(serde::Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    system: String,
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
        .about("Get shell commands using LLM")
        .subcommand(
            SubCommand::with_name("cmd")
                .about("Get a shell command for your task")
                .arg(
                    Arg::with_name("task")
                        .help("Describe what you're trying to do")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("cmd", cmd_matches)) => {
            let task = cmd_matches.value_of("task").unwrap();
            let response = get_shell_command(task)?;
            println!("{}", response);
            Ok(())
        }
        _ => {
            println!("Try using the 'cmd' subcommand followed by your task");
            println!("Example: aish cmd \"find all PDF files modified in the last week\"");
            Ok(())
        }
    }
}

fn get_shell_command(task: &str) -> Result<String, Box<dyn Error>> {
    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    // System prompt to ensure we get just the shell command
    let system_prompt = "You are a helpful shell command assistant. The user will describe a task they want to accomplish using the command line. Your task is to provide ONLY the shell command that would accomplish this task, with no explanation or additional text. Provide the most efficient and accurate command for their use case. Do not include any markdown formatting, just the raw command itself.";

    // Prepare the request
    let client = reqwest::blocking::Client::new();

    let request = AnthropicRequest {
        model: "claude-3-7-sonnet-20250219".to_string(),
        max_tokens: 1000,
        system: system_prompt.to_string(),
        messages: vec![
            Message {
                role: "user".to_string(),
                content: vec![
                    ContentItem {
                        content_type: "text".to_string(),
                        text: format!("I want a shell command to: {}", task),
                    },
                ],
            },
        ],
    };

    // Make the API call
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

    // Clean the response - trim whitespace, remove any markdown code blocks if present
    let clean_response = response_text
        .trim()
        .replace("```bash\n", "")
        .replace("```sh\n", "")
        .replace("```shell\n", "")
        .replace("```\n", "")
        .replace("\n```", "")
        .replace("```", "");

    Ok(clean_response)
}
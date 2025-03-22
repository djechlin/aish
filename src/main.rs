mod anthropic_client;

use clap::{App, Arg, SubCommand};
use std::error::Error;
use std::env;
use std::process;
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
            match call_claude(query) {
                Ok(response) => {
                    print!("{}", response); // Use print! instead of println! - don't add extra newline
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
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

    // Define the system prompt for the assistant's identity
    let system_prompt = "You are aish, an AI shell command assistant that returns exactly ONE shell command per query. NEVER add explanations, backticks, or formatting.";

    // Create the prompt with instructions for structured responses
    let user_prompt = format!(
        "Return ONLY the shell command that would accomplish this task with absolutely no commentary or explanation. Do not wrap the command in backticks or any other formatting.

If you cannot provide a command (due to complexity, ambiguity, or risk), respond with ONLY a single line error message prefixed with 'ERROR: '.

USER REQUEST: {}", query);

    // Make the API call
    let response = client.send_message(
        "claude-3-7-sonnet-20250219",
        1000,
        &user_prompt,
        system_prompt
    )?;

    // Process the response to extract just the relevant parts
    let response = process_claude_response(&response);

    Ok(response)
}

fn process_claude_response(response: &str) -> String {
    // Strip any trailing backticks that might be left in the response
    let clean_response = response.trim().replace("```", "");

    // Extract just the command part - handle various formats

    // Case 1: Command is between [COMMAND_SUCCESS] and end of response
    if let Some(start_marker) = clean_response.find("[COMMAND_SUCCESS]") {
        let after_marker = &clean_response[start_marker + 17..]; // Skip past the marker
        return after_marker.trim().to_string();
    }

    // Case 2: Error message is between [COMMAND_ERROR] and end of response
    if let Some(start_marker) = clean_response.find("[COMMAND_ERROR]") {
        let after_marker = &clean_response[start_marker + 15..]; // Skip past the marker

        // If there's a newline, only take up to that
        if let Some(end_pos) = after_marker.find('\n') {
            return after_marker[..end_pos].trim().to_string();
        }

        return after_marker.trim().to_string();
    }

    // Case 3: No structured markers, just return the cleanest version
    // First, try to find the shell command directly
    if let Some(cmd_line) = clean_response.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('[') && !line.contains("```"))
        .next() {
        return cmd_line.to_string();
    }

    // Last resort: return the first non-empty line
    if let Some(first_line) = clean_response.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .next() {
        return first_line.to_string();
    }

    clean_response.trim().to_string()
}
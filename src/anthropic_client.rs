use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

// Define structures for JSON serialization/deserialization
#[derive(Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
}

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: Vec<ContentItem>,
}

#[derive(Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct AnthropicResponse {
    pub content: Vec<ResponseContent>,
}

#[derive(Deserialize)]
pub struct ResponseContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

pub struct AnthropicClient {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        AnthropicClient {
            api_key,
            client: reqwest::blocking::Client::new(),
        }
    }

    pub fn send_message(&self, model: &str, max_tokens: u32, message_text: &str) -> Result<String, Box<dyn Error>> {
        let request = AnthropicRequest {
            model: model.to_string(),
            max_tokens,
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: vec![
                        ContentItem {
                            content_type: "text".to_string(),
                            text: message_text.to_string(),
                        },
                    ],
                },
            ],
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()?;

        let response_json: AnthropicResponse = response.json()?;

        let response_text = response_json.content
            .iter()
            .filter(|content| content.content_type == "text")
            .map(|content| content.text.clone())
            .collect::<Vec<String>>()
            .join("\n");

        Ok(response_text)
    }
}
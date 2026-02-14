use crate::core::Cli;
use crate::core::log::send_log;

use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct Service {
    pub http: Client,
    pub apikey: Option<String>,
    pub endpoint: String,
    pub model: String,
}

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
}

#[derive(Deserialize)]
pub struct ResponseMessage {
    pub content: String,
}

impl Service {
    pub fn new(args: &Cli) -> Self {
        let url = std::env::var("NETERO_URL")
            .ok()
            .filter(|v| !v.trim().is_empty());

        let model = std::env::var("NETERO_MODEL")
            .ok()
            .filter(|v| !v.trim().is_empty());

        let key = std::env::var("NETERO_API_KEY")
            .ok()
            .filter(|v| !v.trim().is_empty());

        if args.verbose {
            println!("modelo: {:#?}\nurl: {:#?}\n", model, url);
        }

        let (endpoint, model, apikey) = match (url, model) {
            (Some(u), Some(m)) => (u, m, key),
            (None, None) => (
                "https://codestral.mistral.ai/v1/chat/completions".to_string(),
                "codestral-latest".to_string(),
                std::env::var("CODE_API_KEY")
                    .ok()
                    .filter(|v| !v.trim().is_empty()),
            ),
            _ => panic!("NETERO_URL and NETERO_MODEL must be set together"),
        };

        Self {
            http: Client::new(),
            apikey,
            endpoint,
            model,
        }
    }

    pub async fn complete(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        send_log(":: REQUEST ::", content).await;

        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: content.to_string(),
            }],
        };

        let mut req = self.http.post(&self.endpoint).json(&body);

        if let Some(key) = &self.apikey {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req.send().await?.json::<ChatResponse>().await?;

        let content = response
            .choices
            .get(0)
            .ok_or("No choices returned")?
            .message
            .content
            .clone();

        send_log(":: RESPONSE ::", &content).await;

        Ok(content)
    }
}

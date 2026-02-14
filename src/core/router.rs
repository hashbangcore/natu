use crate::core::{Cli, Config};
use crate::core::trace::send_trace;

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
        let config = Config::from_env(args);

        if config.verbose {
            println!("modelo: {:#?}\nurl: {:#?}\n", config.model, config.endpoint);
        }

        Self {
            http: Client::new(),
            apikey: config.apikey,
            endpoint: config.endpoint,
            model: config.model,
        }
    }

    pub async fn complete(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        send_trace(":: REQUEST ::", content).await;

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

        send_trace(":: RESPONSE ::", &content).await;

        Ok(content)
    }
}

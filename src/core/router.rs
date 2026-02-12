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
    pub fn new(provider: Option<&str>) -> Self {
        let provider = provider.unwrap();

        let (envar, endpoint, model, dynamic) = match provider {
            "codestral" => (
                Some("CODE_API_KEY"),
                "https://codestral.mistral.ai/v1/chat/completions",
                "codestral-latest",
                false,
            ),

            _ => (Some("NETERO_API_KEY"), "NETERO_URL", "NETERO_MODEL", true),
        };

        // Shadowing:
        let endpoint = if dynamic {
            std::env::var(endpoint).expect("Endpoint env var not found")
        } else {
            endpoint.to_string()
        };

        let model = if dynamic {
            std::env::var(model).expect("Model env var not found")
        } else {
            model.to_string()
        };
        // end shadowing

        let apikey =
            envar.map(|var| std::env::var(var).expect("API key environment variable not found"));

        Self {
            http: Client::new(),
            apikey,
            endpoint,
            model,
        }
    }

    pub async fn complete(&self, content: &str) -> Result<String, Box<dyn std::error::Error>> {
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

        Ok(content)

    }
}

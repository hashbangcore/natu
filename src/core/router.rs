use crate::core::types::*;
use reqwest::Client;

impl Service {
    pub fn new(provider: Option<&str>) -> Self {
        let provider = provider.unwrap();

        let (envar, endpoint, model) = match provider {
            "codestral" => (
                Some("CODE_API_KEY"),
                "https://codestral.mistral.ai/v1/chat/completions",
                "codestral-latest",
            ),

            "openrouter" => (
                Some("OPENROUTER_API_KEY"),
                "https://openrouter.ai/api/v1/chat/completions",
                "openrouter/free",
            ),

            "ollama" => (
                None,
                "http://localhost:11434/v1/chat/completions",
                "qwen2:0.5b",
            ),

            _ => todo!("{}", provider),
        };

        let apikey = envar.map(|var| std::env::var(var).expect("API key environment variable not found"));

        Self {
            http: Client::new(),
            apikey,
            endpoint: endpoint.to_owned(),
            model: model.to_owned(),
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

        Ok(response.choices[0].message.content.clone())
    }
}

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::rss::Article;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
    #[serde(default)]
    code: i32,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: "gemini-2.5-flash".to_string(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub async fn summarize_articles(&self, articles: &[Article]) -> Result<String> {
        if articles.is_empty() {
            return Ok("No articles to summarize.".to_string());
        }

        let articles_text: String = articles
            .iter()
            .take(20) // Limit to avoid token limits
            .enumerate()
            .map(|(i, a)| {
                format!(
                    "{}. [{}] {}\n   {}\n",
                    i + 1,
                    a.source,
                    a.title,
                    a.summary.as_deref().unwrap_or("No summary available")
                )
            })
            .collect();

        let prompt = format!(
            r#"You are a financial analyst assistant. Analyze these news articles and provide:

1. **Market Overview**: A brief summary of the overall market sentiment
2. **Key Stories**: The most important stories and why they matter
3. **Market Impact Analysis**: How these stories might affect:
   - Stock markets (US, global)
   - Cryptocurrency markets
   - Interest rates and bonds
   - Specific sectors or companies
4. **Actionable Insights**: Key takeaways for investors

Articles:
{}

Provide a concise, well-structured analysis suitable for a morning market briefing."#,
            articles_text
        );

        self.generate(&prompt).await
    }

    async fn generate(&self, prompt: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let raw = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .text()
            .await?;

        let response: GeminiResponse = serde_json::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("Failed to parse Gemini response: {e}\nBody: {raw}"))?;

        if let Some(err) = response.error {
            return Err(anyhow::anyhow!(
                "Gemini API error {}: {}",
                err.code,
                err.message
            ));
        }

        let text = response
            .candidates
            .as_deref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_else(|| "No response generated.".to_string());

        Ok(text)
    }
}

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ResendEmail {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

#[derive(Deserialize)]
struct ResendResponse {
    id: Option<String>,
}

pub struct ResendClient {
    client: Client,
    api_key: String,
    from_email: String,
}

impl ResendClient {
    pub fn new(api_key: String, from_email: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            from_email,
        }
    }

    pub async fn send_digest(&self, to: &str, subject: &str, content: &str) -> Result<String> {
        let html = self.markdown_to_html(content);

        let email = ResendEmail {
            from: self.from_email.clone(),
            to: vec![to.to_string()],
            subject: subject.to_string(),
            html,
        };

        let response = self
            .client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&email)
            .send()
            .await?;

        if response.status().is_success() {
            let res: ResendResponse = response.json().await?;
            Ok(res.id.unwrap_or_else(|| "sent".to_string()))
        } else {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to send email: {}", error_text)
        }
    }

    fn markdown_to_html(&self, markdown: &str) -> String {
        // Simple markdown to HTML conversion
        let html = markdown
            .lines()
            .map(|line| {
                let line = line.trim();
                if line.starts_with("# ") {
                    format!("<h1>{}</h1>", &line[2..])
                } else if line.starts_with("## ") {
                    format!("<h2>{}</h2>", &line[3..])
                } else if line.starts_with("### ") {
                    format!("<h3>{}</h3>", &line[4..])
                } else if line.starts_with("**") && line.ends_with("**") {
                    format!("<p><strong>{}</strong></p>", &line[2..line.len() - 2])
                } else if line.starts_with("- ") {
                    format!("<li>{}</li>", &line[2..])
                } else if line.is_empty() {
                    "<br>".to_string()
                } else {
                    format!("<p>{}</p>", line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; line-height: 1.6; }}
        h1 {{ color: #1a1a1a; border-bottom: 2px solid #0066cc; padding-bottom: 10px; }}
        h2 {{ color: #333; margin-top: 24px; }}
        h3 {{ color: #555; }}
        li {{ margin: 8px 0; }}
        p {{ color: #444; }}
    </style>
</head>
<body>
{}
</body>
</html>"#,
            html
        )
    }
}

use anyhow::Result;
use pulldown_cmark::{html, Options, Parser};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::rss::Article;

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

    pub async fn send_digest(&self, to: &str, subject: &str, content: &str, articles: &[Article]) -> Result<String> {
        let mut html = self.markdown_to_html(content);
        html = self.append_sources(html, articles);

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
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);

        let parser = Parser::new_ext(markdown, options);
        let mut body = String::new();
        html::push_html(&mut body, parser);

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
  body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, sans-serif;
    max-width: 680px;
    margin: 0 auto;
    padding: 32px 24px;
    background: #f9f9f9;
    color: #222;
    line-height: 1.7;
  }}
  .card {{
    background: #fff;
    border-radius: 8px;
    padding: 32px;
    box-shadow: 0 1px 4px rgba(0,0,0,0.08);
  }}
  h1 {{
    font-size: 1.5em;
    color: #111;
    border-bottom: 3px solid #0057ff;
    padding-bottom: 10px;
    margin-top: 0;
  }}
  h2 {{
    font-size: 1.15em;
    color: #0057ff;
    margin-top: 28px;
    margin-bottom: 6px;
  }}
  h3 {{
    font-size: 1em;
    color: #333;
    margin-top: 16px;
    margin-bottom: 4px;
  }}
  p {{ margin: 8px 0 12px; color: #333; }}
  ul, ol {{ padding-left: 20px; margin: 8px 0 12px; }}
  li {{ margin: 6px 0; color: #333; }}
  strong {{ color: #111; }}
  hr {{ border: none; border-top: 1px solid #e5e5e5; margin: 24px 0; }}
  a {{ color: #0057ff; text-decoration: none; }}
  a:hover {{ text-decoration: underline; }}
  .sources {{
    margin-top: 32px;
    border-top: 2px solid #e5e5e5;
    padding-top: 20px;
  }}
  .sources h2 {{ color: #555; font-size: 1em; text-transform: uppercase; letter-spacing: 0.05em; }}
  .sources ul {{ list-style: none; padding-left: 0; }}
  .sources li {{ padding: 4px 0; border-bottom: 1px solid #f0f0f0; }}
  .source-tag {{ color: #999; font-size: 0.82em; margin-left: 6px; }}
</style>
</head>
<body>
<div class="card">
{}
</div>
</body>
</html>"#,
            body
        )
    }

    fn append_sources(&self, html: String, articles: &[Article]) -> String {
        let links: Vec<String> = articles
            .iter()
            .take(20)
            .filter(|a| !a.link.is_empty())
            .map(|a| {
                format!(
                    "<li><a href=\"{}\">{}</a> <span class=\"source-tag\">[{}]</span></li>",
                    a.link, a.title, a.source
                )
            })
            .collect();

        let sources_section = format!(
            "<div class=\"sources\"><h2>Sources</h2><ul>{}</ul></div>",
            links.join("\n")
        );

        html.replace("</body>", &format!("{}\n</body>", sources_section))
    }
}

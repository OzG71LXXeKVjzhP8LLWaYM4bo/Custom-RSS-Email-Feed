mod config;
mod email;
mod gemini;
mod rss;

use anyhow::Result;
use chrono::Local;
use config::Config;
use email::ResendClient;
use gemini::GeminiClient;
use rss::RssFetcher;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.toml".to_string());

    println!("Loading config from: {}", config_path);
    let config = Config::load(&config_path)?;

    println!("Fetching RSS feeds...");
    let fetcher = RssFetcher::new();
    let feeds = config.feeds_as_tuples();
    let articles = fetcher.fetch_all_feeds(&feeds).await;

    println!("Found {} total articles", articles.len());

    if articles.is_empty() {
        println!("No articles found. Exiting.");
        return Ok(());
    }

    println!("Generating market analysis with Gemini...");
    let gemini = GeminiClient::new(config.gemini_api_key.clone());
    let gemini = if let Some(model) = &config.gemini_model {
        gemini.with_model(model)
    } else {
        gemini
    };

    let analysis = gemini.summarize_articles(&articles).await?;

    println!("Sending email digest via Resend...");
    let resend = ResendClient::new(config.resend_api_key.clone(), config.from_email.clone());

    let date = Local::now().format("%B %d, %Y %H:%M");
    let subject = format!("Market Digest - {}", date);

    let email_id = resend
        .send_digest(&config.to_email, &subject, &analysis)
        .await?;

    println!("Email sent successfully! ID: {}", email_id);

    Ok(())
}

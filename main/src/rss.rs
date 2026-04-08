use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use feed_rs::parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub summary: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub source: String,
}

pub struct RssFetcher {
    client: Client,
}

impl RssFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_feed(&self, url: &str, source_name: &str) -> Result<Vec<Article>> {
        let response = self.client.get(url).send().await?.bytes().await?;
        let feed = parser::parse(&response[..])?;

        let articles: Vec<Article> = feed
            .entries
            .into_iter()
            .map(|entry| Article {
                title: entry.title.map(|t| t.content).unwrap_or_default(),
                link: entry
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .unwrap_or_default(),
                summary: entry.summary.map(|s| s.content),
                published: entry.published.map(|d| d.into()),
                source: source_name.to_string(),
            })
            .collect();

        Ok(articles)
    }

    pub async fn fetch_all_feeds(&self, feeds: &[(&str, &str)]) -> Vec<Article> {
        let mut all_articles = Vec::new();

        for (url, name) in feeds {
            match self.fetch_feed(url, name).await {
                Ok(articles) => {
                    println!("Fetched {} articles from {}", articles.len(), name);
                    all_articles.extend(articles);
                }
                Err(e) => {
                    eprintln!("Failed to fetch {}: {}", name, e);
                }
            }
        }

        // Keep only articles from the last 24 hours; fall back to all if none qualify
        let cutoff = Utc::now() - Duration::hours(24);
        let fresh: Vec<Article> = all_articles
            .iter()
            .filter(|a| a.published.map(|d| d > cutoff).unwrap_or(false))
            .cloned()
            .collect();

        let mut result = if fresh.is_empty() {
            eprintln!("Warning: no articles within 24 hours, using all fetched articles");
            all_articles
        } else {
            fresh
        };

        // Sort by published date, most recent first
        result.sort_by(|a, b| b.published.cmp(&a.published));
        result
    }
}

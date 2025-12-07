use anyhow::Result;
use chrono::{DateTime, Utc};
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

        // Sort by published date, most recent first
        all_articles.sort_by(|a, b| b.published.cmp(&a.published));
        all_articles
    }
}

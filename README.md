# RSS Market Digest

A scheduled Rust tool that fetches RSS feeds, summarizes them with Google Gemini, and emails you a market briefing via Resend.

## How it works

1. Fetches articles from configured RSS feeds (financial news, tech, crypto)
2. Sends the top 20 articles to Gemini 2.5 Flash for analysis
3. Gemini returns a structured market briefing: sentiment overview, key stories, impact analysis, and actionable insights
4. The briefing is converted to HTML and emailed via Resend, with a Sources section linking back to original articles

## Setup

### Prerequisites

- [Rust](https://rustup.rs/) (for local builds)
- A [Google AI Studio](https://aistudio.google.com/) API key (free tier works fine)
- A [Resend](https://resend.com/) API key and verified sending domain

### Configuration

```bash
cd main
cp config.example.toml config.toml
```

Edit `config.toml` with your API keys, email addresses, and RSS feeds.

### Run locally

```bash
cd main
cargo run --release -- config.toml
```

### Deploy with GitHub Actions

The included workflow (`.github/workflows/digest.yml`) runs the digest 3x daily at 7am, 1pm, and 11pm Sydney time.

1. Add these secrets to your repo (Settings > Secrets and variables > Actions):
   - `GEMINI_API_KEY`
   - `RESEND_API_KEY`
   - `FROM_EMAIL`
   - `TO_EMAIL`

2. Push to `main` — the workflow will run on schedule, or trigger it manually from the Actions tab.

### Deploy with systemd

```bash
cd main
chmod +x install.sh
./install.sh
```

This builds the binary and installs a user-level systemd timer. Check the paths in `systemd/rss-market-digest.service` match your system before running.

## Customizing feeds

Edit the `[[feeds]]` entries in `config.toml`:

```toml
[[feeds]]
name = "Reuters Business"
url = "https://www.reutersagency.com/feed/?best-topics=business-finance&post_type=best"
```

## Changing the Gemini model

Set `gemini_model` in `config.toml`:

```toml
gemini_model = "gemini-2.5-pro"
```

Default is `gemini-2.5-flash`.

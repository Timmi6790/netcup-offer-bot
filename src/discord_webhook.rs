use rss::Item;
use webhook::client::WebhookClient;

use crate::error::Error;
use crate::feed::Feed;
use crate::Result;

pub struct DiscordWebhook {
    client: WebhookClient,
}

impl std::fmt::Debug for DiscordWebhook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscordWebhook").finish()
    }
}

impl DiscordWebhook {
    pub fn new(url: &str) -> Self {
        DiscordWebhook {
            client: WebhookClient::new(url),
        }
    }

    #[tracing::instrument]
    pub async fn send_discord_message(&self, feed: &Feed, item: Item) -> Result<bool> {
        info!(
            "Sending message for feed {} with title \"{}\"",
            feed.name(),
            item.title().unwrap_or("No title")
        );

        let result = self
            .client
            .send(|message| {
                message
                    .username(&format!("Feed - {}", feed.name()))
                    .embed(|embed| {
                        let embed = embed
                            .title(item.title().unwrap_or("No title"))
                            .description(item.description().unwrap_or("No description"));

                        if let Some(url) = item.link() {
                            embed.url(url);
                        }

                        if let Some(date) = item.pub_date() {
                            embed.field("Date", date, false);
                        }

                        let categories = item
                            .categories()
                            .iter()
                            .map(|category| category.name.clone())
                            .collect::<Vec<String>>()
                            .join(", ");
                        if !categories.is_empty() {
                            embed.field("Categories", &categories, false);
                        }

                        embed
                    })
            })
            .await
            .map_err(|e| Error::custom(e.to_string()))?;

        Ok(result)
    }
}

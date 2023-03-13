#[macro_use]
extern crate tracing;

use opentelemetry::{Context, KeyValue};
use std::fmt::Debug;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::{SpanBackendWithUrl, TracingMiddleware};
use secrecy::ExposeSecret;
use strum::IntoEnumIterator;

use crate::config::Config;
use crate::discord_webhook::DiscordWebhook;
use crate::error::Error;
use crate::feed::Feed;
use crate::feed_state::FeedStates;

pub mod config;
mod discord_webhook;
mod error;
mod feed;
mod feed_state;
pub mod metrics;

pub type Result<T> = anyhow::Result<T, Error>;

#[derive(Debug)]
pub struct FeedChecker {
    client: ClientWithMiddleware,
    states: FeedStates,
    hook: DiscordWebhook,
}

impl FeedChecker {
    pub fn new(client: ClientWithMiddleware, states: FeedStates, webhook: DiscordWebhook) -> Self {
        Self {
            client,
            states,
            hook: webhook,
        }
    }

    pub fn from_config(config: &Config) -> Self {
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::<SpanBackendWithUrl>::new())
            .build();
        let states = FeedStates::load().unwrap();
        let hook = DiscordWebhook::new(config.discord_webhook_url.expose_secret());

        FeedChecker::new(client, states, hook)
    }

    #[tracing::instrument]
    pub async fn check_feeds(&mut self) {
        trace!("Run feed check");

        for feed in Feed::iter() {
            self.check_feed(feed).await;
        }

        if let Err(e) = self.states.save().await {
            error!("Error saving feed states: {}", e);
        }
    }

    #[tracing::instrument]
    pub async fn check_feed(&mut self, feed: Feed) {
        debug!("Checking feed {}", feed.name());

        match feed.fetch(&self.client).await {
            Ok(feed_result) => {
                // Filter out already sent items
                trace!("Found {} items for feed", feed_result.items.len());
                let items = self.states.get_new_feed(&feed, feed_result.items);
                if items.is_empty() {
                    debug!("No new items found");
                    return;
                }

                debug!("Found {} new items", items.len());

                // Increase metrics
                let cx = Context::current();
                metrics::FEED_COUNTER.add(
                    &cx,
                    items.len() as u64,
                    &[KeyValue::new("feed", feed.name().to_string())],
                );

                // Send feed to discord
                for item in items {
                    if let Err(e) = self.hook.send_discord_message(&feed, item).await {
                        error!("Error sending message for feed {}: {}", feed.name(), e);
                    }
                }
            }
            Err(e) => {
                error!("Error fetching feed for {}: {}", feed.name(), e);
            }
        }
    }
}

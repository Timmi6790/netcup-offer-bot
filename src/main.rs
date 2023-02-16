#[macro_use]
extern crate tracing;

use std::env;
use std::net::SocketAddr;
use std::str::FromStr;

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::{SpanBackendWithUrl, TracingMiddleware};
use sentry::ClientInitGuard;
use strum::IntoEnumIterator;
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, Layer};

use crate::config::Config;
use crate::discord_webhook::DiscordWebhook;
use crate::error::Error;
use crate::feed::Feed;
use crate::feed_state::FeedStates;

mod config;
mod discord_webhook;
mod error;
mod feed;
mod feed_state;
mod metrics;

const ENV_SENTRY_DSN: &str = "SENTRY_DSN";
const ENV_LOG_LEVEL: &str = "LOG_LEVEL";

const DEFAULT_LOG_LEVEL: &str = "info";

pub type Result<T> = anyhow::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;

    let dns = match env::var(ENV_SENTRY_DSN) {
        Ok(dns) => Some(dns),
        Err(_) => None,
    };
    // Prevents the process from exiting until all events are sent
    let _sentry = setup_sentry(dns);

    let config = Config::from_env()?;

    setup_metrics(&config.metric_socket)?;

    info!("Starting feed bot");
    let mut checker = FeedChecker::from_config(&config);
    let mut stream = IntervalStream::new(time::interval(config.check_interval));
    while let Some(_ts) = stream.next().await {
        checker.check_feeds().await;
    }

    Ok(())
}

fn setup_tracing() -> Result<()> {
    let level = env::var(ENV_LOG_LEVEL).unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());
    let level = tracing::Level::from_str(&level)?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(filter::LevelFilter::from_level(level)))
        .with(sentry::integrations::tracing::layer().with_filter(filter::LevelFilter::DEBUG))
        .init();

    Ok(())
}

fn setup_sentry(dns: Option<String>) -> Option<ClientInitGuard> {
    // Only enable sentry if the dns is set
    let dns = match dns {
        Some(dns) => dns,
        None => {
            info!("{ENV_SENTRY_DSN} not set, skipping Sentry setup");
            return None;
        }
    };

    // Sentry innit
    Some(sentry::init((
        dns,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            auto_session_tracking: true,
            traces_sample_rate: 1.0,
            enable_profiling: true,
            profiles_sample_rate: 1.0,
            attach_stacktrace: true,
            ..Default::default()
        },
    )))
}

fn setup_metrics(socket: &SocketAddr) -> Result<()> {
    prometheus_exporter::start(*socket)?;
    Ok(())
}

#[derive(Debug)]
struct FeedChecker {
    client: ClientWithMiddleware,
    states: FeedStates,
    hook: DiscordWebhook,
}

impl FeedChecker {
    pub fn from_config(config: &Config) -> Self {
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::<SpanBackendWithUrl>::new())
            .build();
        let states = FeedStates::load().unwrap();
        let hook = DiscordWebhook::new(&config.discord_webhook_url);

        Self {
            client,
            states,
            hook,
        }
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
                let counter = metrics::FEED_COUNTER.with_label_values(&[feed.name()]);
                counter.inc_by(items.len() as u64);

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

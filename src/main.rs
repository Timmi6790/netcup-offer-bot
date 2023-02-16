#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::env;
use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;

use log::LevelFilter;
use pretty_env_logger::env_logger::Builder;
use sentry::{capture_message, ClientInitGuard};
use strum::IntoEnumIterator;
use tokio::time;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

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
    let dns = match env::var(ENV_SENTRY_DSN) {
        Ok(dns) => Some(dns),
        Err(_) => None,
    };
    // Prevents the process from exiting until all events are sent
    let _sentry = setup_logging(dns)?;

    error!("Test error 2 {}", Error::Custom("Test error 2".to_string()));
    capture_message("Test message", sentry::Level::Error);
    let config = Config::from_env()?;

    setup_metrics(&config.metric_socket)?;

    info!("Starting feed bot");
    run(config).await
}

async fn run(config: Config) -> Result<()> {
    let mut states = FeedStates::load()?;
    let hook = DiscordWebhook::new(&config.discord_webhook_url);

    let mut stream = IntervalStream::new(time::interval(config.check_interval));
    while let Some(_ts) = stream.next().await {
        trace!("Run feed check");

        for feed in Feed::iter() {
            trace!("Checking feed {}", feed.name());

            match feed.fetch().await {
                Ok(feed_result) => {
                    // Filter out already sent items
                    let items = states.get_new_feed(&feed, feed_result.items);
                    if items.is_empty() {
                        continue;
                    }

                    // Increase metrics
                    let counter = metrics::FEED_COUNTER.with_label_values(&[feed.name()]);
                    counter.inc_by(items.len() as u64);

                    // Send feed to discord
                    for item in items {
                        if let Err(e) = hook.send_discord_message(&feed, item).await {
                            error!("Error sending message for feed {}: {}", feed.name(), e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error fetching feed for {}: {}", feed.name(), e);
                }
            }
        }

        if let Err(e) = states.save() {
            error!("Error saving feed states: {}", e);
        }
    }

    Ok(())
}

fn build_logger() -> Result<(Builder, LevelFilter)> {
    let mut log_builder = pretty_env_logger::formatted_builder();

    // Set level
    let level = env::var(ENV_LOG_LEVEL).unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());
    let level = LevelFilter::from_str(&level)?;
    log_builder.filter_level(level);

    // Set format
    log_builder.format(|buf, record| {
        let timestamp = buf.timestamp();
        writeln!(
            buf,
            "{}[{}][{}] {}",
            timestamp,
            record.target(),
            record.level(),
            record.args()
        )
    });

    Ok((log_builder, level))
}

fn setup_logging(dns: Option<String>) -> Result<Option<ClientInitGuard>> {
    // Setup logger
    let (mut log_builder, level) = build_logger()?;

    // Only enable sentry if the dns is set
    let dns = match dns {
        Some(dns) => dns,
        None => {
            log_builder.init();

            info!("{ENV_SENTRY_DSN} not set, skipping Sentry setup");
            return Ok(None);
        }
    };

    // Sentry
    // Sentry logging support
    let logger = sentry::integrations::log::SentryLogger::with_dest(log_builder.build());

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(level);

    // Sentry innit
    Ok(Some(sentry::init((
        dns,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ))))
}

fn setup_metrics(socket: &SocketAddr) -> Result<()> {
    prometheus_exporter::start(*socket)?;
    Ok(())
}

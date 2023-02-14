use std::env;

use log::{error, info, trace};
use sentry::ClientInitGuard;
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

const ENV_SENTRY_DSN: &str = "SENTRY_DSN";

pub type Result<T> = anyhow::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let dns = match env::var(ENV_SENTRY_DSN) {
        Ok(dns) => Some(dns),
        Err(_) => None,
    };
    // Prevents the process from exiting until all events are sent
    let _sentry = setup_sentry(dns);

    setup_logging()?;

    info!("Starting feed bot");
    let config = Config::from_env()?;
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
                    for item in states.get_new_feed(&feed, feed_result.items) {
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

fn setup_sentry(dns: Option<String>) -> Option<ClientInitGuard> {
    let dns = match dns {
        Some(dns) => dns,
        None => {
            println!("{ENV_SENTRY_DSN} not set, skipping Sentry setup");
            return None;
        }
    };

    Some(sentry::init((
        dns,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    )))
}

fn setup_logging() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

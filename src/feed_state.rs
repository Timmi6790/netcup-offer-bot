use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, Utc};
use chrono::serde::ts_seconds_option;
use log::{debug, error, info, trace};
use rss::Item;
use serde::{Deserialize, Serialize};

use crate::feed::Feed;

const FEED_STATE_FILE: &str = "./data/feed_state.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedStates {
    feeds: HashMap<Feed, FeedState>,
}

impl FeedStates {
    pub fn load() -> crate::Result<Self> {
        let path = std::path::Path::new(FEED_STATE_FILE);

        if path.exists() {
            info!("Loading feed state from file");

            let content = std::fs::read_to_string(FEED_STATE_FILE)?;
            serde_json::from_str(&content).map_err(|e| e.into())
        } else {
            // Ensure that the path exists
            let prefix = path.parent().ok_or("Invalid FEED_SATE_FILE path")?;
            std::fs::create_dir_all(prefix)?;

            Ok(Self {
                feeds: HashMap::new(),
            })
        }
    }

    fn un_dirty(&mut self) {
        for state in self.feeds.values_mut() {
            state.dirty = false;
        }
    }

    pub fn get_feed_or_create(&mut self, feed: &Feed) -> &mut FeedState {
        self.feeds
            .entry(*feed)
            .or_insert_with(FeedState::default)
    }

    pub fn get_new_feed(&mut self, feed: &Feed, items: Vec<Item>) -> Vec<Item> {
        if items.is_empty() {
            return items;
        }

        let mut last_date = None;
        let mut sorted = Vec::new();

        let feed_state = self.get_feed_or_create(feed);

        for item in items {
            match &item.pub_date {
                Some(date) => {
                    let date = DateTime::parse_from_rfc2822(date);
                    match date {
                        Ok(date) => {
                            if feed_state.is_before(&date) {
                                trace!("Skipping item, already seen {:?}", date);
                                continue;
                            }

                            trace!("Found new item {:?}", date);

                            if last_date.is_none() || date > last_date.unwrap() {
                                last_date = Some(date);
                            }
                            sorted.push(item);
                        }
                        Err(e) => {
                            error!("Error parsing date for feed {}: {}", feed.name(), e);
                        }
                    }
                }
                None => {
                    info!("Skipping item without date on feed {}", feed.name());
                }
            }
        }

        // Store new last date if found
        if let Some(date) = last_date {
            // Convert to UTC
            let date = date.with_timezone(&Utc);
            feed_state.set_last_update(date);
        }

        sorted
    }

    pub fn is_dirty(&self) -> bool {
        self.feeds.values().any(|state| state.dirty)
    }

    pub fn save(&mut self) -> crate::Result<()> {
        if !self.is_dirty() {
            trace!("Feed state is not dirty, skipping save");
            return Ok(());
        }

        debug!("Saving feed state to file");

        // Save to file
        std::fs::write(FEED_STATE_FILE, serde_json::to_string_pretty(self)?)?;

        self.un_dirty();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedState {
    #[serde(with = "ts_seconds_option")]
    last_update: Option<DateTime<Utc>>,
    #[serde(skip_serializing, default)]
    dirty: bool,
}

impl FeedState {
    fn new(last_update: Option<DateTime<Utc>>, dirty: bool) -> Self {
        Self { last_update, dirty }
    }

    pub fn is_before(&self, date: &DateTime<FixedOffset>) -> bool {
        self.last_update
            .map_or(false, |last_update| *date <= last_update)
    }

    pub fn set_last_update(&mut self, date: DateTime<Utc>) {
        self.last_update = Some(date);
        self.dirty = true;
    }
}

impl Default for FeedState {
    fn default() -> Self {
        Self::new(None, false)
    }
}

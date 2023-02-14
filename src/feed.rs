use std::fmt;

use rss::validation::Validate;
use rss::Channel;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, EnumIter, Clone, Copy, Serialize, Deserialize, Eq, Hash)]
pub enum Feed {
    Netcup,
}

impl Feed {
    pub fn name(&self) -> &str {
        match self {
            Feed::Netcup => "Netcup",
        }
    }

    pub fn url(&self) -> &str {
        match self {
            Feed::Netcup => "https://www.netcup-sonderangebote.de/feed/",
        }
    }

    pub async fn fetch(&self) -> crate::Result<Channel> {
        let content = reqwest::get(self.url()).await?.bytes().await?;
        let channel = Channel::read_from(&content[..])?;
        channel.validate()?;
        Ok(channel)
    }
}

impl fmt::Display for Feed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

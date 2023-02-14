use std::env;
use std::time::Duration;

use crate::error::Error;

const ENV_WEB_HOOK: &str = "WEB_HOOK";
const ENV_CHECK_INTERVAL: &str = "CHECK_INTERVAL";

pub struct Config {
    pub discord_webhook_url: String,
    pub check_interval: Duration,
}

impl Config {
    fn get_env(name: &str) -> crate::Result<String> {
        env::var(name).map_err(|e| match e {
            env::VarError::NotPresent => {
                Error::ConfigVar(format!("Missing environment variable {}", name))
            }
            e => Error::ConfigVar(e.to_string()),
        })
    }

    pub fn from_env() -> crate::Result<Self> {
        let discord_webhook_url = Config::get_env(ENV_WEB_HOOK)?;
        let check_interval = Config::get_env(ENV_CHECK_INTERVAL)?.parse::<u64>()?;
        let check_interval = Duration::from_secs(check_interval);

        Ok(Self {
            discord_webhook_url,
            check_interval,
        })
    }
}

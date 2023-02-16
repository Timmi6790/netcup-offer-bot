use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use crate::error::Error;

const ENV_WEB_HOOK: &str = "WEB_HOOK";
const ENV_CHECK_INTERVAL: &str = "CHECK_INTERVAL";
const ENV_METRIC_IP: &str = "METRIC_IP";
const ENV_METRIC_PORT: &str = "METRIC_PORT";

const DEFAULT_METRIC_IP: &str = "127.0.0.1";
const DEFAULT_METRIC_PORT: u16 = 9184;

pub struct Config {
    pub discord_webhook_url: String,
    pub check_interval: Duration,
    pub metric_socket: SocketAddr,
}

impl Config {
    fn get_env(name: &str) -> crate::Result<String> {
        env::var(name).map_err(|e| match e {
            env::VarError::NotPresent => {
                Error::ConfigVar(format!("Missing environment variable {name}"))
            }
            e => Error::ConfigVar(e.to_string()),
        })
    }

    pub fn from_env() -> crate::Result<Self> {
        let discord_webhook_url = Config::get_env(ENV_WEB_HOOK)?;
        let check_interval = Config::get_env(ENV_CHECK_INTERVAL)?.parse::<u64>()?;
        let check_interval = Duration::from_secs(check_interval);

        let metric_ip =
            Config::get_env(ENV_METRIC_IP).unwrap_or_else(|_| DEFAULT_METRIC_IP.to_string());
        let metric_port = Config::get_env(ENV_METRIC_PORT)
            .unwrap_or_else(|_| DEFAULT_METRIC_PORT.to_string())
            .parse::<u16>()?;
        let metric_ip = match metric_ip.parse::<std::net::IpAddr>() {
            Ok(ip) => ip,
            Err(_) => {
                return Err(Error::ConfigVar(format!(
                    "Invalid {ENV_METRIC_IP} address: {metric_ip}"
                )));
            }
        };
        let metric_socket = SocketAddr::new(metric_ip, metric_port);

        Ok(Self {
            discord_webhook_url,
            check_interval,
            metric_socket,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORRECT_WEB_HOOK: &str = "https://discord.com/api/webhooks/";
    const CORRECT_CHECK_INTERVAL: &str = "42";
    const CORRECT_METRIC_IP: &str = "127.0.0.1";
    const CORRECT_METRIC_PORT: &str = "9184";

    #[test]
    fn test_from_env_missing_env() {
        temp_env::with_vars_unset(vec![ENV_WEB_HOOK, ENV_CHECK_INTERVAL], || {
            let result = Config::from_env();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_from_env_minimal() {
        temp_env::with_vars(
            vec![
                (ENV_WEB_HOOK, Some(CORRECT_WEB_HOOK)),
                (ENV_CHECK_INTERVAL, Some(CORRECT_CHECK_INTERVAL)),
            ],
            || {
                let result = Config::from_env();
                assert!(result.is_ok());

                let config = result.unwrap();
                assert_eq!(config.discord_webhook_url, CORRECT_WEB_HOOK);
                assert_eq!(
                    config.check_interval,
                    Duration::from_secs(CORRECT_CHECK_INTERVAL.parse().unwrap())
                );
            },
        );
    }

    #[test]
    fn test_from_env_full() {
        temp_env::with_vars(
            vec![
                (ENV_WEB_HOOK, Some(CORRECT_WEB_HOOK)),
                (ENV_CHECK_INTERVAL, Some(CORRECT_CHECK_INTERVAL)),
                (ENV_METRIC_IP, Some(CORRECT_METRIC_IP)),
                (ENV_METRIC_PORT, Some(CORRECT_METRIC_PORT)),
            ],
            || {
                let result = Config::from_env();
                assert!(result.is_ok());

                let config = result.unwrap();
                assert_eq!(config.discord_webhook_url, CORRECT_WEB_HOOK);
                assert_eq!(
                    config.check_interval,
                    Duration::from_secs(CORRECT_CHECK_INTERVAL.parse().unwrap())
                );
                assert_eq!(
                    config.metric_socket,
                    SocketAddr::new(
                        CORRECT_METRIC_IP.parse().unwrap(),
                        CORRECT_METRIC_PORT.parse().unwrap(),
                    )
                );
            },
        );
    }

    #[test]
    fn test_from_env_invalid_check_interval() {
        temp_env::with_vars(
            vec![
                (ENV_WEB_HOOK, Some(CORRECT_WEB_HOOK)),
                (ENV_CHECK_INTERVAL, Some("d")),
            ],
            || {
                let result = Config::from_env();
                assert!(result.is_err());
            },
        );
    }

    #[test]
    fn test_from_env_invalid_metric_ip() {
        temp_env::with_vars(
            vec![
                (ENV_WEB_HOOK, Some(CORRECT_WEB_HOOK)),
                (ENV_CHECK_INTERVAL, Some(CORRECT_CHECK_INTERVAL)),
                (ENV_METRIC_IP, Some("abcde")),
                (ENV_METRIC_PORT, Some(CORRECT_METRIC_PORT)),
            ],
            || {
                let result = Config::from_env();
                assert!(result.is_err());
            },
        );
    }

    #[test]
    fn test_from_env_invalid_metric_port() {
        temp_env::with_vars(
            vec![
                (ENV_WEB_HOOK, Some(CORRECT_WEB_HOOK)),
                (ENV_CHECK_INTERVAL, Some(CORRECT_CHECK_INTERVAL)),
                (ENV_METRIC_IP, Some(CORRECT_METRIC_IP)),
                (ENV_METRIC_PORT, Some("abcde")),
            ],
            || {
                let result = Config::from_env();
                assert!(result.is_err());
            },
        );
    }

    #[test]
    fn test_from_env_invalid_unicode_character() {
        temp_env::with_vars(
            vec![(ENV_WEB_HOOK, Some("⛷")), (ENV_CHECK_INTERVAL, Some("⛷"))],
            || {
                let result = Config::from_env();
                assert!(result.is_err());
            },
        );
    }
}

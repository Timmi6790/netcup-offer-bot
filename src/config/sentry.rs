//! The `telemetry.sentry` block: error reporting and performance tracing.
//!
//! Described unconditionally, and deliberately so. The client behind these keys is behind the
//! `sentry` feature, but the *keys* are not: a contract that gained and lost rows with a build
//! flag would stop being one document a chart can be checked against. A build without the
//! feature reads the same block and refuses to boot on [`SentryConfig::enabled`], which is the
//! honest answer — see `src/telemetry/sentry.rs`.
//!
//! # The doc comments below are documentation output
//!
//! As everywhere in [`super`]: the first paragraph of each field is the README table's cell and
//! is written for an operator setting the value. Anything under it is for whoever reads the
//! type.

use secrecy::SecretString;
use serde::Deserialize;

use super::SentryLevel;

/// Sentry error reporting and performance tracing.
///
/// Off by default. A DSN is an egress destination for whatever a log line happens to carry, so
/// switching it on is a decision an operator makes once per deployment rather than one this
/// crate makes for them. With [`Self::enabled`] set the boot fails without a usable
/// [`Self::dsn`], instead of installing a reporter that reports nowhere.
#[derive(Debug, Deserialize)]
// As [`super::Config`]: a key nobody declared is a misspelling of one that is here, and this
// block is the one where that costs the most — `smaple_rate` ignored means the volume cap an
// operator set against a quota was never applied.
#[serde(default, deny_unknown_fields)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct SentryConfig {
    /// Initialise the Sentry client. `false` installs no client, no panic hook and no layer, so
    /// every other key here is inert and nothing leaves the process.
    ///
    /// Set on a binary built with `--no-default-features` it fails the boot: that build carries
    /// no client to install, and starting anyway would be the silent no-op this key exists to
    /// avoid.
    pub enabled: bool,
    /// Ingest URL, `https://<key>@<host>/<project>`. Required once `enabled` is set.
    ///
    /// A write credential for the project's event stream, wrapped for the reason
    /// [`DiscordConfig::webhook_url`](super::DiscordConfig::webhook_url) is. Empty is treated as
    /// absent rather than as a malformed URL, because an unfilled chart value and a compose
    /// pass-through both produce an empty string and neither is a typo in a DSN.
    // Not rustdoc: skipped on the way out because `SecretString` has no `Serialize` impl. The
    // key keeps its row and its `secret` flag, which is all a table can usefully say about an
    // optional credential.
    #[serde(skip_serializing)]
    #[cfg_attr(feature = "config-schema", config(secret))]
    pub dsn: Option<SecretString>,
    /// Environment tag on every event. Defaults to `production`, or `development` for a debug
    /// build.
    ///
    /// Always sent, never left for the SDK to fill in. `sentry::init` otherwise reads
    /// `SENTRY_ENVIRONMENT` from the process environment — a second configuration channel that
    /// bypasses the layered loader, its shadow-key rejection and the contract, whose external
    /// surface says this image reads nothing outside the loader's namespace.
    pub environment: Option<String>,
    /// Release tag on every event. Defaults to the version the binary was built from.
    ///
    /// Set explicitly for the same reason [`Self::environment`] is, and this is the field that
    /// makes a regression attributable to a deploy. The default is what
    /// `sentry::release_name!` reads out of the build, which is what the image's debug-file
    /// upload names the symbols under.
    pub release: Option<String>,
    /// Host tag on every event. Unset, Sentry reports none.
    ///
    /// The hostname of a replica is infrastructure detail, and a pod's is a generated string
    /// that changes on every restart, so it is worth a tag only where a deployment has a
    /// stable name to give it.
    pub server_name: Option<String>,
    /// Fraction of captured events actually sent, `0.0`–`1.0`.
    ///
    /// A blunt volume cap: it drops whole issues rather than repetitions of one, so leave it at
    /// `1.0` unless a quota forces otherwise.
    // The interval is the one `check_rate` in `src/telemetry/sentry.rs` already refuses the boot
    // over, inclusive at both ends. Float literals because the field is an `f32`: `min = 0`
    // would publish the integer `0`, a different number to a consumer reading the keywords.
    #[cfg_attr(feature = "config-schema", config(range(min = 0.0, max = 1.0)))]
    pub sample_rate: f32,
    /// Fraction of traces that are recorded, `0.0`–`1.0`. `0.0` records none.
    ///
    /// This process starts every trace it has — nothing hands it one — so at `0.0` no spans are
    /// built at all rather than built and dropped by the sampler. `0.05`–`0.2` is an ordinary
    /// production figure; the previous hard-coded value was `0.2`.
    // As [`Self::sample_rate`]: `check_rate` refuses this one by the same call.
    #[cfg_attr(feature = "config-schema", config(range(min = 0.0, max = 1.0)))]
    pub traces_sample_rate: f32,
    /// Least severe `tracing` level reported as a Sentry issue: `off`, `error`, `warn`, `info`,
    /// `debug` or `trace`.
    #[cfg_attr(feature = "config-schema", config(values))]
    pub capture_level: SentryLevel,
    /// Least severe `tracing` level kept as a breadcrumb — the trail attached to the next issue.
    ///
    /// Records at or above [`Self::capture_level`] become issues instead. Independent of
    /// [`log_level`](super::TelemetryConfig::log_level) on purpose: the layer keeps collecting
    /// the trail however quiet stdout is, so an issue raised from an `ERROR` still arrives with
    /// the round that led to it attached.
    #[cfg_attr(feature = "config-schema", config(values))]
    pub breadcrumb_level: SentryLevel,
    /// How many breadcrumbs one event carries.
    pub max_breadcrumbs: usize,
    /// Attach a stack trace to events that carry none of their own.
    pub attach_stacktraces: bool,
    /// How long process exit waits for queued events to drain.
    ///
    /// The only flush this process performs, and it happens when the guard `main` holds is
    /// dropped. A container stopped with a short grace period loses whatever is still queued
    /// past this window.
    pub shutdown_timeout_secs: u64,
    /// Print the SDK's own diagnostics to stderr. For proving a DSN works, not for running.
    pub debug: bool,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dsn: None,
            environment: None,
            release: None,
            server_name: None,
            sample_rate: 1.0,
            traces_sample_rate: 0.0,
            capture_level: SentryLevel::Error,
            breadcrumb_level: SentryLevel::Info,
            max_breadcrumbs: 100,
            attach_stacktraces: true,
            shutdown_timeout_secs: 2,
            debug: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SentryConfig, SentryLevel};

    /// The block a deployment that has never heard of it gets. Every one of these is a value
    /// that reaches a third party or decides whether anything does, so each is worth pinning
    /// against a careless edit of the `Default` impl above.
    #[test]
    fn the_defaults_report_nothing_anywhere() {
        let config = SentryConfig::default();

        assert!(!config.enabled);
        assert!(config.dsn.is_none());
        assert!(config.environment.is_none());
        assert!(config.release.is_none());
        assert!(config.server_name.is_none());
        assert!((config.sample_rate - 1.0).abs() < f32::EPSILON);
        assert!(config.traces_sample_rate.abs() < f32::EPSILON);
        assert_eq!(config.capture_level, SentryLevel::Error);
        assert_eq!(config.breadcrumb_level, SentryLevel::Info);
    }
}

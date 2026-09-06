//! The severity thresholds this configuration carries, and the one place they leave for
//! `tracing`.
//!
//! One module owns severity, so the spellings an operator may set are written once and read by
//! one thing: `serde`'s own derive. The variant list below *is* the accepted set — there is no
//! parser beside it that would take a spelling the schema does not publish — and the `From`
//! impls at the bottom are where a threshold becomes the `tracing` type the subscriber and the
//! Sentry layer are built from.
//!
//! # The doc comments below are documentation output
//!
//! As everywhere in [`super`]: each variant's comment is what the generated key table and the
//! contract publish as the meaning of that value, so it is written for an operator choosing
//! between them.

use serde::Deserialize;
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;

/// The maximum verbosity one `tracing` sink emits.
///
/// Ordered by verbosity, least first, which is the direction [`Level`] itself orders in: a
/// threshold compares against another the same way here and after the conversion below, so
/// `log_level >= LogLevel::Debug` and `level >= Level::DEBUG` cannot disagree. The order is the
/// declaration order of the variants, so moving one moves the comparison with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// `error` only.
    Error,
    /// `error` and `warn`.
    Warn,
    /// Down to `info`.
    #[default]
    Info,
    /// Down to `debug`.
    Debug,
    /// Everything.
    Trace,
}

/// The `tracing` level a threshold names.
///
/// The whole of the conversion out of this crate's vocabulary and into `tracing`'s, and the
/// reason nothing above the subscriber has to hold a foreign type.
impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Self::ERROR,
            LogLevel::Warn => Self::WARN,
            LogLevel::Info => Self::INFO,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Trace => Self::TRACE,
        }
    }
}

/// The filter a layer is built with.
///
/// Separate from the impl above rather than left to the caller to compose, because
/// `LevelFilter::from` on a [`Level`] and `LevelFilter::from` on an [`Option<Level>`] mean
/// different things — the second treats `None` as "off" — and a layer built through the wrong
/// one is a sink that logs everything or nothing.
impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        Self::from_level(level.into())
    }
}

/// How much of the `tracing` stream one Sentry sink takes.
///
/// Ordered by severity, so a threshold names the *least* severe record it accepts: `warn` means
/// `error` and `warn`. [`Off`](Self::Off) is the variant [`LogLevel`] has no counterpart for —
/// stdout is always written to — which is why the two are separate types rather than one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
#[serde(rename_all = "lowercase")]
pub enum SentryLevel {
    /// Take nothing.
    Off,
    /// `error` only.
    #[default]
    Error,
    /// `error` and `warn`.
    Warn,
    /// Down to `info`.
    Info,
    /// Down to `debug`.
    Debug,
    /// Everything.
    Trace,
}

impl SentryLevel {
    /// The least severe [`Level`] this threshold accepts, or `None` for [`Self::Off`].
    ///
    /// What the layer's own level filter is built from, so a record below the threshold is never
    /// handed to Sentry's layer at all rather than handed over and dropped.
    #[must_use]
    pub fn threshold(self) -> Option<Level> {
        match self {
            Self::Off => None,
            Self::Error => Some(Level::ERROR),
            Self::Warn => Some(Level::WARN),
            Self::Info => Some(Level::INFO),
            Self::Debug => Some(Level::DEBUG),
            Self::Trace => Some(Level::TRACE),
        }
    }

    /// Whether a record at `level` is at least as severe as this threshold.
    ///
    /// [`Level`] orders `ERROR` lowest, so "at least as severe" is `<=`. Inverting it turns
    /// `capture_level = "error"` into "capture everything", which is a bill rather than a
    /// compile error.
    #[must_use]
    pub fn accepts(self, level: Level) -> bool {
        self.threshold().is_some_and(|threshold| level <= threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::{LogLevel, SentryLevel};
    use tracing::Level;
    use tracing_subscriber::filter::LevelFilter;

    /// The default the loader falls back to, and the one the generated table prints.
    #[test]
    fn the_default_log_level_is_info() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }

    /// The direction the type documents, pinned so a reordered variant list fails here rather
    /// than at the one comparison in `main` that decides whether the boot explains its layers.
    #[test]
    fn log_levels_order_from_least_to_most_verbose() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }

    /// The claim the ordering above is only useful for: it is [`Level`]'s own direction, so a
    /// comparison written against either type answers the same question.
    #[test]
    fn the_order_is_the_one_tracing_uses() {
        let levels = [
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];

        for pair in levels.windows(2) {
            let (quieter, louder) = (pair[0], pair[1]);
            assert!(quieter < louder);
            assert!(Level::from(quieter) < Level::from(louder));
        }
    }

    #[test]
    fn every_log_level_converts_to_its_tracing_level() {
        assert_eq!(Level::from(LogLevel::Error), Level::ERROR);
        assert_eq!(Level::from(LogLevel::Warn), Level::WARN);
        assert_eq!(Level::from(LogLevel::Info), Level::INFO);
        assert_eq!(Level::from(LogLevel::Debug), Level::DEBUG);
        assert_eq!(Level::from(LogLevel::Trace), Level::TRACE);
    }

    /// The filter a layer is built with, which is the form the subscriber actually consumes.
    #[test]
    fn every_log_level_converts_to_its_filter() {
        assert_eq!(LevelFilter::from(LogLevel::Error), LevelFilter::ERROR);
        assert_eq!(LevelFilter::from(LogLevel::Warn), LevelFilter::WARN);
        assert_eq!(LevelFilter::from(LogLevel::Info), LevelFilter::INFO);
        assert_eq!(LevelFilter::from(LogLevel::Debug), LevelFilter::DEBUG);
        assert_eq!(LevelFilter::from(LogLevel::Trace), LevelFilter::TRACE);
    }

    /// The inversion that a value assertion cannot catch: every level is "at least as severe as"
    /// something, and getting the comparison the wrong way round turns the default threshold
    /// into "capture everything".
    #[test]
    fn a_threshold_accepts_only_levels_at_least_as_severe() {
        assert!(SentryLevel::Error.accepts(Level::ERROR));
        assert!(!SentryLevel::Error.accepts(Level::WARN));
        assert!(!SentryLevel::Error.accepts(Level::TRACE));

        assert!(SentryLevel::Info.accepts(Level::ERROR));
        assert!(SentryLevel::Info.accepts(Level::WARN));
        assert!(SentryLevel::Info.accepts(Level::INFO));
        assert!(!SentryLevel::Info.accepts(Level::DEBUG));

        for level in [
            Level::ERROR,
            Level::WARN,
            Level::INFO,
            Level::DEBUG,
            Level::TRACE,
        ] {
            assert!(!SentryLevel::Off.accepts(level));
            assert!(SentryLevel::Trace.accepts(level));
        }
    }

    #[test]
    fn off_is_the_one_threshold_with_no_level() {
        assert_eq!(SentryLevel::Off.threshold(), None);
        assert_eq!(SentryLevel::Error.threshold(), Some(Level::ERROR));
        assert_eq!(SentryLevel::Trace.threshold(), Some(Level::TRACE));
    }
}

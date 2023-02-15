use lazy_static::lazy_static;
use prometheus::{register_int_counter_vec, IntCounterVec};

lazy_static! {
    pub static ref FEED_COUNTER: IntCounterVec =
        register_int_counter_vec!("feed_counter", "Number of send feeds", &["feed"])
            .expect("Failed to register feed counter metric");
}

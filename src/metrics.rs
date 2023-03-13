use actix_web::{web, App, HttpServer};
use actix_web_opentelemetry::{PrometheusMetricsHandler, RequestMetricsBuilder, RequestTracing};
use lazy_static::lazy_static;
use opentelemetry::global;
use opentelemetry::metrics::Counter;
use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use opentelemetry_prometheus::PrometheusExporter;
use std::net::SocketAddr;
use tokio::task::JoinHandle;

// TODO: Add system metrics
fn get_meter() -> opentelemetry::metrics::Meter {
    global::meter("")
}

lazy_static! {
    pub static ref FEED_COUNTER: Counter<u64> = get_meter()
        .u64_counter("feed_counter")
        .with_description("Number of send feeds")
        .init();
}

pub struct Metrics {
    address: SocketAddr,
    exporter: PrometheusExporter,
}

impl Metrics {
    fn init_meter() -> PrometheusExporter {
        let controller = controllers::basic(
            processors::factory(
                selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
                aggregation::cumulative_temporality_selector(),
            )
            .with_memory(true),
        )
        .build();

        opentelemetry_prometheus::exporter(controller).init()
    }

    pub fn new(address: SocketAddr) -> Self {
        let exporter = Self::init_meter();
        Self { address, exporter }
    }

    pub async fn run_until_stopped(&self) -> crate::Result<JoinHandle<std::io::Result<()>>> {
        let meter = get_meter();
        let request_metrics = RequestMetricsBuilder::new().build(meter);

        let exporter = self.exporter.clone();
        let server = HttpServer::new(move || {
            App::new()
                .wrap(RequestTracing::new())
                .wrap(request_metrics.clone())
                .route(
                    "/metrics",
                    web::get().to(PrometheusMetricsHandler::new(exporter.clone())),
                )
        })
        .bind(self.address)?
        .run();

        let run = tokio::spawn(async move { server.await });
        Ok(run)
    }
}

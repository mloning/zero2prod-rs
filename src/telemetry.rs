use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Registry};

fn build_tracing_subscriber(name: String, level: String) -> impl Subscriber + Send + Sync {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

fn init_tracing_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // redirect all log's events to our tracing subscriber
    LogTracer::init().expect("Failed to set logger");

    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn configure_tracing() {
    let level = "info".to_string();
    let name = "zero2prod".to_string();
    let subscriber = build_tracing_subscriber(name, level);
    init_tracing_subscriber(subscriber);
}

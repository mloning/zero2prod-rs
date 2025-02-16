use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Registry};

// using generic Sink type constrained in the where clause below
fn build_tracing_subscriber<Sink>(
    name: String,
    level: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
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

pub fn configure_tracing<Sink>(name: String, level: String, sink: Sink)
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let subscriber = build_tracing_subscriber(name, level, sink);
    init_tracing_subscriber(subscriber);
}

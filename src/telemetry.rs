use tracing::{Subscriber, subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

/// Compose multiple layer into a `tracing`'s subscriber
pub fn get_subscriber<S, L, Sink>(name: S, default_level: L, sink: Sink) -> impl Subscriber
where
    S: Into<String>,
    L: AsRef<str>,
    Sink: for<'a> MakeWriter<'a> + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let formatting_layer = BunyanFormattingLayer::new(name.into(), sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as a global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set log redirecting to tracing");
    subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

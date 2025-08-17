use tracing_subscriber::{fmt::format::Pretty, prelude::*};
use tracing_web::{MakeWebConsoleWriter, performance_layer};

pub fn init_logging() {
    console_error_panic_hook::set_once();
    color_eyre::install().unwrap();

    let filter_layer =
        tracing_subscriber::filter::Targets::new().with_default(tracing::Level::INFO);

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers, see note below
        .with_writer(MakeWebConsoleWriter::new()); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .with(filter_layer)
        .init(); // Install these as subscribers to tracing events
}

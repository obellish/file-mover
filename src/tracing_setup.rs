use anyhow::Result;
use tracing::Subscriber;
use tracing_subscriber::{
	fmt::{
		self,
		format::{Format, Pretty},
	},
	prelude::*,
	EnvFilter, Layer,
};

pub async fn setup_tracing() -> Result<()> {
	let log_filter_layer = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new("debug,tokio=error,runtime=error"))?;

	let log_fmt_layer = setup_console();

	tracing_subscriber::registry()
		.with(log_fmt_layer.with_filter(log_filter_layer))
		.try_init()?;
	Ok(())
}

fn setup_console<S>() -> fmt::Layer<S, Pretty, Format<Pretty>>
where
	S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
	fmt::layer()
		.pretty()
		.with_ansi(true)
		.with_thread_ids(false)
		.with_file(false)
		.with_target(false)
		.with_line_number(false)
		.with_thread_names(true)
}

use std::path::Path;

use anyhow::Result;
use tracing::Subscriber;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Layer};

pub async fn setup_tracing(args: &crate::Args) -> Result<()> {
	let log_filter_layer =
		EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("debug"))?;

	let log_fmt_layer = setup_console();

	let registry = tracing_subscriber::registry()
		.with(log_filter_layer)
		.with(log_fmt_layer);

	if let Some(log_file) = args.log_file.as_deref() {
		let log_fs_layer = setup_file(log_file).await?;
		registry.with(log_fs_layer).try_init()?;
	} else {
		registry.try_init()?;
	}

	Ok(())
}

fn setup_console<S>() -> impl Layer<S>
where
	S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
	fmt::layer()
		.pretty()
		.with_ansi(true)
		.with_thread_ids(false)
		.with_file(false)
		.with_thread_names(true)
}

async fn setup_file<P, S>(output_file: P) -> Result<impl Layer<S>>
where
	P: AsRef<Path> + Send,
	S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
	let output_folder = output_file.as_ref();

	let output_log_file = tokio::fs::File::create(output_folder)
		.await?
		.into_std()
		.await;

	let log_fs_layer = fmt::layer()
		.compact()
		.with_ansi(false)
		.with_writer(output_log_file);

	Ok(log_fs_layer)
}

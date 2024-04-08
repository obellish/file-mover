use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use anyhow::Result;
use clap::Parser;
use file_mover::{setup_tracing, zip_files, Args};
use tokio::runtime::Builder;

static THREAD_ID: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
	let args = Args::try_parse()?;

	Builder::new_current_thread()
		.enable_all()
		.thread_name_fn(|| {
			let id = THREAD_ID.fetch_add(1, SeqCst) + 1;
			let output = String::from("file-mover-pool-");
			output + &id.to_string()
		})
		.on_thread_stop(|| {
			THREAD_ID.fetch_sub(1, SeqCst);
		})
		.build()?
		.block_on(run(args))
}

async fn run(args: Args) -> Result<()> {
	setup_tracing(&args).await?;

	zip_files(&args.input_folder, args.output_folder).await?;

	Ok(())
}

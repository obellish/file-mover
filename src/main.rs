use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use anyhow::Result;
use clap::Parser;
use file_mover::{copy_dir_all, setup_tracing, Args, MoveFileError};
use futures::{stream::FuturesUnordered, TryFutureExt};
use tokio::runtime::Builder;
use tracing::{event, Level};

static THREAD_ID: AtomicUsize = AtomicUsize::new(1);

fn main() -> Result<()> {
	let args = Args::try_parse()?;

	Builder::new_multi_thread()
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
		.block_on(catch_error(args));

	Ok(())
}

async fn catch_error(args: Args) {
	if let Err(error) = run(args).await {
		event!(Level::ERROR, ?error);
	}
}

async fn run(args: Args) -> Result<()> {
	setup_tracing(&args).await?;

	let output: FuturesUnordered<_> = copy_dir_all(&args.input_folder, &args.output_folder).await?;

	futures::future::try_join_all(output)
		.map_ok(|values| values.into_iter().collect::<Result<(), MoveFileError>>())
		.await??;

	// copy_dir_all_old(&args.input_folder, args.output_folder).await?;

	Ok(())
}

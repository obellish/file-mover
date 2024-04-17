use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use anyhow::Result;
use clap::Parser;
use file_mover::{copy_dir_all, setup_tracing, Args};
use futures::{stream::FuturesUnordered, TryStreamExt as _};
use tokio::{
	runtime::Builder,
	signal::windows::{ctrl_break, ctrl_c},
};
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
		.block_on(run(args))?;

	Ok(())
}

async fn run(args: Args) -> Result<()> {
	setup_tracing().await?;

	match copy_dir_all::<FuturesUnordered<_>>(&args.input_folder, &args.output_folder, args.remove)
		.await
	{
		Ok(mut output) => {
			let mut sig_c = ctrl_c()?;
			let mut sig_break = ctrl_break()?;

			loop {
				tokio::select! {
					_ = sig_c.recv() => {
						event!(Level::INFO, "received CTRL+C");
						break;
					}
					_ = sig_break.recv() => {
						event!(Level::INFO, "received CTRL+BREAK");
						break;
					}
					value = output.try_next() => {
						match value {
							Ok(Some(Ok(()))) => {},
							Ok(Some(Err(e))) => {
								event!(Level::ERROR, ?e);
								break;
							}
							Ok(None) => {
								event!(Level::INFO, "finished copying files");
								break;
							}
							Err(e) => {
								event!(Level::ERROR, ?e);
								break;
							}
						}
					}
				}
			}
		}
		Err(error) => event!(Level::ERROR, ?error),
	}

	Ok(())
}

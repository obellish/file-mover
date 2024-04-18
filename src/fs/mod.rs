mod error;

use std::{
	fmt::Debug,
	path::{Path, PathBuf},
};

use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt as _, TryStreamExt as _};
use tokio::{fs, task::JoinHandle};
use tokio_stream::wrappers::ReadDirStream;
use tracing::{event, span, Instrument, Level};

pub use self::error::*;

pub fn copy_dir_all<'a, I>(
	src: impl AsRef<Path> + Debug + Send + 'a,
	dst: impl AsRef<Path> + Debug + Send + 'a,
	delete: bool,
) -> BoxFuture<'a, Result<I, CopyDirError>>
where
	I: Extend<JoinHandle<Result<(), MoveFileError>>> + Default + Send,
{
	let src = src.as_ref().to_path_buf();
	let dst = dst.as_ref().to_path_buf();
	let src_display = src.display().to_string();
	let dst_display = dst.display().to_string();
	let span = span!(
		Level::INFO,
		"copying_folder",
		src = src_display,
		dst = dst_display
	);
	async move {
		let mut output = I::default();
		event!(Level::DEBUG, "copying folder");
		fs::create_dir_all(&dst).await?;
		let mut stream = ReadDirStream::new(fs::read_dir(src).await?);
		while let Some(entry) = stream.try_next().await? {
			let ty = entry.file_type().await?;
			if ty.is_file() {
				let dst = dst.clone();
				output.extend(std::iter::once(tokio::spawn(async move {
					move_file(entry.path(), dst.join(entry.file_name()), delete).await
				})));
			} else if ty.is_dir() {
				let inner_output: FuturesUnordered<_> =
					copy_dir_all(entry.path(), dst.join(entry.file_name()), delete).await?;
				output.extend(inner_output);
			}
		}

		Ok(output)
	}
	.instrument(span)
	.boxed()
}

// #[tracing::instrument(skip_all)]
async fn move_file(from: PathBuf, to: PathBuf, delete: bool) -> Result<(), MoveFileError> {
	event!(Level::TRACE, ?from, ?to, "copying file");
	fs::copy(&from, to)
		.await
		.map_err(|source| MoveFileError::Copy {
			source,
			path: from.clone(),
		})?;
	if delete {
		event!(Level::TRACE, ?from, "deleting file");
		fs::remove_file(&from)
			.await
			.map_err(|source| MoveFileError::Delete { source, path: from })?;
	}
	Ok(())
}

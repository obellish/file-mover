mod error;

use std::{
	fmt::Debug,
	path::{Path, PathBuf},
};

use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt as _, TryStreamExt as _};
use tokio::{fs, task::JoinHandle};
use tokio_stream::wrappers::ReadDirStream;
use tracing::{event, Level};

pub use self::error::*;

pub fn copy_dir_all<'a, I>(
	src: impl AsRef<Path> + Debug + Send + 'a,
	dst: impl AsRef<Path> + Debug + Send + 'a,
) -> BoxFuture<'a, Result<I, CopyDirError>>
where
	I: Extend<JoinHandle<Result<(), MoveFileError>>> + Default + Send,
{
	async move {
		let mut output = I::default();
		let src = src.as_ref();
		let dst = dst.as_ref();
		event!(Level::DEBUG, ?src, ?dst, "copying folder");
		fs::create_dir_all(&dst).await?;
		let mut stream = ReadDirStream::new(fs::read_dir(src).await?);

		while let Some(entry) = stream.try_next().await? {
			let metadata = entry.metadata().await?;
			if metadata.is_file() {
				let dst = dst.to_path_buf();
				output.extend(std::iter::once(tokio::spawn(async move {
					move_file(entry.path(), dst).await
				})));
			} else if metadata.is_dir() {
				let inner_output =
					copy_dir_all::<FuturesUnordered<_>>(entry.path(), dst.join(entry.file_name()))
						.await?;
				output.extend(inner_output);
			}
		}

		Ok(output)
	}
	.boxed()
}

async fn move_file(from: PathBuf, to: PathBuf) -> Result<(), MoveFileError> {
	event!(Level::TRACE, ?from, ?to, "copying file");
	fs::copy(&from, to)
		.await
		.map_err(|source| MoveFileError::Copy {
			source,
			path: from.clone(),
		})?;
	event!(Level::TRACE, ?from, "deleting file");
	fs::remove_file(&from)
		.await
		.map_err(|source| MoveFileError::Delete { source, path: from })
}

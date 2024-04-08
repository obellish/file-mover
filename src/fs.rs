use std::path::Path;

use futures::{future::BoxFuture, FutureExt, TryFutureExt, TryStreamExt as _};
use tokio::fs;
use tokio_stream::wrappers::ReadDirStream;
use tracing::{event, Level};

pub fn copy_dir_all<'a, I, O>(src: I, dst: O) -> BoxFuture<'a, std::io::Result<()>>
where
	I: AsRef<Path> + Send + 'a,
	O: AsRef<Path> + Send + Sync + 'a,
{
	async move {
		let src = src.as_ref();
		let dst = dst.as_ref();
		event!(Level::DEBUG, ?src, ?dst, "copying folder");
		fs::create_dir_all(&dst).await?;
		let mut stream = ReadDirStream::new(fs::read_dir(src).await?);

		let mut futures = Vec::new();

		while let Some(entry) = stream.try_next().await? {
			let ty = entry.file_type().await?;
			if ty.is_dir() {
				copy_dir_all(entry.path(), dst.join(entry.file_name())).await?;
			} else {
				let dst = dst.to_path_buf();
				futures.push(tokio::spawn(async move {
					event!(Level::TRACE, "copying file {}", entry.path().display());
					fs::copy(entry.path(), dst.join(entry.file_name())).await?;
					Ok::<(), std::io::Error>(())
				}));
			}
		}

		futures::future::try_join_all(futures)
			.map_ok(|values| values.into_iter().collect::<std::io::Result<()>>())
			.await??;

		Ok(())
	}
	.boxed()
}

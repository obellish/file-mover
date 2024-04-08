use std::{
	path::{Path, PathBuf},
	pin::pin,
};

use anyhow::Result;
use async_zip::{
	tokio::write::ZipFileWriter, Compression, StringEncoding, ZipEntryBuilder, ZipString,
};
use futures::{stream::FuturesUnordered, StreamExt, TryStreamExt as _};
use tokio::{fs::File, io::AsyncReadExt as _};
use tracing::{event, Level};

use crate::visit;

pub async fn zip_files<I, O>(input_folder: I, output_file: O) -> Result<()>
where
	I: AsRef<Path> + Send,
	O: AsRef<Path> + Send,
{
	let input_folder = input_folder.as_ref();
	// let mut stream = pin!(visit(&input_folder));

	let output_file = File::create(output_file).await?;

	let mut zip_writer = ZipFileWriter::with_tokio(output_file);

	let path_to_remove = input_folder.iter().collect::<Vec<_>>();

	let mut stream = visit(&input_folder)
		.map_ok(|entry| async move {
			let data = {
				let mut file = File::open(entry.path()).await.unwrap();

				let mut buffer = Vec::new();

				file.read_to_end(&mut buffer).await.unwrap();

				buffer
			};

			Ok::<(PathBuf, Vec<u8>), anyhow::Error>((entry.path(), data))

			// futures::future::ok((entry.path(), data))
		})
		.try_collect::<FuturesUnordered<_>>()
		.await?;

	while let Some(res) = stream.next().await {
		let (path, data) = res?;
	}

	// while let Some(item) = stream.try_next().await? {
	// 	event!(Level::TRACE, "zipping file {}", item.path().display());
	// 	let file_name = ZipString::new(
	// 		{
	// 			let path = item.path();

	// 			path.iter()
	// 				.filter(|item| !path_to_remove.contains(item))
	// 				.collect::<PathBuf>()
	// 				.as_os_str()
	// 				.as_encoded_bytes()
	// 				.to_vec()
	// 		},
	// 		StringEncoding::Raw,
	// 	);

	// 	let entry = ZipEntryBuilder::new(file_name, Compression::Stored);

	// 	let data = {
	// 		let mut file = File::open(item.path()).await?;

	// 		let mut buffer = Vec::with_capacity(file.metadata().await?.len() as usize);

	// 		file.read_to_end(&mut buffer).await?;

	// 		buffer
	// 	};

	// 	zip_writer.write_entry_whole(entry, &data).await?;
	// }

	zip_writer.close().await?;

	Ok(())
}

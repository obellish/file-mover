use std::{
	path::{Path, PathBuf},
	pin::pin,
};

use anyhow::Result;
use async_zip::{
	tokio::write::ZipFileWriter, Compression, StringEncoding, ZipEntryBuilder, ZipString,
};
use futures::TryStreamExt as _;
use tokio::{
	fs::File,
	io::AsyncReadExt as _,
	signal::windows::{ctrl_break, ctrl_c, ctrl_close, ctrl_logoff, ctrl_shutdown},
};
use tracing::{event, Level};

use crate::visit;

pub async fn zip_files<I, O>(input_folder: I, output_file: O) -> Result<()>
where
	I: AsRef<Path> + Send,
	O: AsRef<Path> + Send,
{
	let input_folder = input_folder.as_ref();

	let output_file = File::create(output_file).await?;

	let mut zip_writer = ZipFileWriter::with_tokio(output_file);

	let path_to_remove = input_folder.iter().collect::<Vec<_>>();

	let mut stream = pin!(visit(&input_folder));

	let mut ctrl_c_handler = ctrl_c()?;
	let mut ctrl_break_handler = ctrl_break()?;
	let mut ctrl_close_handler = ctrl_close()?;
	let mut ctrl_logoff_handler = ctrl_logoff()?;
	let mut ctrl_shutdown_handler = ctrl_shutdown()?;

	loop {
		tokio::select! {
				_ = ctrl_c_handler.recv() => {
					event!(Level::INFO, "received CTRL+C");
					break;
				},
				_ = ctrl_break_handler.recv() => {
					event!(Level::INFO, "received CTRL+BREAK");
					break;
				},
				_ = ctrl_close_handler.recv() => {
					event!(Level::INFO, "received CTRL+CLOSE");
					break;
				}
				_ = ctrl_logoff_handler.recv() => {
					event!(Level::INFO, "received CTRL+LOGOFF");
					break;
				},
				_ = ctrl_shutdown_handler.recv() => {
					event!(Level::INFO, "received CTRL+SHUTDOWN");
					break;
				}
				Ok(Some(entry)) = stream.try_next() => {
					let path = entry.path();
					event!(Level::TRACE, ?path, "zipping file");
					let filename = ZipString::new(
						path.iter()
							.filter(|item| !path_to_remove.contains(item))
							.collect::<PathBuf>()
							.as_os_str()
							.as_encoded_bytes()
							.to_vec(),
						StringEncoding::Raw
					);

					let entry = ZipEntryBuilder::new(filename, Compression::Lzma);

					let data = {
						let mut file = File::open(path).await?;

						let mut buffer = Vec::new();

						file.read_to_end(&mut buffer).await?;

						buffer
					};

					zip_writer.write_entry_whole(entry, &data).await?;
				}
		}
	}

	zip_writer.close().await?;

	Ok(())
}

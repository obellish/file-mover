mod fs;
mod tracing_setup;
mod util;
mod zip;

use std::path::PathBuf;

use clap::Parser;

pub use self::{fs::*, tracing_setup::setup_tracing, util::*, zip::*};

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// The input folder to move from.
	#[arg(short, long, value_name = "DIRECTORY")]
	pub input_folder: PathBuf,
	#[arg(short, long, value_name = "DIRECTORY")]
	pub output_folder: PathBuf,
	/// Folder for logging.
	#[arg(long, value_name = "DIRECTORY")]
	pub log_file: Option<PathBuf>,
}

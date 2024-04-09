use std::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	io::Error as IoError,
	path::PathBuf,
};

#[derive(Debug)]
pub enum CopyDirError {
	Io(IoError),
	MoveFile(MoveFileError),
}

impl Display for CopyDirError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Io(e) => {
				f.write_str("an error occurred copying a directory: ")?;
				Display::fmt(e, f)
			}
			Self::MoveFile(e) => Display::fmt(e, f),
		}
	}
}

impl From<IoError> for CopyDirError {
	fn from(value: IoError) -> Self {
		Self::Io(value)
	}
}

impl From<MoveFileError> for CopyDirError {
	fn from(value: MoveFileError) -> Self {
		Self::MoveFile(value)
	}
}

impl StdError for CopyDirError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Io(e) => Some(e),
			Self::MoveFile(e) => Some(e),
		}
	}
}

#[derive(Debug)]
pub enum MoveFileError {
	Copy { source: IoError, path: PathBuf },
	Delete { source: IoError, path: PathBuf },
}

impl Display for MoveFileError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::Copy { source, path } => {
				// f.write_str("an error occurred moving a file: ")?;
				f.write_str("an error occurred moving file ")?;
				Display::fmt(&path.display(), f)?;
				f.write_str(": ")?;
				Display::fmt(source, f)
			}
			Self::Delete { source, path } => {
				f.write_str("an error occurred deleting file ")?;
				Display::fmt(&path.display(), f)?;
				f.write_str(": ")?;
				Display::fmt(source, f)
			}
		}
	}
}

impl StdError for MoveFileError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		match self {
			Self::Copy { source, .. } | Self::Delete { source, .. } => Some(source),
		}
	}
}

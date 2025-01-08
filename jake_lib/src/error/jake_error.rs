use kstring::KString;
use relative_path::RelativePathBuf;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum JakeError {
	LayoutNotFound(KString),
	FileNotUtf8(RelativePathBuf),
	UnexpectedFilePath(PathBuf),
	Misc(&'static str),
}

impl std::error::Error for JakeError {}

impl std::fmt::Display for JakeError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			JakeError::LayoutNotFound(path) => write!(f, "Layout not found: '{path}'"),
			JakeError::FileNotUtf8(path) => write!(f, "File is not valid UTF-8: '{path}'"),
			JakeError::UnexpectedFilePath(path) => write!(f, "BUG: File not in expected directory: '{}'", path.display()),
			JakeError::Misc(e) => write!(f, "BUG: Unknown error: '{e}'"),
		}
	}
}

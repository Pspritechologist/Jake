use kstring::KString;
use relative_path::RelativePathBuf;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum HydeError {
	LayoutNotFound(KString),
	FileNotUtf8(RelativePathBuf),
	UnexpectedFilePath(PathBuf),
	Misc(&'static str),
}

impl std::error::Error for HydeError {}

impl std::fmt::Display for HydeError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			HydeError::LayoutNotFound(path) => write!(f, "Layout not found: '{path}'"),
			HydeError::FileNotUtf8(path) => write!(f, "File is not valid UTF-8: '{path}'"),
			HydeError::UnexpectedFilePath(path) => write!(f, "BUG: File not in expected directory: '{}'", path.display()),
			HydeError::Misc(e) => write!(f, "BUG: Unknown error: '{e}'"),
		}
	}
}

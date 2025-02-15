use crate::frontmatter::FrontMatter;
use relative_path::RelativePathBuf;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct JakeFileT1 {
	pub source: RelativePathBuf,
	pub front_matter: FrontMatter,
	pub content: FileContent<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JakeFileT2 {
	pub source: FileSource,
	pub output: RelativePathBuf,
	pub front_matter: FrontMatter,
	pub content: FileContent<String>,
	pub to_write: bool,
	#[serde(skip)]
	pub post_processor: Vec<mlua::Function>,
}

#[derive()]
pub struct JakeFileT3 {
	pub source: FileSource,
	pub output: RelativePathBuf,
	pub front_matter: FrontMatter,
	pub template: FileContent<liquid::Template>,
	pub to_write: bool,
	pub post_processor: Vec<mlua::Function>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct JakeConfig {
	pub project_dir: PathBuf,
	pub output_dir: PathBuf,
	pub source_dir: PathBuf,
	pub plugins_dir: PathBuf,
	pub layout_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "T: Clone + serde::Serialize + for<'a> serde::Deserialize<'a>")]
#[serde(into = "Option<T>", from = "Option<T>")]
pub enum FileContent<T> {
	Utf8(T),
	Binary,
}

impl<T: Default> Default for FileContent<T> {
	fn default() -> Self {
		Self::Utf8(T::default())
	}
}

impl<T, O: Into<T>> From<Option<O>> for FileContent<T> {
	fn from(opt: Option<O>) -> Self {
		match opt {
			Some(s) => Self::Utf8(s.into()),
			None => Self::Binary,
		}
	}
}

impl<T> From<FileContent<T>> for Option<T> {
	fn from(fc: FileContent<T>) -> Option<T> {
		match fc {
			FileContent::Utf8(s) => Some(s),
			FileContent::Binary => None,
		}
	}
}

impl<T> FileContent<T> {
	pub fn as_option(&self) -> Option<&T> {
		match self {
			Self::Utf8(s) => Some(s),
			Self::Binary => None,
		}
	}

	pub fn into_option(self) -> Option<T> {
		match self {
			Self::Utf8(s) => Some(s),
			Self::Binary => None,
		}
	}

	pub fn is_binary(&self) -> bool {
		matches!(self, Self::Binary)
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FileSource<T = RelativePathBuf> {
	Src(T),
	Lua,
}

impl<T> From<Option<T>> for FileSource<T> {
	fn from(opt: Option<T>) -> Self {
		match opt {
			Some(p) => Self::Src(p),
			None => Self::Lua,
		}
	}
}

impl<T> From<FileSource<T>> for Option<T> {
	fn from(fs: FileSource<T>) -> Option<T> {
		match fs {
			FileSource::Src(p) => Some(p),
			FileSource::Lua => None,
		}
	}
}

impl<T> FileSource<T> {
	pub fn as_option(&self) -> Option<&T> {
		match self {
			Self::Src(p) => Some(p),
			Self::Lua => None,
		}
	}

	pub fn into_option(self) -> Option<T> {
		match self {
			Self::Src(p) => Some(p),
			Self::Lua => None,
		}
	}

	pub fn is_lua(&self) -> bool {
		matches!(self, Self::Lua)
	}
}

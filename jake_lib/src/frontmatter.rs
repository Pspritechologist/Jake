use crate::error::{Error, ResultExtensions};
use kstring::KString;
use std::{collections::BTreeMap, fs::File, io::{BufRead, BufReader, ErrorKind, Read, Seek}, path::Path};

pub type FrontMatter = BTreeMap<KString, serde_json::Value>;

/// Reads the frontmatter of a file, if present.
/// 
/// # Returns
/// - `Ok(Some(frontmatter))` if the file has frontmatter.
/// - `Ok(None)` if the file does not have frontmatter.
/// - `Err(e)` if an I/O error occurs.
pub fn file_frontmatter(path: impl AsRef<Path>) -> Result<Option<FrontMatter>, Error> {
	let mut file = BufReader::new(File::open(&path)?);
	
	parse_frontmatter(&mut file, path.as_ref())
}

/// Reads the content of a file, skipping the frontmatter if present.
/// 
/// # Returns
/// - `Ok(Some(content))` if the file is valid UTF-8.
/// - `Ok(None)` if the file is not valid UTF-8.
/// - `Err(e)` if an I/O error occurs.
pub fn file_content(path: impl AsRef<Path>) -> Result<Option<String>, Error> {
	let mut file = BufReader::new(File::open(path)?);
	let mut buf = String::new();

	{
		let mut lines = (&mut file).lines();

		match lines.next().transpose() {
			Ok(Some(line)) => {
				if is_frontmatter_delimiter(line) {
					lines.take_while(|line|
						!line.as_ref().map(is_frontmatter_delimiter).unwrap_or(true)
					).for_each(drop);
				} else {
					file.seek(std::io::SeekFrom::Start(0))?;
				}
			},
			Ok(None) => (),
			Err(e) if e.kind() == std::io::ErrorKind::InvalidData => return Ok(None),
			Err(e) => return Err(e)?,
		}
	}
	
	if !read_utf8_to_string(file, &mut buf)? {
		return Ok(None);
	}

	Ok(Some(buf))
}

/// Reads the content of a file, separating the frontmatter if present.
/// 
/// # Returns
/// - `Ok(Some((frontmatter, content)))` if the file is valid UTF-8.
///   - `frontmatter` is `None` if the file does not have frontmatter.
/// - `Ok(None)` if the file is not valid UTF-8.
/// - `Err(e)` if an I/O error occurs.
pub fn file_frontmatter_content(path: impl AsRef<Path>) -> Result<Option<(Option<FrontMatter>, String)>, Error> {
	let mut file = BufReader::new(File::open(&path)?);
	let mut buf = String::new();

	let frontmatter = parse_frontmatter(&mut file, path.as_ref())?;

	if frontmatter.is_none() {
		file.seek(std::io::SeekFrom::Start(0))?;
	}

	if !read_utf8_to_string(file, &mut buf)? {
		return Ok(None);
	}

	Ok(Some((frontmatter, buf)))
}

pub fn combine_frontmatters<const S: usize>(objs: [std::borrow::Cow<liquid::Object>; S]) -> liquid::Object {
	objs.into_iter().flat_map(|o| o.into_owned()).collect()
}

/// Reads the content of a reader, skipping the frontmatter if present.  
/// Does not return Err if the reader is not valid UTF-8.
/// 
/// # Returns
/// - `Ok(true)` if the reader is valid UTF-8 (`buf` was written to).
/// - `Ok(false)` if the reader is not valid UTF-8 (`buf` was not written to).
/// - `Err(e)` if a non-UTF-8 I/O error occurs.
fn read_utf8_to_string<R: Read>(reader: R, buf: &mut String) -> Result<bool, Error> {
	let mut reader = BufReader::new(reader);

	match reader.read_to_string(buf) {
		Ok(_) => Ok(true),
		Err(e) if e.kind() == ErrorKind::InvalidData => Ok(false),
		Err(e) => Err(e)?,
	}
}

/// Parses the frontmatter of a file.
/// 
/// # Returns
/// - `Ok(Some(frontmatter))` if the file has frontmatter.
/// - `Ok(None)` if the file does not have frontmatter or isn't valid UTF-8.
/// - `Err(e)` if an I/O error occurs.
fn parse_frontmatter(buf: &mut BufReader<File>, path: &Path) -> Result<Option<FrontMatter>, Error> {
	let mut lines = buf.lines();

	match lines.next().transpose() {
		Ok(Some(line)) => if !is_frontmatter_delimiter(line) {
			return Ok(None);
		}
		Ok(None) => return Ok(None),
		Err(e) if e.kind() == ErrorKind::InvalidData => return Ok(None),
		Err(e) => return Err(e)?,
	}

	let yaml: String = lines
		.take_while(|line| !line.as_ref().map(is_frontmatter_delimiter).unwrap_or(true))
		.intersperse_with(|| Ok(String::from('\n')))
		.try_collect()?;

	Ok(Some(serde_yaml::from_str(&yaml).into_error_result_with(
		|| path.strip_prefix(&crate::config().project_dir).expect("File not in project dir").to_string_lossy()
	)?))
}

fn is_frontmatter_delimiter(line: impl AsRef<str>) -> bool {
	line.as_ref().trim_end() == "---"
}

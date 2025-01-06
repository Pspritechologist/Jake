use crate::error::Error;
use std::{fs::File, io::{BufRead, BufReader, Read, Seek}, path::Path};

pub type FrontMatter = serde_json::Map<String, serde_json::Value>;

pub fn file_frontmatter(path: impl AsRef<Path>) -> Result<Option<FrontMatter>, Error> {
	let mut file = BufReader::new(File::open(path)?);
	
	parse_frontmatter(&mut file)
}

pub fn file_content(path: impl AsRef<Path>) -> Result<String, Error> {
	let mut file = BufReader::new(File::open(path)?);
	let mut buf = String::new();

	{
		let mut lines = (&mut file).lines();

		if let Some(line) = lines.next().transpose()? {
			if is_frontmatter_delimiter(line) {
				lines.take_while(|line| !line.as_ref().map(is_frontmatter_delimiter).unwrap_or(true))
					.for_each(drop);
			} else {
				file.seek(std::io::SeekFrom::Start(0))?;
			}
		}
	}
	
	file.read_to_string(&mut buf)?;

	Ok(buf)
}

pub fn file_frontmatter_content(path: impl AsRef<Path>) -> Result<(Option<FrontMatter>, String), Error> {
	let mut file = BufReader::new(File::open(path)?);
	let mut buf = String::new();

	let frontmatter = parse_frontmatter(&mut file)?;

	if frontmatter.is_none() {
		file.seek(std::io::SeekFrom::Start(0))?;
	}

	file.read_to_string(&mut buf)?;

	Ok((frontmatter, buf))
}

fn parse_frontmatter(buf: &mut BufReader<File>) -> Result<Option<FrontMatter>, Error> {
	let mut lines = buf.lines();

	if let Some(line) = lines.next().transpose()? {
		if is_frontmatter_delimiter(line) {
			return Ok(None);
		}
	}

	let yaml: String = lines
		.take_while(|line| !line.as_ref().map(is_frontmatter_delimiter).unwrap_or(true))
		.intersperse_with(|| Ok(String::from('\n')))
		.try_collect()?;

	Ok(serde_yaml::from_str(&yaml)?)
}

fn is_frontmatter_delimiter(line: impl AsRef<str>) -> bool {
	line.as_ref().trim_end() == "---"
}

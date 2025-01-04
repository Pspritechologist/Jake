use crate::error::Error;
use std::{fs::File, io::{BufRead, BufReader}, path::Path};

pub type FrontMatter = serde_json::Map<String, serde_json::Value>;

pub fn parse_frontmatter(path: impl AsRef<Path>) -> Result<Option<FrontMatter>, Error> {
	let file = BufReader::new(File::open(path)?);
	let mut lines = file.lines();

	if let Some(Ok(line)) = lines.next() {
		if line.trim() != "---" {
			return Ok(Default::default());
		}
	}

	let yaml: String = lines
		.take_while(|line| line.as_ref().map(|l| l.trim_end() != "---").unwrap_or(true))
		.intersperse_with(|| Ok(String::from('\n')))
		.try_collect()?;

	Ok(serde_yaml::from_str(&yaml)?)
}

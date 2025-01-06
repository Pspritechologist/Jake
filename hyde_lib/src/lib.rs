#![feature(try_blocks)]
#![feature(let_chains)]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]
#![feature(extend_one)]
#![feature(iter_intersperse)]

#![warn(
	clippy::todo,
	clippy::unimplemented,
)]

#![deny(clippy::unwrap_used,
	// clippy::expect_used,
	clippy::panic,
	clippy::panic_in_result_fn,
	clippy::panicking_overflow_checks,
)]

use std::{collections::{BTreeMap, HashMap}, path::PathBuf};

use error::Error;
use frontmatter::FrontMatter;
use liquid::ValueView;
use lua::general_api::file::HydeFile;
use relative_path::RelativePathBuf;

pub mod error;
mod lua;
mod frontmatter;

#[derive(Debug, Clone)]
pub struct HydeConfig {
	pub project_dir: PathBuf,
	pub output_dir: PathBuf,
	pub source_dir: PathBuf,
	pub plugins_dir: PathBuf,
	pub layout_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct HydeProject {
	pub files: Vec<HydeFile>,
	pub layouts: HashMap<String, String>,
}

pub fn process_project(config: &HydeConfig) -> Result<(), Error> {
	let lua = unsafe { mlua::Lua::unsafe_new() };
	let lua::LuaResult { tags, converters, filters, mut files } = lua::setup_lua_state(&lua, config, collect_src(config)?)?;

	let mut liquid_builder = liquid::ParserBuilder::with_stdlib();

	liquid_builder = liquid_builder.block(lua::block::LuaBlock { lua: lua.clone() });

	for (tag, func) in tags {
		liquid_builder = liquid_builder.tag(lua::tag::LuaTag { tag, func, lua: lua.clone() });
	}

	for (filter, func) in filters {
		liquid_builder = liquid_builder.filter(lua::filter::Lua { filter, func, lua: lua.clone() });
	}

	files.retain(|f| f.to_write);

	let liquid = liquid_builder.build()?;

	for file in files {
		// process_file(config, &liquid, file)?;
	}

	Ok(())
}

fn collect_src(config: &HydeConfig) -> Result<Vec<HydeFile>, Error> {
	let HydeConfig { source_dir, output_dir, plugins_dir, layout_dir, .. } = config;

	std::fs::create_dir_all(output_dir)?;
	std::fs::create_dir_all(plugins_dir)?;
	std::fs::create_dir_all(layout_dir)?;

	let mut files = Vec::with_capacity(16); // Better than starting at 0.
	
	let dir = walkdir::WalkDir::new(source_dir)
		.into_iter()
		.filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
		.filter_map(Result::ok);

	const DEFAULT_FRONTMATTER_FILE: &str = ".hyde.yml";

	let mut frontmatter_glob: Vec<(globset::GlobMatcher, FrontMatter)> = Default::default();

	for entry in dir {
		if entry.file_type().is_dir() {
			let conf_path = entry.path().join(DEFAULT_FRONTMATTER_FILE);
			if let Some(config) = conf_path.exists()
				.then(|| std::fs::File::open(conf_path))
				.transpose()?
				.map(serde_yaml::from_reader::<_, HashMap<String, FrontMatter>>)
				.transpose()? {

				frontmatter_glob.extend_reserve(config.len());
				for (glob, frontmatter) in config {
					frontmatter_glob.push((globset::GlobBuilder::new(&format!("{}/{}", entry.path().to_string_lossy(), glob))
						.backslash_escape(true)
						.empty_alternates(true)
						.build()?
						.compile_matcher(), frontmatter));
				}
			}

			continue;
		}

		let rel_path = entry.path()
			.strip_prefix(source_dir)
			.ok()
			.and_then(|p| RelativePathBuf::from_path(p).ok())
			.expect("File not in source directory");

		let mut front_matter = FrontMatter::default();

		for (glob, fm) in &frontmatter_glob {
			if glob.is_match(entry.path()) {
				front_matter.extend(fm.clone());
			}
		}

		if let Some(fm) = frontmatter::file_frontmatter(entry.path())? {
			front_matter.extend(fm);
		}

		files.push(HydeFile {
			to_write: false,
			source: Some(rel_path.clone()),
			output: rel_path,
			front_matter,
			content: String::new(),
		});
	}

	Ok(files)
}

// pub fn process_file(config: &HydeConfig, liquid: &liquid::Parser, file: HydeFile) -> Result<(), Error> {
// 	let HydeConfig { source_dir, output_dir, .. } = config;

// 	let output = output_dir.join(file.output);

// 	std::fs::create_dir_all(output.parent().expect("File has no parent"))?;

// 	let content = file.source.map(|src| std::fs::read_to_string(src));
// 	let content = parse_content(config, liquid, content, liquid::Object::new());

// 	fn markdown_ops() -> markdown::Options {
// 		markdown::Options {
// 			compile: markdown::CompileOptions {
// 				allow_dangerous_html: true,
// 				allow_dangerous_protocol: true,
// 				gfm_tagfilter: false,
// 				..markdown::CompileOptions::gfm()
// 			},
// 			parse: markdown::ParseOptions::gfm()
// 		}
// 	}

// 	match file.path().extension().and_then(std::ffi::OsStr::to_str) {
// 		Some("md") => std::fs::write(output.with_extension("html"), markdown::to_html_with_options(&content?, &markdown_ops()).expect("Markdown doesn't panic"))?,
// 		Some("scss") => std::fs::write(output.with_extension("css"), grass::from_string(content?.as_str(), &grass::Options::default())?)?,
// 		_ => std::fs::write(output, content.map(|c| c.as_bytes().to_vec()).or_else(|_| std::fs::read(input))?)?,
// 	}

// 	Ok(())
// }

fn parse_content(config: &HydeConfig, liquid: &liquid::Parser, content: String, mut frontmatter: liquid::Object) -> Result<String, Error> {
	let mut content = liquid.parse(&content)?.render(&frontmatter)?;

	if let Some(layout) = frontmatter.get("layout") {
		let layout = config.layout_dir.join(layout.to_kstr());

		let layout = std::fs::read_to_string(layout.with_extension("html"))?;

		frontmatter.remove("layout");
		frontmatter.insert("content".into(), liquid_core::Value::scalar(content));

		content = parse_content(config, liquid, layout, frontmatter)?;
	}

	return Ok(content);
}

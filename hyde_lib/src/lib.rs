#![feature(try_blocks)]
#![feature(let_chains)]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

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

use std::path::PathBuf;

use error::Error;
use liquid::ValueView;
use lua::general_api::file::HydeFile;
use relative_path::RelativePathBuf;

pub mod error;
mod lua;

#[derive(Debug, Clone)]
pub struct HydeConfig {
	pub project_dir: PathBuf,
	pub output_dir: PathBuf,
	pub source_dir: PathBuf,
	pub plugins_dir: PathBuf,
	pub layout_dir: PathBuf,
}

pub fn process_dir(config: &HydeConfig) -> Result<(), Error> {
	let files = collect_src(config)?;

	for file in &files {
		println!("{file:#?}");
	}
	println!();
	println!();

	let lua = unsafe { mlua::Lua::unsafe_new() };

	let lua::LuaResult { tags, converters, filters, files } = lua::setup_lua_state(&lua, config, files)?;

	let mut liquid_builder = liquid::ParserBuilder::with_stdlib();

	for (tag, func) in tags {
		liquid_builder = liquid_builder.tag(lua::tag::LuaTag { tag, func, lua: lua.clone() });
	}

	for (filter, func) in filters {
		liquid_builder = liquid_builder.filter(lua::filter::Lua { filter, func, lua: lua.clone() });
	}

	liquid_builder = liquid_builder.block(lua::block::LuaBlock { lua: lua.clone() });

	// We leak our Lua state here so it remains valid for the rest of the program's lifetime.
	// Box::leak(Box::new(lua));

	let liquid = liquid_builder.build()?;

	// walkdir::WalkDir::new(&config.source_dir)
	// 	.into_iter()
	// 	.filter_map(Result::<_, _>::ok)
	// 	.filter(|e| e.file_type().is_file())
	// 	.try_for_each(|e| process_file(config, &liquid, e))?;
	// 	// .map(|e| process_file(config, &liquid, e))
	// 	// .collect::<Result<_, _>>()?;

	for file in files {
		// process_file(config, &liquid, file)?;
		// Print all the data for each file in a pretty way.
		println!("{file:#?}");
	}

	Ok(())
}

fn collect_src(config: &HydeConfig) -> Result<Vec<HydeFile>, Error> {
	let HydeConfig { source_dir, output_dir, plugins_dir, layout_dir, .. } = config;

	std::fs::create_dir_all(output_dir)?;
	std::fs::create_dir_all(plugins_dir)?;
	std::fs::create_dir_all(layout_dir)?;
	
	let files: Vec<_> = walkdir::WalkDir::new(source_dir)
		.into_iter()
		.filter_map(Result::<_, _>::ok)
		.filter(|e| e.file_type().is_file())
		.map(|entry| {
		
		let path = entry.path()
			.strip_prefix(source_dir)
			.ok()
			.and_then(|p| RelativePathBuf::from_path(p).ok())
			.expect("File not in source directory");
		
		HydeFile {
			to_write: false,
			source: Some(path.clone()),
			output: path,
			front_matter: Default::default(), //TODO
			content: String::new(),
		}
	}).collect();

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
	let mut lines = content.lines();
	if let Some(line) = lines.next() && line.trim_end() == "---" {
		// let end = lines.position(|l| l.trim_end() == "---").ok_or("No closing frontmatter")?;

		let raw_frontmatter: String = lines.take_while(|l| l.trim_end() != "---").collect::<Vec<_>>().join("\n");

		if !raw_frontmatter.is_empty() {
			let parsed_frontmatter: serde_json::Value = serde_yaml::from_str(&raw_frontmatter)?;
	
			frontmatter.extend(liquid::model::to_object(&parsed_frontmatter)?);
		}
		
		let content: String = content.lines().skip(1).skip_while(|l| l.trim_end() != "---").skip(1).collect::<Vec<_>>().join("\n");
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
	
	Ok(content)
}

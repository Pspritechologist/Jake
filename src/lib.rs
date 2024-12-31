#![feature(try_blocks)]
#![feature(let_chains)]
#![feature(iterator_try_collect)]

#![deny(clippy::unwrap_used,
	// clippy::expect_used,
	clippy::panic,
	clippy::panic_in_result_fn,
	clippy::panicking_overflow_checks,
)]

use std::path::PathBuf;

use error::Error;
use liquid::ValueView;

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
	let lua = unsafe { mlua::Lua::unsafe_new() };

	let lua::LuaResult { tags, converters, filters } = lua::setup_lua(&lua, config)?;

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

	walkdir::WalkDir::new(&config.source_dir)
		.into_iter()
		.filter_map(Result::<_, _>::ok)
		.filter(|e| e.file_type().is_file())
		.try_for_each(|e| process_file(config, &liquid, e))?;
		// .map(|e| process_file(config, &liquid, e))
		// .collect::<Result<_, _>>()?;

	Ok(())
}

pub fn process_file(config: &HydeConfig, liquid: &liquid::Parser, file: walkdir::DirEntry) -> Result<(), Error> {
	let HydeConfig { source_dir, output_dir, .. } = config;

	let input = file.path();
	let file_path = file.path().strip_prefix(source_dir).expect("File not in source directory");
	let output = output_dir.join(file_path);

	std::fs::create_dir_all(output.parent().expect("File has no parent"))?;

	let content = std::fs::read_to_string(input)?;
	let content = parse_content(config, liquid, content, liquid::Object::new());

	fn markdown_ops() -> markdown::Options {
		markdown::Options {
			compile: markdown::CompileOptions {
				allow_dangerous_html: true,
				allow_dangerous_protocol: true,
				..markdown::CompileOptions::gfm()
			},
			parse: markdown::ParseOptions::gfm()
		}
	}

	match file.path().extension().and_then(std::ffi::OsStr::to_str) {
		Some("md") => std::fs::write(output.with_extension("html"), markdown::to_html_with_options(&content?, &markdown_ops()).expect("Markdown doesn't panic"))?,
		Some("scss") => std::fs::write(output.with_extension("css"), grass::from_string(content?.as_str(), &grass::Options::default())?)?,
		_ => std::fs::write(output, content.map(|c| c.as_bytes().to_vec()).or_else(|_| std::fs::read(input))?)?,
	}

	Ok(())
}

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

// #[derive(Debug, Clone)]
// struct LuaBlock {
// 	lua: mlua::Lua,
// }

// impl liquid_core::BlockReflection for LuaBlock {
// 	fn start_tag(&self) -> &str {
// 		"lua"
// 	}

// 	fn end_tag(&self) -> &str {
// 		"endlua"
// 	}

// 	fn description(&self) -> &str {
// 		"A block of Lua code to be executed"
// 	}
// }

// impl liquid_core::ParseBlock for LuaBlock {
// 	fn parse(
// 		&self,
// 		arguments: liquid_core::TagTokenIter,
// 		mut block: liquid_core::TagBlock,
// 		options: &liquid_core::Language,
// 	) -> liquid_core::Result<Box<dyn liquid_core::Renderable>> {
// 		Ok(Box::new(LuaBlockRenderer { function: self.lua.load(&block.parse_all(&options).unwrap().concat::<String>()).into_function().unwrap() }))
// 	}

// 	fn reflection(&self) -> &dyn liquid_core::BlockReflection {
// 		self
// 	}
// }

// #[derive(Debug, Clone)]
// struct LuaBlockRenderer {
// 	function: mlua::Function,
// }

// impl liquid_core::Renderable for LuaBlockRenderer {
// 	fn render_to(&self, writer: &mut dyn std::io::Write, runtime: &dyn liquid_core::Runtime) -> liquid_core::Result<()> {
// 		let res = self.lua.load(&self.script).into_function().unwrap();

// 		Ok(())
// 	}
// }

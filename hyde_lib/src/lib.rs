#![feature(
	try_blocks,
	let_chains,
	iterator_try_collect,
	once_cell_try,
	extend_one,
	iter_intersperse,
	// result_flattening,
)]

#![warn(
	clippy::todo,
	clippy::unimplemented,
	// clippy::expect_used,
)]

#![deny(clippy::unwrap_used,
	clippy::panic,
	clippy::panic_in_result_fn,
	clippy::panicking_overflow_checks,
)]

pub mod error;

pub use data_strctures::HydeConfig;

mod lua;
mod frontmatter;
mod jsonify_tag;
pub(crate) mod data_strctures;

use error::{Error, HydeError::*, ResultExtensions};
use frontmatter::{combine_frontmatters, FrontMatter};
use data_strctures::{FileContent, HydeFileT1};
use kstring::{backend::{BoxedStr, HeapStr}, KString};
use liquid::ValueView;
use relative_path::{RelativePath, RelativePathBuf};
use std::collections::HashMap;
use std::borrow::Cow::{Owned as Cowned, Borrowed as Cowed};

static CONFIG: std::sync::OnceLock<&HydeConfig> = std::sync::OnceLock::new();
#[inline]
#[allow(clippy::expect_used)]
fn config<'a>() -> &'a HydeConfig {
	CONFIG.get().expect("Config not set")
}

pub fn process_project(config: &HydeConfig) -> Result<(), Error> {
	// SAFETY: Config is reset at the start of this function and
	// is only accessed during the function's execution.
	#[allow(clippy::missing_transmute_annotations, clippy::expect_used)]
	CONFIG.set(unsafe { std::mem::transmute(config) }).expect("Config already set");

	let lua = unsafe { mlua::Lua::unsafe_new() };
	let lua::LuaResult { tags, converters, filters, mut files } = lua::setup_lua_state(&lua, collect_src()?)?;

	let mut liquid_builder = liquid::ParserBuilder::with_stdlib();

	liquid_builder = liquid_builder.block(lua::block::LuaBlock { lua: lua.clone() })
		.filter(jsonify_tag::Jsonify);

	for (tag, func) in tags {
		liquid_builder = liquid_builder.tag(lua::tag::LuaTag { tag, func, lua: lua.clone() });
	}

	for (filter, func) in filters {
		liquid_builder = liquid_builder.filter(lua::filter::Lua { filter, func, lua: lua.clone() });
	}

	files.retain(|f| f.to_write);

	let layouts = collect_layouts()?;

	let liquid = liquid_builder.build()?;

	let global: liquid::Object = serde_yaml::from_str(&std::fs::read_to_string(config.project_dir.join("hyde.yml"))?)?;

	for file in files {
		if !file.to_write { continue; }
		
		match file.content {
			FileContent::Utf8(content) => {
				let objs = [ Cowed(&global), Cowned(liquid::to_object(&file.front_matter)?) ];
				let mut content = parse_content(&layouts, &liquid, content, file.source.as_option(), combine_frontmatters(objs))?;

				let output = file.output.to_logical_path(&config.output_dir);
				std::fs::create_dir_all(output.parent().ok_or_else(|| UnexpectedFilePath(output.clone()))?)?;

				// if let Some(ext) = output.extension() && ext == "html" {
				// 	if true {
				// 		let mut buf = Vec::new();
				// 		let mut rewriter = lol_html::HtmlRewriter::new(
				// 			lol_html::Settings {
				// 				element_content_handlers: vec![

				// 				],
				// 				..Default::default()
				// 			},
				// 			|c: &[u8]| buf.extend(c.iter().copied())
				// 		);
				// 		rewriter.write(content.as_bytes()).expect("owo");
				// 		rewriter.end().expect("uwu");

				// 		content = String::from_utf8(buf).expect("Rewritten HTML is valid UTF8");
				// 	}

				// 	if true {
				// 		let min = minify_html::minify(content.as_bytes(), &minify_html::Cfg { minify_css: true, minify_js: true, ..Default::default() });
				// 		content = String::from_utf8(min).expect("Minified HTML is valid UTF8");
				// 	}
				// }

				std::fs::write(output, content)?;
			},
			FileContent::Binary => {
				let output = file.output.to_logical_path(&config.output_dir);
				std::fs::create_dir_all(output.parent().ok_or_else(|| UnexpectedFilePath(output.clone()))?)?;

				const MSG: &str = "Only files with a src can be binary";
				let source = file.source.into_option().expect(MSG).to_logical_path(&config.source_dir);

				if let (Ok(src), Ok(out)) = (source.metadata().and_then(|src| src.modified()), output.metadata().and_then(|src| src.modified())) && src < out {
				} else {
					std::fs::copy(source, output)?;
					// std::os::unix::fs::symlink(source, output)?; //? This is really really funny.
				}
			},
		}
	}

	Ok(())
}

fn collect_src() -> Result<Vec<HydeFileT1>, Error> {
	let HydeConfig { project_dir, source_dir, output_dir, plugins_dir, layout_dir, .. } = config();

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
			let get_rel_conf_path = || conf_path.strip_prefix(project_dir).expect("File not in proj directory").to_string_lossy();

			if let Some(config) = conf_path.exists()
				.then(|| std::fs::File::open(&conf_path))
				.transpose()?
				.map(serde_yaml::from_reader::<_, HashMap<String, FrontMatter>>)
				.transpose()
				.into_error_result_with(get_rel_conf_path)? {

				frontmatter_glob.extend_reserve(config.len());
				for (glob, frontmatter) in config {
					frontmatter_glob.push((globset::GlobBuilder::new(&format!("{}/{}", entry.path().to_string_lossy(), glob))
						.backslash_escape(true)
						.empty_alternates(true)
						.build()
						.into_error_result_with(get_rel_conf_path)?
						.compile_matcher(), frontmatter));
				}
			}

			continue;
		}

		let rel_path = entry.path()
			.strip_prefix(source_dir)
			.ok()
			.and_then(|p| RelativePathBuf::from_path(p).ok())
			.ok_or_else(|| UnexpectedFilePath(entry.path().to_owned()))?;

		let mut front_matter = FrontMatter::default();

		for (glob, fm) in &frontmatter_glob {
			if glob.is_match(entry.path()) {
				front_matter.extend(fm.clone());
			}
		}

		let content = if let Some((fm, content)) = frontmatter::file_frontmatter_content(entry.path())? {
			if let Some(fm) = fm {
				front_matter.extend(fm);
			}
			FileContent::Utf8(content)
		} else {
			FileContent::Binary
		};

		files.push(HydeFileT1 {
			source: rel_path,
			front_matter,
			content,
			// to_write: true,
			// output: rel_path,
			// content: String::new(),
		});
	}

	Ok(files)
}

fn parse_content(
	layouts: &HashMap<KString, HydeLayout>,
	liquid: &liquid::Parser,
	content: impl AsRef<str>,
	path: Option<impl AsRef<RelativePath>>,
	mut frontmatter: liquid::Object
) -> Result<String, Error> {
	fn markdown_ops() -> markdown::Options {
		markdown::Options {
			compile: markdown::CompileOptions {
				allow_dangerous_html: true,
				allow_dangerous_protocol: true,
				gfm_tagfilter: false,
				..markdown::CompileOptions::gfm()
			},
			parse: markdown::ParseOptions {
				constructs: markdown::Constructs {
					code_indented: false,
					..markdown::Constructs::gfm()
				},
				..markdown::ParseOptions::gfm()
			},
		}
	}

	let context = || path.as_ref().map_or(String::from("Lua-generated File"), |p| p.as_ref().to_string());

	let content = liquid.parse(content.as_ref())
		.into_error_result_with(context)?
		.render(&frontmatter)
		.into_error_result_with(context)?;

	let mut content = if let Some(path) = &path { match path.as_ref().extension() {
		Some("md") => {
			markdown::to_html_with_options(&content, &markdown_ops()).expect("Markdown doesn't panic")
		},
		// Some("scss") => grass::from_string(&content, &grass::Options::default())?,
		_ => content,
	} } else { content };

	if let Some(layout) = frontmatter.get("layout") {
		let layout = layouts.get(layout.to_kstr().as_str()).ok_or_else(|| LayoutNotFound(layout.to_kstr().into()))?;

		frontmatter.remove("layout");

		let mut frontmatter = if let Some(layout_frontmatter) = layout.frontmatter.as_ref() {
			combine_frontmatters([ Cowned(frontmatter), Cowned(liquid::to_object(&layout_frontmatter)?) ])
		} else {
			frontmatter
		};

		frontmatter.insert("content".into(), liquid_core::Value::scalar(content));

		content = parse_content(layouts, liquid, &layout.content, Some(&layout.path), frontmatter)
			.into_error_result_with(|| format!("{} + {}", context(), layout.path))?;
	}

	Ok(content)
}

struct HydeLayout {
	pub path: RelativePathBuf,
	pub frontmatter: Option<FrontMatter>,
	pub content: BoxedStr,
}

fn collect_layouts() -> Result<HashMap<KString, HydeLayout>, Error> {
	let HydeConfig { layout_dir, .. } = config();

	let mut layouts = HashMap::new();

	let dir = walkdir::WalkDir::new(layout_dir)
		.into_iter()
		.filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
		.filter_map(Result::ok)
		.filter(|f| f.file_type().is_file());

	for entry in dir {
		let rel_path = entry.path()
			.strip_prefix(layout_dir)
			.ok()
			.and_then(|p| RelativePathBuf::from_path(p).ok())
			.ok_or_else(|| UnexpectedFilePath(entry.path().to_owned()))?;

		let name = KString::from_ref(rel_path.file_stem().ok_or(Misc("Layout file has no name"))?);

		let (frontmatter, content) = frontmatter::file_frontmatter_content(entry.path())
			.into_error_result_with(|| rel_path.as_str())?
			.ok_or(FileNotUtf8(rel_path.clone()))?;

		let layout = HydeLayout {
			path: rel_path,
			frontmatter,
			content: BoxedStr::from_string(content),
		};

		layouts.insert(name, layout);
	}

	Ok(layouts)
}

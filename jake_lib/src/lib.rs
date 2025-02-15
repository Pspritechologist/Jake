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

pub use data_strctures::JakeConfig;

mod lua;
mod frontmatter;
mod jsonify_tag;
pub(crate) mod data_strctures;

use error::{Error, JakeError::*, ResultExtensions};
use frontmatter::FrontMatter;
use data_strctures::{FileContent, JakeFileT1, JakeFileT3};
use kstring::KString;
use liquid::ValueView;
use liquid_core::{runtime, Renderable, Runtime};
use relative_path::{RelativePath, RelativePathBuf};
use std::collections::HashMap;

pub fn process_project(config: &JakeConfig) -> Result<(), Error> {
	let lua = unsafe { mlua::Lua::unsafe_new() };
	let lua::LuaResult { tags, converters, filters, mut files } = lua::setup_lua_state(&lua, config, collect_src(config)?)?;

	let mut liquid_builder = liquid::ParserBuilder::with_stdlib();

	liquid_builder = liquid_builder.block(lua::liquid_api::block::LuaBlock { lua: lua.clone() })
		.filter(jsonify_tag::Jsonify);

	for (tag, func) in tags {
		liquid_builder = liquid_builder.tag(lua::liquid_api::tag::LuaTag { tag, func, lua: lua.clone() });
	}

	for (filter, func) in filters {
		liquid_builder = liquid_builder.filter(lua::liquid_api::filter::Lua { filter, func, lua: lua.clone() });
	}

	files.retain(|f| f.to_write);

	let liquid_parser = liquid_builder.build()?;

	let files: Vec<_> = files.into_iter().map(|f| Ok::<_, Error>(JakeFileT3 {
		source: f.source,
		output: f.output,
		front_matter: f.front_matter,
		template: if let FileContent::Utf8(content) = f.content {
			FileContent::Utf8(liquid_parser.parse(&content)?)
		} else { FileContent::Binary },
		to_write: f.to_write,
		post_processor: f.post_processor,
	})).try_collect()?;

	let layouts = collect_layouts(config, &liquid_parser)?;

	let liquid_site_scope: liquid::Object = serde_yaml::from_str(&std::fs::read_to_string(config.project_dir.join("jake.yml"))?)?;

	let liquid_runtime = liquid_core::runtime::RuntimeBuilder::new()
		// .set_partials(values)
		.set_globals(&liquid_site_scope)
		.build();

	// let liquid_lua_scope = lua::liquid_api::liquid_view::LuaValueView::new(lua.globals(), &lua)?;
	// let liquid_lua_scope = liquid_core::runtime::StackFrame::new(liquid_runtime, liquid_lua_scope);

	for file in files {
		if !file.to_write { continue; }
		
		match file.template {
			FileContent::Utf8(template) => {
				// let scope = [ liquid_site_scope.to_owned(), liquid::to_object(&file.front_matter)? ].into_iter().flatten().collect();
				let data = liquid::to_object(&file.front_matter)?;
				let scope = liquid_core::runtime::StackFrame::new(&liquid_runtime, &data);
				let mut content = parse_content(&layouts, &template, file.source.as_option(), &scope)?;

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

fn collect_src(config: &JakeConfig) -> Result<Vec<JakeFileT1>, Error> {
	let JakeConfig { project_dir, source_dir, output_dir, plugins_dir, layout_dir, .. } = config;

	std::fs::create_dir_all(output_dir)?;
	std::fs::create_dir_all(plugins_dir)?;
	std::fs::create_dir_all(layout_dir)?;

	let mut files = Vec::with_capacity(16); // Better than starting at 0.
	
	let dir = walkdir::WalkDir::new(source_dir)
		.into_iter()
		.filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
		.filter_map(Result::ok);

	const DEFAULT_FRONTMATTER_FILE: &str = ".jake.yml";

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

		let context = || entry.path().strip_prefix(project_dir).unwrap_or(entry.path()).to_string_lossy();
		let content = if let Some((fm, content)) = frontmatter::file_frontmatter_content(entry.path()).into_error_result_with(context)? {
			if let Some(fm) = fm {
				front_matter.extend(fm);
			}
			FileContent::Utf8(content)
		} else {
			FileContent::Binary
		};

		files.push(JakeFileT1 {
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
	layouts: &HashMap<KString, JakeLayout>,
	template: &liquid::Template,
	path: Option<impl AsRef<RelativePath>>,
	liquid_runtime: &dyn liquid_core::runtime::Runtime,
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

	pub struct TemplateMirror {
		template: runtime::Template,
		#[allow(dead_code)]
		partials: Option<std::sync::Arc<dyn liquid_core::runtime::PartialStore + Send + Sync>>,
	}

	let content = unsafe { std::mem::transmute::<&liquid::Template, &TemplateMirror>(template) } // :T
		.template.render(&liquid_runtime)
		.into_error_result_with(context)?;

	let mut content = if let Some(path) = &path { match path.as_ref().extension() {
		Some("md") => {
			markdown::to_html_with_options(&content, &markdown_ops()).expect("Markdown doesn't panic")
		},
		// Some("scss") => grass::from_string(&content, &grass::Options::default())?,
		_ => content,
	} } else { content };

	if let Some(layout) = liquid_runtime.try_get(&[ "layout".into() ]) && !layout.is_nil() {
		let layout = layouts.get(layout.to_kstr().as_str()).ok_or_else(|| LayoutNotFound(layout.to_kstr().into()))?;

		let mut frontmatter = liquid::to_object(&layout.frontmatter)?;
		frontmatter.insert("layout".into(), liquid::model::Value::Nil);

		let runtime = liquid_core::runtime::StackFrame::new(&liquid_runtime, frontmatter);

		runtime.set_global("content".into(), liquid::model::Value::scalar(content));

		content = parse_content(layouts, &layout.template, Some(&layout.path), &runtime)
			.into_error_result_with(|| format!("{} + {}", context(), layout.path))?;
	}

	Ok(content)
}

struct JakeLayout {
	pub path: RelativePathBuf,
	pub frontmatter: Option<FrontMatter>,
	pub template: liquid::Template,
}

fn collect_layouts(config: &JakeConfig, parser: &liquid::Parser) -> Result<HashMap<KString, JakeLayout>, Error> {
	let JakeConfig { layout_dir, .. } = config;

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

		let template = parser.parse(&content)
			.into_error_result_with(|| rel_path.as_str())?;

		let layout = JakeLayout {
			path: rel_path,
			frontmatter,
			template,
		};

		layouts.insert(name, layout);
	}

	Ok(layouts)
}

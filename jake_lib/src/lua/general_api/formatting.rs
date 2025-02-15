pub fn render_markdown(content: &str) -> String {
	markdown::to_html_with_options(content, &markdown_ops()).expect("Basic Markdown doesn't error.")
}

pub fn minify_html(lua: &mlua::Lua, content: &str) -> mlua::Result<mlua::String> {
	lua.create_string(minify_html::minify(content.as_bytes(), &minify_html_conf()))
}

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

fn minify_html_conf() -> minify_html::Cfg {
	minify_html::Cfg {
		minify_css: true,
		minify_js: true,
		..Default::default()
	}
}

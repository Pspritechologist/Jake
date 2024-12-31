use super::*;

#[derive(Debug, Clone)]
pub struct LuaTag {
	pub tag: String,
	pub func: mlua::Function,
	pub lua: mlua::Lua,
}

impl liquid_core::TagReflection for LuaTag {
	fn tag(&self) -> &str {
		&self.tag
	}

	fn description(&self) -> &str {
		"Custom tag registered from Lua"
	}
}

impl liquid_core::ParseTag for LuaTag {
	fn parse(&self, mut arguments: liquid_core::TagTokenIter, _options: &liquid_core::Language) -> liquid_core::Result<Box<dyn liquid_core::Renderable>> {
		let mut args = vec![];

		while let Ok(arg) = arguments.expect_next("") {
			args.push(arg.expect_value().into_result()?);
		}
		
		Ok(Box::new(LuaTagRenderer { func: self.func.clone(), args, lua: self.lua.clone() }))
	}

	fn reflection(&self) -> &dyn liquid_core::TagReflection {
		self
	}
}

#[derive(Debug)]
struct LuaTagRenderer {
	args: Vec<liquid_core::Expression>,
	func: mlua::Function,
	lua: mlua::Lua,
}

impl liquid_core::Renderable for LuaTagRenderer {
	fn render_to(&self, writer: &mut dyn std::io::Write, runtime: &dyn liquid_core::Runtime) -> liquid_core::Result<()> {
		let args: mlua::MultiValue = self.args.iter().map(
			|arg| arg.evaluate(runtime).and_then(
				|v| self.lua.to_value(&v.into_owned()).map_err(|e| liquid::Error::with_msg(format!("Error while converting argument to Lua value: {e}")))
			)
		).collect::<Result<_, _>>()?;

		let res: mlua::Value = self.func.call(args)
			.map_err(|e| liquid_core::Error::with_msg(format!("Lua error: {e}")))?;
		let res = res.to_string().map_err(|e| liquid_core::Error::with_msg(format!("Error while rendering Lua result: {e}")))?;
		
		writer.write_all(res.as_bytes())
			.map_err(|e| liquid_core::Error::with_msg(format!("Error while writing Lua result: {e}")))?;

		Ok(())
	}
}

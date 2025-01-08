use super::*;
use mlua::LuaSerdeExt;
use liquid_core::{
	Expression,
	Language,
	ParseTag,
	Renderable,
	Runtime,
	TagReflection,
	TagTokenIter,
};

#[derive(Debug, Clone)]
pub struct LuaTag {
	pub tag: String,
	pub func: mlua::Function,
	pub lua: mlua::Lua,
}

impl TagReflection for LuaTag {
	fn tag(&self) -> &str {
		&self.tag
	}

	fn description(&self) -> &str {
		"Custom tag registered from Lua"
	}
}

impl ParseTag for LuaTag {
	fn parse(&self, mut arguments: TagTokenIter, _options: &Language) -> liquid_core::Result<Box<dyn Renderable>> {
		let mut args = vec![];

		while let Ok(arg) = arguments.expect_next("") {
			args.push(arg.expect_value().into_result()?);
		}
		
		Ok(Box::new(LuaTagRenderer { func: self.func.clone(), args, lua: self.lua.clone() }))
	}

	fn reflection(&self) -> &dyn TagReflection {
		self
	}
}

#[derive(Debug)]
struct LuaTagRenderer {
	args: Vec<Expression>,
	func: mlua::Function,
	lua: mlua::Lua,
}

impl Renderable for LuaTagRenderer {
	fn render_to(&self, writer: &mut dyn std::io::Write, runtime: &dyn Runtime) -> liquid_core::Result<()> {
		let args: mlua::MultiValue = self.args.iter().map(|arg| arg.evaluate(runtime))
			.map(|arg| self.lua.to_value(&arg?.into_owned()).map_err(Error::from))
			.try_collect()?;

		let res: mlua::Value = self.func.call(args).map_err(Error::from)?;
		let res = res.to_string().map_err(Error::from)?;
		
		writer.write_all(res.as_bytes()).map_err(Error::from)?;

		Ok(())
	}
}

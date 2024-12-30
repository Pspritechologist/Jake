use mlua::LuaSerdeExt;

use crate::{error::Error, HydeConfig};

#[derive(Debug, Clone, Default)]
pub struct LuaResult {
	// pub tags: Vec<(String, mlua::Function)>,
	// pub converters: Vec<(String, mlua::Function)>,
	// pub filters: Vec<(String, mlua::Function)>,
	pub tags: std::collections::BTreeMap<String, mlua::Function>,
	pub converters: std::collections::BTreeMap<String, mlua::Function>,
	pub filters: std::collections::BTreeMap<String, mlua::Function>,
}

pub fn setup_lua(lua: &mlua::Lua, config: &HydeConfig) -> Result<LuaResult, Error> {
	let plugins_root = if config.plugins_dir.join("init.lua").exists() {
		config.plugins_dir.clone()
	} else if config.plugins_dir.join("init/init.lua").exists() {
		config.plugins_dir.join("init")
	} else {
		return Ok(LuaResult::default());
	};

	let init = std::fs::read_to_string(plugins_root.join("init.lua"))?;
	
	let global = lua.globals();

	global.set("TAGS", lua.create_table()?)?;
	global.set("FILTERS", lua.create_table()?)?;
	global.set("CONVERTERS", lua.create_table()?)?;

	// global.set("LUA_PATH", plugins_root.to_string_lossy())?;

	// let mut tags: Vec<(String, mlua::Function)> = vec![];
	// let mut filters: Vec<(String, mlua::Function)> = vec![];
	// let mut converters: Vec<(String, mlua::Function)> = vec![];

	// lua.scope(|scope| {
	// 	global.set("tag", scope.create_function_mut(|_, (tag, func): (String, mlua::Function)| {
	// 		tags.push((tag, func));
	// 		Ok(())
	// 	})?)?;

	// 	global.set("filter", scope.create_function_mut(|_, (filter, func): (String, mlua::Function)| {
	// 		filters.push((filter, func));
	// 		Ok(())
	// 	})?)?;

	// 	global.set("converter", scope.create_function_mut(|_, (ext, func): (String, mlua::Function)| {
	// 		converters.push((ext, func));
	// 		Ok(())
	// 	})?)?;

		lua.load(&init)
			.set_name(plugins_root.join("init.lua").strip_prefix(&config.project_dir).unwrap().to_string_lossy())
			.exec()?;

	// 	Ok(())
	// })?;

	let tags: std::collections::BTreeMap<String, mlua::Function> = global.get("TAGS")?;
	let filters: std::collections::BTreeMap<String, mlua::Function> = global.get("FILTERS")?;
	let converters: std::collections::BTreeMap<String, mlua::Function> = global.get("CONVERTERS")?;

	Ok(LuaResult { tags, converters, filters })
}

#[derive(Debug, Clone)]
pub struct LuaBlock {
	pub lua: mlua::Lua,
}

impl liquid_core::BlockReflection for LuaBlock {
	fn start_tag(&self) -> &str {
		"lua"
	}

	fn end_tag(&self) -> &str {
		"endlua"
	}

	fn description(&self) -> &str {
		"A block of Lua code to be executed."
	}
}

impl liquid_core::ParseBlock for LuaBlock {
	fn parse(&self, mut arguments: liquid_core::TagTokenIter, mut block: liquid_core::TagBlock, options: &liquid_core::Language) -> liquid_core::Result<Box<dyn liquid_core::Renderable>> {
		arguments.expect_nothing()?;

		let lua = self.lua.clone();
		let code = block.escape_liquid(true)?;
		let func = lua.load(code).into_function().map_err(|e| liquid_core::Error::with_msg(format!("Error while loading Lua code: {e}")))?;

		Ok(Box::new(LuaBlockRenderer { func, lua }))
	}

	fn reflection(&self) -> &dyn liquid_core::BlockReflection {
		self
	}
}

#[derive(Debug, Clone)]
struct LuaBlockRenderer {
	func: mlua::Function,
	lua: mlua::Lua,
}

impl liquid_core::Renderable for LuaBlockRenderer {
	fn render_to(&self, writer: &mut dyn std::io::Write, runtime: &dyn liquid_core::Runtime) -> liquid_core::Result<()> {
		let e = self.lua.scope(|scope| {
			let env = self.func.environment().unwrap_or(self.lua.globals());
			
			env.set("write", scope.create_function_mut(|_, args: mlua::MultiValue| {
				let res = args.into_iter().map(|arg| arg.to_string()).collect::<Result<String, _>>()?;
				
				writer.write_all(res.as_bytes())
					.map_err(mlua::Error::external)?;

				Ok(())
			})?)?;

			// env.set("get", scope.create_function_mut(|lua, key: String| {
			// 	runtime.try_get(&[liquid::model::Scalar::new(key)])
			// 		.map(|v| lua.to_value(&v.into_owned()))
			// 		.unwrap_or(Ok(mlua::Value::Nil))
			// })?)?;

			// env.set("set", scope.create_function_mut(|_, (key, value): (String, mlua::Value)| {
			// 	runtime.set_global(liquid::model::KString::new(key), value.into_owned());
			// 	Ok(())
			// })?);

			// let env = self.lua.create_table()?;
			// let mt = self.lua.create_table()?;
			// mt.set("__index", self.lua.create_function_mut(|lua, (name, value): (mlua::String, Option<mlua::Value>)| {
			// 	if let Some(value) = value {

			// 	} else {

			// 	}

			// 	Ok(())
			// })?)?;

			self.func.call(())
				.map_err(mlua::Error::external)?;

			Ok(())
		}).err();

		if let Some(e) = e && let Some(e) = e.downcast_ref::<liquid::Error>() {
			return Err(e.to_owned());
		}

		Ok(())
	}
}

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

#[derive(Debug, Clone)]
pub struct LuaFilter {
	pub filter: String,
	pub func: mlua::Function,
}

impl std::fmt::Display for LuaFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.filter)
	}
}

impl liquid_core::FilterReflection for LuaFilter {
	fn name(&self) -> &str {
		&self.filter
	}

	fn description(&self) -> &str {
		"Custom filter registered from Lua"
	}

	fn positional_parameters(&self) -> &'static [liquid_core::parser::ParameterReflection] {
		todo!()
	}

	fn keyword_parameters(&self) -> &'static [liquid_core::parser::ParameterReflection] {
		todo!()
	}
}

impl liquid_core::ParseFilter for LuaFilter {
	fn parse(&self, arguments: liquid_core::parser::FilterArguments) -> liquid_core::Result<Box<dyn liquid_core::Filter>> {
		todo!()
	}

	fn reflection(&self) -> &dyn liquid_core::FilterReflection {
		self
	}
}

#[derive(Debug, liquid_core::Display_filter)]
#[name = "strip"]
struct LuaFilterEr;

impl liquid_core::Filter for LuaFilter {
    fn evaluate(&self, input: &dyn liquid_core::ValueView, _runtime: &dyn liquid_core::Runtime) -> Result<liquid_core::Value, liquid_core::Error> {
        let input = input.to_kstr();
        Ok(liquid_core::Value::scalar(input.trim().to_owned()))
    }
}

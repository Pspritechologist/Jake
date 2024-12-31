use crate::error::ErrorExtensions;

use super::*;

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
					.map_err(crate::error::ErrorExtensions::into_lua_error)?;

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

			self.func.call::<()>(())
				.map_err(crate::error::ErrorExtensions::into_lua_error)?;

			Ok(())
		}).err();

		if let Some(e) = e {
			return Err(e.into_liquid_error())
		}

		Ok(())
	}
}

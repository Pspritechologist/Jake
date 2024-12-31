pub mod tag;
pub mod filter;
pub mod block;
pub mod converter;

use mlua::LuaSerdeExt;

use crate::{error::Error, HydeConfig};

#[derive(Debug, Clone, Default)]
pub struct LuaResult {
	pub tags: std::collections::HashMap<String, mlua::Function>,
	pub converters: std::collections::HashMap<String, mlua::Function>,
	pub filters: std::collections::HashMap<String, mlua::Function>,
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

	const TAGS_TABLE: &str = "TAGS";
	const FILTERS_TABLE: &str = "FILTERS";
	const CONVERTERS_TABLE: &str = "CONVERTERS";

	global.set(TAGS_TABLE, lua.create_table()?)?;
	global.set(FILTERS_TABLE, lua.create_table()?)?;
	global.set(CONVERTERS_TABLE, lua.create_table()?)?;

	lua.load(&init)
		.set_name(plugins_root.join("init.lua")
			.strip_prefix(&config.project_dir)
			.expect("init.lua not in plugins dir")
			.to_string_lossy())
		.exec()?;

	let tags = global.get(TAGS_TABLE)?;
	let filters = global.get(FILTERS_TABLE)?;
	let converters = global.get(CONVERTERS_TABLE)?;

	Ok(LuaResult { tags, converters, filters })
}

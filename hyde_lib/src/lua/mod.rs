pub mod tag;
pub mod filter;
pub mod block;
pub mod converter;
pub mod liquid_user_data;
pub mod liquid_view;
pub mod general_api;

use crate::{error::Error, HydeConfig};
use general_api::{file::{FileUserData, HydeFile}, path::PathUserData};
use mlua::LuaSerdeExt;
use std::collections::HashMap;

const INIT_LUA_PATHS: &[&str] = &[
	"init.lua",
	"init/init.lua",
];

// Global variable names.
const TAGS_TABLE: &str = "TAGS";
const FILTERS_TABLE: &str = "FILTERS";
const CONVERTERS_TABLE: &str = "CONVERTERS";
const SITE_DATA: &str = "SITE";

// Site data keys.
const DIR_PROJ: &str = "project_dir";
const DIR_SRC: &str = "source_dir";
const DIR_OUT: &str = "output_dir";
const DIR_PLUG: &str = "plugins_dir";
const DIR_LAY: &str = "layout_dir";
const FILES: &str = "files";

#[derive(Debug, Clone, Default)]
pub struct LuaResult {
	pub tags: HashMap<String, mlua::Function>,
	pub converters: HashMap<String, mlua::Function>,
	pub filters: HashMap<String, mlua::Function>,

	pub files: Vec<HydeFile>,
}

pub fn setup_lua_state(lua: &mlua::Lua, config: &HydeConfig, files: Vec<HydeFile>) -> Result<LuaResult, Error> {
	let Some(init_file) = INIT_LUA_PATHS.iter()
		.map(|path| config.plugins_dir.join(path))
		.find(|path| path.exists()) else {
			return Ok(LuaResult::default());
		};

	let init = std::fs::read_to_string(&init_file)?;
	
	let global = lua.globals();

	// Make sure require searches for modules relative to the plugins directory.
	let package: mlua::Table = global.get("package")?;
	let path = format!("{dir}/?.lua;{dir}/?/init.lua", dir = config.plugins_dir.to_string_lossy());
	package.set("path", path)?;
	
	global.set(PathUserData::CLASS_NAME, lua.create_proxy::<PathUserData>()?)?;
	global.set(FileUserData::CLASS_NAME, lua.create_proxy::<FileUserData>()?)?;

	global.set(TAGS_TABLE, lua.create_table()?)?;
	global.set(FILTERS_TABLE, lua.create_table()?)?;
	global.set(CONVERTERS_TABLE, lua.create_table()?)?;

	let site_data = lua.create_table()?;
	site_data.set(DIR_PROJ, config.project_dir.as_os_str())?;
	site_data.set(DIR_SRC, config.source_dir.as_os_str())?;
	site_data.set(DIR_OUT, config.output_dir.as_os_str())?;
	site_data.set(DIR_PLUG, config.plugins_dir.as_os_str())?;
	site_data.set(DIR_LAY, config.layout_dir.as_os_str())?;

	let site_files = lua.create_table_from(
		files.into_iter().enumerate().map(|(i, file)| (i + 1, FileUserData::from_file(file, lua)))
		// files.into_iter().enumerate().map(|(i, f)| (i + 1, lua.to_value(&f).expect("Known data")))
	)?;
	site_data.set(FILES, &site_files)?;

	// let config_clone = config.clone();
	// let new_file = lua.create_function(move |lua, ()| {
	// 	FileUserData::new(&config_clone, lua)
	// })?;
	// site_data.set(NEW_FILE, new_file)?;

	global.set(SITE_DATA, site_data)?;

	lua.load(&init)
		.set_name(init_file.strip_prefix(&config.project_dir)
			.expect("init.lua not in plugins dir")
			.to_string_lossy())
		.exec()?;

	let tags = global.get(TAGS_TABLE)?;
	let filters = global.get(FILTERS_TABLE)?;
	let converters = global.get(CONVERTERS_TABLE)?;

	let files: Vec<HydeFile> = site_files.sequence_values().map(|f| f.and_then(|f: FileUserData| f.into_file(lua))).collect::<Result<_, _>>()?;
	// let files = site_files.sequence_values::<mlua::Value>().map(|v| lua.from_value(v?)).collect::<Result<_, _>>()?;
	// let files = site_files.sequence_values().map(|v| lua.from_value(v?)).collect::<Result<_, _>>()?;

	Ok(LuaResult { tags, converters, filters, files })
}

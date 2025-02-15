pub mod typed;
pub mod general_api;
pub mod liquid_api;

use crate::{JakeConfig, data_strctures::{JakeFileT1, JakeFileT2}, error::{Error, JakeError, ResultExtensions}};
use general_api::{file::FileUserData, path::PathUserData};

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
	pub tags: Vec<(String, mlua::Function)>,
	pub converters: Vec<(String, mlua::Function)>,
	pub filters: Vec<(String, mlua::Function)>,

	pub files: Vec<JakeFileT2>,
}

pub fn setup_lua_state(lua: &mlua::Lua, config: &JakeConfig, files: Vec<JakeFileT1>) -> Result<LuaResult, Error> {
	let Some(init_file) = INIT_LUA_PATHS.iter()
		.map(|path| config.plugins_dir.join(path))
		.find(|path| path.exists()) else {
			return Ok(LuaResult {
				files: files.into_iter()
					.flat_map(|f| FileUserData::from_file(f, lua).map(|f| f.into_file(lua)))
					.collect::<Result<_, _>>()?,
				..Default::default()
			});
		};

	let init = std::fs::read_to_string(&init_file)?;
	
	let global = lua.globals();

	// Make sure require searches for modules relative to the plugins directory.
	let package: mlua::Table = global.get("package")?;
	let path = format!("{dir}/share/lua/5.1/?.lua;{dir}/share/lua/5.1/?/init.lua;{dir}/?.lua;{dir}/?/init.lua", dir = config.plugins_dir.to_string_lossy());
	package.set("path", path)?;
	let cpath = format!("{dir}/lib/lua/5.1/?.so;{dir}/?.so", dir = config.plugins_dir.to_string_lossy());
	package.set("cpath", cpath)?;
	
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
		files.into_iter().enumerate().map(|(i, file)| (i + 1, FileUserData::from_file(file, lua).expect("Userdata failed uwu"))) //TODO: Iter tools thing
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
			.map_err(|_| JakeError::UnexpectedFilePath(init_file.clone()))?
			.to_string_lossy())
		.exec()?;

	let tags = global.get::<mlua::Table>(TAGS_TABLE)?.pairs().try_collect()?;
	let filters = global.get::<mlua::Table>(FILTERS_TABLE)?.pairs().try_collect()?;
	let converters = global.get::<mlua::Table>(CONVERTERS_TABLE)?.pairs().try_collect()?;

	let files: Vec<JakeFileT2> = site_files.sequence_values().map(|f|
		f.and_then(|f: FileUserData| {
			let clone = f.output.clone();
			let context = || clone.borrow().map_or(String::from("Unknown file"), |p| p.path().to_string());
			f.into_file(lua).into_error_result_with(context).into_lua_result()
		})
	).collect::<Result<_, _>>()?;

	// let files = site_files.sequence_values::<mlua::Value>().map(|v| lua.from_value(v?)).collect::<Result<_, _>>()?;
	// let files = site_files.sequence_values().map(|v| lua.from_value(v?)).collect::<Result<_, _>>()?;

	// let post_processor: Option<mlua::Either<mlua::Function, HashMap<mlua::String, mlua::Function>>> = global.get(FILE_POSTPROC_FUNC)?;
	// let post_processor = post_processor.map(|e| match e {
	// 	mlua::Either::Left(func) => Ok(mlua::Either::Left(func)),
	// 	mlua::Either::Right(table) => {
	// 		let table: HashMap<mlua::String, mlua::Function> = table.pairs().map(|pair| {
	// 			let (k, v): (mlua::String, mlua::Function) = pair?;
	// 			Ok((k, v))
	// 		}).try_collect()?;

	// 		Ok(mlua::Either::Right(table))
	// 	},
	// }).transpose()?;

	Ok(LuaResult { tags, converters, filters, files, /* post_processor */ })
}

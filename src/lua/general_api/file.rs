use std::{cell::OnceCell, collections::HashMap, fmt::Write, ops::Deref, path::PathBuf};
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, SerializeOptions, UserData};
use serde::Deserialize;
use crate::{lua, HydeConfig};

use super::{path::PathUserData, *};

pub const SOURCE_FIELD: &str = "source";
pub const OUTPUT_FIELD: &str = "path";
pub const CONTENT_FIELD: &str = "content";
pub const DATA_FIELD: &str = "data";
pub const TO_WRITE_FIELD: &str = "to_write";
pub const IGNORE_METHOD: &str = "ignore";
pub const IS_ABSOLUTE_FIELD: &str = "is_absolute";

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HydeFile {
	pub to_write: bool,
	pub source: Option<PathBuf>,
	pub output: PathBuf,
	pub content: String,
	pub front_matter: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct FileUserData {
	pub to_write: bool,
	pub source: Option<PathUserData>,
	pub output: TypedUserData<PathUserData>,
	pub content: String,
	pub data: mlua::Table,
	lua_string: OnceCell<mlua::String>,
}

impl FileUserData {
	pub const CLASS_NAME: &'static str = "File";

	pub fn from_file(file: HydeFile, lua: &Lua) -> Self {
		Self {
			to_write: file.to_write,
			content: file.content,
			source: file.source.map(PathUserData::new),
			output: PathUserData::new(file.output).to_typed(lua),
			data: lua.create_table_from(
				file.front_matter.into_iter().map(|(k, v)| (k, lua.to_value(&v).expect("Value failed :(")))
			).expect("Table failed :("),
			lua_string: OnceCell::new(),
		}
	}

	pub fn into_file(self, lua: &Lua) -> mlua::Result<HydeFile> {
		Ok(HydeFile {
			to_write: self.to_write,
			source: self.source.map(|path| path.path),
			output: self.output.borrow()?.path.clone(),
			content: self.content,
			front_matter: lua.from_value(mlua::Value::Table(self.data))?,
		})
	}

	pub fn new(lua: &Lua) -> mlua::Result<Self> {
		Ok(Self {
			to_write: true,
			content: String::new(),
			source: None,
			output: TypedUserData::from_ser_data(PathUserData::default(), lua),
			data: lua.create_table()?,
			lua_string: OnceCell::new(),
		})
	}
}

impl FromLua for FileUserData {
	fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
		if let Some(userdata) = value.as_userdata() && userdata.is::<FileUserData>() {
			let userdata = userdata.borrow::<FileUserData>()?;
			
			Ok(userdata.clone())
		} else if value.is_table() {
			let file = lua.from_value(value)?;

			Ok(FileUserData::from_file(file, lua))
		} else {
			Err(mlua::Error::runtime(format!("Expected a table or a userdata, got {:?}", value.type_name())))
		}
	}
}

impl UserData for FileUserData {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get(SOURCE_FIELD, |lua, this| {
			this.source.clone().into_lua(lua)
		});

		fields.add_field_method_get(DATA_FIELD, |_, this| Ok(this.data.clone()));
		fields.add_field_method_set(DATA_FIELD, |_, this, data: mlua::Table| {
			this.data = data;
			Ok(())
		});

		fields.add_field_method_get(OUTPUT_FIELD, |lua, this| this.output.userdata().into_lua(lua));
		fields.add_field_method_set(OUTPUT_FIELD, |lua, this, path: mlua::Value| {
			if let Some(path) = path.as_userdata() && let Ok(path) = TypedUserData::from_userdata(path.clone()) {
				this.output = path;
			} else if let Some(path) = path.as_str() {
				this.output = PathUserData::new(PathBuf::from(&*path)).to_typed(lua);
			} else {
				return Err(mlua::Error::runtime("`path` must be a Path or a String"));
			}

			Ok(())
		});

		fields.add_field_method_get(CONTENT_FIELD, |lua, this| {
			Ok(this.lua_string.get_or_try_init(|| lua.create_string(this.content.as_str()))?.clone())
		});
		fields.add_field_method_set(CONTENT_FIELD, |_, this, content: mlua::String| {
			this.content.clear();
			this.content.write_str(&content.to_str()?).expect("Writing to a String");
			this.lua_string.set(content).expect("Failed to set OnceCell");
			Ok(())
		});

		fields.add_field_method_get(TO_WRITE_FIELD, |_, this| Ok(this.to_write));
		fields.add_field_method_set(TO_WRITE_FIELD, |_, this, to_write: bool| {
			this.to_write = to_write;
			Ok(())
		});

		fields.add_field_method_get(IS_ABSOLUTE_FIELD, |_, this| Ok(this.output.borrow()?.path.has_root()));
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut(IGNORE_METHOD, |_, this, ignore: Option<bool>| {
			this.to_write = ignore.unwrap_or(false);
			Ok(())
		});
	}
}

// impl FromLua for HydeFile {
// 	fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
// 		if let Some(userdata) = value.as_userdata() {
// 			let userdata = userdata.borrow::<FileUserData>()?;
			
// 			Ok(Self {
// 				to_write: userdata.to_write,
// 				source: userdata.source.as_ref().map(|path| path.path.clone()),
// 				output: userdata.output.borrow()?.path.clone(),
// 				content: userdata.content.clone(),
// 				front_matter: lua.from_value(mlua::Value::Table(userdata.data.clone()))?,
// 			})
// 		} else {
// 			Err(mlua::Error::runtime(format!("Expected a table or a userdata, got {:?}", value.type_name())))
// 		}
// 	}
// }

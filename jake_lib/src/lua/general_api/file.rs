use super::{path::PathUserData, *};
use crate::data_strctures::{JakeFileT1, JakeFileT2};
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, UserData};
use relative_path::RelativePathBuf;

pub const SOURCE_FIELD: &str = "source";
pub const OUTPUT_FIELD: &str = "path";
pub const CONTENT_FIELD: &str = "content";
pub const DATA_FIELD: &str = "data";
pub const TO_WRITE_FIELD: &str = "to_write";
pub const IGNORE_METHOD: &str = "ignore";
pub const POSTPROC_FIELD: &str = "post_proc";
pub const IS_TEXT_FIELD: &str = "is_text";
pub const IS_BIN_FIELD: &str = "is_binary";

#[derive(Debug, Clone)]
pub struct FileUserData {
	pub to_write: bool,
	pub source: Option<PathUserData>,
	pub output: TypedUserData<PathUserData>,
	pub content: Option<mlua::String>,
	pub data: mlua::Table,
	pub post_processor: Option<mlua::Function>,
}

impl FileUserData {
	pub const CLASS_NAME: &'static str = "File";

	pub fn from_file(file: JakeFileT1, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self {
			to_write: true,
			content: file.content.into_option().map(|c| lua.create_string(c)).transpose()?,
			source: PathUserData::new(&file.source).into(),
			output: PathUserData::new(file.source).to_typed(lua),
			data: lua.create_table_from(
				file.front_matter.into_iter().map(|(k, v)| (mlua::String::wrap(k), lua.to_value(&v).expect("All frontmatter values are valid Lua values"))),
			)?,
			post_processor: None,
		})
	}

	pub fn into_file(self, lua: &Lua) -> mlua::Result<JakeFileT2> {
		Ok(JakeFileT2 {
			to_write: self.to_write,
			source: self.source.map(|path| path.into_path()).into(),
			output: self.output.borrow()?.path().to_owned(),
			content: self.content.map(|c| c.to_string_lossy()).into(), //TODO: and here...
			front_matter: lua.from_value(mlua::Value::Table(self.data))?,
			post_processor: self.post_processor,
		})
	}

	pub fn new(lua: &Lua) -> mlua::Result<Self> {
		Ok(Self {
			to_write: true,
			content: Some(lua.create_string("")?),
			source: None,
			output: TypedUserData::from_ser_data(PathUserData::default(), lua),
			data: lua.create_table()?,
			post_processor: None,
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

			FileUserData::from_file(file, lua)
		} else {
			Err(mlua::Error::runtime(format!("Expected a table or a {FILE}, got {:?}", value.type_name(), FILE = FileUserData::CLASS_NAME)))
		}
	}
}

#[allow(clippy::unit_arg)]
impl UserData for FileUserData {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get(SOURCE_FIELD, |_, this| Ok(this.source.clone()));

		fields.add_field_method_get(DATA_FIELD, |_, this| Ok(this.data.clone()));
		fields.add_field_method_set(DATA_FIELD, |_, this, data: mlua::Table| Ok(this.data = data));

		fields.add_field_method_get(OUTPUT_FIELD, |lua, this| this.output.userdata().into_lua(lua));
		fields.add_field_method_set(OUTPUT_FIELD, |lua, this, path: mlua::Value| {
			if let Some(path) = path.as_userdata() && let Ok(path) = TypedUserData::from_userdata(path.clone()) {
				this.output = path;
			} else if let Some(path) = path.as_str() {
				this.output = PathUserData::new(RelativePathBuf::from(&*path)).to_typed(lua);
			} else {
				return Err(mlua::Error::runtime("`path` must be a Path or a String"));
			}

			Ok(())
		});

		fields.add_field_method_get(CONTENT_FIELD, |_, this| Ok(this.content.clone()));
		fields.add_field_method_set(CONTENT_FIELD, |_, this, content: mlua::String| {
			if this.content.is_some() {
				Ok(this.content = Some(content))
			} else {
				let msg = format!(
					"Cannot set content of a binary file: {}",
					this.source.as_ref().map_or_else(|| this.output.borrow()
						.map(|p| p.path().to_string())
						.unwrap_or(String::from("Unknown file")), |p| p.path().to_string())
				);
				Err(mlua::Error::RuntimeError(msg))
			}
		});

		fields.add_field_method_get(TO_WRITE_FIELD, |_, this| Ok(this.to_write));
		fields.add_field_method_set(TO_WRITE_FIELD, |_, this, to_write: bool| {
			Ok(this.to_write = to_write)
		});

		fields.add_field_method_get(POSTPROC_FIELD, |_, this| {
			Ok(this.post_processor.clone())
		});
		fields.add_field_method_set(POSTPROC_FIELD, |_, this, post_processor: Option<mlua::Function>| {
			Ok(this.post_processor = post_processor)
		});

		fields.add_field_method_get(IS_TEXT_FIELD, |_, this| Ok(this.content.is_some()));
		fields.add_field_method_get(IS_BIN_FIELD, |_, this| Ok(this.content.is_none()));
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(mlua::MetaMethod::Index, |_, this, name: mlua::String| {
			this.data.raw_get::<mlua::Value>(name)
		});
		methods.add_meta_method(mlua::MetaMethod::NewIndex, |_, this, (name, value): (mlua::String, mlua::Value)| {
			this.data.raw_set(name, value)
		});

		methods.add_method_mut(IGNORE_METHOD, |_, this, ignore: Option<bool>| {
			Ok(this.to_write = ignore.unwrap_or(false))
		});

		methods.add_function(super::NEW_FUNCTION, |lua, value: Option<mlua::Table>| {
			if let Some(value) = value {
				let content = value.get::<Option<_>>("content").transpose();
				let data = value.get::<Option<_>>("data").transpose();
				let output = value.get::<Option<PathUserData>>("output")?;
				let post_processor = value.get("post_processor")?;
				
				Ok(FileUserData {
					content: Some(content.unwrap_or_else(|| lua.create_string(""))?),
					data: data.unwrap_or_else(|| lua.create_table())?,
					output: TypedUserData::from_ser_data(output.unwrap_or_default(), lua),
					post_processor,
					..FileUserData::new(lua)?
				})
			} else {
				FileUserData::new(lua)
			}
		});
	}
}

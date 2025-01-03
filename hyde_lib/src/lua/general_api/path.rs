use mlua::{IntoLua, IntoLuaMulti, UserData};
use relative_path::{RelativePath, RelativePathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PathUserData {
	pub path: RelativePathBuf,
}

impl From<RelativePathBuf> for PathUserData {
	fn from(path: RelativePathBuf) -> Self {
		Self { path }
	}
}

impl PathUserData {
	pub const CLASS_NAME: &'static str = "Path";

	pub fn new(path: impl Into<RelativePathBuf>) -> Self {
		Self { path: path.into() }
	}

	pub fn from(path: impl AsRef<RelativePath>) -> Self {
		Self { path: path.as_ref().to_relative_path_buf() }
	}
}

impl UserData for PathUserData {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("ext", |lua, this| {
			this.path.extension().map(|p| p.into_lua(lua)).unwrap_or(Ok(mlua::Nil))
		});
		fields.add_field_method_set("ext", |_, this, ext: mlua::String| {
			this.path.set_extension(&*ext.to_str()?);
			Ok(())
		});

		fields.add_field_method_get("parent", |lua, this| {
			this.path.parent().map(|p| PathUserData::new(p).into_lua(lua)).unwrap_or(Ok(mlua::Nil))
		});
		
		fields.add_field_method_get("end", |lua, this| {
			this.path.file_name().map(|p| p.into_lua(lua)).unwrap_or(Ok(mlua::Nil))
		});
		fields.add_field_method_set("end", |_, this, name: mlua::String| {
			this.path.set_file_name(&*name.to_str()?);
			Ok(())
		});

		fields.add_field_method_get("name", |lua, this| {
			this.path.file_stem().map(|p| p.into_lua(lua)).unwrap_or(Ok(mlua::Nil))
		});
		fields.add_field_method_set("name", |_, this, name: mlua::String| {
			let name = name.to_str()?;
			let mut name = RelativePathBuf::from(&*name);
			
			if name.extension().is_none() {
				name.set_extension(this.path.extension().unwrap_or_default());
			}

			this.path.set_file_name(&*name);

			Ok(())
		});

		// fields.add_field_method_get("is_dir", |_, this| Ok(this.path.is_dir()));
		// fields.add_field_method_get("is_file", |_, this| Ok(this.path.is_file()));

		// fields.add_field_method_get("is_absolute", |_, this| Ok(this.path.is_absolute()));
		// fields.add_field_method_get("is_relative", |_, this| Ok(this.path.is_relative()));

		// fields.add_field_method_get("exists", |_, this| Ok(this.path.exists()));
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(mlua::MetaMethod::ToString, |lua, this, ()| this.path.as_str().into_lua(lua));

		methods.add_method_mut("push", |_, this, path: mlua::Value| {
			if let Some(path) = path.as_userdata() && path.is::<PathUserData>() {
				let path = path.borrow::<PathUserData>().expect("Verified above");
				this.path.push(&path.path);
			} else if let Some(path) = path.as_str() {
				this.path.push(&*path);
			} else if path.is_nil() {
				
			} else {
				this.path.push(&path.to_string()?);
			}

			Ok(())
		});

		methods.add_function("join", |lua, paths: mlua::MultiValue| {
			let path = join_paths(lua, paths)?;
			path.into_lua(lua)
		});
	}
}

fn join_paths(lua: &mlua::Lua, paths: impl IntoLuaMulti) -> mlua::Result<PathUserData> {
	let mut path = RelativePathBuf::new();
	for p in paths.into_lua_multi(lua)? {
		if let Some(p) = p.as_userdata() {
			let p = p.borrow::<PathUserData>()?;
			path.push(&p.path);
		} else if let Some(p) = p.as_str() {
			path.push(&*p);
		} else if p.is_nil() {

		} else {
			path.push(&p.to_string()?);
		}
	}

	Ok(PathUserData { path })
}

pub trait IntoPathUserData {
	fn to_path_userdata(self) -> PathUserData;
}

// impl<T: Into<PathBuf>> IntoPathUserData for T {
// 	fn to_path_userdata(self) -> PathUserData {
// 		PathUserData::new(self)
// 	}
// }

// impl<T: AsRef<std::path::Path>> IntoPathUserData for T {
// 	fn to_path_userdata(self) -> PathUserData {
// 		PathUserData::new(self)
// 	}
// }
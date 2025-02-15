use mlua::{FromLua, IntoLua, UserData};
use relative_path::{RelativePath, RelativePathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PathUserData {
	path: RelativePathBuf,
}

impl From<RelativePathBuf> for PathUserData {
	fn from(path: RelativePathBuf) -> Self {
		Self { path }
	}
}

impl PathUserData {
	pub const CLASS_NAME: &'static str = "Path";

	/// Should always contain a normalized path.
	pub fn new(path: impl Into<RelativePathBuf>) -> Self {
		Self { path: path.into() }
	}

	pub fn from(path: impl AsRef<RelativePath>) -> Self {
		Self { path: path.as_ref().normalize() }
	}

	pub fn path(&self) -> &RelativePath {
		&self.path
	}

	pub fn into_path(self) -> RelativePathBuf {
		self.path
	}
}

impl FromLua for PathUserData {
	fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
		if let Some(userdata) = value.as_userdata() && userdata.is::<PathUserData>() {
			Ok(userdata.borrow::<PathUserData>()?.clone())
		} else if let Some(value) = value.as_str() {
			Ok(PathUserData::new(RelativePath::new(&value).normalize()))
		} else {
			Err(mlua::Error::runtime(format!("Expected a string or a {CLASS}, got {:?}", value.type_name(), CLASS = PathUserData::CLASS_NAME)))
		}
	}
}

#[allow(clippy::unit_arg)]
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
		
		fields.add_field_method_get("last", |lua, this| {
			this.path.file_name().map(|p| p.into_lua(lua)).unwrap_or(Ok(mlua::Nil))
		});
		fields.add_field_method_set("last", |_, this, name: mlua::String| {
			Ok(this.path.set_file_name(&*name.to_str()?))
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
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(mlua::MetaMethod::ToString, |lua, this, ()|
			this.path.as_str().into_lua(lua)
		);
		methods.add_meta_method(mlua::MetaMethod::Concat, |_, this, other: PathUserData| {
			let mut path = this.path.clone();
			path.push(&other.path);
			Ok(PathUserData::new(path))
		});
		methods.add_meta_method(mlua::MetaMethod::Add, |_, this, other: PathUserData| {
			let mut path = this.path.clone();
			path.push(&other.path);
			Ok(PathUserData::new(path))
		});
		methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: PathUserData|
			// Paths are inherently normalized.
			Ok(this.path == other.path)
		);
		methods.add_meta_method(mlua::MetaMethod::Len, |_, this, ()|
			Ok(this.path.components().count())
		);
		methods.add_meta_method(mlua::MetaMethod::Index, |lua, this, i: usize| {
			let i = i.saturating_sub(1); // Lua is 1-indexed.
			this.path.components().nth(i).map(|c| c.as_str().into_lua(lua)).transpose()
		});

		methods.add_method_mut("push", |_, this, path: mlua::Variadic<Option<PathUserData>>| {
			Ok(path.into_iter().flatten().for_each(|p| this.path.push(&p.path)))
		});

		methods.add_method("parts", |lua, this, ()| {
			let parts = this.path.components().map(|c| lua.create_string(c.as_str())).collect::<Vec<_>>();
			let mut parts = parts.into_iter();
			
			lua.create_function_mut(move |_, ()| parts.next().transpose())
		});

		methods.add_method("strip", |_, this, prefix: Option<PathUserData>| {
			let Some(prefix) = prefix else {
				return Ok(Some(PathUserData::new(&this.path)));
			};
			Ok(this.path.strip_prefix(prefix.path).ok().map(PathUserData::new))
		});

		methods.add_function("join", |_, paths: mlua::Variadic<Option<PathUserData>>| {
			let mut path = RelativePathBuf::new();
			paths.into_iter().flatten().for_each(|p| path.push(&p.path));
			Ok(PathUserData::new(path))
		});

		methods.add_function(super::NEW_FUNCTION, |_, path: Option<PathUserData>| Ok(path));
	}
}

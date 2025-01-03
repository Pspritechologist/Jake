use std::{marker::PhantomData, ops::{Deref, DerefMut}};
use mlua::{AnyUserData, FromLua, IntoLua, Lua, MaybeSend, UserData, UserDataRef};

#[derive(Debug, Clone)]
pub struct TypedUserData<T>(AnyUserData, PhantomData<T>);

pub trait TypedUserDataExt: UserData + MaybeSend + 'static {
	fn to_typed(self, lua: &Lua) -> TypedUserData<Self> {
		TypedUserData::from_data(self, lua)
	}
}

impl<T: UserData + MaybeSend + 'static> TypedUserDataExt for T {}

impl<T: UserData + 'static> TypedUserData<T> {
	pub fn from_userdata(userdata: AnyUserData) -> mlua::Result<Self> {
		if !userdata.is::<T>() {
			return Err(mlua::Error::FromLuaConversionError {
				from: "userdata",
				to: std::any::type_name::<T>().to_string(),
				message: None,
			});
		}

		Ok(TypedUserData(userdata, PhantomData))
	}

	pub fn userdata(&self) -> &AnyUserData {
		&self.0
	}

	pub fn from_data(data: T, lua: &Lua) -> Self where T: MaybeSend {
		TypedUserData(lua.create_userdata(data).expect("Failed to create UserData"), PhantomData)
	}

	pub fn from_ser_data(data: T, lua: &Lua) -> Self where T: MaybeSend + serde::Serialize {
		TypedUserData(lua.create_ser_userdata(data).expect("Failed to create UserData"), PhantomData)
	}

	pub fn borrow<>(&self) -> mlua::Result<UserDataRef<T>> {
		self.0.borrow()
	}

	pub fn borrow_mut<>(&self) -> mlua::Result<mlua::UserDataRefMut<T>> {
		self.0.borrow_mut()
	}
}

impl<T: UserData> IntoLua for TypedUserData<T> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		self.0.into_lua(lua)
	}
}

impl<T: UserData + 'static> FromLua for TypedUserData<T> {
	fn from_lua(value: mlua::Value, lua: &Lua) -> mlua::Result<Self> {
		let type_name = value.type_name();

		let userdata = AnyUserData::from_lua(value, lua)?;

		if !userdata.is::<T>() {
			return Err(mlua::Error::FromLuaConversionError {
				from: type_name,
				to: std::any::type_name::<T>().to_string(),
				message: None,
			});
		}

		Ok(TypedUserData(userdata, PhantomData))
	}
}

impl<T> Deref for TypedUserData<T> {
	type Target = AnyUserData;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for TypedUserData<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

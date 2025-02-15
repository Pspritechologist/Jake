use crate::error::ErrorExtensions;
use mlua::ObjectLike;
use liquid::{model::{DisplayCow, KStringCow, State}, ValueView};

pub fn is_value_default(value: &mlua::Value) -> bool {
	match value {
		mlua::Value::Nil => true,
		mlua::Value::Boolean(b) => !b,
		mlua::Value::Integer(i) => *i == 0,
		mlua::Value::Number(n) => *n == 0.0,
		mlua::Value::String(s) => s.as_bytes().is_empty(),
		mlua::Value::LightUserData(lu) => lu.0.is_null(),
		_ => false,
	}
}

pub fn is_value_blank(value: &mlua::Value) -> bool {
	match value {
		mlua::Value::Nil => true,
		mlua::Value::String(s) => s.as_bytes().is_empty(),
		mlua::Value::Table(t) => t.is_empty(),
		_ => false,
	}
}

fn handle_lua_err<T>(res: mlua::Result<T>) -> Option<T> {
	match res {
		Ok(v) => Some(v),
		// Err(e) => panic!("Lua error in Liquid callback- This is not allowed to happen! - {e}"),
		Err(e) => {
			e.into_error_with("Occurred within Liquid callback - This disallows error handling!").print_error();
			None
		},
	}
}

#[derive(Clone, Debug)]
struct DebugDisplay<T>(T);
impl<T: std::fmt::Debug> std::fmt::Display for DebugDisplay<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.0)
	}
}

#[derive(Debug)]
pub struct LuaValueView(mlua::Value);

impl LuaValueView {
	pub fn new(value: impl mlua::IntoLua, lua: &mlua::Lua) -> mlua::Result<Self> {
		Ok(Self(value.into_lua(lua)?))
	}
}

impl ValueView for LuaValueView {
	fn as_debug(&self) -> &dyn std::fmt::Debug { self }

	fn render(&self) -> DisplayCow<'_> {
		match self.0.to_string() {
			Ok(s) => DisplayCow::Owned(Box::new(s)),
			Err(e) => {
				eprintln!("Error rendering LuaValueView: {e}");
				DisplayCow::Owned(Box::new(String::from("")))
			}
		}
	}

	fn source(&self) -> DisplayCow<'_> {
		DisplayCow::Owned(Box::new(DebugDisplay(&self.0)))
	}

	fn type_name(&self) -> &'static str {
		self.0.type_name()
	}

	fn query_state(&self, state: State) -> bool {
		match state {
			State::Truthy => self.0.as_boolean().unwrap_or_else(|| !self.0.is_nil()),
			State::DefaultValue => is_value_default(&self.0),
			State::Empty => if let mlua::Value::Table(t) = &self.0 { t.is_empty() } else { false },
			State::Blank => is_value_blank(&self.0),
		}
	}

	fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
		KStringCow::from_string(self.0.to_string().unwrap_or(String::from("")))
	}

	fn to_value(&self) -> liquid_core::Value {
		liquid::model::to_value(&self.0).unwrap_or(liquid_core::Value::Nil)
	}
	
	fn as_scalar(&self) -> Option<liquid::model::ScalarCow<'_>> {
		Some(match &self.0 {
			mlua::Value::Boolean(b) => (*b).into(),
			mlua::Value::Integer(i) => (*i).into(),
			mlua::Value::Number(n) => (*n).into(),
			mlua::Value::String(s) => s.to_string_lossy().into(),
			_ => return None,
		})
	}
	
	fn is_scalar(&self) -> bool {
		self.0.is_boolean()
			|| self.0.is_integer()
			|| self.0.is_number()
			|| self.0.is_string()
	}
	
	fn as_array(&self) -> Option<&dyn liquid::model::ArrayView> {
		if !self.0.is_table() && !self.0.is_userdata() {
			return None;
		}

		Some(self)
	}

	fn is_array(&self) -> bool {
		self.0.is_table() || self.0.is_userdata()
	}
	
	fn as_object(&self) -> Option<&dyn liquid::ObjectView> {
		if !self.0.is_table() && !self.0.is_userdata() {
			return None;
		}

		Some(self)
	}

	fn is_object(&self) -> bool {
		self.0.is_table() || self.0.is_userdata()
	}
	
	fn is_nil(&self) -> bool {
		self.0.is_nil()
	}
}

const OBJECT_ERROR_MESSAGE: &str = "Should not be possible to obtain ObjectView from non table or userdata Lua value";

pub fn clear_values() {
	VALUE_STORAGE.with(|s| s.borrow_mut().clear());
}

// This solution sucks.
thread_local! {
	static VALUE_STORAGE: std::cell::RefCell<Vec<LuaValueView>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn store_value<'a>(value: mlua::Value) -> &'a LuaValueView {
	VALUE_STORAGE.with_borrow_mut(|s| {
		let len = s.len();
		s.push(LuaValueView(value));
		unsafe { std::mem::transmute(s.get_unchecked(len)) }
	})
}

impl liquid_core::ObjectView for LuaValueView {
	fn as_value(&self) -> &dyn ValueView { self }

	fn size(&self) -> i64 {
		match &self.0 {
			mlua::Value::Table(t) => t.len().unwrap_or(0),
			mlua::Value::UserData(u) => u.call_method(mlua::MetaMethod::Len.name(), ()).unwrap_or(0),
			_ => unreachable!("{OBJECT_ERROR_MESSAGE}"),
		}
	}

	fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
		match &self.0 {
			mlua::Value::Table(t) => Box::new(t.pairs::<mlua::String, mlua::Value>().filter_map(|r| {
				let (key, _) = handle_lua_err(r)?;
				let key = handle_lua_err(key.to_str())?;
				Some(liquid::model::KStringCow::from_string(key.to_owned()))
			})),
			mlua::Value::UserData(_) => Box::new(std::iter::empty()),
			_ => unreachable!("{OBJECT_ERROR_MESSAGE}"),
		}
	}

	fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
		match &self.0 {
			mlua::Value::Table(t) => {
				Box::new(t.pairs::<mlua::Value, mlua::Value>().filter_map(|r| {
					let (_, value) = handle_lua_err(r)?;
					Some(store_value(value) as &dyn ValueView)
				}))
			},
			mlua::Value::UserData(_) => Box::new(std::iter::empty()),
			_ => unreachable!("{OBJECT_ERROR_MESSAGE}"),
		}
	}

	fn iter<'k>(&'k self) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
		match &self.0 {
			mlua::Value::Table(t) => {
				Box::new(t.pairs::<mlua::String, mlua::Value>().filter_map(|r| {
					let (key, value) = handle_lua_err(r)?;
					Some((
						liquid::model::KStringCow::from_string(handle_lua_err(key.to_str()).map_or(String::default(), |s| s.to_owned())),
						store_value(value) as &dyn ValueView
					))
				}))
			},
			mlua::Value::UserData(_) => Box::new(std::iter::empty()),
			_ => unreachable!("{OBJECT_ERROR_MESSAGE}"),
		}
	}

	fn contains_key(&self, index: &str) -> bool {
		match &self.0 {
			mlua::Value::Table(t) => handle_lua_err(t.contains_key(index)).unwrap_or(false),
			mlua::Value::UserData(u) => !handle_lua_err(u.get::<mlua::Value>(index)).map_or(false, |v| v.is_nil()),
			_ => unreachable!("{OBJECT_ERROR_MESSAGE}"),
		}
	}

	fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
		match &self.0 {
			mlua::Value::Table(t) => {
				let value = handle_lua_err(t.get(index))?;
				Some(store_value(value) as &dyn ValueView)
			},
			mlua::Value::UserData(u) => {
				let value = handle_lua_err(u.get::<mlua::Value>(index))?;
				if value.is_nil() {
					None
				} else {
					Some(store_value(value) as &dyn ValueView)
				}
			},
			_ => unreachable!("{OBJECT_ERROR_MESSAGE}"),
		}
	}
}

const ARRAY_ERROR_MESSAGE: &str = "Should not be possible to obtain ArrayView from non table or userdata Lua value";

impl liquid_core::model::ArrayView for LuaValueView {
	fn as_value(&self) -> &dyn ValueView { self }

	fn size(&self) -> i64 {
		match &self.0 {
			mlua::Value::Table(t) => t.len().unwrap_or(0),
			mlua::Value::UserData(u) => u.call_method(mlua::MetaMethod::Len.name(), ()).unwrap_or(0),
			_ => unreachable!("{ARRAY_ERROR_MESSAGE}"),
		}
	}

	fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
		match &self.0 {
			mlua::Value::Table(t) => {
				Box::new(t.sequence_values().filter_map(|v| {
					let value = handle_lua_err(v)?;
					Some(store_value(value) as &dyn ValueView)
				}))
			},
			mlua::Value::UserData(_) => Box::new(std::iter::empty()),
			_ => unreachable!("{ARRAY_ERROR_MESSAGE}"),
		}
	}

	fn contains_key(&self, index: i64) -> bool {
		// Lua is 1-indexed.
		let index = 1 + if index.is_positive() {
			index
		} else {
			self.size() + index
		};

		match &self.0 {
			mlua::Value::Table(t) => handle_lua_err(t.contains_key(index)).unwrap_or(false),
			mlua::Value::UserData(u) => !handle_lua_err(u.get::<mlua::Value>(index)).map_or(false, |v| v.is_nil()),
			_ => unreachable!("{ARRAY_ERROR_MESSAGE}"),
		}
	}

	fn get(&self, index: i64) -> Option<&dyn ValueView> {
		// Lua is 1-indexed.
		let index = 1 + if index.is_positive() {
			index
		} else {
			self.size() + index
		};

		match &self.0 {
			mlua::Value::Table(t) => {
				let value = handle_lua_err(t.get(index))?;
				Some(store_value(value) as &dyn ValueView)
			},
			mlua::Value::UserData(u) => {
				let value = handle_lua_err(u.get(index))?;
				Some(store_value(value) as &dyn ValueView)
			},
			_ => unreachable!("{ARRAY_ERROR_MESSAGE}"),
		}
	}
}

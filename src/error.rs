use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone)]
pub enum Error {
	Liquid(liquid::Error),
	Lua(mlua::Error),
	Grass(Box<grass::Error>),
	WalkDir(Arc<walkdir::Error>),
	Io(Arc<std::io::Error>),
	Serde(SerdeError)
}

impl std::error::Error for Error {}

impl From<liquid::Error> for Error {
	fn from(e: liquid::Error) -> Self { Error::Liquid(e) }
}

impl From<mlua::Error> for Error {
	fn from(e: mlua::Error) -> Self { Error::Lua(e) }
}

impl From<Box<grass::Error>> for Error {
	fn from(e: Box<grass::Error>) -> Self { Error::Grass(e) }
}

impl From<walkdir::Error> for Error {
	fn from(e: walkdir::Error) -> Self { Error::WalkDir(e.into()) }
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self { Error::Io(e.into()) }
}

impl<T: Into<SerdeError>> From<T> for Error {
	fn from(e: T) -> Self { Error::Serde(e.into()) }
}

#[derive(Debug, Clone)]
pub enum SerdeError {
	Json(Arc<serde_json::Error>),
	Yaml(Arc<serde_yaml::Error>)
}

impl From<serde_json::Error> for SerdeError {
	fn from(e: serde_json::Error) -> Self { SerdeError::Json(e.into()) }
}

impl From<serde_yaml::Error> for SerdeError {
	fn from(e: serde_yaml::Error) -> Self { SerdeError::Yaml(e.into()) }
}

impl Error {
	/// If this error is a Liquid or Lua error containing a
	/// dynamic Error, downcast it to that Error.  
	/// Otherwise, return the error as-is.
	pub fn downcast(&self) -> &Error {
		match self {
			Error::Lua(mlua::Error::ExternalError(e)) if e.is::<Error>()
				=> e.downcast_ref::<Error>().expect("Validated above"),
			liquid @ Error::Liquid(e) => match std::error::Error::source(e).and_then(|e| e.downcast_ref::<Error>()) {
				Some(e) => e,
				None => liquid
			},
			e => e
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.downcast() {
			Error::Lua(e) => write!(f, "Lua error: {e}"),
			Error::Liquid(e) => write!(f, "Liquid error: {e}"),
			Error::Grass(e) => write!(f, "Grass error: {e}"),
			Error::WalkDir(e) => write!(f, "WalkDir error: {e}"),
			Error::Io(e) => write!(f, "IO error: {e}"),
			Error::Serde(SerdeError::Json(e)) => write!(f, "JSON error: {e}"),
			Error::Serde(SerdeError::Yaml(e)) => write!(f, "YAML error: {e}"),
		}
	}
}

impl From<Error> for mlua::Error {
	fn from(value: Error) -> Self {
		match value {
			Error::Lua(e) => e,
			e => mlua::Error::external(e)
		}
	}	
}

impl From<Error> for liquid::Error {
	fn from(e: Error) -> Self {
		match e {
			Error::Liquid(e) => e,
			e => liquid::Error::with_msg(format!("External error -> {e}")).cause(e)
		}
	}
}

pub trait ErrorExtensions: Into<Error> {
	fn into_liquid_error(self) -> liquid::Error {
		self.into().into()
	}

	fn into_lua_error(self) -> mlua::Error {
		mlua::Error::external(self.into())
	}
}

impl<E: Into<Error>> ErrorExtensions for E {}

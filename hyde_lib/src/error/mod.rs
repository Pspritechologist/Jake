mod hyde_error;

pub use hyde_error::HydeError;

use std::{fmt::Display, sync::Arc};

#[derive(Debug, Clone)]
pub enum Error {
	Liquid(liquid::Error),
	Lua(mlua::Error),
	Grass(Box<grass::Error>),
	WalkDir(Arc<walkdir::Error>),
	Io(Arc<std::io::Error>),
	Serde(SerdeError),
	Glob(globset::Error),
	HydeError(HydeError),
	WithContext { context: String, error: Box<Error> },
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

impl From<globset::Error> for Error {
	fn from(e: globset::Error) -> Self { Error::Glob(e) }
}

impl From<HydeError> for Error {
	fn from(e: HydeError) -> Self { Error::HydeError(e) }
}

impl<P: Into<String>, E: Into<Error>> From<(P, E)> for Error {
	fn from((context, error): (P, E)) -> Self {
		Error::WithContext { context: context.into(), error: Box::new(error.into()) }
	}
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
	/// dynamic Error, recursively downcast it to that Error.  
	/// Otherwise, return the error as-is.
	pub fn downcast(&self) -> std::borrow::Cow<Error> {
		match self {
			Error::Lua(mlua::Error::ExternalError(e)) if e.is::<Error>()
				=> e.downcast_ref::<Error>().expect("Validated above").downcast(),
			orig @ Error::Liquid(e) => match std::error::Error::source(e).and_then(|e| e.downcast_ref::<Error>()) {
				Some(e) => e.downcast(),
				None => std::borrow::Cow::Borrowed(orig),
			},
			Error::WithContext { context, error } => std::borrow::Cow::Owned(Error::from((context, error.downcast().into_owned()))),
			e => std::borrow::Cow::Borrowed(e),
		}
	}

	pub fn print_error(self) {
		eprintln!("{}", self.downcast());
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Lua(e) => write!(f, "Lua error: {e}"),
			Error::Liquid(e) => write!(f, "Liquid error: {e}"),
			Error::Grass(e) => write!(f, "Grass error: {e}"),
			Error::WalkDir(e) => write!(f, "WalkDir error: {e}"),
			Error::Io(e) => write!(f, "IO error: {e}"),
			Error::Serde(SerdeError::Json(e)) => write!(f, "JSON error: {e}"),
			Error::Serde(SerdeError::Yaml(e)) => write!(f, "YAML error: {e}"),
			Error::Glob(e) => write!(f, "Glob pattern error: {e}"),
			Error::HydeError(e) => write!(f, "Hyde error: {e}"),
			Error::WithContext { context, error } => write!(f, "{error} (context - {context})"),
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
		self.into().into()
	}

	fn into_error(self) -> Error {
		self.into()
	}

	fn into_error_with(self, file: impl LazyContext) -> Error {
		Error::WithContext { context: file.eval(), error: Box::new(self.into_error()) }
	}

	fn print_as_error(self) {
		self.into_error().print_error();
	}
}

impl<E: Into<Error>> ErrorExtensions for E {}

pub trait ResultExtensions<T> {
	fn into_liquid_result(self) -> Result<T, liquid::Error>;
	fn into_lua_result(self) -> Result<T, mlua::Error>;
	fn into_error_result(self) -> Result<T, Error>;
	fn into_error_result_with(self, context: impl LazyContext) -> Result<T, Error> where Self: Sized;
	fn handle_as_error(self) where Self: Sized {
		if let Err(e) = self.into_error_result() {
			e.print_error();
		}
	}
}

impl<E: ErrorExtensions, T> ResultExtensions<T> for Result<T, E> {
	fn into_liquid_result(self) -> Result<T, liquid::Error> {
		self.map_err(E::into_liquid_error)
	}

	fn into_lua_result(self) -> Result<T, mlua::Error> {
		self.map_err(E::into_lua_error)
	}

	fn into_error_result(self) -> Result<T, Error> {
		self.map_err(E::into_error)
	}

	fn into_error_result_with(self, context: impl LazyContext) -> Result<T, Error> where Self: Sized {
		self.map_err(|e| e.into_error_with(context))
	}
}

pub trait LazyContext {
    fn eval(self) -> String;
}

impl LazyContext for String {
    fn eval(self) -> String { self }
}

impl LazyContext for &str {
	fn eval(self) -> String { self.to_string() }
}

impl<T: FnOnce() -> S, S: Into<String>> LazyContext for T {
    fn eval(self) -> String {
        self().into()
    }
}

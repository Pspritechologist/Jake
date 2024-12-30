#[derive(Debug)]
pub enum Error {
	Liquid(liquid::Error),
	Lua(mlua::Error),
	Grass(Box<grass::Error>),
	WalkDir(walkdir::Error),
	Io(std::io::Error),
	Serde(SerdeError)
}

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
	fn from(e: walkdir::Error) -> Self { Error::WalkDir(e) }
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self { Error::Io(e) }
}

impl<T: Into<SerdeError>> From<T> for Error {
	fn from(e: T) -> Self { Error::Serde(e.into()) }
}

#[derive(Debug)]
pub enum SerdeError {
	Json(serde_json::Error),
	Yaml(serde_yaml::Error)
}

impl From<serde_json::Error> for SerdeError {
	fn from(e: serde_json::Error) -> Self { SerdeError::Json(e) }
}

impl From<serde_yaml::Error> for SerdeError {
	fn from(e: serde_yaml::Error) -> Self { SerdeError::Yaml(e) }
}

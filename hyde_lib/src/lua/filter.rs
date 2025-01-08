use super::*;
use mlua::LuaSerdeExt;
use std::collections::HashMap;
use liquid_core::{
	parser::{FilterArguments, ParameterReflection},
	Expression,
	Filter,
	FilterReflection,
	ParseFilter,
	Runtime,
	Value,
	ValueView
};

#[derive(Debug, Clone)]
pub struct Lua {
	pub filter: String,
	pub func: mlua::Function,
	pub lua: mlua::Lua,
}

impl FilterReflection for Lua {
	fn name(&self) -> &str {
		&self.filter
	}

	fn description(&self) -> &str {
		"Custom filter registered from Lua"
	}

	fn positional_parameters(&self) -> &'static [ParameterReflection] { &[] }
	fn keyword_parameters(&self) -> &'static [ParameterReflection] { &[] }
}

impl ParseFilter for Lua {
	fn parse(&self, arguments: FilterArguments) -> liquid_core::Result<Box<dyn Filter>> {
		Ok(Box::new(LuaFilter {
			filter: self.filter.clone(),
			func: self.func.clone(),
			pos_args: arguments.positional.collect(),
			key_args: arguments.keyword.map(|(k, v)| (k.to_string(), v)).collect(),
			lua: self.lua.clone(),
		}))
	}

	fn reflection(&self) -> &dyn FilterReflection {
		self
	}
}

#[derive(Debug, Clone)]
pub struct LuaFilter {
	filter: String,
	func: mlua::Function,
	pos_args: Vec<Expression>,
	key_args: HashMap<String, Expression>,
	lua: mlua::Lua,
}

impl std::fmt::Display for LuaFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.filter)
	}
}

impl Filter for LuaFilter {
    fn evaluate(&self, input: &dyn ValueView, runtime: &dyn Runtime) -> liquid_core::Result<Value> {
		let pos_args: mlua::MultiValue = self.pos_args.iter()
			.map(|v| v.evaluate(runtime))
			.map(|v| self.lua.to_value(&v?.into_owned()).map_err(Error::from))
			.try_collect()?;

		let input = self.lua.to_value(&input.to_value().to_owned()).map_err(Error::from)?;

		let key_args: HashMap<_, _> = self.key_args.iter().map(|(k, v)| (k, v.evaluate(runtime)))
			.map(|(k, v)| Ok::<_, Error>((k.clone(), self.lua.to_value(&v?.into_owned())?)))
			.try_collect()?;

		let key_args = self.lua.create_table_from(key_args).map_err(Error::from)?;

		let result: mlua::Value = self.func.call((input, key_args, pos_args)).map_err(Error::from)?;

		let result = liquid::model::to_value(&result)?;

		Ok(result)
    }
}

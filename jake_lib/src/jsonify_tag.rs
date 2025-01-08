use crate::error::ResultExtensions;
use liquid_core::{
	parser::{FilterArguments, ParameterReflection},
	Filter,
	FilterReflection,
	ParseFilter,
	Runtime,
	Value,
	ValueView
};

#[derive(Debug, Clone)]
pub struct Jsonify;

impl FilterReflection for Jsonify {
	fn name(&self) -> &str {
		"jsonify"
	}

	fn description(&self) -> &str {
		"Converts a value to a JSON string."
	}

	fn positional_parameters(&self) -> &'static [ParameterReflection] { &[] }
	fn keyword_parameters(&self) -> &'static [ParameterReflection] { &[] }
}

impl ParseFilter for Jsonify {
	fn parse(&self, _arguments: FilterArguments) -> liquid_core::Result<Box<dyn Filter>> {
		Ok(Box::new(JsonifyFilter))
	}

	fn reflection(&self) -> &dyn FilterReflection {
		self
	}
}

#[derive(Debug, Clone)]
pub struct JsonifyFilter;

impl std::fmt::Display for JsonifyFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "jsonify")
	}
}

impl Filter for JsonifyFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> liquid_core::Result<Value> {
		let arg = serde_json::to_string(&input.to_value()).into_liquid_result()?;
		Ok(Value::scalar(arg))
    }
}

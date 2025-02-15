use crate::error::ResultExtensions;
use liquid_core::{
	Display_filter,
	Filter,
	FilterReflection,
	ParseFilter,
	Runtime,
	Value,
	ValueView,
};

#[derive(Debug, Clone, FilterReflection, ParseFilter)]
#[filter(
	name="jsonify",
	description="Converts a value to a JSON string.",
	parsed(JsonifyFilter),
)]
pub struct Jsonify;

#[derive(Debug, Clone, Default, Display_filter)]
#[name="jsonify"]
pub struct JsonifyFilter;

impl Filter for JsonifyFilter {
    fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> liquid_core::Result<Value> {
		let arg = serde_json::to_string(&input.to_value()).into_liquid_result()?;
		Ok(Value::scalar(arg))
    }
}

#[derive(Debug, Clone, FilterReflection, ParseFilter)]
#[filter(
	name="render",
	description="Renders Markdown into HTML.",
	parsed(RenderFilter),
)]
pub struct Render;

#[derive(Debug, Clone, Default, Display_filter)]
#[name="render"]
pub struct RenderFilter;

impl Filter for RenderFilter {
	fn evaluate(&self, input: &dyn ValueView, _runtime: &dyn Runtime) -> liquid_core::Result<Value> {
		let html = crate::lua::general_api::formatting::render_markdown(&input.to_kstr());
		Ok(Value::scalar(html))
	}
}

use jaq_core::{
  load::{Arena, File, Loader},
  Compiler, Ctx, Native, RcIter,
};
use jaq_json::Val;
use serde_json::Value;

pub type Filter = jaq_core::Filter<Native<Val>>;

#[derive(thiserror::Error, Debug)]
pub enum JqError {
  #[error("Interpret error: {0}")]
  InterpretError(String),
  #[error("Parse error: {0:?}")]
  ParseError(String),
  #[error("Unable to parse")]
  UnableToParse,
  #[error("Conversion error: {0}")]
  Infallible(#[from] std::convert::Infallible),
}

impl From<jaq_core::Error<Val>> for JqError {
  fn from(value: jaq_core::Error<Val>) -> Self {
    JqError::InterpretError(value.to_string())
  }
}

pub struct JqFilter {
  filter: Filter,
}

impl JqFilter {
  pub fn new(query: &str) -> Result<Self, JqError> {
    let filter = make_jq_filter(query)?;
    Ok(Self { filter })
  }

  pub fn run(&self, json: Value) -> Result<Vec<Value>, JqError> {
    let jq_val: Val = json.into();

    let null = Box::new(core::iter::once(Ok(Val::Null))) as Box<dyn Iterator<Item = _>>;
    let null = RcIter::new(null);
    let null_ctx = Ctx::new(vec![], &null);

    let results = self
      .filter
      .run((null_ctx.clone(), jq_val))
      .map(|x| x.map(Into::into))
      .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
  }
}

pub fn make_jq_filter(query: &str) -> Result<Filter, JqError> {
  let program = File { code: query, path: () };

  let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
  let arena = Arena::default();
  let modules = loader.load(&arena, program).map_err(|_| JqError::UnableToParse)?;
  let filter = Compiler::default()
    .with_funs(jaq_std::funs().chain(jaq_json::funs()))
    .compile(modules)
    .map_err(|e| {
      for (file, e) in e {
        tracing::error!("Error while compiling: `{:?}`: \n {e:?}", file.code);
      }
      JqError::UnableToParse
    })?;

  Ok(filter)
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use test_case::test_case;

  #[test_case(r#"{ "answer": 42 }"#, ".answer", &["42"])]
  #[test_case(
    r#"{"fruit":{"name":"apple","color":"green","price":1.20}}"#,
    ".",
    &[r#"{"fruit":{"name":"apple","color":"green","price":1.20}}"#]
  )]
  #[test_case(
    r#"{"fruit":{"name":"apple","color":"green","price":1.20}}"#,
    "del(.fruit.name)",
    &[r#"{"fruit":{"color":"green","price":1.20}}"#]
  )]
  #[test_case(r#"{"fruit":{"name":"apple","color":"green","price":1.20}}"#, ".fruit.name", &[r#""apple""#])]
  #[test_case(r#"{"fruit":{"name":"apple","color":"green","price":1.20}}"#, ".fruit.price", &[r"1.20"])]
  #[test_case(r#"["x","y","z"]"#, ".[]", &[r#""x""#, r#""y""#, r#""z""#])]
  #[test_case(
    r#"[{"product":{"name":"apple","color":"green","price":1.20}}]"#,
    ".[] | .product.name",
    &[r#""apple""#]
  )]
  #[test_case(
    r#"[{"product":{"name":"apple","color":"green","price":1.20}}]"#,
    ".[].product.color",
    &[r#""green""#]
  )]
  #[test_case(
    r#"{"fruit":{"name":"apple","color":"green","price":1.20}}"#,
    r".fruit | keys[]",
    &[r#""color""#,r#""name""#,r#""price""#]
  )]
  fn jq_test(json: &str, query: &str, expected: &[&str]) -> anyhow::Result<()> {
    pretty_env_logger::try_init().ok();

    let json: Value = serde_json::from_str(json)?;
    let expected: Vec<Value> = expected.iter().map(|s| serde_json::from_str(s)).collect::<Result<_, _>>()?;

    let result = JqFilter::new(query)?.run(json)?;
    assert_eq!(expected, result);

    Ok(())
  }
}

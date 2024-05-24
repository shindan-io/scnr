// helping examples :
// see here : https://github.com/01mf02/jaq/blob/main/jaq-interpret/tests/common/mod.rs
// and here : https://github.com/01mf02/jaq/blob/c776647e66e3c481a505bd34e333678acb0141d8/jaq/src/main.rs#L402

use jaq_interpret::{Ctx, FilterT, RcIter, Val};
use serde_json::Value;

pub type Filter = jaq_syn::Main;

#[derive(thiserror::Error, Debug)]
pub enum JqError {
  #[error("Interpret error: {0}")]
  InterpretError(String),
  #[error("Parse error: {0:?}")]
  ParseError(String),
  #[error("Unable to parse")]
  UnableToParse,
}

impl From<Vec<jaq_parse::Error>> for JqError {
  fn from(value: Vec<jaq_parse::Error>) -> Self {
    let string = value.into_iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ");
    JqError::ParseError(string)
  }
}

impl From<jaq_interpret::Error> for JqError {
  fn from(value: jaq_interpret::Error) -> Self {
    JqError::InterpretError(value.to_string())
  }
}

pub struct JqFilter {
  owned: jaq_interpret::Filter,
}

impl JqFilter {
  pub fn new(query: &str) -> Result<Self, JqError> {
    let filter = make_jq_filter(query)?;
    Ok(Self::from_filter(filter))
  }

  pub fn from_filter(filter: Filter) -> Self {
    let mut ctx = make_default_context();
    let owned = ctx.compile(filter);

    Self { owned }
  }

  pub fn run(&self, json: Value) -> Result<Vec<Value>, JqError> {
    let jq_val: Val = json.into();

    let null = Box::new(core::iter::once(Ok(Val::Null))) as Box<dyn Iterator<Item = _>>;
    let null = RcIter::new(null);
    let null_ctx = Ctx::new(vec![], &null);

    let results = self
      .owned
      .run((null_ctx.clone(), jq_val))
      .map(|x| x.map(Into::into))
      .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
  }
}

pub fn make_jq_filter(query: &str) -> Result<Filter, JqError> {
  let (main, errs) = jaq_parse::parse(query, jaq_parse::main());
  if !errs.is_empty() {
    return Err(errs.into());
  }
  let Some(main) = main else {
    return Err(JqError::UnableToParse);
  };
  Ok(main)
}

fn make_default_context() -> jaq_interpret::ParseCtx {
  let mut ctx = jaq_interpret::ParseCtx::new(Vec::new());
  ctx.insert_natives(jaq_core::core());
  ctx.insert_defs(jaq_std::std());
  ctx
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
    let json: Value = serde_json::from_str(json)?;
    let expected: Vec<Value> = expected.iter().map(|s| serde_json::from_str(s)).collect::<Result<_, _>>()?;

    let result = JqFilter::new(query)?.run(json)?;
    assert_eq!(expected, result);

    Ok(())
  }
}

use jaq_core::{
  Ctx, Vars,
  data::JustLut,
  load::{Arena, File, Loader},
};
use jaq_json::{Num, Val};
use serde::Deserialize;
use serde_json::Value;

pub type Filter = jaq_core::Filter<JustLut<Val>>;

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
    let jq_val = Val::deserialize(json).map_err(|e| JqError::ParseError(e.to_string()))?;

    let ctx = Ctx::<JustLut<Val>>::new(&self.filter.lut, Vars::new([]));

    let results = self
      .filter
      .id
      .run((ctx, jq_val))
      .map(|x| {
        x.map(val_to_json).map_err(|e| match e.get_err() {
          Ok(e) => JqError::from(e),
          Err(e) => match e.get_halt() {
            Ok(code) => JqError::InterpretError(format!("halted with exit code {code}")),
            Err(_) => JqError::InterpretError("unexpected exception".to_string()),
          },
        })
      })
      .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
  }
}

fn val_to_json(val: Val) -> Value {
  match val {
    Val::Null => Value::Null,
    Val::Bool(b) => Value::Bool(b),
    Val::Num(n) => num_to_json(&n),
    Val::TStr(b) | Val::BStr(b) => Value::String(String::from_utf8_lossy(&b).into_owned()),
    Val::Arr(a) => Value::Array(a.iter().cloned().map(val_to_json).collect()),
    Val::Obj(o) => Value::Object(
      o.iter()
        .map(|(k, v)| {
          let k = match k {
            Val::TStr(b) | Val::BStr(b) => String::from_utf8_lossy(b).into_owned(),
            k => k.to_string(),
          };
          (k, val_to_json(v.clone()))
        })
        .collect(),
    ),
  }
}

fn num_to_json(n: &Num) -> Value {
  match n {
    Num::Int(i) => Value::from(*i),
    Num::Float(f) => serde_json::Number::from_f64(*f).map_or(Value::Null, Value::Number),
    // big ints and decimals: rely on serde_json parsing, fallback to float or null
    n => {
      let s = n.to_string();
      serde_json::from_str::<serde_json::Number>(&s).map_or(Value::Null, Value::Number)
    }
  }
}

pub fn make_jq_filter(query: &str) -> Result<Filter, JqError> {
  let program = File { code: query, path: () };

  let loader = Loader::new(jaq_core::defs().chain(jaq_std::defs()).chain(jaq_json::defs()));
  let arena = Arena::default();
  let modules = loader.load(&arena, program).map_err(|_| JqError::UnableToParse)?;
  let filter = jaq_core::Compiler::default()
    .with_funs(jaq_core::funs().chain(jaq_std::funs()).chain(jaq_json::funs()))
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

  // conversion edge cases introduced with jaq 3 (manual Val -> serde_json conversion)
  #[test_case("null", "nan", &["null"] ; "nan becomes null")]
  #[test_case("null", "infinite", &["null"] ; "infinity becomes null")]
  #[test_case("null", "-infinite", &["null"] ; "negative infinity becomes null")]
  #[test_case("null", "1.5", &["1.5"] ; "float")]
  #[test_case("null", "-42", &["-42"] ; "negative int")]
  #[test_case("null", "123456789012345678901234567890", &["123456789012345678901234567890"] ; "big int")]
  #[test_case(r#"{"n": 1.20}"#, ".n", &["1.20"] ; "decimal input")]
  #[test_case("null", r#""abc" | tobytes"#, &[r#""abc""#] ; "byte string becomes string")]
  #[test_case("null", r#""é😀""#, &[r#""é😀""#] ; "utf8 string")]
  #[test_case("null", "{(1): 2, (null): 3}", &[r#"{"1": 2, "null": 3}"#] ; "non string keys are stringified")]
  #[test_case("null", "empty", &[] ; "empty output")]
  fn jq_conversion_test(json: &str, query: &str, expected: &[&str]) -> anyhow::Result<()> {
    jq_test(json, query, expected)
  }

  #[test]
  fn jq_object_key_order_is_preserved() -> anyhow::Result<()> {
    let result = JqFilter::new(".")?.run(serde_json::from_str(r#"{"b":1,"a":2}"#)?)?;
    let keys: Vec<_> = result[0].as_object().unwrap().keys().cloned().collect();
    assert_eq!(vec!["b", "a"], keys);
    Ok(())
  }

  #[test_case("halt" ; "halt")]
  #[test_case("halt_error" ; "halt_error")]
  #[test_case(r#""bye" | halt_error(3)"#; "halt_error with code")]
  fn jq_halt_returns_error_instead_of_exiting(query: &str) -> anyhow::Result<()> {
    let result = JqFilter::new(query)?.run(Value::Null);
    assert!(matches!(&result, Err(JqError::InterpretError(msg)) if msg.starts_with("halted with exit code")), "{result:?}");
    Ok(())
  }

  #[test_case(r#"error("boom")"#, "boom" ; "user error")]
  #[test_case(".a | keys", "" ; "type error")]
  fn jq_runtime_error(query: &str, contains: &str) -> anyhow::Result<()> {
    let result = JqFilter::new(query)?.run(serde_json::json!({"a": 1}));
    assert!(matches!(&result, Err(JqError::InterpretError(msg)) if msg.contains(contains)), "{result:?}");
    Ok(())
  }

  #[test]
  fn jq_invalid_query() {
    assert!(matches!(JqFilter::new(".[[["), Err(JqError::UnableToParse)));
  }
}

#![allow(clippy::default_trait_access, clippy::module_name_repetitions, clippy::wildcard_imports)]

use plist::{from_reader, Value};
use scnr_core::*;
use serde_json::{Map, Number};

#[derive(Debug)]
pub struct PlistPlugin;

impl ScanPlugin for PlistPlugin {
  #[tracing::instrument(err)]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    let seekable = reader.into_seekable()?;
    let json = from_reader::<_, Value>(seekable)?;
    let content = Content::Json(plist_to_json(json, context.bin_repr, context.date_repr)?);
    context.send_content(content)?;
    Ok(())
  }
}

fn plist_to_json(plist: Value, bin_repr: BinRepr, date_repr: DateRepr) -> Result<serde_json::Value, ScanError> {
  use serde_json::Value as J;
  Ok(match plist {
    Value::Array(a) => J::Array(
      a.into_iter()
        .map(|v| plist_to_json(v, bin_repr, date_repr))
        .collect::<Result<Vec<_>, _>>()?,
    ),
    Value::Dictionary(d) => J::Object(
      d.into_iter()
        .map(|(k, v)| plist_to_json(v, bin_repr, date_repr).map(|v| (k, v)))
        .collect::<Result<Map<_, _>, _>>()?,
    ),
    Value::Boolean(b) => J::Bool(b),
    Value::Data(bytes) => J::String(bin_repr.to_string(&bytes)),
    Value::Date(d) => J::String(date_repr.to_string(d.into())?),
    Value::Real(f) => Number::from_f64(f).map_or(J::Null, Into::into),
    Value::Integer(i) => {
      if let Some(i) = i.as_unsigned() {
        J::Number(i.into())
      } else if let Some(i) = i.as_signed() {
        J::Number(i.into())
      } else {
        J::Null
      }
    }
    Value::String(s) => J::String(s),
    Value::Uid(uid) => J::Number(uid.get().into()),
    _ => J::Null,
  })
}

#[cfg(test)]
mod tests {
  // use super::*;

  // #[test]
  // fn it_works() {
  //   let result = add(2, 2);
  //   assert_eq!(result, 4);
  // }
}

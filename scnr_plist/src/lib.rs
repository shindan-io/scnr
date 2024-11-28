#![allow(clippy::default_trait_access, clippy::module_name_repetitions, clippy::wildcard_imports)]
#![deny(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use plist::{from_reader, Value};
use scnr_core::*;
use serde_json::{Map, Number};

#[derive(Debug)]
pub struct PlistPlugin;

impl ScanPlugin for PlistPlugin {
  #[tracing::instrument(level = "debug", err)]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    let seekable = reader.into_seekable()?;
    let plist_value = from_reader::<_, Value>(seekable)?;
    let content = Content::Json(plist_to_json(plist_value, context.bin_repr, context.date_repr)?);
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
  use super::*;
  use crate::{
    tests_helpers::{exec_plugin_scan, get_samples_path},
    ScanReader,
  };

  fn get_plist_content(sample_path: &str) -> anyhow::Result<ScanContent> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/{sample_path}"))?;
    let results = exec_plugin_scan(ScanReader::read_seek(&mut file), &PlistPlugin)?;
    assert_eq!(results.len(), 1);
    let result = results.into_iter().next().expect("?")?;
    Ok(result)
  }

  #[test]
  fn test_xml() -> anyhow::Result<()> {
    let result = get_plist_content("sampled.xml.plist");

    let Ok(ScanContent { rel_path, content: Content::Json(_json) }) = result else {
      anyhow::bail!("Expected a json content, got {:?}", result)
    };
    assert_eq!(rel_path.as_os_str(), "");

    Ok(())
  }

  #[test]
  fn test_bin() -> anyhow::Result<()> {
    let result = get_plist_content("sampled.plist");

    let Ok(ScanContent { rel_path, content: Content::Json(_json) }) = result else {
      anyhow::bail!("Expected a json content, got {:?}", result)
    };
    assert_eq!(rel_path.as_os_str(), "");

    Ok(())
  }

  #[test]
  fn cmp_bin_and_xml() -> anyhow::Result<()> {
    let result_xml = get_plist_content("sampled.xml.plist")?;
    let result_bin = get_plist_content("sampled.plist")?;

    assert_eq!(result_xml.rel_path, result_bin.rel_path);
    assert_eq!(result_xml.content, result_bin.content);

    Ok(())
  }

  #[test]
  fn failing_test() -> anyhow::Result<()> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/w.tar.gz"))?;

    let result = exec_plugin_scan(ScanReader::read_seek(&mut file), &PlistPlugin);
    assert!(result.is_err());

    Ok(())
  }

  #[test]
  fn bad_file_fail_test() -> anyhow::Result<()> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/bad_file.plist"))?;

    let result = exec_plugin_scan(ScanReader::read_seek(&mut file), &PlistPlugin);
    assert!(result.is_err());

    Ok(())
  }

  #[test]
  fn should_fail_on_read_bad_plist_content() -> anyhow::Result<()> {
    let bad_plist = b"not a plist";
    let reader = std::io::Cursor::new(bad_plist);
    let plist_value = plist::from_reader::<_, plist::Value>(reader);
    assert!(plist_value.is_err());
    // assert_eq!(format!("{plist_value:?}"), "String(\"not\")");
    Ok(())
  }
}

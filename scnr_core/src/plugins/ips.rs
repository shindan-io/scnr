use super::*;
use serde::Serialize;

#[derive(Debug)]
pub struct IpsPlugin;

#[derive(Debug, Serialize)]
struct IpsValue {
  pub meta: Option<serde_json::Value>,
  pub data: Option<serde_json::Value>,
}

impl ScanPlugin for IpsPlugin {
  #[tracing::instrument(level = "debug", err)]
  fn scan(&self, context: &ScanContext, mut reader: ScanReader<'_>) -> ScanPluginResult {
    // read the first line
    let mut file_content = String::new();
    reader.read_to_string(&mut file_content)?;

    let (line, meta_json) = match file_content.lines().next() {
      Some(line) => (line, serde_json::from_str(line)?),
      None => ("", None),
    };

    let rest = &file_content[line.len()..];
    let data_json = serde_json::from_str(rest)?;

    let value = IpsValue { meta: meta_json, data: data_json };
    let json = serde_json::to_value(value)?;
    let content = Content::Json(json);
    context.send_content(content)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tests_helpers::exec_plugin_scan;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  #[test]
  fn test() -> anyhow::Result<()> {
    let content = r#"{ "prop": "value" }
     { 
      "prop2": 
        "value2" 
     }
    "#
    .bytes()
    .collect::<Vec<_>>();

    let results = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), &IpsPlugin)?;
    assert_eq!(results.len(), 1);

    let result = results.into_iter().next().expect("?")?;
    assert_eq!(result.content, Content::Json(json! ( {"meta":  { "prop": "value" }, "data": { "prop2": "value2" }})));

    Ok(())
  }

  #[test]
  fn failing_test() {
    let content = r"not_json".bytes().collect::<Vec<_>>();
    let result = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), &IpsPlugin);
    assert!(result.is_err());
  }
}

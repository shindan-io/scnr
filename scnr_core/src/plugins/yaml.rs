use super::*;

#[derive(Debug)]
pub struct YamlPlugin;

impl ScanPlugin for YamlPlugin {
  #[tracing::instrument(err)]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    let json: serde_json::Value = serde_yaml::from_reader(reader)?;
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
    let content = r#"prop: "value""#.bytes().collect::<Vec<_>>();

    let results = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), &YamlPlugin)?;
    assert_eq!(results.len(), 1);

    let result = results.into_iter().next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.content == Content::Json(json! ( { "prop": "value" }))));

    Ok(())
  }

  #[test]
  fn failing_test() {
    let content = r"not :  yaml  : ".bytes().collect::<Vec<_>>();
    let result = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), &YamlPlugin);
    assert!(result.is_err());
  }
}

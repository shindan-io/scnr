use quickxml_to_serde::xml_str_to_json;

use super::*;

#[derive(Debug)]
pub struct XmlPlugin;

impl ScanPlugin for XmlPlugin {
  #[tracing::instrument(level = "debug", err)]
  fn scan(&self, context: &ScanContext, mut reader: ScanReader<'_>) -> ScanPluginResult {
    let mut xml = String::new();
    reader.read_to_string(&mut xml)?;
    let json = xml_str_to_json(&xml, &Default::default())?;
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
    let content = r"<prop>value</prop>".bytes().collect::<Vec<_>>();

    let results = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), &XmlPlugin)?;
    assert_eq!(results.len(), 1);

    let result = results.into_iter().next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.content == Content::Json(json! ( { "prop": "value" }))));

    Ok(())
  }

  #[test]
  fn failing_test() {
    let content = r"not_xml".bytes().collect::<Vec<_>>();
    let result = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), &XmlPlugin);
    assert!(result.is_err());
  }
}

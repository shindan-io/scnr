use super::*;

#[derive(Debug)]
pub struct TextPlugin;

impl ScanPlugin for TextPlugin {
  #[tracing::instrument(level = "debug", err)]
  fn scan(&self, context: &ScanContext, mut reader: ScanReader<'_>) -> ScanPluginResult {
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes).to_string();
    let content = Content::Text(text);
    context.send_content(content)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tests_helpers::exec_plugin_scan;
  use pretty_assertions::assert_eq;

  #[test]
  fn test() -> anyhow::Result<()> {
    let content = "test".bytes().collect::<Vec<_>>();

    let results = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), &TextPlugin)?;
    assert_eq!(results.len(), 1);

    let result = results.into_iter().next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.content == Content::Text("test".into())));

    Ok(())
  }
}

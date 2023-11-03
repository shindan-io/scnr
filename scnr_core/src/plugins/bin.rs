use super::*;

/// This plugin acts exactly like the last resort plugin, but it never says `true` to the `can_scan` question.
/// So it's never executed.
/// Unless you specify some configuration to force some binary export of some patterns (see `scnr --help`).
#[derive(Debug)]
pub struct BinPlugin;

impl ScanPlugin for BinPlugin {
  #[tracing::instrument(skip(reader))]
  fn scan(&self, context: &ScanContext, mut reader: ScanReader<'_>) -> ScanPluginResult {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let content = Content::Bytes(bytes);
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

    let results = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), BinPlugin)?;
    assert_eq!(results.len(), 1);

    let result = results.into_iter().next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.content == Content::Bytes(content)));

    Ok(())
  }
}

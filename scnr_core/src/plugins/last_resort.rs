use super::{bin::BinPlugin, *};

/// This plugin acts exactly like the bin plugin, but it always says `true` to the `can_scan` question.
/// So it's always executed.
#[derive(Debug)]
pub struct LastResortPlugin;

impl ScanPlugin for LastResortPlugin {
  #[tracing::instrument(level = "debug", skip(reader))]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    BinPlugin.scan(context, reader)
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

    let results = exec_plugin_scan(ScanReader::read_only(&mut content.as_slice()), &LastResortPlugin)?;
    assert_eq!(results.len(), 1);

    let result = results.into_iter().next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.content == Content::Bytes(content)));

    Ok(())
  }
}

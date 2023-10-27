use super::{bin::BinPlugin, *};

/// This plugin acts exactly like the bin plugin, but it always says `true` to the `can_scan` question.
/// So it's always executed.
#[derive(Debug)]
pub struct LastResortPlugin;

impl ScanPlugin for LastResortPlugin {
  #[tracing::instrument(skip(reader))]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    BinPlugin.scan(context, reader)
  }
}

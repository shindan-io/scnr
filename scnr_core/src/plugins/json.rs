use super::*;

#[derive(Debug)]
pub struct JsonPlugin;

impl ScanPlugin for JsonPlugin {
  #[tracing::instrument(err)]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    let json: serde_json::Value = serde_json::from_reader(reader).unwrap();
    let content = Content::Json(json);
    context.send_content(content)?;
    Ok(())
  }
}

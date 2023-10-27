use super::*;

#[derive(Debug)]
pub struct TextPlugin;

impl ScanPlugin for TextPlugin {
  #[tracing::instrument(err)]
  fn scan(&self, context: &ScanContext, mut reader: ScanReader<'_>) -> ScanPluginResult {
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes).to_string();
    let content = Content::Text(text);
    context.send_content(content)?;
    Ok(())
  }
}

use super::*;

#[derive(Debug)]
pub struct TarGzPlugin;

impl ScanPlugin for TarGzPlugin {
  fn can_recurse(&self) -> bool {
    true
  }

  #[tracing::instrument(skip(reader))]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    let tar = flate2::read::GzDecoder::new(reader);
    let mut archive = tar::Archive::new(tar);

    for entry in archive.entries()? {
      let mut entry = entry?;
      if entry.header().entry_type() != tar::EntryType::file() {
        continue;
      }
      let path = entry.path()?.to_path_buf();
      context.recurse(path, ScanReader::read_only(&mut entry))?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {}

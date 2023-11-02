use std::io::BufReader;

use super::*;

#[derive(Debug)]
pub struct TarXzPlugin;

impl ScanPlugin for TarXzPlugin {
  fn can_recurse(&self) -> bool {
    true
  }

  #[tracing::instrument(skip(reader))]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    let mut reader = BufReader::new(reader);

    let mut decomp: Vec<u8> = Vec::new();
    lzma_rs::xz_decompress(&mut reader, &mut decomp)?;

    let mut archive = tar::Archive::new(&decomp[..]);

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
mod tests {
  //
  // TODO: add tests
  //
}

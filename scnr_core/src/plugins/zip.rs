use super::*;

#[derive(Debug)]
pub struct ZipPlugin;

impl ScanPlugin for ZipPlugin {
  fn can_recurse(&self) -> bool {
    true
  }

  #[tracing::instrument(skip(reader))]
  fn scan(&self, context: &ScanContext, reader: ScanReader<'_>) -> ScanPluginResult {
    // the BufReader here adapt from not Sized to Sized in order to be able to read
    let mut reader = reader.into_seekable()?;

    let mut zip = ::zip::ZipArchive::new(&mut reader).unwrap();

    for i in 0..zip.len() {
      let mut entry = zip.by_index(i).unwrap();
      if entry.is_dir() {
        continue;
      }
      let file_name = entry.name().to_string();
      let readonly_scan_reader = ScanReader::read_only(&mut entry);
      context.recurse(file_name, readonly_scan_reader)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{env, path::Path};

  #[test]
  fn read_a_simple_zip() -> anyhow::Result<()> {
    let dir = env::var("CARGO_MANIFEST_DIR")?;
    let zip_path = Path::new(&dir).join("samples/z.zip");

    let zip = std::fs::File::open(zip_path)?;
    let mut zip = ::zip::ZipArchive::new(zip)?;

    for i in 0..zip.len() {
      let file = zip.by_index(i)?;
      println!("Filename: {}", file.name());
    }

    Ok(())
  }
}

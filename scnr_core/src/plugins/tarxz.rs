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
  use super::*;
  use crate::{
    tests_helpers::{exec_plugin_scan, get_samples_path},
    ScanReader,
  };

  #[test]
  fn test() -> anyhow::Result<()> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/y.tar.xz"))?;

    let results = exec_plugin_scan(ScanReader::read_seek(&mut file), &TarXzPlugin)?;
    assert_eq!(results.len(), 2);

    let mut iter = results.into_iter();

    let result = iter.next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.rel_path.as_os_str() == "y/c.txt"));

    let result = iter.next().expect("?");
    assert!(matches!(result, Ok(scan) if scan.rel_path.as_os_str() == "y/z.zip"));

    Ok(())
  }

  #[test]
  fn failing_test() -> anyhow::Result<()> {
    let samples_dir = get_samples_path()?;
    let mut file = std::fs::File::open(format!("{samples_dir}/z.zip"))?;

    let result = exec_plugin_scan(ScanReader::read_seek(&mut file), &TarXzPlugin);
    assert!(result.is_err());

    Ok(())
  }
}
